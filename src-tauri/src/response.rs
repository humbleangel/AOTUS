use crate::provider::{Answer, ToolCallDelta, Usage};

pub fn parse_batch_response(body: &serde_json::Value) -> Result<Answer, crate::ChatError> {
    let choices = body["choices"]
        .as_array()
        .ok_or_else(|| crate::ChatError::Parse("missing choices".into()))?;

    let choice = choices.first()
        .ok_or_else(|| crate::ChatError::Parse("empty choices".into()))?;

    let msg = choice["message"].as_object()
        .ok_or_else(|| crate::ChatError::Parse("missing message".into()))?;

    let content = msg.get("content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| crate::ChatError::Parse("missing content".into()))?
        .to_string();

    let reasoning = msg.get("reasoning_content")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string();

    let model = body["model"].as_str().unwrap_or("").to_string();

    let usage: Option<Usage> = serde_json::from_value(body["usage"].clone()).ok();

    let tool_calls: Vec<ToolCallDelta> = msg.get("tool_calls")
        .and_then(|tc| tc.as_array())
        .map(|arr| {
            arr.iter().map(|tc| {
                let f = tc.get("function");
                ToolCallDelta {
                    id: tc["id"].as_str().map(String::from),
                    call_type: tc["type"].as_str().map(String::from),
                    function: f.map(|f| crate::provider::ToolCallFunc {
                        name: f["name"].as_str().map(String::from),
                        arguments: f["arguments"].as_str().map(String::from),
                    }),
                }
            }).collect()
        })
        .unwrap_or_default();

    Ok(Answer { content, model, usage, reasoning_content: reasoning, tool_calls })
}

pub fn parse_stream_delta(body: &serde_json::Value) -> Result<Answer, crate::ChatError> {
    let choices = body["choices"]
        .as_array()
        .ok_or_else(|| crate::ChatError::Parse("missing choices".into()))?;

    let choice = choices.first()
        .ok_or_else(|| crate::ChatError::Parse("empty choices".into()))?;

    let delta = choice["delta"].as_object()
        .ok_or_else(|| crate::ChatError::Parse("missing delta".into()))?;

    let content = delta.get("content")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    let reasoning = delta.get("reasoning_content")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string();

    let model = body["model"].as_str().unwrap_or("").to_string();

    let tool_calls: Vec<ToolCallDelta> = delta.get("tool_calls")
        .and_then(|tc| tc.as_array())
        .map(|arr| {
            arr.iter().map(|tc| {
                let f = tc.get("function");
                ToolCallDelta {
                    id: tc["id"].as_str().map(String::from),
                    call_type: tc["type"].as_str().map(String::from),
                    function: f.map(|f| crate::provider::ToolCallFunc {
                        name: f["name"].as_str().map(String::from),
                        arguments: f["arguments"].as_str().map(String::from),
                    }),
                }
            }).collect()
        })
        .unwrap_or_default();

    Ok(Answer { content, model, usage: None, reasoning_content: reasoning, tool_calls })
}

pub fn extract_usage(body: &serde_json::Value) -> Option<Usage> {
    serde_json::from_value(body["usage"].clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_batch_response_basic() {
        let json = serde_json::json!({
            "id": "chatcmpl-123",
            "choices": [{"index": 0, "message": {"content": "Hello!", "role": "assistant"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
            "model": "nvidia/llama"
        });
        let answer = parse_batch_response(&json).unwrap();
        assert_eq!(answer.content, "Hello!");
        assert_eq!(answer.model, "nvidia/llama");
        assert_eq!(answer.usage.unwrap().total_tokens, 15);
    }

    #[test]
    fn test_parse_batch_response_with_reasoning() {
        let json = serde_json::json!({
            "choices": [{"message": {"content": "answer", "reasoning_content": "thinking..."}}],
            "model": "test"
        });
        let answer = parse_batch_response(&json).unwrap();
        assert_eq!(answer.reasoning_content, "thinking...");
    }

    #[test]
    fn test_parse_batch_response_missing_choices() {
        let json = serde_json::json!({});
        assert!(parse_batch_response(&json).is_err());
    }

    #[test]
    fn test_parse_batch_response_empty_choices() {
        let json = serde_json::json!({"choices": []});
        assert!(parse_batch_response(&json).is_err());
    }

    #[test]
    fn test_parse_stream_delta_basic() {
        let json = serde_json::json!({
            "choices": [{"index": 0, "delta": {"content": "World"}, "finish_reason": null}]
        });
        let answer = parse_stream_delta(&json).unwrap();
        assert_eq!(answer.content, "World");
    }

    #[test]
    fn test_parse_stream_delta_with_reasoning() {
        let json = serde_json::json!({
            "choices": [{"delta": {"content": "x", "reasoning_content": "step by step"}}]
        });
        let answer = parse_stream_delta(&json).unwrap();
        assert_eq!(answer.reasoning_content, "step by step");
    }

    #[test]
    fn test_parse_stream_delta_empty() {
        let json = serde_json::json!({
            "choices": [{"delta": {}}]
        });
        let answer = parse_stream_delta(&json).unwrap();
        assert_eq!(answer.content, "");
    }

    #[test]
    fn test_extract_usage() {
        let json = serde_json::json!({"usage": {"prompt_tokens": 1, "completion_tokens": 2, "total_tokens": 3}});
        let u = extract_usage(&json).unwrap();
        assert_eq!(u.total_tokens, 3);
    }

    #[test]
    fn test_extract_usage_missing() {
        let json = serde_json::json!({});
        assert!(extract_usage(&json).is_none());
    }
}
