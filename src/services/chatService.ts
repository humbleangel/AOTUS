import { invoke } from '@tauri-apps/api/core'
import type { ChatEvent, Session, ChatParams } from '../lib/types'

export async function sendStream(
  runtime: import('../lib/interactionRuntime').InteractionRuntime,
  params: ChatParams,
  onEvent: (event: ChatEvent) => void,
): Promise<string> {
  const { Channel } = await import('@tauri-apps/api/core')
  const channel = new Channel<ChatEvent>()
  channel.onmessage = onEvent
  return invoke<string>('chat_stream', {
    message: params.message,
    sessionId: params.sessionId,
    modelName: params.modelName,
    channel,
  })
}

export async function sendBatch(
  runtime: import('../lib/interactionRuntime').InteractionRuntime,
  params: ChatParams,
): Promise<void> {
  const answer = await invoke<import('../lib/types').Answer>('chat_batch', {
    message: params.message,
    sessionId: params.sessionId,
    modelName: params.modelName,
  })
  const id = runtime.start(params.sessionId, params.message, params.modelName)
  const interaction = runtime.interactions.find((i: { id: string }) => i.id === id)
  if (interaction) {
    interaction.answer = answer.content
  }
  runtime.setDone(id)
}

export async function cancel(requestId: string): Promise<void> {
  return invoke('cancel_request', { requestId })
}

export async function getSessions(): Promise<Session[]> {
  return invoke('get_sessions')
}

export async function createSession(name: string): Promise<Session> {
  return invoke('create_session', { name })
}

export async function deleteSession(id: string): Promise<void> {
  return invoke('delete_session', { id })
}

export async function getMessages(sessionId: string): Promise<import('../lib/types').Message[]> {
  return invoke('get_messages', { sessionId })
}

export async function saveMessage(
  sessionId: string,
  role: string,
  content: string,
  model?: string,
): Promise<void> {
  return invoke('save_message', { sessionId, role, content, model })
}

export async function getModels(): Promise<import('../lib/types').ModelInfo[]> {
  return invoke('get_models')
}
