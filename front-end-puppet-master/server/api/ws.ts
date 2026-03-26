/**
 * WebSocket Endpoint
 *
 * Main WebSocket handler for real-time communication.
 * Supports authentication, room subscriptions, and message broadcasting.
 *
 * Connection URL: ws://localhost:3000/api/ws
 *
 * Message Protocol:
 *   - ping/pong: Heartbeat to keep connection alive
 *   - subscribe: Join a room (e.g., { type: 'subscribe', payload: { room: 'support' } })
 *   - unsubscribe: Leave a room
 *   - message: Send message to a room
 *
 * Authentication:
 *   - Uses existing session cookie (pm-session)
 *   - Some rooms require authentication (see ROOM_CONFIG)
 */
import { defineWebSocketHandler } from 'h3'
import { wsHub, authenticateWebSocket, WS_RATE_LIMIT } from '../lib/websocket'
import {
  WS_MESSAGE_TYPES,
  createWSMessage,
  isWSMessage,
  type WSMessage,
  type WSConnectedPayload,
  type WSSubscribePayload,
  type WSRoomMessagePayload
} from '../../types/websocket'
import { logger } from '../utils/logger'

// Map peer to connectionId for cleanup
const peerConnectionMap = new WeakMap<object, string>()

export default defineWebSocketHandler({
  /**
   * Handle new WebSocket connection
   */
  async open(peer) {
    try {
      // Authenticate from upgrade request
      const request = peer.request as Request | undefined
      const user = request ? await authenticateWebSocket(request) : null

      // Add to hub
      const connection = wsHub.addConnection(peer, user)
      peerConnectionMap.set(peer, connection.connectionId)

      // Send connected message
      wsHub.send(
        connection.connectionId,
        createWSMessage<WSConnectedPayload>(WS_MESSAGE_TYPES.CONNECTED, {
          userId: user?.userId ?? 0,
          connectionId: connection.connectionId
        })
      )

      logger.info(
        {
          connectionId: connection.connectionId,
          userId: user?.userId,
          authenticated: !!user
        },
        'WebSocket connected'
      )
    } catch (error) {
      logger.error({ error }, 'WebSocket open error')
      peer.close(1011, 'Internal error')
    }
  },

  /**
   * Handle incoming WebSocket message
   */
  async message(peer, message) {
    const connectionId = peerConnectionMap.get(peer)
    if (!connectionId) {
      logger.warn('Message from unknown peer')
      return
    }

    const connection = wsHub.getConnection(connectionId)
    if (!connection) {
      logger.warn({ connectionId }, 'Message from disconnected connection')
      return
    }

    // Rate limiting
    if (!wsHub.checkRateLimit(connectionId, WS_RATE_LIMIT.maxMessages, WS_RATE_LIMIT.windowMs)) {
      wsHub.sendError(connectionId, 'RATE_LIMITED', 'Too many messages, please slow down')
      return
    }

    try {
      // Parse message
      const text = message.text()
      const data = JSON.parse(text)

      if (!isWSMessage(data)) {
        wsHub.sendError(connectionId, 'INVALID_MESSAGE', 'Invalid message format')
        return
      }

      // Route message by type
      await handleMessage(connectionId, data)
    } catch (error) {
      logger.error({ connectionId, error }, 'WebSocket message error')
      wsHub.sendError(connectionId, 'PARSE_ERROR', 'Failed to parse message')
    }
  },

  /**
   * Handle WebSocket close
   */
  close(peer, details) {
    const connectionId = peerConnectionMap.get(peer)
    if (connectionId) {
      const connection = wsHub.getConnection(connectionId)
      logger.info(
        {
          connectionId,
          userId: connection?.user?.userId,
          code: details.code,
          reason: details.reason
        },
        'WebSocket disconnected'
      )
      wsHub.removeConnection(connectionId)
    }
  },

  /**
   * Handle WebSocket error
   */
  error(peer, error) {
    const connectionId = peerConnectionMap.get(peer)
    logger.error(
      { connectionId, error: error.message },
      'WebSocket error'
    )
    if (connectionId) {
      wsHub.removeConnection(connectionId)
    }
  }
})

/**
 * Route and handle incoming messages
 */
async function handleMessage(connectionId: string, message: WSMessage): Promise<void> {
  const connection = wsHub.getConnection(connectionId)
  if (!connection) return

  switch (message.type) {
    case WS_MESSAGE_TYPES.PING:
      // Respond with pong
      wsHub.send(connectionId, createWSMessage(WS_MESSAGE_TYPES.PONG))
      break

    case WS_MESSAGE_TYPES.SUBSCRIBE: {
      const payload = message.payload as WSSubscribePayload | undefined
      if (!payload?.room) {
        wsHub.sendError(connectionId, 'INVALID_PAYLOAD', 'Room name required')
        return
      }
      wsHub.subscribe(connectionId, payload.room)
      break
    }

    case WS_MESSAGE_TYPES.UNSUBSCRIBE: {
      const payload = message.payload as WSSubscribePayload | undefined
      if (!payload?.room) {
        wsHub.sendError(connectionId, 'INVALID_PAYLOAD', 'Room name required')
        return
      }
      wsHub.unsubscribe(connectionId, payload.room)
      break
    }

    case WS_MESSAGE_TYPES.MESSAGE: {
      const payload = message.payload as WSRoomMessagePayload | undefined
      if (!payload?.room || payload.content === undefined) {
        wsHub.sendError(connectionId, 'INVALID_PAYLOAD', 'Room and content required')
        return
      }

      // Check if user is in the room
      if (!connection.rooms.has(payload.room)) {
        wsHub.sendError(connectionId, 'NOT_IN_ROOM', 'You must subscribe to the room first')
        return
      }

      // Broadcast to room (include sender info)
      const broadcastPayload: WSRoomMessagePayload = {
        room: payload.room,
        content: payload.content,
        from: connection.user
          ? { userId: connection.user.userId, name: connection.user.name ?? undefined }
          : undefined
      }

      wsHub.broadcastToRoom(
        payload.room,
        createWSMessage(WS_MESSAGE_TYPES.MESSAGE, broadcastPayload, message.id)
      )
      break
    }

    default:
      logger.debug(
        { connectionId, type: message.type },
        'Unknown message type'
      )
      wsHub.sendError(connectionId, 'UNKNOWN_TYPE', `Unknown message type: ${message.type}`)
  }
}
