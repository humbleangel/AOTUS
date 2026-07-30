pub mod config;
pub mod provider;
pub mod request;
pub mod response;
pub mod stream;
pub mod chat_engine;
pub mod tool_orchestrator;
pub mod db;

mod commands;
pub use commands::*;

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

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            chat_stream, chat_batch, cancel_request, save_message, get_models,
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
