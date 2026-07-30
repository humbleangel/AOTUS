use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    fn execute(&self, args: serde_json::Value) -> Result<String, crate::ChatError>;
}

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: RwLock::new(HashMap::new()) }
    }

    pub fn register(&self, tool: Arc<dyn Tool>) {
        let mut map = self.tools.write().unwrap();
        map.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let map = self.tools.read().unwrap();
        map.get(name).cloned()
    }

    pub fn list_all(&self) -> Vec<Arc<dyn Tool>> {
        let map = self.tools.read().unwrap();
        map.values().cloned().collect()
    }

    pub fn tool_defs(&self) -> Vec<crate::request::ToolDef> {
        let map = self.tools.read().unwrap();
        map.values().map(|t| crate::request::ToolDef {
            name: t.name().to_string(),
            description: t.description().to_string(),
            parameters: t.parameters(),
        }).collect()
    }
}

use crate::provider::ToolCallDelta;

pub enum CycleResult {
    Continue,
    Done,
    MaxTurnsReached,
    CycleDetected,
}

pub struct CycleDetector {
    history: VecDeque<(String, String)>,
    max_len: usize,
}

impl CycleDetector {
    pub fn new(max_len: usize) -> Self {
        Self { history: VecDeque::new(), max_len }
    }

    pub fn record(&mut self, name: &str, args: &str) {
        self.history.push_back((name.to_string(), args.to_string()));
        if self.history.len() > self.max_len {
            self.history.pop_front();
        }
    }

    pub fn is_cycle(&self) -> bool {
        if self.history.len() < self.max_len { return false }
        let first = &self.history[0];
        self.history.iter().all(|e| e == first)
    }

    pub fn reset(&mut self) {
        self.history.clear()
    }
}

pub struct ToolOrchestrator {
    registry: ToolRegistry,
    cycle_detector: std::sync::Mutex<CycleDetector>,
}

impl ToolOrchestrator {
    pub fn new(registry: ToolRegistry) -> Self {
        Self { registry, cycle_detector: std::sync::Mutex::new(CycleDetector::new(3)) }
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub async fn run_cycle(
        &self,
        bundle: &mut crate::request::RequestBundle,
        tool_calls: &[ToolCallDelta],
        on_event: Option<&tokio::sync::mpsc::Sender<crate::chat_engine::ChatEvent>>,
    ) -> CycleResult {
        let mut results = vec![];
        for tc in tool_calls {
            let name = match tc.function.as_ref().and_then(|f| f.name.as_deref()) {
                Some(n) => n,
                None => return CycleResult::Done,
            };

            let args_str = tc.function.as_ref()
                .and_then(|f| f.arguments.as_deref())
                .unwrap_or("{}");

            {
                let mut cd = self.cycle_detector.lock().unwrap();
                cd.record(name, args_str);
                if cd.is_cycle() {
                    return CycleResult::CycleDetected;
                }
            }

            let args: serde_json::Value = match serde_json::from_str(args_str) {
                Ok(v) => v,
                Err(_) => return CycleResult::Done,
            };

            let tool = match self.registry.get(name) {
                Some(t) => t,
                None => return CycleResult::Done,
            };

            let output = match tool.execute(args) {
                Ok(o) => o,
                Err(_) => return CycleResult::Done,
            };

            let result = ToolResult {
                tool_call_id: tc.id.clone().unwrap_or_default(),
                content: output,
            };

            if let Some(tx) = on_event {
                let _ = tx.send(crate::chat_engine::ChatEvent::ToolResult {
                    call_id: result.tool_call_id.clone(),
                    output: result.content.clone(),
                }).await;
            }

            results.push(result);
        }

        for r in &results {
            bundle.push_tool(&r.tool_call_id, &r.content);
        }

        CycleResult::Continue
    }
}

pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoTool;

    impl Tool for EchoTool {
        fn name(&self) -> &str { "echo" }
        fn description(&self) -> &str { "echoes input" }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({"type": "object", "properties": {"text": {"type": "string"}}})
        }
        fn execute(&self, args: serde_json::Value) -> Result<String, crate::ChatError> {
            Ok(args["text"].as_str().unwrap_or("").to_string())
        }
    }

    #[test]
    fn test_tool_registry_register_and_get() {
        let registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool));
        let tool = registry.get("echo").unwrap();
        assert_eq!(tool.name(), "echo");
    }

    #[test]
    fn test_tool_registry_list_all() {
        let registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool));
        assert_eq!(registry.list_all().len(), 1);
    }

    #[test]
    fn test_tool_registry_tool_defs() {
        let registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool));
        let defs = registry.tool_defs();
        assert_eq!(defs[0].name, "echo");
    }

    #[test]
    fn test_tool_registry_unknown() {
        let registry = ToolRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_cycle_detector_no_cycle() {
        let mut cd = CycleDetector::new(3);
        cd.record("get_weather", "{}");
        cd.record("search", "{}");
        cd.record("get_weather", "{}");
        assert!(!cd.is_cycle());
    }

    #[test]
    fn test_cycle_detector_detects_cycle() {
        let mut cd = CycleDetector::new(3);
        cd.record("get_weather", "{\"city\":\"NYC\"}");
        cd.record("get_weather", "{\"city\":\"NYC\"}");
        cd.record("get_weather", "{\"city\":\"NYC\"}");
        assert!(cd.is_cycle());
    }

    #[test]
    fn test_cycle_detector_reset() {
        let mut cd = CycleDetector::new(3);
        cd.record("a", "");
        cd.record("a", "");
        cd.record("a", "");
        assert!(cd.is_cycle());
        cd.reset();
        assert!(!cd.is_cycle());
    }

    #[test]
    fn test_tool_result_struct() {
        let result = super::ToolResult {
            tool_call_id: "call_1".into(),
            content: "result".into(),
        };
        assert_eq!(result.tool_call_id, "call_1");
        assert_eq!(result.content, "result");
    }

    #[tokio::test]
    async fn test_orchestrator_run_cycle() {
        let registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool));
        let orch = ToolOrchestrator::new(registry);

        let mut bundle = crate::request::RequestBundle::new(
            crate::config::ModelConfig {
                name: "test".into(), provider: "nvidia".into(),
                alias: "Test".into(), temperature: 0.7, max_tokens: 4096,
                kwargs: None, body_extra: None, reasoning_budget: None,
            }
        );

        let calls = vec![ToolCallDelta {
            id: Some("call_1".into()),
            call_type: Some("function".into()),
            function: Some(crate::provider::ToolCallFunc {
                name: Some("echo".into()),
                arguments: Some(r#"{"text":"hello"}"#.into()),
            }),
        }];

        let cycle = orch.run_cycle(&mut bundle, &calls, None).await;
        assert!(matches!(cycle, CycleResult::Continue));
        assert_eq!(bundle.messages.len(), 1);
        assert_eq!(bundle.messages[0].role, "tool");
        assert_eq!(bundle.messages[0].content, "hello");
    }
}
