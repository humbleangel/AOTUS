<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import ChatForm from './ChatForm.svelte'
  import ChatMessages from './ChatMessages.svelte'
  import { InteractionRuntime } from '$lib/runtime.svelte'
  import { getSessions, createSession, deleteSession, sendStream, getMessages } from '$lib/chat-service'
  import type { ChatEvent, Session } from '$lib/types'

  let runtime = $state(new InteractionRuntime())
  let sessions: Session[] = $state([])
  let currentSessionId: string = $state('')
  let model = $state('nvidia/llama-3.1-8b-instruct')

  async function loadSessions() {
    sessions = await getSessions()
  }

  async function handleNewSession() {
    const session = await createSession('New chat')
    sessions = [session, ...sessions]
    currentSessionId = session.id
    loadHistory(session.id)
  }

  async function handleSelectSession(id: string) {
    currentSessionId = id
    loadHistory(id)
  }

  async function handleDeleteSession(id: string) {
    await deleteSession(id)
    sessions = sessions.filter(s => s.id !== id)
    if (currentSessionId === id) {
      currentSessionId = ''
      runtime.reset()
    }
  }

  async function loadHistory(sessionId: string) {
    const msgs = await getMessages(sessionId)
    runtime.reset()
    runtime.load(sessionId, msgs)
  }

  async function handleSend(message: string) {
    if (!currentSessionId) {
      await handleNewSession()
    }
    const id = runtime.start(currentSessionId, message, model)
    try {
      await sendStream(message, currentSessionId, model, (event: ChatEvent) => {
        handleEvent(id, event)
      })
    } catch (e) {
      runtime.setError(id, String(e))
    }
  }

  function handleEvent(interactionId: string, event: ChatEvent) {
    switch (event.type) {
      case 'Token':
        runtime.appendToAnswer(interactionId, event.data.text)
        break
      case 'ToolCalls':
        runtime.setToolCalls(interactionId, event.data.calls)
        break
      case 'ToolResult':
        runtime.addToolResult(interactionId, event.data.call_id, event.data.output)
        break
      case 'Done':
        runtime.setDone(interactionId, undefined, event.data.ttft_ms, event.data.duration_ms)
        break
      case 'Error':
        runtime.setError(interactionId, event.data.message)
        break
    }
  }

  function handleCancel() {
    runtime.cancel()
  }

  $effect(() => { loadSessions() })
</script>

<div class="app-shell">
  <Sidebar
    {sessions}
    {currentSessionId}
    onNewSession={handleNewSession}
    onSelectSession={handleSelectSession}
    onDeleteSession={handleDeleteSession}
  />
  <main class="main-content">
    <header class="header">
      <h1>AOTUS</h1>
    </header>
    <ChatMessages interactions={runtime.interactions} {runtime} />
    <ChatForm
      onSend={handleSend}
      onCancel={handleCancel}
      disabled={runtime.activeId !== null}
    />
  </main>
</div>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .header {
    padding: 12px 20px;
    border-bottom: 1px solid #1e2329;
    flex-shrink: 0;
  }
  .header h1 {
    font-size: 14px;
    font-weight: 600;
    color: #e2e8f0;
    margin: 0;
  }
</style>
