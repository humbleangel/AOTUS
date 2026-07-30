<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import ChatForm from './ChatForm.svelte'
  import ChatMessages from './ChatMessages.svelte'
  import { InteractionRuntime } from '$lib/interactionRuntime'
  import { sendStream, cancel } from './services/chatService'
  import type { ChatEvent } from '$lib/types'

  let runtime = $state(new InteractionRuntime())
  let currentSessionId: string = $state('')
  let model = $state('nvidia/llama-3.1-8b-instruct')
  let sessions = $state<Array<{ id: string; name: string; created_at: string }>>([])

  function handleModelChange(m: string) {
    model = m
  }

  async function loadSessions() {
    sessions = await runtime.getSessions()
  }

  $effect(() => {
    loadSessions()
  })

  async function handleNewSession() {
    const session = await runtime.createSession('New chat')
    sessions = [session, ...sessions]
    currentSessionId = session.id
    await loadHistory(session.id)
  }

  async function handleSelectSession(id: string) {
    currentSessionId = id
    await loadHistory(id)
  }

  async function handleDeleteSession(id: string) {
    await runtime.deleteSession(id)
    sessions = sessions.filter(s => s.id !== id)
    if (currentSessionId === id) {
      currentSessionId = ''
    }
  }

  async function loadHistory(sessionId: string) {
    await runtime.load(sessionId)
  }

  let currentRequestId = $state<string | null>(null)

  async function handleSend(message: string) {
    if (!currentSessionId) {
      await handleNewSession()
    }
    await runtime.saveUserMessage(currentSessionId, message, model)
    const id = await runtime.start(currentSessionId, message, model)
    const params = { message, sessionId: currentSessionId, modelName: model }
    try {
      currentRequestId = await sendStream(runtime, params, (event: ChatEvent) => {
        handleEvent(id, event)
      })
    } catch (e) {
      runtime.setError(id, String(e), currentSessionId, model)
      if (currentRequestId) {
        cancel(currentRequestId)
      }
    } finally {
      currentRequestId = null
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
      case 'Done':
        runtime.setDone(interactionId, currentSessionId, model)
        break
      case 'Error':
        runtime.setError(interactionId, event.data.message, currentSessionId, model)
        break
    }
  }

  function handleCancel() {
    if (currentRequestId) {
      cancel(currentRequestId)
    }
    runtime.cancel()
  }

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
      selectedModel={model}
      onSend={handleSend}
      onCancel={handleCancel}
      disabled={runtime.activeId !== null}
      onModelChange={handleModelChange}
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
