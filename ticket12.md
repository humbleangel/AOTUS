**What to build:** Implement the SSE streaming select! loop in chat_engine.rs. Takes a reqwest::Response, Channel<ChatEvent>, and CancellationToken. The loop select!s between: new SSE bytes (parsed via stream.rs -> emit ChatEvent), cancel.cancelled() (return Cancelled), and a 120s timeout (return Timeout). Returns StreamResult { answer: Answer, ttft_ms }. Tool execution happens inline when tool_calls detected.

**Blocked by:** #8, #11, #12

**Status:** ready-for-agent

- [ ] select! loop with 3 branches
- [ ] SSE bytes -> ChatEvent emission
- [ ] CancellationToken support
- [ ] 120s stream timeout
- [ ] On tool_calls: handoff to ToolOrchestrator (stub)

**Blocked by:** #8, #11, #12

**Status:** ready-for-agent
