use crate::*;

#[tauri::command]
pub async fn chat_stream(
    state: tauri::State<'_, AppState>,
    message: String,
    session_id: String,
    model_name: String,
    channel: tauri::ipc::Channel<chat_engine::ChatEvent>,
) -> Result<String, ChatError> {
    let model_config = state.config.resolve_model(&model_name)
        .ok_or_else(|| ChatError::Unknown(format!("unknown model: {}", model_name)))?
        .clone();

    let adapter = provider::resolve_adapter(&state.config, &model_config)?;
    let api_key = state.config.api_key(&model_config.provider)
        .ok_or_else(|| ChatError::Auth(format!("no key for provider: {}", model_config.provider)))?
        .clone();
    let timeout = state.config.stream_timeout_secs;

    let mut bundle = request::RequestBundle::new(model_config.clone());
    if let Ok(history) = state.db.get_messages(&session_id) {
        for msg in &history {
            bundle.messages.push(provider::ChatMessage {
                role: msg.role.clone(), content: msg.content.clone(),
                tool_calls: None, tool_call_id: None,
            });
        }
    }
    bundle.push_user(&message);

    let request_id = uuid::Uuid::new_v4().to_string();
    let cancel_token = tokio_util::sync::CancellationToken::new();
    state.cancel_map.insert(request_id.clone(), cancel_token.clone());

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<chat_engine::ChatEvent>(256);

    let _bridge = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let _ = channel.send(event);
        }
    });

    let _engine = tokio::spawn(async move {
        let mut bundle = bundle;
        let orchestrator = tool_orchestrator::ToolOrchestrator::new(
            tool_orchestrator::ToolRegistry::new()
        );
        let client = reqwest::Client::new();
        let mut rounds = 0;

        loop {
            bundle.tool_defs = orchestrator.registry().tool_defs();
            let input = request::BuildRequestInput {
                model: bundle.model_config.name.clone(),
                messages: bundle.messages.clone(),
                temperature: bundle.model_config.temperature,
                max_tokens: bundle.model_config.max_tokens,
                stream: true,
                tool_defs: bundle.tool_defs.clone(),
            };
            let body = match request::build_request(&input, &bundle.model_config, &*adapter) {
                Ok(b) => b,
                Err(e) => {
                    let _ = event_tx.send(chat_engine::ChatEvent::Error {
                        message: e.to_string(),
                    }).await;
                    return;
                }
            };
            let url = adapter.endpoint(&bundle.model_config);

            let response = match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key.clone()))
                .header("Accept", "text/event-stream")
                .json(&body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = event_tx.send(chat_engine::ChatEvent::Error {
                        message: e.to_string(),
                    }).await;
                    return;
                }
            };

            let result = chat_engine::run_stream(
                response, event_tx.clone(), cancel_token.clone(), timeout,
            ).await;

            let stream_result = match result {
                Ok(r) => r,
                Err(e) => {
                    let _ = event_tx.send(chat_engine::ChatEvent::Error {
                        message: e.to_string(),
                    }).await;
                    return;
                }
            };

            if stream_result.answer.tool_calls.is_empty() {
                let _ = event_tx.send(chat_engine::ChatEvent::Done {
                    answer: stream_result.answer.content.clone(),
                    duration_ms: stream_result.duration_ms,
                    ttft_ms: stream_result.ttft_ms,
                }).await;
                return;
            }

            if rounds >= 5 {
                let _ = event_tx.send(chat_engine::ChatEvent::Done {
                    answer: stream_result.answer.content.clone(),
                    duration_ms: stream_result.duration_ms,
                    ttft_ms: stream_result.ttft_ms,
                }).await;
                return;
            }
            rounds += 1;

            let tool_calls = stream_result.answer.tool_calls.clone();
            let _ = event_tx.send(chat_engine::ChatEvent::ToolCalls {
                calls: tool_calls.clone(),
            }).await;

            bundle.push_assistant(&stream_result.answer.content, tool_calls.clone());

            let cycle = orchestrator.run_cycle(
                &mut bundle, &tool_calls, Some(&event_tx),
            ).await;
            bundle.tool_defs = orchestrator.registry().tool_defs();

            match cycle {
                tool_orchestrator::CycleResult::Continue => {},
                _ => {
                    let _ = event_tx.send(chat_engine::ChatEvent::Done {
                        answer: stream_result.answer.content.clone(),
                        duration_ms: stream_result.duration_ms,
                        ttft_ms: stream_result.ttft_ms,
                    }).await;
                    return;
                }
            }
        }
    });

    Ok(request_id)
}

#[tauri::command]
pub async fn chat_batch(
    state: tauri::State<'_, AppState>,
    message: String,
    session_id: String,
    model_name: String,
) -> Result<provider::Answer, ChatError> {
    let model_config = state.config.resolve_model(&model_name)
        .ok_or_else(|| ChatError::Unknown(format!("unknown model: {}", model_name)))?
        .clone();
    let adapter = provider::resolve_adapter(&state.config, &model_config)?;
    let api_key = state.config.api_key(&model_config.provider)
        .ok_or_else(|| ChatError::Auth(format!("no key for provider: {}", model_config.provider)))?
        .clone();

    let mut messages = vec![];
    if let Ok(history) = state.db.get_messages(&session_id) {
        for msg in &history {
            messages.push(provider::ChatMessage {
                role: msg.role.clone(), content: msg.content.clone(),
                tool_calls: None, tool_call_id: None,
            });
        }
    }
    messages.push(request::build_user_message(&message));

    let input = request::BuildRequestInput {
        model: model_name.clone(),
        messages,
        temperature: model_config.temperature,
        max_tokens: model_config.max_tokens,
        stream: false,
        tool_defs: vec![],
    };
    let body = request::build_request(&input, &model_config, adapter.as_ref())?;
    let url = adapter.endpoint(&model_config);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| ChatError::Network(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(ChatError::Network(format!("HTTP {}: {}", status, text)));
    }

    let text = response.text().await
        .map_err(|e| ChatError::Network(e.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ChatError::Parse(e.to_string()))?;
    crate::response::parse_batch_response(&value)
}

#[tauri::command]
pub fn cancel_request(
    state: tauri::State<'_, AppState>,
    request_id: String,
) -> Result<(), ChatError> {
    state.cancel_map.cancel(&request_id);
    Ok(())
}

#[tauri::command]
pub fn get_sessions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<db::SessionRow>, ChatError> {
    state.db.get_sessions()
}

#[tauri::command]
pub fn create_session(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<db::SessionRow, ChatError> {
    let id = uuid::Uuid::new_v4().to_string();
    state.db.as_ref().create_session(&id, &name)
}

#[tauri::command]
pub fn delete_session(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), ChatError> {
    state.db.as_ref().delete_session(&id)
}

#[tauri::command]
pub fn save_message(
    state: tauri::State<'_, AppState>,
    session_id: String,
    role: String,
    content: String,
    model: Option<String>,
) -> Result<db::MessageRow, ChatError> {
    state.db.insert_message(db::NewMessage {
        session_id, role, content, model, tool_calls: None,
    })
}

#[tauri::command]
pub fn get_models(state: tauri::State<'_, AppState>) -> Result<Vec<crate::config::ModelConfig>, ChatError> {
    Ok(state.config.models.clone())
}

#[tauri::command]
pub fn get_messages(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Vec<db::MessageRow>, ChatError> {
    state.db.get_messages(&session_id)
}
