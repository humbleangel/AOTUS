pub mod config;
pub mod provider;
pub mod request;
pub mod response;
pub mod stream;
pub mod chat_engine;
pub mod tool_orchestrator;
pub mod db;

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

pub struct AppState {
    pub config: config::AppConfig,
    pub db: Box<dyn db::MessageDb>,
    pub cancel_map: CancellationMap,
}

pub struct CancellationMap {
    tokens: Mutex<HashMap<String, CancellationToken>>,
}

impl CancellationMap {
    pub fn new() -> Self {
        Self { tokens: Mutex::new(HashMap::new()) }
    }

    pub fn insert(&self, id: String, token: CancellationToken) {
        let mut map = self.tokens.lock().unwrap();
        map.insert(id, token);
    }

    pub fn cancel(&self, id: &str) {
        let mut map = self.tokens.lock().unwrap();
        if let Some(token) = map.remove(id) {
            token.cancel();
        }
    }

    pub fn remove(&self, id: &str) {
        let mut map = self.tokens.lock().unwrap();
        map.remove(id);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Timeout")]
    Timeout,
    #[error("Database error: {0}")]
    Db(String),
    #[error("Cancelled")]
    Cancel,
    #[error("Tool error: {0}")]
    Tool(String),
    #[error("{0}")]
    Unknown(String),
}

impl From<ChatError> for tauri::ipc::InvokeError {
    fn from(e: ChatError) -> Self {
        tauri::ipc::InvokeError::from(e.to_string())
    }
}

#[tauri::command]
async fn chat_stream(
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
    let endpoint = adapter.endpoint(&model_config);
    let timeout = state.config.stream_timeout_secs;

    let mut messages = vec![];
    if let Ok(history) = state.db.get_messages(&session_id) {
        for msg in &history {
            messages.push(provider::ChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }
    messages.push(request::build_user_message(&message));

    let input = request::BuildRequestInput {
        model: model_name.clone(),
        messages,
        temperature: model_config.temperature,
        max_tokens: model_config.max_tokens,
        stream: true,
        tool_defs: vec![],
    };
    let body = request::build_request(&input, &model_config, adapter.as_ref())?;

    let request_id = uuid::Uuid::new_v4().to_string();
    let cancel_token = tokio_util::sync::CancellationToken::new();
    state.cancel_map.insert(request_id.clone(), cancel_token.clone());

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<chat_engine::ChatEvent>(256);

    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let _ = channel.send(event);
        }
    });

    tokio::spawn(async move {
        let _ = chat_engine::run_stream(
            endpoint, body, api_key, event_tx, cancel_token, timeout,
        ).await;
    });

    Ok(request_id)
}

#[tauri::command]
fn cancel_request(
    state: tauri::State<'_, AppState>,
    request_id: String,
) -> Result<(), ChatError> {
    state.cancel_map.cancel(&request_id);
    Ok(())
}

#[tauri::command]
fn get_sessions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<db::SessionRow>, ChatError> {
    state.db.get_sessions()
}

#[tauri::command]
fn create_session(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<db::SessionRow, ChatError> {
    let id = uuid::Uuid::new_v4().to_string();
    state.db.as_ref().create_session(&id, &name)
}

#[tauri::command]
fn delete_session(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), ChatError> {
    state.db.as_ref().delete_session(&id)
}

#[tauri::command]
fn get_messages(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Vec<db::MessageRow>, ChatError> {
    state.db.get_messages(&session_id)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            chat_stream, cancel_request,
            get_sessions, create_session, delete_session, get_messages,
        ])
        .setup(|app| {
            let config_path = std::env::var("AOTUS_CONFIG")
                .unwrap_or_else(|_| "config.example.json".to_string());
            let config = config::load_or_create_config(&config_path)
                .expect("failed to load config");

            let db_path = app.path().app_data_dir()
                .map(|p| p.join("aotus.db"))
                .unwrap_or_else(|_| std::path::PathBuf::from("aotus.db"));

            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let db: Box<dyn db::MessageDb> = Box::new(
                db::SqliteDb::open(db_path.to_str().unwrap_or("aotus.db"))
                    .expect("failed to open database")
            );

            app.manage(AppState {
                config,
                db,
                cancel_map: CancellationMap::new(),
            });

            let _window = app.get_webview_window("main").unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running AOTUS");
}
