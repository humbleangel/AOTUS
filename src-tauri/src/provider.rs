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
    fn endpoint(&self, model: &ModelConfig) -> String;
    fn build_body(&self, params: &ChatParams, model: &ModelConfig) -> serde_json::Value;
    fn parse_response(&self, body: &serde_json::Value) -> Result<Answer, crate::ChatError>;
}

pub fn resolve_adapter(
    _config: &AppConfig,
    model: &ModelConfig,
) -> Result<Box<dyn ProviderAdapter>, crate::ChatError> {
    let provider = Provider::from_name(&model.provider)
        .ok_or_else(|| crate::ChatError::Unknown(format!("unknown provider: {}", model.provider)))?;
    match provider {
        Provider::Nvidia => Ok(Box::new(NvidiaAdapter::new())),
        Provider::OpenRouter => Ok(Box::new(OpenRouterAdapter::new())),
        Provider::Zen => Ok(Box::new(ZenAdapter::new())),
        Provider::Go => Ok(Box::new(GoAdapter::new())),
    }
}

pub struct NvidiaAdapter;

impl NvidiaAdapter {
    pub fn new() -> Self {
        Self
    }

    fn default_endpoint() -> &'static str {
        "https://api.nvidia.com/v1/chat/completions"
    }
}

impl ProviderAdapter for NvidiaAdapter {
    fn provider(&self) -> Provider { Provider::Nvidia }

    fn endpoint(&self, model: &ModelConfig) -> String {
        model.kwargs
            .as_ref()
            .and_then(|k| k.get("base_url"))
            .and_then(|v| v.as_str())
            .unwrap_or(Self::default_endpoint())
            .to_string()
    }

    fn build_body(&self, params: &ChatParams, model: &ModelConfig) -> serde_json::Value {
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
        body
    }

    fn parse_response(&self, body: &serde_json::Value) -> Result<Answer, crate::ChatError> {
        if body["choices"][0]["delta"].is_object() {
            crate::response::parse_stream_delta(body)
        } else {
            crate::response::parse_batch_response(body)
        }
    }
}

// ── OpenRouter ──────────────────────────────────────────

pub struct OpenRouterAdapter;

impl OpenRouterAdapter {
    pub fn new() -> Self { Self }
    fn default_endpoint() -> &'static str {
        "https://openrouter.ai/api/v1/chat/completions"
    }
}

impl ProviderAdapter for OpenRouterAdapter {
    fn provider(&self) -> Provider { Provider::OpenRouter }
    fn endpoint(&self, model: &ModelConfig) -> String {
        model.kwargs.as_ref()
            .and_then(|k| k.get("base_url"))
            .and_then(|v| v.as_str())
            .unwrap_or(Self::default_endpoint())
            .to_string()
    }
    fn build_body(&self, params: &ChatParams, model: &ModelConfig) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": params.model,
            "messages": params.messages,
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
            "stream": params.stream,
        });
        if let Some(extra) = &model.body_extra {
            if let Some(obj) = extra.as_object() {
                for (k, v) in obj { body[k] = v.clone(); }
            }
        }
        body
    }
    fn parse_response(&self, body: &serde_json::Value) -> Result<Answer, crate::ChatError> {
        if body["choices"][0]["delta"].is_object() {
            crate::response::parse_stream_delta(body)
        } else {
            crate::response::parse_batch_response(body)
        }
    }
}

// ── Zen ────────────────────────────────────────────────

pub struct ZenAdapter;

impl ZenAdapter {
    pub fn new() -> Self { Self }
    fn default_endpoint() -> &'static str {
        "https://api.zen.com/v1/chat/completions"
    }
}

impl ProviderAdapter for ZenAdapter {
    fn provider(&self) -> Provider { Provider::Zen }
    fn endpoint(&self, model: &ModelConfig) -> String {
        model.kwargs.as_ref()
            .and_then(|k| k.get("base_url"))
            .and_then(|v| v.as_str())
            .unwrap_or(Self::default_endpoint())
            .to_string()
    }
    fn build_body(&self, params: &ChatParams, model: &ModelConfig) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": params.model,
            "messages": params.messages,
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
            "stream": params.stream,
        });
        if let Some(extra) = &model.body_extra {
            if let Some(obj) = extra.as_object() {
                for (k, v) in obj { body[k] = v.clone(); }
            }
        }
        body
    }
    fn parse_response(&self, body: &serde_json::Value) -> Result<Answer, crate::ChatError> {
        if body["choices"][0]["delta"].is_object() {
            crate::response::parse_stream_delta(body)
        } else {
            crate::response::parse_batch_response(body)
        }
    }
}

// ── GO (subscription) ──────────────────────────────────

pub struct GoAdapter;

impl GoAdapter {
    pub fn new() -> Self { Self }
    fn default_endpoint() -> &'static str {
        "https://api.go.com/v1/chat/completions"
    }
}

impl ProviderAdapter for GoAdapter {
    fn provider(&self) -> Provider { Provider::Go }
    fn endpoint(&self, model: &ModelConfig) -> String {
        model.kwargs.as_ref()
            .and_then(|k| k.get("base_url"))
            .and_then(|v| v.as_str())
            .unwrap_or(Self::default_endpoint())
            .to_string()
    }
    fn build_body(&self, params: &ChatParams, model: &ModelConfig) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": params.model,
            "messages": params.messages,
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
            "stream": params.stream,
        });
        if let Some(reasoning) = &model.reasoning_budget {
            body["reasoning_budget"] = serde_json::json!(reasoning);
        }
        if let Some(extra) = &model.body_extra {
            if let Some(obj) = extra.as_object() {
                for (k, v) in obj { body[k] = v.clone(); }
            }
        }
        body
    }
    fn parse_response(&self, body: &serde_json::Value) -> Result<Answer, crate::ChatError> {
        if body["choices"][0]["delta"].is_object() {
            crate::response::parse_stream_delta(body)
        } else {
            crate::response::parse_batch_response(body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAdapter;

    impl ProviderAdapter for MockAdapter {
        fn provider(&self) -> Provider { Provider::Nvidia }
        fn endpoint(&self, _model: &ModelConfig) -> String {
            "https://test.api.com/v1/chat".into()
        }
        fn build_body(&self, params: &ChatParams, _model: &ModelConfig) -> serde_json::Value {
            serde_json::json!({
                "model": params.model,
                "messages": params.messages,
                "stream": params.stream,
            })
        }
        fn parse_response(&self, body: &serde_json::Value) -> Result<Answer, crate::ChatError> {
            let answer = Answer {
                content: body["choices"][0]["message"]["content"]
                    .as_str().unwrap_or("").into(),
                model: body["model"].as_str().unwrap_or("").into(),
                usage: None,
                reasoning_content: String::new(),
                tool_calls: vec![],
            };
            Ok(answer)
        }
    }

    #[test]
    fn test_trait_compiles_with_mock() {
        let adapter = MockAdapter;
        assert_eq!(adapter.provider(), Provider::Nvidia);

        let model = ModelConfig {
            name: "test/model".into(),
            provider: "nvidia".into(),
            alias: "Test".into(),
            temperature: 0.7,
            max_tokens: 4096,
            kwargs: None,
            body_extra: None,
            reasoning_budget: None,
        };
        let ep = adapter.endpoint(&model);
        assert!(ep.contains("test.api.com"));

        let params = ChatParams {
            model: "test/model".into(),
            messages: vec![],
            temperature: 0.7,
            max_tokens: 4096,
            stream: true,
        };
        let body = adapter.build_body(&params, &model);
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
        let adapter = NvidiaAdapter::new();
        let model = ModelConfig {
            name: "test/model".into(), provider: "nvidia".into(),
            alias: "Test".into(), temperature: 0.7, max_tokens: 4096,
            kwargs: None, body_extra: None, reasoning_budget: None,
        };
        let ep = adapter.endpoint(&model);
        assert_eq!(ep, "https://api.nvidia.com/v1/chat/completions");
    }

    #[test]
    fn test_nvidia_endpoint_with_base_url_kwarg() {
        let adapter = NvidiaAdapter::new();
        let model = ModelConfig {
            name: "test/model".into(), provider: "nvidia".into(),
            alias: "Test".into(), temperature: 0.7, max_tokens: 4096,
            kwargs: Some(serde_json::json!({"base_url": "https://custom.nvidia.com/v1/chat"})),
            body_extra: None, reasoning_budget: None,
        };
        let ep = adapter.endpoint(&model);
        assert_eq!(ep, "https://custom.nvidia.com/v1/chat");
    }

    #[test]
    fn test_nvidia_build_body() {
        let adapter = NvidiaAdapter::new();
        let model = ModelConfig {
            name: "test/model".into(), provider: "nvidia".into(),
            alias: "Test".into(), temperature: 0.7, max_tokens: 4096,
            kwargs: None, body_extra: None, reasoning_budget: None,
        };
        let params = ChatParams {
            model: "nvidia/llama".into(),
            messages: vec![ChatMessage {
                role: "user".into(), content: "hi".into(),
                tool_calls: None, tool_call_id: None,
            }],
            temperature: 0.5, max_tokens: 2048, stream: true,
        };
        let body = adapter.build_body(&params, &model);
        assert_eq!(body["model"], "nvidia/llama");
        assert_eq!(body["messages"][0]["content"], "hi");
        assert_eq!(body["temperature"], 0.5);
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn test_nvidia_parse_batch_response() {
        let adapter = NvidiaAdapter::new();
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
        let adapter = NvidiaAdapter::new();
        let json = serde_json::json!({
            "choices": [{"index": 0, "delta": {"content": "World"}, "finish_reason": null}]
        });
        let answer = adapter.parse_response(&json).unwrap();
        assert_eq!(answer.content, "World");
    }

    #[test]
    fn test_nvidia_parse_with_reasoning() {
        let adapter = NvidiaAdapter::new();
        let json = serde_json::json!({
            "choices": [{"index": 0, "delta": {"content": "answer", "reasoning_content": "thinking..."}}]
        });
        let answer = adapter.parse_response(&json).unwrap();
        assert_eq!(answer.content, "answer");
        assert_eq!(answer.reasoning_content, "thinking...");
    }

    // ── Multi-provider integration tests ──────────────

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
    fn test_openrouter_endpoint() {
        let adapter = OpenRouterAdapter::new();
        assert_eq!(adapter.endpoint(&test_model()), "https://openrouter.ai/api/v1/chat/completions");
    }

    #[test]
    fn test_openrouter_build_body() {
        let adapter = OpenRouterAdapter::new();
        let body = adapter.build_body(&test_params(), &test_model());
        assert_eq!(body["model"], "test/model");
        assert_eq!(body["messages"][0]["content"], "hi");
    }

    #[test]
    fn test_openrouter_parse() {
        let adapter = OpenRouterAdapter::new();
        let answer = adapter.parse_response(&batch_response("hello", "openrouter/model")).unwrap();
        assert_eq!(answer.content, "hello");
        assert_eq!(answer.model, "openrouter/model");
    }

    #[test]
    fn test_zen_endpoint() {
        let adapter = ZenAdapter::new();
        assert_eq!(adapter.endpoint(&test_model()), "https://api.zen.com/v1/chat/completions");
    }

    #[test]
    fn test_zen_build_body() {
        let adapter = ZenAdapter::new();
        let body = adapter.build_body(&test_params(), &test_model());
        assert_eq!(body["model"], "test/model");
    }

    #[test]
    fn test_zen_parse() {
        let adapter = ZenAdapter::new();
        let answer = adapter.parse_response(&batch_response("zen answer", "zen/model")).unwrap();
        assert_eq!(answer.content, "zen answer");
    }

    #[test]
    fn test_go_endpoint() {
        let adapter = GoAdapter::new();
        assert_eq!(adapter.endpoint(&test_model()), "https://api.go.com/v1/chat/completions");
    }

    #[test]
    fn test_go_build_body() {
        let adapter = GoAdapter::new();
        let body = adapter.build_body(&test_params(), &test_model());
        assert_eq!(body["model"], "test/model");
    }

    #[test]
    fn test_go_parse() {
        let adapter = GoAdapter::new();
        let answer = adapter.parse_response(&batch_response("go answer", "go/model")).unwrap();
        assert_eq!(answer.content, "go answer");
    }

    #[test]
    fn test_go_reasoning_budget() {
        let adapter = GoAdapter::new();
        let mut model = test_model();
        model.reasoning_budget = Some(1024);
        let body = adapter.build_body(&test_params(), &model);
        assert_eq!(body["reasoning_budget"], 1024);
    }

    #[test]
    fn test_openrouter_body_extra() {
        let adapter = OpenRouterAdapter::new();
        let mut model = test_model();
        model.body_extra = Some(serde_json::json!({"stop": ["END"]}));
        let body = adapter.build_body(&test_params(), &model);
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
