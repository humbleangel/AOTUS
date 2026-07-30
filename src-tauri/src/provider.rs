use crate::config::{AppConfig, ModelConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    Nvidia,
    OpenRouter,
    Zen,
    Go,
}

impl Provider {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "nvidia" => Some(Self::Nvidia),
            "openrouter" => Some(Self::OpenRouter),
            "zen" => Some(Self::Zen),
            "go" => Some(Self::Go),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatParams {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f64,
    pub max_tokens: usize,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: Option<ToolCallFunc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunc {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub content: String,
    pub model: String,
    pub usage: Option<Usage>,
    #[serde(default)]
    pub reasoning_content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCallDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub trait ProviderAdapter: Send + Sync {
    fn provider(&self) -> Provider;
    fn endpoint(&self, model: &str) -> String;
    fn build_body(&self, params: &ChatParams) -> serde_json::Value;
    fn parse_response(&self, body: &serde_json::Value) -> Result<Answer, crate::ChatError> {
        if body["choices"][0]["delta"].is_object() {
            crate::response::parse_stream_delta(body)
        } else {
            crate::response::parse_batch_response(body)
        }
    }
}

pub fn resolve_adapter(
    _config: &AppConfig,
    model: &ModelConfig,
) -> Result<Box<dyn ProviderAdapter>, crate::ChatError> {
    let provider = Provider::from_name(&model.provider)
        .ok_or_else(|| crate::ChatError::Unknown(format!("unknown provider: {}", model.provider)))?;
    match provider {
        Provider::Nvidia => Ok(Box::new(NvidiaAdapter::new(model.clone()))),
        Provider::OpenRouter => Ok(Box::new(OpenRouterAdapter::new(model.clone()))),
        Provider::Zen => Ok(Box::new(ZenAdapter::new(model.clone()))),
        Provider::Go => Ok(Box::new(GoAdapter::new(model.clone()))),
    }
}

fn build_base_body(model: &ModelConfig, params: &ChatParams) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": params.model,
        "messages": params.messages,
        "temperature": params.temperature,
        "max_tokens": params.max_tokens,
        "stream": params.stream,
    });
    if let Some(kwargs) = &model.kwargs {
        if let Some(obj) = kwargs.as_object() {
            for (k, v) in obj {
                if k != "base_url" {
                    body[k] = v.clone();
                }
            }
        }
    }
    if let Some(extra) = &model.body_extra {
        if let Some(obj) = extra.as_object() {
            for (k, v) in obj {
                body[k] = v.clone();
            }
        }
    }
    if let Some(reasoning) = &model.reasoning_budget {
        body["reasoning_budget"] = serde_json::json!(reasoning);
    }
    body
}

fn resolve_endpoint(model: &ModelConfig, default: &str) -> String {
    model.kwargs
        .as_ref()
        .and_then(|k| k.get("base_url"))
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

pub struct NvidiaAdapter {
    model: ModelConfig,
}

impl NvidiaAdapter {
    pub fn new(model: ModelConfig) -> Self { Self { model } }
    fn default_endpoint() -> &'static str { "https://api.nvidia.com/v1/chat/completions" }
}

impl ProviderAdapter for NvidiaAdapter {
    fn provider(&self) -> Provider { Provider::Nvidia }
    fn endpoint(&self, _model: &str) -> String {
        resolve_endpoint(&self.model, Self::default_endpoint())
    }
    fn build_body(&self, params: &ChatParams) -> serde_json::Value {
        build_base_body(&self.model, params)
    }
}

// ── OpenRouter ──────────────────────────────────────────

pub struct OpenRouterAdapter {
    model: ModelConfig,
}

impl OpenRouterAdapter {
    pub fn new(model: ModelConfig) -> Self { Self { model } }
    fn default_endpoint() -> &'static str { "https://openrouter.ai/api/v1/chat/completions" }
}

impl ProviderAdapter for OpenRouterAdapter {
    fn provider(&self) -> Provider { Provider::OpenRouter }
    fn endpoint(&self, _model: &str) -> String {
        resolve_endpoint(&self.model, Self::default_endpoint())
    }
    fn build_body(&self, params: &ChatParams) -> serde_json::Value {
        build_base_body(&self.model, params)
    }
}

// ── Zen ────────────────────────────────────────────────

pub struct ZenAdapter {
    model: ModelConfig,
}

impl ZenAdapter {
    pub fn new(model: ModelConfig) -> Self { Self { model } }
    fn default_endpoint() -> &'static str { "https://api.zen.com/v1/chat/completions" }
}

impl ProviderAdapter for ZenAdapter {
    fn provider(&self) -> Provider { Provider::Zen }
    fn endpoint(&self, _model: &str) -> String {
        resolve_endpoint(&self.model, Self::default_endpoint())
    }
    fn build_body(&self, params: &ChatParams) -> serde_json::Value {
        build_base_body(&self.model, params)
    }
}

// ── GO (subscription) ──────────────────────────────────

pub struct GoAdapter {
    model: ModelConfig,
}

impl GoAdapter {
    pub fn new(model: ModelConfig) -> Self { Self { model } }
    fn default_endpoint() -> &'static str { "https://api.go.com/v1/chat/completions" }
}

impl ProviderAdapter for GoAdapter {
    fn provider(&self) -> Provider { Provider::Go }
    fn endpoint(&self, _model: &str) -> String {
        resolve_endpoint(&self.model, Self::default_endpoint())
    }
    fn build_body(&self, params: &ChatParams) -> serde_json::Value {
        build_base_body(&self.model, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> ModelConfig {
        ModelConfig {
            name: "test/model".into(), provider: "test".into(),
            alias: "Test".into(), temperature: 0.7, max_tokens: 4096,
            kwargs: None, body_extra: None, reasoning_budget: None,
        }
    }

    fn test_params() -> ChatParams {
        ChatParams {
            model: "test/model".into(),
            messages: vec![ChatMessage {
                role: "user".into(), content: "hi".into(),
                tool_calls: None, tool_call_id: None,
            }],
            temperature: 0.7, max_tokens: 4096, stream: true,
        }
    }

    fn batch_response(text: &str, model: &str) -> serde_json::Value {
        serde_json::json!({
            "choices": [{"index": 0, "message": {"content": text, "role": "assistant"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
            "model": model
        })
    }

    #[test]
    fn test_trait_compiles_with_mock() {
        struct MockAdapter;
        impl ProviderAdapter for MockAdapter {
            fn provider(&self) -> Provider { Provider::Nvidia }
            fn endpoint(&self, _model: &str) -> String { "https://test.api.com/v1/chat".into() }
            fn build_body(&self, params: &ChatParams) -> serde_json::Value {
                serde_json::json!({
                    "model": params.model,
                    "messages": params.messages,
                    "stream": params.stream,
                })
            }
        }

        let adapter = MockAdapter;
        assert_eq!(adapter.provider(), Provider::Nvidia);
        let ep = adapter.endpoint("test/model");
        assert!(ep.contains("test.api.com"));
        let body = adapter.build_body(&test_params());
        assert_eq!(body["model"], "test/model");
        let json = serde_json::json!({
            "choices": [{"message": {"content": "hello"}}],
            "model": "test/model"
        });
        let answer = adapter.parse_response(&json).unwrap();
        assert_eq!(answer.content, "hello");
    }

    #[test]
    fn test_nvidia_endpoint_default() {
        let adapter = NvidiaAdapter::new(test_model());
        assert_eq!(adapter.endpoint("test/model"), "https://api.nvidia.com/v1/chat/completions");
    }

    #[test]
    fn test_nvidia_endpoint_with_base_url_kwarg() {
        let mut model = test_model();
        model.kwargs = Some(serde_json::json!({"base_url": "https://custom.nvidia.com/v1/chat"}));
        let adapter = NvidiaAdapter::new(model);
        assert_eq!(adapter.endpoint("test/model"), "https://custom.nvidia.com/v1/chat");
    }

    #[test]
    fn test_nvidia_build_body() {
        let adapter = NvidiaAdapter::new(test_model());
        let params = ChatParams {
            model: "nvidia/llama".into(),
            messages: vec![ChatMessage {
                role: "user".into(), content: "hi".into(),
                tool_calls: None, tool_call_id: None,
            }],
            temperature: 0.5, max_tokens: 2048, stream: true,
        };
        let body = adapter.build_body(&params);
        assert_eq!(body["model"], "nvidia/llama");
        assert_eq!(body["messages"][0]["content"], "hi");
        assert_eq!(body["temperature"], 0.5);
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn test_nvidia_parse_batch_response() {
        let adapter = NvidiaAdapter::new(test_model());
        let json = serde_json::json!({
            "id": "chatcmpl-123",
            "choices": [{"index": 0, "message": {"content": "Hello!", "role": "assistant"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
            "model": "nvidia/llama"
        });
        let answer = adapter.parse_response(&json).unwrap();
        assert_eq!(answer.content, "Hello!");
        assert_eq!(answer.model, "nvidia/llama");
        assert_eq!(answer.usage.unwrap().total_tokens, 15);
    }

    #[test]
    fn test_nvidia_parse_stream_delta() {
        let adapter = NvidiaAdapter::new(test_model());
        let json = serde_json::json!({
            "choices": [{"index": 0, "delta": {"content": "World"}, "finish_reason": null}]
        });
        let answer = adapter.parse_response(&json).unwrap();
        assert_eq!(answer.content, "World");
    }

    #[test]
    fn test_nvidia_parse_with_reasoning() {
        let adapter = NvidiaAdapter::new(test_model());
        let json = serde_json::json!({
            "choices": [{"index": 0, "delta": {"content": "answer", "reasoning_content": "thinking..."}}]
        });
        let answer = adapter.parse_response(&json).unwrap();
        assert_eq!(answer.content, "answer");
        assert_eq!(answer.reasoning_content, "thinking...");
    }

    #[test]
    fn test_openrouter_endpoint() {
        let adapter = OpenRouterAdapter::new(test_model());
        assert_eq!(adapter.endpoint("test/model"), "https://openrouter.ai/api/v1/chat/completions");
    }

    #[test]
    fn test_openrouter_build_body() {
        let adapter = OpenRouterAdapter::new(test_model());
        let body = adapter.build_body(&test_params());
        assert_eq!(body["model"], "test/model");
        assert_eq!(body["messages"][0]["content"], "hi");
    }

    #[test]
    fn test_openrouter_parse() {
        let adapter = OpenRouterAdapter::new(test_model());
        let answer = adapter.parse_response(&batch_response("hello", "openrouter/model")).unwrap();
        assert_eq!(answer.content, "hello");
        assert_eq!(answer.model, "openrouter/model");
    }

    #[test]
    fn test_zen_endpoint() {
        let adapter = ZenAdapter::new(test_model());
        assert_eq!(adapter.endpoint("test/model"), "https://api.zen.com/v1/chat/completions");
    }

    #[test]
    fn test_zen_build_body() {
        let adapter = ZenAdapter::new(test_model());
        let body = adapter.build_body(&test_params());
        assert_eq!(body["model"], "test/model");
    }

    #[test]
    fn test_zen_parse() {
        let adapter = ZenAdapter::new(test_model());
        let answer = adapter.parse_response(&batch_response("zen answer", "zen/model")).unwrap();
        assert_eq!(answer.content, "zen answer");
    }

    #[test]
    fn test_go_endpoint() {
        let adapter = GoAdapter::new(test_model());
        assert_eq!(adapter.endpoint("test/model"), "https://api.go.com/v1/chat/completions");
    }

    #[test]
    fn test_go_build_body() {
        let adapter = GoAdapter::new(test_model());
        let body = adapter.build_body(&test_params());
        assert_eq!(body["model"], "test/model");
    }

    #[test]
    fn test_go_parse() {
        let adapter = GoAdapter::new(test_model());
        let answer = adapter.parse_response(&batch_response("go answer", "go/model")).unwrap();
        assert_eq!(answer.content, "go answer");
    }

    #[test]
    fn test_go_reasoning_budget() {
        let mut model = test_model();
        model.reasoning_budget = Some(1024);
        let adapter = GoAdapter::new(model);
        let body = adapter.build_body(&test_params());
        assert_eq!(body["reasoning_budget"], 1024);
    }

    #[test]
    fn test_openrouter_body_extra() {
        let mut model = test_model();
        model.body_extra = Some(serde_json::json!({"stop": ["END"]}));
        let adapter = OpenRouterAdapter::new(model);
        let body = adapter.build_body(&test_params());
        assert_eq!(body["stop"], serde_json::json!(["END"]));
    }

    #[test]
    fn test_resolve_adapter_nvidia() {
        let mut model = test_model();
        model.provider = "nvidia".into();
        let config = crate::config::default_config();
        let adapter = resolve_adapter(&config, &model).unwrap();
        assert_eq!(adapter.provider(), Provider::Nvidia);
    }

    #[test]
    fn test_resolve_adapter_openrouter() {
        let mut model = test_model();
        model.provider = "openrouter".into();
        let config = crate::config::default_config();
        let adapter = resolve_adapter(&config, &model).unwrap();
        assert_eq!(adapter.provider(), Provider::OpenRouter);
    }

    #[test]
    fn test_resolve_adapter_zen() {
        let mut model = test_model();
        model.provider = "zen".into();
        let config = crate::config::default_config();
        let adapter = resolve_adapter(&config, &model).unwrap();
        assert_eq!(adapter.provider(), Provider::Zen);
    }

    #[test]
    fn test_resolve_adapter_go() {
        let mut model = test_model();
        model.provider = "go".into();
        let config = crate::config::default_config();
        let adapter = resolve_adapter(&config, &model).unwrap();
        assert_eq!(adapter.provider(), Provider::Go);
    }
}
