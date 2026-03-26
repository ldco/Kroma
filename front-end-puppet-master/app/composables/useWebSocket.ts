/**
 * WebSocket Composable
 *
 * Client-side composable for WebSocket communication.
 * Provides automatic connection, reconnection, and typed messaging.
 *
 * Usage:
 *   const { connected, send, on, subscribe, unsubscribe } = useWebSocket()
 *
 *   // Listen for messages
 *   on('message', (msg) => console.log(msg))
 *
 *   // Subscribe to room
 *   subscribe('support')
 *
 *   // Send message
 *   send({ type: 'message', payload: { room: 'support', content: 'Hello!' } })
 */
import {
  WS_MESSAGE_TYPES,
  createWSMessage,
  isWSMessage,
  type WSMessage,
  type WSMessageType,
  type WSSubscribePayload,
  type WSRoomMessagePayload,
  type WSConnectedPayload,
  type WSErrorPayload
} from '../../types/websocket'

interface UseWebSocketOptions {
  /** Auto-connect on mount (default: false) */
  autoConnect?: boolean
  /** Reconnect on disconnect (default: true) */
  reconnect?: boolean
  /** Maximum reconnection attempts (default: 5) */
  maxReconnectAttempts?: number
  /** Base reconnection delay in ms (default: 1000) */
  reconnectDelay?: number
}

type MessageHandler<T = unknown> = (payload: T, message: WSMessage<T>) => void

// Shared state across all useWebSocket instances
const sharedState = {
  ws: null as WebSocket | null,
  connected: ref(false),
  connectionId: ref<string | null>(null),
  userId: ref<number | null>(null),
  reconnectAttempts: 0,
  reconnectTimeout: null as ReturnType<typeof setTimeout> | null,
  handlers: new Map<string, Set<MessageHandler>>(),
  pendingMessages: [] as WSMessage[]
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const {
    autoConnect = false,
    reconnect = true,
    maxReconnectAttempts = 5,
    reconnectDelay = 1000
  } = options

  // Reactive state
  const connected = sharedState.connected
  const connectionId = sharedState.connectionId
  const userId = sharedState.userId

  /**
   * Get WebSocket URL based on current location
   */
  function getWebSocketUrl(): string {
    if (import.meta.server) return ''

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const host = window.location.host
    return `${protocol}//${host}/api/ws`
  }

  /**
   * Connect to WebSocket server
   */
  function connect(): void {
    if (import.meta.server) return
    if (sharedState.ws?.readyState === WebSocket.OPEN) return
    if (sharedState.ws?.readyState === WebSocket.CONNECTING) return

    const url = getWebSocketUrl()
    if (!url) return

    try {
      sharedState.ws = new WebSocket(url)

      sharedState.ws.onopen = () => {
        sharedState.reconnectAttempts = 0
        // connected state is set when we receive CONNECTED message
      }

      sharedState.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          if (!isWSMessage(data)) return

          handleMessage(data)
        } catch {
          console.error('[WS] Failed to parse message:', event.data)
        }
      }

      sharedState.ws.onclose = (event) => {
        sharedState.connected.value = false
        sharedState.connectionId.value = null

        if (reconnect && !event.wasClean && sharedState.reconnectAttempts < maxReconnectAttempts) {
          scheduleReconnect()
        }
      }

      sharedState.ws.onerror = (error) => {
        console.error('[WS] Error:', error)
      }
    } catch (error) {
      console.error('[WS] Failed to connect:', error)
    }
  }

  /**
   * Schedule reconnection with exponential backoff
   */
  function scheduleReconnect(): void {
    if (sharedState.reconnectTimeout) {
      clearTimeout(sharedState.reconnectTimeout)
    }

    const delay = reconnectDelay * Math.pow(2, sharedState.reconnectAttempts)
    sharedState.reconnectAttempts++

    console.log(`[WS] Reconnecting in ${delay}ms (attempt ${sharedState.reconnectAttempts}/${maxReconnectAttempts})`)

    sharedState.reconnectTimeout = setTimeout(() => {
      connect()
    }, delay)
  }

  /**
   * Disconnect from WebSocket server
   */
  function disconnect(): void {
    if (sharedState.reconnectTimeout) {
      clearTimeout(sharedState.reconnectTimeout)
      sharedState.reconnectTimeout = null
    }

    if (sharedState.ws) {
      sharedState.ws.close(1000, 'Client disconnect')
      sharedState.ws = null
    }

    sharedState.connected.value = false
    sharedState.connectionId.value = null
    sharedState.reconnectAttempts = 0
  }

  /**
   * Handle incoming messages
   */
  function handleMessage(message: WSMessage): void {
    // Handle system messages
    switch (message.type) {
      case WS_MESSAGE_TYPES.CONNECTED: {
        const payload = message.payload as WSConnectedPayload
        sharedState.connected.value = true
        sharedState.connectionId.value = payload.connectionId
        sharedState.userId.value = payload.userId || null

        // Send pending messages
        for (const pending of sharedState.pendingMessages) {
          send(pending)
        }
        sharedState.pendingMessages = []
        break
      }

      case WS_MESSAGE_TYPES.PONG:
        // Heartbeat response - could track latency here
        break

      case WS_MESSAGE_TYPES.ERROR: {
        const payload = message.payload as WSErrorPayload
        console.error(`[WS] Error: ${payload.code} - ${payload.message}`)
        break
      }
    }

    // Call registered handlers
    const handlers = sharedState.handlers.get(message.type)
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(message.payload, message)
        } catch (error) {
          console.error('[WS] Handler error:', error)
        }
      }
    }

    // Call wildcard handlers
    const wildcardHandlers = sharedState.handlers.get('*')
    if (wildcardHandlers) {
      for (const handler of wildcardHandlers) {
        try {
          handler(message.payload, message)
        } catch (error) {
          console.error('[WS] Wildcard handler error:', error)
        }
      }
    }
  }

  /**
   * Send a message
   */
  function send(message: WSMessage): boolean {
    if (!sharedState.ws || sharedState.ws.readyState !== WebSocket.OPEN) {
      if (sharedState.connected.value === false) {
        // Queue message for when connected
        sharedState.pendingMessages.push(message)
      }
      return false
    }

    try {
      sharedState.ws.send(JSON.stringify(message))
      return true
    } catch (error) {
      console.error('[WS] Send error:', error)
      return false
    }
  }

  /**
   * Send typed message (convenience wrapper)
   */
  function sendMessage<T>(type: WSMessageType | string, payload?: T, id?: string): boolean {
    return send(createWSMessage(type, payload, id))
  }

  /**
   * Subscribe to a room
   */
  function subscribe(room: string): boolean {
    return sendMessage<WSSubscribePayload>(WS_MESSAGE_TYPES.SUBSCRIBE, { room })
  }

  /**
   * Unsubscribe from a room
   */
  function unsubscribe(room: string): boolean {
    return sendMessage<WSSubscribePayload>(WS_MESSAGE_TYPES.UNSUBSCRIBE, { room })
  }

  /**
   * Send a message to a room
   */
  function sendToRoom(room: string, content: unknown): boolean {
    return sendMessage<WSRoomMessagePayload>(WS_MESSAGE_TYPES.MESSAGE, { room, content })
  }

  /**
   * Register a message handler
   */
  function on<T = unknown>(type: WSMessageType | string | '*', handler: MessageHandler<T>): () => void {
    const handlers = sharedState.handlers.get(type) || new Set()
    handlers.add(handler as MessageHandler)
    sharedState.handlers.set(type, handlers)

    // Return unsubscribe function
    return () => {
      handlers.delete(handler as MessageHandler)
      if (handlers.size === 0) {
        sharedState.handlers.delete(type)
      }
    }
  }

  /**
   * Register a one-time message handler
   */
  function once<T = unknown>(type: WSMessageType | string, handler: MessageHandler<T>): () => void {
    const wrappedHandler: MessageHandler<T> = (payload, message) => {
      unsubscribe()
      handler(payload, message)
    }

    const unsubscribe = on(type, wrappedHandler)
    return unsubscribe
  }

  /**
   * Send ping to keep connection alive
   */
  function ping(): boolean {
    return sendMessage(WS_MESSAGE_TYPES.PING)
  }

  // Auto-connect if option is set
  if (autoConnect && import.meta.client) {
    onMounted(() => {
      connect()
    })

    onUnmounted(() => {
      // Don't disconnect on component unmount - shared connection
      // Only disconnect if explicitly called
    })
  }

  return {
    // State
    connected: readonly(connected),
    connectionId: readonly(connectionId),
    userId: readonly(userId),

    // Connection
    connect,
    disconnect,

    // Messaging
    send,
    sendMessage,
    sendToRoom,
    ping,

    // Rooms
    subscribe,
    unsubscribe,

    // Events
    on,
    once
  }
}
