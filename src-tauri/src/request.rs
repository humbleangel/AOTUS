use crate::config::ModelConfig;
use crate::provider::{ChatMessage, ChatParams, ProviderAdapter, ToolCallDelta};

pub struct RequestBundle {
    pub messages: Vec<ChatMessage>,
    pub model_config: ModelConfig,
    pub tool_defs: Vec<ToolDef>,
}

impl RequestBundle {
    pub fn new(model_config: ModelConfig) -> Self {
        Self { messages: vec![], model_config, tool_defs: vec![] }
    }

    pub fn push_user(&mut self, content: &str) {
        self.messages.push(build_user_message(content));
    }

    pub fn push_assistant(&mut self, content: &str, tool_calls: Vec<ToolCallDelta>) {
        self.messages.push(build_assistant_message(content, tool_calls));
    }

    pub fn push_tool(&mut self, tool_call_id: &str, content: &str) {
        self.messages.push(build_tool_message(tool_call_id, content));
    }
}

#[derive(Debug, Clone)]
pub struct BuildRequestInput {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f64,
    pub max_tokens: usize,
    pub stream: bool,
    pub tool_defs: Vec<ToolDef>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

pub fn build_request(
    input: &BuildRequestInput,
    model_config: &ModelConfig,
    adapter: &dyn ProviderAdapter,
) -> Result<serde_json::Value, crate::ChatError> {
    let params = ChatParams {
        model: input.model.clone(),
        messages: input.messages.clone(),
        temperature: input.temperature,
        max_tokens: input.max_tokens,
        stream: input.stream,
    };

    let mut body = adapter.build_body(&params, model_config);

    if !input.tool_defs.is_empty() {
        let tools: Vec<serde_json::Value> = input.tool_defs.iter().map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            })
        }).collect();
        body["tools"] = serde_json::Value::Array(tools);
    }

    if let Some(extra) = &model_config.body_extra {
        if let Some(obj) = extra.as_object() {
            for (k, v) in obj {
                body[k] = v.clone();
            }
        }
    }

    Ok(body)
}

pub fn build_user_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".into(),
        content: content.to_string(),
        tool_calls: None,
        tool_call_id: None,
    }
}

pub fn build_assistant_message(
    content: &str,
    tool_calls: Vec<ToolCallDelta>,
) -> ChatMessage {
    let calls = if tool_calls.is_empty() { None } else { Some(tool_calls) };
    ChatMessage {
        role: "assistant".into(),
        content: content.to_string(),
        tool_calls: calls,
        tool_call_id: None,
    }
}

pub fn build_tool_message(tool_call_id: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: "tool".into(),
        content: content.to_string(),
        tool_calls: None,
        tool_call_id: Some(tool_call_id.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAdapter;
    impl ProviderAdapter for MockAdapter {
        fn provider(&self) -> crate::provider::Provider {
            crate::provider::Provider::Nvidia
        }
        fn endpoint(&self, _model: &ModelConfig) -> String {
            "https://test.api.com/v1/chat".into()
        }
        fn build_body(&self, params: &ChatParams, _model: &ModelConfig) -> serde_json::Value {
            serde_json::json!({
                "model": params.model,
                "messages": params.messages,
                "temperature": params.temperature,
                "max_tokens": params.max_tokens,
                "stream": params.stream,
            })
        }
        fn parse_response(&self, _body: &serde_json::Value) -> Result<crate::provider::Answer, crate::ChatError> {
            unimplemented!()
        }
    }

    fn make_model() -> ModelConfig {
        ModelConfig {
            name: "nvidia/llama".into(),
            provider: "nvidia".into(),
            alias: "Test".into(),
            temperature: 0.7, max_tokens: 4096,
            kwargs: None, body_extra: None, reasoning_budget: None,
        }
    }

    #[test]
    fn test_build_request_basic() {
        let adapter = MockAdapter;
        let input = BuildRequestInput {
            model: "nvidia/llama".into(),
            messages: vec![build_user_message("hello")],
            temperature: 0.5, max_tokens: 2048, stream: true,
            tool_defs: vec![],
        };
        let body = build_request(&input, &make_model(), &adapter).unwrap();
        assert_eq!(body["model"], "nvidia/llama");
        assert_eq!(body["messages"][0]["content"], "hello");
        assert_eq!(body["temperature"], 0.5);
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn test_build_request_with_tools() {
        let adapter = MockAdapter;
        let input = BuildRequestInput {
            model: "test".into(),
            messages: vec![],
            temperature: 0.7, max_tokens: 4096, stream: false,
            tool_defs: vec![ToolDef {
                name: "get_weather".into(),
                description: "Get weather".into(),
                parameters: serde_json::json!({"type": "object"}),
            }],
        };
        let body = build_request(&input, &make_model(), &adapter).unwrap();
        assert!(body["tools"].is_array());
        assert_eq!(body["tools"][0]["function"]["name"], "get_weather");
    }

    #[test]
    fn test_build_request_with_body_extra() {
        let adapter = MockAdapter;
        let mut model = make_model();
        model.body_extra = Some(serde_json::json!({"stop": ["END"]}));
        let input = BuildRequestInput {
            model: "test".into(), messages: vec![],
            temperature: 0.7, max_tokens: 4096, stream: false,
            tool_defs: vec![],
        };
        let body = build_request(&input, &model, &adapter).unwrap();
        assert_eq!(body["stop"], serde_json::json!(["END"]));
    }

    #[test]
    fn test_build_message_helpers() {
        let user = build_user_message("hi");
        assert_eq!(user.role, "user");
        assert_eq!(user.content, "hi");

        let assistant = build_assistant_message("hey", vec![]);
        assert_eq!(assistant.role, "assistant");
        assert!(assistant.tool_calls.is_none());

        let tool = build_tool_message("call_123", "result");
        assert_eq!(tool.role, "tool");
        assert_eq!(tool.tool_call_id.unwrap(), "call_123");
    }
}
