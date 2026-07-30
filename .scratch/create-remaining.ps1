$root = "C:\Users\SSDGO\Desktop\AOTUS"
Set-Location $root

$tickets = @(
  ,@(13, '13 — 120s timeout + CancellationToken integration', @'
**What to build:** Wire timeout and cancellation into the chat_engine. CancellationToken created per-request, stored in a HashMap by request_id. cancel_request drops the token.

**Blocked by:** #13

- [ ] Per-request CancellationToken
- [ ] cancel_request wiring
- [ ] Test: cancel mid-stream
'@)

  ,@(14, '14 — MessageDb trait (insert, get, delete)', @'
**What to build:** MessageDb trait with Send + Sync. Methods: insert_message, get_messages, delete_message. NewMessage and MessageRow structs.

**Blocked by:** #4

- [ ] MessageDb trait definition
- [ ] NewMessage, MessageRow structs
- [ ] Trait compiles
'@)

  ,@(15, '15 — InMemoryMessageDb impl', @'
**What to build:** Implement MessageDb with HashMap backend. Thread-safe via Mutex. Used for unit tests.

**Blocked by:** #14

- [ ] InMemoryMessageDb struct
- [ ] All trait methods implemented
- [ ] Thread-safe
'@)

  ,@(16, '16 — SQLite schema: sessions table', @'
**What to build:** CREATE TABLE sessions (id TEXT PK, name TEXT, created_at TEXT). Migration system with schema_version table. rusqlite connection.

**Blocked by:** #14

- [ ] sessions table
- [ ] schema_version table
- [ ] Migration runner
'@)

  ,@(17, '17 — SQLite schema: messages table', @'
**What to build:** CREATE TABLE messages (id INTEGER PK, session_id FK, role TEXT, content TEXT, model TEXT, tool_calls TEXT, created_at TEXT). Foreign key to sessions.

**Blocked by:** #14

- [ ] messages table with tool_calls column
- [ ] Foreign key to sessions
- [ ] Migration applied in order
'@)

  ,@(18, '18 — Migration system', @'
**What to build:** Sequential migration runner. Migration files numbered (001_create_sessions.sql, 002_create_messages.sql). Applied in order. Schema version tracked.

**Blocked by:** #16, #17

- [ ] Migration file loader
- [ ] Sequential application
- [ ] Idempotent
'@)

  ,@(19, '19 — AppState setup in lib.rs', @'
**What to build:** AppState struct holding AppConfig, MessageDb, CancellationTokens map. Constructed in lib.rs setup(). Managed via tauri::State.

**Blocked by:** #4, #18

- [ ] AppState struct
- [ ] lib.rs wiring
- [ ] Tauri State management
'@)

  ,@(20, '20 — chat_stream command', @'
**What to build:** Tauri command chat_stream. Resolves provider via adapter, builds request, runs chat_engine stream, returns StreamResult.

**Blocked by:** #13, #19

- [ ] chat_stream command
- [ ] Provider resolution
- [ ] Engine invocation
'@)

  ,@(21, '21 — cancel_request command', @'
**What to build:** cancel_request command. Looks up CancellationToken by request_id and drops it.

**Blocked by:** #13, #19

- [ ] cancel_request command
- [ ] Token lookup and drop
'@)

  ,@(22, '22 — get_sessions + create_session commands', @'
**What to build:** get_sessions returns all sessions. create_session takes name, inserts, returns session.

**Blocked by:** #18, #19

- [ ] get_sessions command
- [ ] create_session command
'@)

  ,@(23, '23 — delete_session + get_messages commands', @'
**What to build:** delete_session removes session and its messages. get_messages returns messages for a session.

**Blocked by:** #18, #19

- [ ] delete_session command
- [ ] get_messages command
'@)

  ,@(24, '24 — Error handling (ChatError)', @'
**What to build:** ChatError enum: Network, Parse, Auth, Timeout, Db, Cancel, Tool, Unknown. Implement Into<InvokeError>. All commands use Result<_, ChatError>.

**Blocked by:** #20

- [ ] ChatError enum
- [ ] Into<InvokeError> impl
'@)

  ,@(25, '25 — Interaction type definitions (TypeScript)', @'
**What to build:** TypeScript types: Interaction, Message, ChatEvent, ChatParams, ChatResult. Matches Rust serialization.

**Blocked by:** #10

- [ ] Interaction type
- [ ] ChatEvent type
- [ ] ChatParams type
'@)

  ,@(26, '26 — InteractionRuntime class with $state', @'
**What to build:** InteractionRuntime with Svelte 5 $state. Fields: interactions, activeId, error. Methods: start, cancel, appendToAnswer, setToolCalls, setDone, setError, load.

**Blocked by:** #25

- [ ] InteractionRuntime class
- [ ] $state interactions array
'@)

  ,@(27, '27 — start() method — creates pending interaction', @'
**What to build:** Runtime.start() creates pending Interaction, adds to array, calls ChatService.sendStream(). Async, returns immediately.

**Blocked by:** #26

- [ ] start() creates pending
- [ ] Calls ChatService.sendStream()
'@)

  ,@(28, '28 — appendToAnswer() + setToolCalls()', @'
**What to build:** appendToAnswer appends to answer during streaming. setToolCalls sets ToolExecuting status. Updates $state.

**Blocked by:** #26

- [ ] appendToAnswer works
- [ ] setToolCalls sets ToolExecuting
'@)

  ,@(29, '29 — setDone() + setError()', @'
**What to build:** setDone sets status=done, persists to DB. setError sets status=error, persists.

**Blocked by:** #26

- [ ] setDone persists
- [ ] setError persists
'@)

  ,@(30, '30 — loadInteractions(sessionId)', @'
**What to build:** loadInteractions calls get_messages backend, populates interactions array. Falls back to new session if none.

**Blocked by:** #26, #22

- [ ] Loads from backend
- [ ] Fallback to new session
'@)

  ,@(31, '31 — ChatService.ts (sendStream, sendBatch, cancel)', @'
**What to build:** 3 exported functions. sendStream: creates Channel, invokes chat_stream, forwards events. sendBatch: invokes chat_batch. cancel: invokes cancel_request.

**Blocked by:** #20, #26

- [ ] sendStream with event forwarding
- [ ] sendBatch command
- [ ] cancel command
'@)

  ,@(32, '32 — App.svelte shell + view routing', @'
**What to build:** App.svelte with header and main. View routing (chat view). Sidebar slot. Dark theme CSS.

**Blocked by:** #31

- [ ] App.svelte shell
- [ ] View routing
- [ ] Dark theme
'@)

  ,@(33, '33 — Sidebar.svelte — session list', @'
**What to build:** Sidebar with session list. Click to switch. New session button. View icons at bottom.

**Blocked by:** #32, #22

- [ ] Session list
- [ ] Click to switch
- [ ] New session button
'@)

  ,@(34, '34 — ChatForm.svelte — input + send button', @'
**What to build:** Textarea, send button, model selector dropdown. Enter to send, Shift+Enter newline.

**Blocked by:** #32

- [ ] Textarea + send button
- [ ] Model selector dropdown
'@)

  ,@(35, '35 — ChatMessages.svelte — render interactions', @'
**What to build:** Renders all interactions in order. Auto-scroll on new content. Scroll-anchor for streaming.

**Blocked by:** #32

- [ ] Render interaction list
- [ ] Auto-scroll behavior
'@)

  ,@(36, '36 — Interaction.svelte — chat bubble', @'
**What to build:** One chat bubble: user prompt + assistant response. Streaming display. Collapsible reasoning. Tool calls section.

**Blocked by:** #35

- [ ] Prompt + response display
- [ ] Streaming content
- [ ] Reasoning + tool sections
'@)

  ,@(37, '37 — Auto-scroll to bottom on new tokens', @'
**What to build:** Scroll to bottom on new tokens if user hasn't scrolled up. 50px threshold. Respects manual scroll position.

**Blocked by:** #36

- [ ] Auto-scroll during streaming
- [ ] Respects manual scroll
'@)

  ,@(38, '38 — Wire ChatForm -> ChatService -> Runtime -> UI', @'
**What to build:** End-to-end: ChatForm.send -> ChatService.sendStream -> Tauri -> Runtime.appendToAnswer -> UI re-render.

**Blocked by:** #34, #35, #31

- [ ] End-to-end send cycle works
- [ ] Streaming visible in UI
'@)

  ,@(39, '39 — Tool trait + ToolRegistry', @'
**What to build:** Tool trait: name, description, parameters, execute. ToolRegistry: register, get, list_all. Thread-safe.

**Blocked by:** #4

- [ ] Tool trait definition
- [ ] ToolRegistry methods
'@)

  ,@(40, '40 — CycleDetector', @'
**What to build:** Records last 3 (name, args) pairs. Returns true if same call 3 times in a row.

**Blocked by:** #4

- [ ] Records last 3 calls
- [ ] Cycle detection
'@)

  ,@(41, '41 — ToolOrchestrator.run_cycle()', @'
**What to build:** Parse tool_calls -> execute via ToolRegistry -> inject results -> build follow-up request -> send HTTP -> parse response -> CycleResult.

**Blocked by:** #39, #40

- [ ] Parse tool_calls
- [ ] Execute tools
- [ ] Inject results
- [ ] Follow-up request
'@)

  ,@(42, '42 — Tool result injection into bundle.messages', @'
**What to build:** Tool results as tool-role messages with tool_call_id. LLM sees tool output.

**Blocked by:** #41

- [ ] Results as tool-role messages
- [ ] tool_call_id linking
'@)

  ,@(43, '43 — Inline tool execution in chat_engine', @'
**What to build:** When chat_engine detects tool_calls, pause streaming, call ToolOrchestrator.run_cycle(), emit events, resume.

**Blocked by:** #13, #42

- [ ] Detect tool_calls in stream
- [ ] Inline ToolOrchestrator call
- [ ] Resume streaming
'@)

  ,@(44, '44 — ToolExecuting state in InteractionRuntime', @'
**What to build:** Interaction status includes ToolExecuting. setToolCalls sets it. setDone from final answer.

**Blocked by:** #29, #43

- [ ] ToolExecuting status variant
- [ ] setToolCalls -> ToolExecuting
'@)

  ,@(45, '45 — ToolCallCard.svelte', @'
**What to build:** Tool name, status indicator, expandable result view. Collapsed by default.

**Blocked by:** #44

- [ ] Tool name + status
- [ ] Expandable results
'@)

  ,@(46, '46 — Tool status indicators', @'
**What to build:** Spinner (running), checkmark (done), X (error). Animated.

**Blocked by:** #45

- [ ] Spinner animation
- [ ] Checkmark on success
- [ ] X on error
'@)

  ,@(47, '47 — Expandable tool result display', @'
**What to build:** Expand/collapse toggle. Code formatting. Error results highlighted red.

**Blocked by:** #45

- [ ] Expand/collapse
- [ ] Code formatting
'@)

  ,@(48, '48 — OpenRouter adapter impl', @'
**What to build:** ProviderAdapter for OpenRouter. OpenAI-compatible format. body_extra support.

**Blocked by:** #6

- [ ] OpenRouter endpoint
- [ ] Request/response handling
- [ ] body_extra support
'@)

  ,@(49, '49 — Zen adapter impl', @'
**What to build:** ProviderAdapter for Zen. Custom auth, request/response format.

**Blocked by:** #6

- [ ] Zen endpoint
- [ ] Auth mechanism
- [ ] Request/response handling
'@)

  ,@(50, '50 — GO (subscription) adapter impl', @'
**What to build:** ProviderAdapter for GO. Custom endpoint, auth, format.

**Blocked by:** #6

- [ ] GO endpoint
- [ ] Auth mechanism
- [ ] Request/response handling
'@)

  ,@(51, '51 — Provider extras per model', @'
**What to build:** kwargs (NVIDIA), body_extra (OpenRouter), reasoning_budget (conditional). Applied in each adapter.

**Blocked by:** #48, #49, #50

- [ ] kwargs support
- [ ] body_extra support
- [ ] reasoning_budget conditional
'@)

  ,@(52, '52 — Model selector dropdown in ChatForm', @'
**What to build:** Dropdown with all models from AppConfig. Grouped by provider. Selection persisted in localStorage.

**Blocked by:** #51, #34

- [ ] Dropdown with all models
- [ ] Grouped by provider
- [ ] Persisted selection
'@)

  ,@(53, '53 — Session rename + search in sidebar', @'
**What to build:** Inline rename. Search/filter list. Delete with confirmation.

**Blocked by:** #33

- [ ] Inline rename
- [ ] Search/filter
- [ ] Delete confirmation
'@)

  ,@(54, '54 — Auto-restore last session on launch', @'
**What to build:** Load last active session on launch (localStorage). Fallback to new session.

**Blocked by:** #30, #53

- [ ] Last session stored
- [ ] Restore on launch
'@)

  ,@(55, '55 — Multi-provider integration tests', @'
**What to build:** Integration tests for all 4 providers. Mock/fixture responses for each format.

**Blocked by:** #48, #49, #50, #51

- [ ] NVIDIA fixture test
- [ ] OpenRouter fixture test
- [ ] Zen fixture test
- [ ] GO fixture test
'@)
)

$mapping = @{}
foreach ($t in $tickets) {
  $num = $t[0]
  $title = $t[1]
  $body = $t[2]
  $bodyFile = ".scratch\ticket-body.md"
  $body | Set-Content -Path $bodyFile -Encoding UTF8
  $url = gh issue create --title "$title" --body-file $bodyFile --label ready-for-agent 2>&1
  if ($url -match "issues/(\d+)$") {
    $issueNum = $Matches[1]
    $mapping[$num] = [int]$issueNum
    Write-Host "Ticket $num -> Issue #$issueNum"
  } else {
    Write-Host "FAILED: Ticket $num - $url"
  }
  if (Test-Path $bodyFile) { Remove-Item $bodyFile -Force }
}
$mapping | ConvertTo-Json | Set-Content -Path ".scratch\ticket-issue-map.json" -Encoding UTF8
Write-Host "DONE. Total created: $($mapping.Count)"
