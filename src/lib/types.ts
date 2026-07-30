export type Role = 'user' | 'assistant' | 'system' | 'tool'

export interface Message {
  role: Role
  content: string
  tool_call_id?: string
  tool_calls?: ToolCallDelta[]
}

export interface ToolCallDelta {
  id?: string
  type?: string
  function?: {
    name?: string
    arguments?: string
  }
}

export type InteractionStatus = 'pending' | 'streaming' | 'tool_executing' | 'done' | 'error'

export interface Interaction {
  id: string
  sessionId: string
  prompt: string
  answer: string
  reasoning: string
  status: InteractionStatus
  model: string
  toolCalls: ToolCallDelta[]
  toolResults: ToolResult[]
  usage?: Usage
  ttftMs?: number
  durationMs?: number
  createdAt: string
}

export interface ToolResult {
  callId: string
  output: string
}

export interface Usage {
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
}

export type ChatEvent =
  | { type: 'Token'; data: { text: string } }
  | { type: 'Done'; data: { answer: string; duration_ms: number; ttft_ms: number } }
  | { type: 'Error'; data: { message: string } }
  | { type: 'ToolCalls'; data: { calls: ToolCallDelta[] } }
  | { type: 'ToolResult'; data: { call_id: string; output: string } }

export interface ChatParams {
  message: string
  sessionId: string
  modelName: string
}

export interface Answer {
  content: string
  model: string
  usage?: Usage
  reasoning_content?: string
}

export interface Session {
  id: string
  name: string
  created_at: string
}
