import { invoke } from '@tauri-apps/api/core'
import type { ChatEvent, Session } from './types'

export async function sendStream(
  message: string,
  sessionId: string,
  modelName: string,
  onEvent: (event: ChatEvent) => void,
): Promise<string> {
  const { Channel } = await import('@tauri-apps/api/core')
  const channel = new Channel<ChatEvent>()
  channel.onmessage = onEvent
  return invoke<string>('chat_stream', { message, sessionId, modelName, channel })
}

export async function cancelRequest(requestId: string): Promise<void> {
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

export async function getMessages(sessionId: string): Promise<import('./types').Message[]> {
  return invoke('get_messages', { sessionId })
}
