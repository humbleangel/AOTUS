use crate::provider::{Answer, ToolCallDelta};
use crate::stream::SseParser;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ChatEvent {
    Token { text: String },
    Done { answer: String, duration_ms: u64, ttft_ms: u64 },
    Error { message: String },
    ToolCalls { calls: Vec<ToolCallDelta> },
    ToolResult { call_id: String, output: String },
}

pub struct StreamResult {
    pub answer: Answer,
    pub ttft_ms: u64,
    pub duration_ms: u64,
}

pub async fn run_stream(
    url: String,
    body: serde_json::Value,
    api_key: String,
    event_tx: mpsc::Sender<ChatEvent>,
    cancel: CancellationToken,
    timeout_secs: u64,
) -> Result<StreamResult, crate::ChatError> {
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "text/event-stream")
        .json(&body)
        .send()
        .await
        .map_err(|e| crate::ChatError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        return if status == 401 || status == 403 {
            Err(crate::ChatError::Auth(text))
        } else {
            Err(crate::ChatError::Network(format!("HTTP {}: {}", status, text)))
        };
    }

    let start = std::time::Instant::now();
    let mut ttft_ms: Option<u64> = None;
    let mut full_content = String::new();
    let mut full_reasoning = String::new();
    let mut full_tool_calls: Vec<ToolCallDelta> = vec![];
    let mut model_name = String::new();
    let mut usage = None;

    let mut byte_stream = response.bytes_stream();
    let mut parser = SseParser::new();
    let deadline = tokio::time::Instant::now()
        + tokio::time::Duration::from_secs(timeout_secs);

    let build_result = || async {
        loop {
            tokio::select! {
                biased;

                _ = cancel.cancelled() => {
                    return Err(crate::ChatError::Cancel);
                }

                chunk = futures_util::StreamExt::next(&mut byte_stream) => {
                    match chunk {
                        Some(Ok(bytes)) => {
                            let text = String::from_utf8_lossy(&bytes);
                            for line in text.lines() {
                                if let Some(sse_event) = parser.feed_line(line) {
                                    match sse_event {
                                        crate::stream::SseEvent::Data(json_str) => {
                                            let value: serde_json::Value =
                                                serde_json::from_str(&json_str)
                                                .map_err(|e| crate::ChatError::Parse(e.to_string()))?;

                                            let has_message = value["choices"][0]["message"].is_object();
                                            if has_message {
                                                if let Ok(answer) = crate::response::parse_batch_response(&value) {
                                                    full_content.push_str(&answer.content);
                                                    full_reasoning.push_str(&answer.reasoning_content);
                                                    if !answer.tool_calls.is_empty() {
                                                        full_tool_calls = answer.tool_calls;
                                                    }
                                                    if !answer.model.is_empty() {
                                                        model_name = answer.model;
                                                    }
                                                    if answer.usage.is_some() {
                                                        usage = answer.usage;
                                                    }
                                                }
                                            } else if let Ok(answer) = crate::response::parse_stream_delta(&value) {
                                                if ttft_ms.is_none() && !answer.content.is_empty() {
                                                    ttft_ms = Some(start.elapsed().as_millis() as u64);
                                                }
                                                if !answer.content.is_empty() {
                                                    full_content.push_str(&answer.content);
                                                    event_tx.send(ChatEvent::Token {
                                                        text: answer.content,
                                                    }).await.ok();
                                                }
                                                if !answer.reasoning_content.is_empty() {
                                                    full_reasoning.push_str(&answer.reasoning_content);
                                                    event_tx.send(ChatEvent::Token {
                                                        text: answer.reasoning_content,
                                                    }).await.ok();
                                                }
                                                if !answer.model.is_empty() {
                                                    model_name = answer.model;
                                                }
                                                if !answer.tool_calls.is_empty() {
                                                    full_tool_calls = answer.tool_calls.clone();
                                                    event_tx.send(ChatEvent::ToolCalls {
                                                        calls: answer.tool_calls,
                                                    }).await.ok();
                                                }
                                            }
                                            if let Some(finish) = value["choices"][0]["finish_reason"].as_str() {
                                                if !finish.is_empty() && finish != "null" {
                                                    let duration = start.elapsed().as_millis() as u64;
                                                    let ttft = ttft_ms.unwrap_or(duration);
                                                    event_tx.send(ChatEvent::Done {
                                                        answer: full_content.clone(),
                                                        duration_ms: duration,
                                                        ttft_ms: ttft,
                                                    }).await.ok();
                                                    let duration_ms = start.elapsed().as_millis() as u64;
                                                    return Ok(StreamResult {
                                                        answer: Answer {
                                                            content: full_content,
                                                            model: model_name,
                                                            usage,
                                                            reasoning_content: full_reasoning,
                                                            tool_calls: full_tool_calls,
                                                        },
                                                        ttft_ms: ttft,
                                                        duration_ms,
                                                    });
                                                }
                                            }
                                        }
                                        crate::stream::SseEvent::Done => {
                                            let duration = start.elapsed().as_millis() as u64;
                                            let ttft = ttft_ms.unwrap_or(duration);
                                            event_tx.send(ChatEvent::Done {
                                                answer: full_content.clone(),
                                                duration_ms: duration,
                                                ttft_ms: ttft,
                                            }).await.ok();
                                            return Ok(StreamResult {
                                                answer: Answer {
                                                    content: full_content,
                                                    model: model_name,
                                                    usage,
                                                    reasoning_content: full_reasoning,
                                                    tool_calls: full_tool_calls,
                                                },
                                                ttft_ms: ttft,
                                                duration_ms: duration,
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            return Err(crate::ChatError::Network(e.to_string()));
                        }
                        None => {
                            let duration = start.elapsed().as_millis() as u64;
                            let ttft = ttft_ms.unwrap_or(duration);
                            event_tx.send(ChatEvent::Done {
                                answer: full_content.clone(),
                                duration_ms: duration,
                                ttft_ms: ttft,
                            }).await.ok();
                            return Ok(StreamResult {
                                answer: Answer {
                                    content: full_content,
                                    model: model_name,
                                    usage,
                                    reasoning_content: full_reasoning,
                                    tool_calls: full_tool_calls,
                                },
                                ttft_ms: ttft,
                                duration_ms: duration,
                            });
                        }
                    }
                }

                _ = tokio::time::sleep_until(deadline) => {
                    return Err(crate::ChatError::Timeout);
                }
            }
        }
    };

    build_result().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_event_serialize_token() {
        let event = ChatEvent::Token { text: "hello".into() };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"Token\""));
    }

    #[test]
    fn test_chat_event_serialize_done() {
        let event = ChatEvent::Done {
            answer: "done".into(), duration_ms: 1000, ttft_ms: 200,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"Done\""));
    }

    #[test]
    fn test_chat_event_serialize_tool_calls() {
        let event = ChatEvent::ToolCalls { calls: vec![] };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"ToolCalls\""));
    }
}
