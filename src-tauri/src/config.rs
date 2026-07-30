use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub models: Vec<ModelConfig>,
    pub api_keys: HashMap<String, String>,
    pub default_provider: String,
    pub default_model: String,
    pub stream_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelConfig {
    pub name: String,
    pub provider: String,
    pub alias: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kwargs: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_extra: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_budget: Option<u32>,
}

fn default_temperature() -> f64 { 0.7 }
fn default_max_tokens() -> usize { 4096 }

impl AppConfig {
    pub fn resolve_model(&self, name: &str) -> Option<&ModelConfig> {
        self.models.iter().find(|m| m.name == name)
    }

    pub fn api_key(&self, provider: &str) -> Option<&String> {
        self.api_keys.get(provider)
    }
}

pub fn load_config(path: &str) -> Result<AppConfig, crate::ChatError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| crate::ChatError::Network(e.to_string()))?;
    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| crate::ChatError::Parse(e.to_string()))?;
    Ok(config)
}

pub fn default_config() -> AppConfig {
    AppConfig {
        models: vec![ModelConfig {
            name: "nvidia/llama-3.1-8b-instruct".into(),
            provider: "nvidia".into(),
            alias: "Llama 3.1 8B".into(),
            temperature: 0.7,
            max_tokens: 4096,
            kwargs: None,
            body_extra: None,
            reasoning_budget: None,
        }],
        api_keys: [("nvidia".into(), "nvapi-...".into())].into(),
        default_provider: "nvidia".into(),
        default_model: "nvidia/llama-3.1-8b-instruct".into(),
        stream_timeout_secs: 120,
    }
}

pub fn load_or_create_config(path: &str) -> Result<AppConfig, crate::ChatError> {
    if std::path::Path::new(path).exists() {
        load_config(path)
    } else {
        let cfg = default_config();
        let json = serde_json::to_string_pretty(&cfg)
            .map_err(|e| crate::ChatError::Parse(e.to_string()))?;
        std::fs::write(path, json)
            .map_err(|e| crate::ChatError::Network(e.to_string()))?;
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_nvidia() {
        let cfg = default_config();
        assert_eq!(cfg.default_provider, "nvidia");
        assert_eq!(cfg.models.len(), 1);
        assert_eq!(cfg.models[0].name, "nvidia/llama-3.1-8b-instruct");
    }

    #[test]
    fn test_load_config_from_json() {
        let json = r#"{
            "models": [{"name":"test/model","provider":"nvidia","alias":"Test","temperature":0.5,"max_tokens":2048}],
            "api_keys": {"nvidia": "key"},
            "default_provider": "nvidia",
            "default_model": "test/model",
            "stream_timeout_secs": 60
        }"#;
        let cfg: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.models.len(), 1);
        assert_eq!(cfg.api_key("nvidia").unwrap(), "key");
        assert_eq!(cfg.stream_timeout_secs, 60);
    }

    #[test]
    fn test_resolve_model() {
        let cfg = default_config();
        let m = cfg.resolve_model("nvidia/llama-3.1-8b-instruct").unwrap();
        assert_eq!(m.provider, "nvidia");
        assert!(cfg.resolve_model("nonexistent").is_none());
    }
}
