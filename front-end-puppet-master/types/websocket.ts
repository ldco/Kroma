/**
 * WebSocket Message Types
 *
 * Shared types for WebSocket communication between client and server.
 * Used by both server/api/ws.ts and composables/useWebSocket.ts
 */

// Message type constants
export const WS_MESSAGE_TYPES = {
  // Connection management
  PING: 'ping',
  PONG: 'pong',
  CONNECTED: 'connected',
  ERROR: 'error',

  // Room/channel management
  SUBSCRIBE: 'subscribe',
  UNSUBSCRIBE: 'unsubscribe',
  SUBSCRIBED: 'subscribed',
  UNSUBSCRIBED: 'unsubscribed',

  // Generic messaging
  MESSAGE: 'message',
  BROADCAST: 'broadcast',

  // Notifications
  NOTIFICATION: 'notification'
} as const

export type WSMessageType = (typeof WS_MESSAGE_TYPES)[keyof typeof WS_MESSAGE_TYPES]

/**
 * Base WebSocket message structure
 */
export interface WSMessage<T = unknown> {
  type: WSMessageType | string
  payload?: T
  timestamp: number
  id?: string // For message acknowledgment
}

/**
 * Payload types for specific message types
 */
export interface WSConnectedPayload {
  userId: number
  connectionId: string
}

export interface WSErrorPayload {
  code: string
  message: string
}

export interface WSSubscribePayload {
  room: string
}

export interface WSSubscribedPayload {
  room: string
  success: boolean
}

export interface WSRoomMessagePayload {
  room: string
  content: unknown
  from?: {
    userId: number
    name?: string
  }
}

export interface WSNotificationPayload {
  title: string
  body?: string
  type?: 'info' | 'success' | 'warning' | 'error'
  action?: {
    label: string
    url: string
  }
}

/**
 * Helper to create typed messages
 */
export function createWSMessage<T>(
  type: WSMessageType | string,
  payload?: T,
  id?: string
): WSMessage<T> {
  return {
    type,
    payload,
    timestamp: Date.now(),
    id
  }
}

/**
 * Type guard for WebSocket messages
 */
export function isWSMessage(data: unknown): data is WSMessage {
  return (
    typeof data === 'object' &&
    data !== null &&
    'type' in data &&
    typeof (data as WSMessage).type === 'string' &&
    'timestamp' in data &&
    typeof (data as WSMessage).timestamp === 'number'
  )
}
