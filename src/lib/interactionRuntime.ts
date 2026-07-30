import { invoke } from '@tauri-apps/api/core'
import type { Interaction, InteractionStatus, Message, ToolCallDelta, ChatEvent, Session } from './types'

let nextId = 1
function genId(): string {
  return `interaction_${nextId++}_${Date.now()}`
}

export class InteractionRuntime {
  interactions = $state<Interaction[]>([])
  activeId = $state<string | null>(null)
  error = $state<string | null>(null)

  async start(sessionId: string, prompt: string, model: string): Promise<string> {
    const id = genId()
    const interaction: Interaction = {
      id,
      sessionId,
      prompt,
      answer: '',
      reasoning: '',
      status: 'pending',
      model,
      toolCalls: [],
      toolResults: [],
      createdAt: new Date().toISOString(),
    }
    this.interactions.push(interaction)
    this.activeId = id
    this.error = null
    return id
  }

  cancel(): void {
    const current = this.current
    if (current) current.status = 'done'
    this.activeId = null
  }

  get current(): Interaction | undefined {
    return this.interactions.find(i => i.id === this.activeId)
  }

  appendToAnswer(interactionId: string, text: string): void {
    const interaction = this.interactions.find(i => i.id === interactionId)
    if (!interaction) return
    interaction.answer += text
    if (interaction.status === 'pending') {
      interaction.status = 'streaming'
    }
  }

  setToolCalls(interactionId: string, calls: ToolCallDelta[]): void {
    const interaction = this.interactions.find(i => i.id === interactionId)
    if (!interaction) return
    interaction.toolCalls = calls
    interaction.status = 'tool_executing'
  }

  async setDone(interactionId: string, sessionId: string, model: string): Promise<void> {
    const interaction = this.interactions.find(i => i.id === interactionId)
    if (!interaction || interaction.status === 'error') return
    interaction.status = 'done'
    await invoke('save_message', { sessionId, role: 'assistant', content: interaction.answer, model })
  }

  async setError(interactionId: string, message: string, sessionId: string, model: string): Promise<void> {
    const interaction = this.interactions.find(i => i.id === interactionId)
    if (!interaction) return
    interaction.status = 'error'
    this.error = message
    await invoke('save_message', { sessionId, role: 'assistant', content: `Error: ${message}`, model })
  }

  async load(sessionId: string): Promise<void> {
    const messages: Message[] = await invoke('get_messages', { sessionId })
    const interactions: Interaction[] = []
    for (let i = 0; i < messages.length; i += 2) {
      const userMsg = messages[i]
      const assistantMsg = messages[i + 1]
      if (!userMsg || userMsg.role !== 'user') continue
      interactions.push({
        id: genId(),
        sessionId,
        prompt: userMsg.content,
        answer: assistantMsg?.content ?? '',
        reasoning: '',
        status: 'done',
        model: '',
        toolCalls: assistantMsg?.tool_calls ?? [],
        toolResults: [],
        createdAt: '',
      })
    }
    this.interactions = interactions
  }

  async getSessions(): Promise<Session[]> {
    return invoke('get_sessions')
  }

  async createSession(name: string): Promise<Session> {
    return invoke('create_session', { name })
  }

  async deleteSession(id: string): Promise<void> {
    return invoke('delete_session', { id })
  }

  async saveUserMessage(sessionId: string, content: string, model: string): Promise<void> {
    await invoke('save_message', { sessionId, role: 'user', content, model })
  }
}
