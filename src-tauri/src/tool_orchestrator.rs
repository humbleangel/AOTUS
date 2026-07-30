use crate::provider::ToolCallDelta;

pub struct ToolOrchestrator;

impl ToolOrchestrator {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_cycle(
        &self,
        _tool_calls: &[ToolCallDelta],
    ) -> Result<Vec<ToolResult>, crate::ChatError> {
        Ok(vec![])
    }
}

pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
}
