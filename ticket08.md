**What to build:** Define `ChatEvent` enum as a tagged union with `#[serde(tag = "type", content = "data")]`: `Token { text }`, `Done { answer, duration_ms, ttft_ms }`, `Error { message }`, `ToolCalls { calls }`, `ToolResult { call_id, output }`. All types must implement Serialize for Tauri IPC Channel.

**Blocked by:** #4

**Status:** ready-for-agent

- [ ] ChatEvent enum with all variants
- [ ] Serialize derives
- [ ] Unit test: serialize/deserialize round-trip
