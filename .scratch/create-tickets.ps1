$root = "C:\Users\SSDGO\Desktop\AOTUS"
Set-Location $root

$tickets = @(
  @{num=1; title="01 — Scaffold: npm create tauri-app (Svelte 5, TS)"; body="**What to build:** Run `npm create tauri-app` to scaffold the project. Select Svelte 5 + TypeScript template. Verify `npm run tauri dev` opens a window.

**Blocked by:** None — can start immediately.

- [ ] `npm create tauri-app` with Svelte 5 + TS
- [ ] `npm run tauri dev` opens empty window"}

  @{num=2; title="02 — Rust module skeleton"; body="**What to build:** Create the Rust module structure under src-tauri/src/: lib.rs, main.rs, and empty module files for config.rs, provider.rs, request.rs, response.rs, stream.rs, chat_engine.rs, tool_orchestrator.rs, db/mod.rs.

**Blocked by:** #1

- [ ] All module files created
- [ ] cargo build compiles"}

  @{num=3; title="03 — AppConfig struct + JSON deserialization"; body="**What to build:** Define AppConfig struct in config.rs with fields: models, api_keys, default_provider, default_model, stream_timeout_secs. Define ModelConfig. Implement Deserialize + Serialize.

**Blocked by:** #2

- [ ] AppConfig struct with all fields
- [ ] Deserialize from JSON file"}

  @{num=4; title="04 — config.json with 1 provider (NVIDIA)"; body="**What to build:** Create config.json with one NVIDIA provider entry and one model. Placeholder API key. Config loader reads on startup. Default config created if missing.

**Blocked by:** #3

- [ ] config.json with 1 provider, 1 model
- [ ] Config loader creates default on missing"}

  @{num=5; title="05 — AppConfig parse test"; body="**What to build:** Unit tests for AppConfig: valid JSON parses, missing fields error, unknown fields rejected. Use JSON fixtures.

**Blocked by:** #4

- [ ] Valid config test
- [ ] Malformed JSON error test
- [ ] Missing fields error test"}

  @{num=6; title="06 — Provider enum + ProviderAdapter trait"; body="**What to build:** Provider enum (Nvidia, OpenRouter, Zen, Go). ProviderAdapter trait: endpoint(), build_body(), parse_response(). Answer struct with content, model, usage.

**Blocked by:** #4

- [ ] Provider enum with 4 variants
- [ ] ProviderAdapter trait
- [ ] Answer struct"}

  @{num=7; title="07 — NVIDIA adapter impl"; body="**What to build:** Implement ProviderAdapter for NVIDIA. Endpoint, request body builder, batch response parser, stream SSE line parser, usage extraction.

**Blocked by:** #6

- [ ] NVIDIA endpoint
- [ ] Request body builder
- [ ] Batch + stream response parser
- [ ] Usage extraction"}

  @{num=8; title="08 — ChatEvent enum (tagged union)"; body="**What to build:** ChatEvent enum with serde(tag = type, content = data): Token, Done, Error, ToolCalls, ToolResult. All variants implement Serialize for Tauri IPC Channel.

**Blocked by:** #4

- [ ] ChatEvent enum with all variants
- [ ] Serialize derives
- [ ] Round-trip serialization test"}

  @{num=9; title="09 — SSE line parser (stream.rs)"; body="**What to build:** Parse raw SSE bytes into structured lines. Handle multi-line data, [DONE], keep-alive comments, malformed lines. parse_sse_line() -> Option of parsed JSON.

**Blocked by:** #8

- [ ] Basic SSE line parser
- [ ] Multi-line data support
- [ ] [DONE] signal
- [ ] Unit tests"}

  @{num=10; title="10 — Response parser for NVIDIA format"; body="**What to build:** Parse NVIDIA chat completion JSON. Extract content, reasoning, model, usage. Handle batch and stream delta.

**Blocked by:** #7

- [ ] Batch response parser
- [ ] Stream delta parser
- [ ] Usage extraction"}

  @{num=11; title="11 — Chat request builder (request.rs)"; body="**What to build:** Module that builds HTTP request JSON body. Takes ChatParams and ProviderAdapter. Returns JSON Value. Includes conversation history and tool definitions.

**Blocked by:** #6

- [ ] build_request(params, adapter) -> Value
- [ ] Includes history + tool defs
- [ ] Unit test with mock adapter"}

  @{num=12; title="12 — chat_engine select! loop"; body="**What to build:** SSE streaming select! loop. Three branches: SSE bytes -> ChatEvent, cancel -> Cancelled, 120s timeout -> Timeout. Returns StreamResult.

**Blocked by:** #9, #10, #11

- [ ] select! loop with 3 branches
- [ ] SSE -> ChatEvent emission
- [ ] CancellationToken support
- [ ] 120s timeout"}

  @{num=13; title="13 — 120s timeout + CancellationToken integration"; body="**What to build:** Wire timeout and cancellation into the chat_engine. CancellationToken created per-request, stored in a HashMap by request_id. cancel_request drops the token.

**Blocked by:** #12

- [ ] Per-request CancellationToken
- [ ] cancel_request wiring
- [ ] Test: cancel mid-stream"}

  @{num=14; title="14 — MessageDb trait (insert, get, delete)"; body="**What to build:** MessageDb trait with Send + Sync. Methods: insert_message, get_messages, delete_message. NewMessage and MessageRow structs.

**Blocked by:** #4

- [ ] MessageDb trait definition
- [ ] NewMessage, MessageRow structs
- [ ] Trait compiles"}

  @{num=15; title="15 — InMemoryMessageDb impl"; body="**What to build:** Implement MessageDb with HashMap backend. Thread-safe via Mutex. Used for unit tests.

**Blocked by:** #14

- [ ] InMemoryMessageDb struct
- [ ] All trait methods implemented
- [ ] Thread-safe"}

  @{num=16; title="16 — SQLite schema: sessions table"; body="**What to build:** CREATE TABLE sessions (id TEXT PK, name TEXT, created_at TEXT). Migration system with schema_version table. rusqlite connection.

**Blocked by:** #14

- [ ] sessions table
- [ ] schema_version table
- [ ] Migration runner"}

  @{num=17; title="17 — SQLite schema: messages table"; body="**What to build:** CREATE TABLE messages (id INTEGER PK, session_id FK, role TEXT, content TEXT, model TEXT, tool_calls TEXT, created_at TEXT). Foreign key to sessions.

**Blocked by:** #14

- [ ] messages table with tool_calls column
- [ ] Foreign key to sessions
- [ ] Migration applied in order"}

  @{num=18; title="18 — Migration system"; body="**What to build:** Sequential migration runner. Migration files numbered (001_create_sessions.sql, 002_create_messages.sql). Applied in order. Schema version tracked. Down migrations not required.

**Blocked by:** #16, #17

- [ ] Migration file loader
- [ ] Sequential application
- [ ] Idempotent (skip applied)"}

  @{num=19; title="19 — AppState setup in lib.rs"; body="**What to build:** AppState struct holding AppConfig, MessageDb (Arc<Mutex<dyn MessageDb>>), CancellationTokens map. Constructed in lib.rs setup(). Managed via tauri::State.

**Blocked by:** #4, #18

- [ ] AppState struct
- [ ] lib.rs setup() wiring
- [ ] Tauri State management"}

  @{num=20; title="20 — chat_stream command"; body="**What to build:** Tauri command chat_stream: (params, channel) -> Result. Resolves provider via adapter, builds request via request.rs, runs chat_engine stream, returns StreamResult. Wires everything end to end.

**Blocked by:** #12, #19

- [ ] chat_stream command
- [ ] Provider resolution
- [ ] Engine invocation
- [ ] Result returned"}

  @{num=21; title="21 — cancel_request command"; body="**What to build:** cancel_request command. Looks up CancellationToken in AppState by request_id. Drops it. Engine detects cancellation, stops streaming.

**Blocked by:** #13, #19

- [ ] cancel_request command
- [ ] Token lookup and drop
- [ ] Side-effect: stream stops"}

  @{num=22; title="22 — get_sessions + create_session commands"; body="**What to build:** get_sessions returns all sessions. create_session takes name, inserts, returns session. Both use MessageDb.

**Blocked by:** #18, #19

- [ ] get_sessions command
- [ ] create_session command
- [ ] Integration test"}

  @{num=23; title="23 — delete_session + get_messages commands"; body="**What to build:** delete_session removes session and its messages. get_messages returns messages for a session ordered by id.

**Blocked by:** #18, #19

- [ ] delete_session command
- [ ] get_messages command
- [ ] Cascade delete test"}

  @{num=24; title="24 — Error handling (ChatError)"; body="**What to build:** ChatError enum with variants: Network, Parse, Auth, Timeout, Db, Cancel, Tool, Unknown. Implement Into<InvokeError> for Tauri. All commands wrap errors.

**Blocked by:** #20

- [ ] ChatError enum
- [ ] Into<InvokeError> impl
- [ ] All commands use Result<_, ChatError>"}

  @{num=25; title="25 — Interaction type definitions (TypeScript)"; body="**What to build:** TypeScript types matching Rust structs: Interaction, Message, ChatEvent, ChatParams, ChatResult, StreamResult. Generated via ts-rs or hand-written. Used by InteractionRuntime.

**Blocked by:** #8

- [ ] Interaction type
- [ ] ChatEvent type
- [ ] ChatParams type
- [ ] Matches Rust serialization"}

  @{num=26; title="26 — InteractionRuntime class with \$state"; body="**What to build:** InteractionRuntime class with Svelte 5 \$state. Fields: interactions, activeId, error. Methods: start(), cancel(), appendToAnswer(), setToolCalls(), setDone(), setError(), load(sessionId).

**Blocked by:** #25

- [ ] InteractionRuntime class
- [ ] \$state interactions array
- [ ] All methods defined"}

  @{num=27; title="27 — start() method — creates pending interaction"; body="**What to build:** Runtime.start() creates a new Interaction with status=pending, adds to array, calls ChatService.sendStream(). Returns immediately (async). Interaction visible in UI immediately.

**Blocked by:** #26

- [ ] start() creates pending interaction
- [ ] Calls ChatService.sendStream()
- [ ] Interaction visible immediately"}

  @{num=28; title="28 — appendToAnswer() + setToolCalls()"; body="**What to build:** appendToAnswer(id, text) appends to the interaction's answer during streaming. setToolCalls(id, calls) sets tool calls and switches status to ToolExecuting. Updates \$state.

**Blocked by:** #26

- [ ] appendToAnswer works during streaming
- [ ] setToolCalls switches to ToolExecuting
- [ ] UI re-renders on call"}

  @{num=29; title="29 — setDone() + setError()"; body="**What to build:** setDone(id) sets status=done, persists to DB. setError(id, msg) sets status=error, persists. No summary generation — just persistence.

**Blocked by:** #26

- [ ] setDone persists + updates state
- [ ] setError persists + updates state
- [ ] Error state preserved on reload"}

  @{num=30; title="30 — loadInteractions(sessionId)"; body="**What to build:** loadInteractions() called on init. Invokes get_messages backend command for the last active session. Populates interactions array. Falls back to new session if none.

**Blocked by:** #26, #22

- [ ] Loads messages from backend
- [ ] Populates interactions array
- [ ] Creates new session if none"}

  @{num=31; title="31 — ChatService.ts (sendStream, sendBatch, cancel)"; body="**What to build:** ChatService with 3 exported functions. sendStream: creates Tauri Channel, invokes chat_stream, forwards events (Token, Done, Error, ToolCalls) to Runtime callback. sendBatch: invokes chat_batch. cancel: invokes cancel_request.

**Blocked by:** #20, #26

- [ ] sendStream with Channel event forwarding
- [ ] sendBatch command
- [ ] cancel command
- [ ] Timeout: also calls cancel_request"}

  @{num=32; title="32 — App.svelte shell + view routing"; body="**What to build:** App.svelte shell with header (logo, version) and main content area. Simple view routing: chat view. Sidebar slot on the left. CSS variables for dark theme.

**Blocked by:** #31

- [ ] App.svelte with header + main
- [ ] View routing (chat only)
- [ ] Dark theme CSS"}

  @{num=33; title="33 — Sidebar.svelte — session list"; body="**What to build:** Sidebar component. Top section: session list (name, created_at). Click to switch session. New session button. Bottom: compact icon buttons for views.

**Blocked by:** #32, #22

- [ ] Session list from backend
- [ ] Click to switch session
- [ ] New session button"}

  @{num=34; title="34 — ChatForm.svelte — input + send button"; body="**What to build:** Chat input form. Textarea for prompt. Send button. Model selector dropdown (reads from config). SmartContext toggle (persisted to localStorage). Enter to send, Shift+Enter for newline.

**Blocked by:** #32

- [ ] Textarea input
- [ ] Send button
- [ ] Model selector dropdown"}

  @{num=35; title="35 — ChatMessages.svelte — render interactions"; body="**What to build:** ChatMessages component. Renders all interactions in order. Auto-scrolls to bottom on new content. Scroll-anchor for streaming (don't force scroll up when reading history).

**Blocked by:** #32

- [ ] Render interaction list
- [ ] Auto-scroll on new tokens
- [ ] Scroll-anchor behavior"}

  @{num=36; title="36 — Interaction.svelte — chat bubble"; body="**What to build:** Interaction component. Renders one chat bubble: user prompt + assistant response. Streaming: shows content as it arrives. Reasoning section (collapsible). Tool calls section (when status=ToolExecuting).

**Blocked by:** #35

- [ ] User prompt display
- [ ] Streaming answer display
- [ ] Reasoning section (collapsible)
- [ ] Tool calls section"}

  @{num=37; title="37 — Auto-scroll to bottom on new tokens"; body="**What to build:** Auto-scroll behavior: scroll to bottom when new tokens arrive and user hasn't scrolled up. Scroll anchor at bottom. User scrolled up? Don't force scroll. threshold = 50px.

**Blocked by:** #36

- [ ] Auto-scroll during streaming
- [ ] Respect user scroll position
- [ ] 50px threshold"}

  @{num=38; title="38 — Wire ChatForm -> ChatService -> Runtime -> UI"; body="**What to build:** Final wiring: ChatForm.send() -> ChatService.sendStream(rt, params) -> Tauri invoke -> Runtime.appendToAnswer() -> UI re-render. End-to-end demoable: type, see stream, done.

**Blocked by:** #34, #35, #31

- [ ] End-to-end send cycle works
- [ ] Streaming visible in UI
- [ ] Done state displayed"}

  @{num=39; title="39 — Tool trait + ToolRegistry"; body="**What to build:** Tool trait: name(), description(), parameters() returning JSON Schema, execute(args, ctx) -> Result<String>. ToolRegistry: register, get, list_all. Thread-safe (Arc<RwLock>).

**Blocked by:** #4

- [ ] Tool trait definition
- [ ] ToolRegistry with register/get/list
- [ ] Thread-safe"}

  @{num=40; title="40 — CycleDetector"; body="**What to build:** CycleDetector records last 3 consecutive (name, args) pairs. check() returns true if same call 3 times in a row. reset() clears history.

**Blocked by:** #4

- [ ] Records last 3 calls
- [ ] Returns true on repeat x3
- [ ] reset() method"}

  @{num=41; title="41 — ToolOrchestrator.run_cycle()"; body="**What to build:** ToolOrchestrator.run_cycle(): parse tool_calls from LLM response -> execute each via ToolRegistry -> collect results -> inject into bundle.messages -> build follow-up request -> send HTTP -> parse response -> return CycleResult. Max turns limit.

**Blocked by:** #39, #40

- [ ] Parse tool_calls from response
- [ ] Execute tools via registry
- [ ] Inject results into bundle
- [ ] Follow-up LLM request"}

  @{num=42; title="42 — Tool result injection into bundle.messages"; body="**What to build:** When tools execute, results must be injected into RequestBundle.messages as tool-role messages. Without this the LLM never sees tool output. Each result gets {role: tool, tool_call_id, content}.

**Blocked by:** #41

- [ ] Results as tool-role messages
- [ ] tool_call_id linking
- [ ] Content from tool output"}

  @{num=43; title="43 — Inline tool execution in chat_engine"; body="**What to build:** When chat_engine detects tool_calls in the SSE stream, pausing token emission, call ToolOrchestrator.run_cycle(). Emit ToolCalls ChatEvent before cycle, resume streaming on follow-up response. Single loop.

**Blocked by:** #12, #42

- [ ] Detect tool_calls in stream
- [ ] Call ToolOrchestrator inline
- [ ] Emit ToolCalls/ToolResult events
- [ ] Resume streaming"}

  @{num=44; title="44 — ToolExecuting state in InteractionRuntime"; body="**What to build:** Interaction status includes ToolExecuting. setToolCalls(id, calls) sets this status. UI shows different state. setDone() called when final answer arrives after tool cycle.

**Blocked by:** #29, #43

- [ ] ToolExecuting status variant
- [ ] setToolCalls switches status
- [ ] setDone from final answer"}

  @{num=45; title="45 — ToolCallCard.svelte — name, status, expand"; body="**What to build:** ToolCallCard component showing: tool name, status indicator (running/done/error), expandable result view. Collapsed by default.

**Blocked by:** #44

- [ ] Tool name display
- [ ] Status indicator
- [ ] Expandable results"}

  @{num=46; title="46 — Tool status indicators"; body="**What to build:** Visual indicators for tool execution status: spinner (running), green checkmark (done), red X (error). Animated transitions between states.

**Blocked by:** #45

- [ ] Spinner animation
- [ ] Checkmark on success
- [ ] X on error
- [ ] Smooth transitions"}

  @{num=47; title="47 — Expandable tool result display"; body="**What to build:** Tool results displayed in an expandable section. Click to expand/collapse. Code blocks formatted. Error results highlighted red.

**Blocked by:** #45

- [ ] Expand/collapse toggle
- [ ] Code formatting
- [ ] Error highlight"}

  @{num=48; title="48 — OpenRouter adapter impl"; body="**What to build:** Implement ProviderAdapter for OpenRouter. Differs from NVIDIA: different base URL, API key header (HTTP_REFERER optional), response format is OpenAI-compatible (same as NVIDIA). body_extra support for OpenRouter-specific params.

**Blocked by:** #6

- [ ] OpenRouter endpoint
- [ ] Request body builder (with body_extra merge)
- [ ] Response parser (OpenAI-compatible)
- [ ] Usage extraction"}

  @{num=49; title="49 — Zen adapter impl"; body="**What to build:** Implement ProviderAdapter for Zen. Different base URL, auth mechanism, request/response format. May need custom body structure.

**Blocked by:** #6

- [ ] Zen endpoint
- [ ] Auth mechanism
- [ ] Request body builder
- [ ] Response parser"}

  @{num=50; title="50 — GO adapter impl"; body="**What to build:** Implement ProviderAdapter for GO (subscription). Different base URL, auth, request/response format.

**Blocked by:** #6

- [ ] GO endpoint
- [ ] Auth mechanism
- [ ] Request body builder
- [ ] Response parser"}

  @{num=51; title="51 — Provider extras per model (kwargs, body_extra, rb)"; body="**What to build:** Per-provider extra fields in ModelConfig: chat_template_kwargs (NVIDIA), body_extra (OpenRouter), reasoning_budget (NVIDIA except some models). Applied in each adapter's build_body.

**Blocked by:** #48, #49, #50

- [ ] kwargs support in NVIDIA adapter
- [ ] body_extra in OpenRouter adapter
- [ ] reasoning_budget conditional
- [ ] Tests for each"}

  @{num=52; title="52 — Model selector dropdown in ChatForm"; body="**What to build:** Dropdown in ChatForm showing all models from AppConfig. Grouped by provider. Selected model sent with chat request. Current selection persisted in localStorage.

**Blocked by:** #51, #34

- [ ] Dropdown with all models
- [ ] Grouped by provider
- [ ] Selection persisted"}

  @{num=53; title="53 — Session rename + search in sidebar"; body="**What to build:** Rename session: inline edit on click. Search: filter session list by name. Delete: confirmation dialog before delete.

**Blocked by:** #33

- [ ] Inline rename
- [ ] Search/filter
- [ ] Delete with confirmation"}

  @{num=54; title="54 — Auto-restore last session on launch"; body="**What to build:** On app launch, load last active session (stored in localStorage). If none, create new session with default name. Populate interactions from backend.

**Blocked by:** #30, #53

- [ ] Last session stored in localStorage
- [ ] Restore on launch
- [ ] Fallback to new session"}

  @{num=55; title="55 — Multi-provider tests"; body="**What to build:** Integration tests for all 4 providers: each adapter builds correct request body, parses mock responses correctly. Use recorded fixtures for each provider format.

**Blocked by:** #48, #49, #50, #51

- [ ] NVIDIA fixture test
- [ ] OpenRouter fixture test
- [ ] Zen fixture test
- [ ] GO fixture test"}
)

$mapping = @{}
foreach ($t in $tickets) {
  $bodyFile = ".scratch\ticket$($t.num).md"
  $t.body | Set-Content -Path $bodyFile -Encoding UTF8
  $url = gh issue create --title $t.title --body-file $bodyFile --label ready-for-agent 2>&1
  if ($url -match "github.com/(\d+)$") {
    $issueNum = $Matches[1]
    $mapping[$t.num] = $issueNum
    Write-Host "Ticket $($t.num) -> Issue #$issueNum"
  } else {
    Write-Host "FAILED: Ticket $($t.num) - $url"
  }
  Remove-Item $bodyFile
}

$mapping | ConvertTo-Json | Set-Content -Path ".scratch\ticket-issue-map.json" -Encoding UTF8
Write-Host "DONE. Mapping saved to .scratch/ticket-issue-map.json"
