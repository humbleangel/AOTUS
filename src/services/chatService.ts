import { invoke } from '@tauri-apps/api/core'
import type { ChatEvent, ChatParams } from '../lib/types'

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
  const id = await runtime.start(params.sessionId, params.message, params.modelName)
  const interaction = runtime.interactions.find((i: { id: string }) => i.id === id)
  if (interaction) {
    interaction.answer = answer.content
  }
  await runtime.setDone(id, params.sessionId, params.modelName)
}

export async function cancel(requestId: string): Promise<void> {
  return invoke('cancel_request', { requestId })
}
