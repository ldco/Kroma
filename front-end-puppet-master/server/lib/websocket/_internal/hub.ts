/**
 * WebSocket Hub
 *
 * Central manager for WebSocket connections, rooms, and message broadcasting.
 * Provides utilities for connection tracking, room subscriptions, and messaging.
 *
 * Usage:
 *   import { wsHub } from './hub'
 *
 *   // Add connection
 *   wsHub.addConnection(peer, user)
 *
 *   // Subscribe to room
 *   wsHub.subscribe(connectionId, 'support')
 *
 *   // Broadcast to room
 *   wsHub.broadcastToRoom('support', { type: 'message', ... })
 */
import type { Peer } from 'crossws'
import type { WSConnection, WSUserContext } from './types'
import { ROOM_CONFIG, WS_LIMITS } from './types'
import { hasRole } from '../../../utils/roles'
import type { UserRole } from '../../../database/schema'
import {
  createWSMessage,
  WS_MESSAGE_TYPES,
  type WSMessage,
  type WSSubscribedPayload,
  type WSErrorPayload
} from '../../../../types/websocket'
import { logger } from '../../../utils/logger'

/**
 * In-memory WebSocket connection hub
 */
class WebSocketHub {
  /** All active connections by connection ID */
  private connections = new Map<string, WSConnection>()

  /** Connections grouped by user ID for quick lookup */
  private userConnections = new Map<number, Set<string>>()

  /** Room subscriptions: room name -> Set of connection IDs */
  private rooms = new Map<string, Set<string>>()

  /** Rate limiting: connection ID -> message timestamps */
  private rateLimits = new Map<string, number[]>()

  /**
   * Generate unique connection ID
   */
  private generateConnectionId(): string {
    return `ws_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`
  }

  /**
   * Add a new connection
   */
  addConnection(peer: Peer, user: WSUserContext | null): WSConnection {
    const connectionId = this.generateConnectionId()

    // Check connection limit per user
    if (user) {
      const userConns = this.userConnections.get(user.userId) || new Set()
      if (userConns.size >= WS_LIMITS.maxConnectionsPerUser) {
        // Close oldest connection
        const oldestId = userConns.values().next().value
        if (oldestId) {
          this.removeConnection(oldestId)
          logger.info({ userId: user.userId }, 'Closed oldest connection due to limit')
        }
      }
    }

    const connection: WSConnection = {
      peer,
      user,
      connectionId,
      connectedAt: Date.now(),
      rooms: new Set()
    }

    this.connections.set(connectionId, connection)

    // Track user connections
    if (user) {
      const userConns = this.userConnections.get(user.userId) || new Set()
      userConns.add(connectionId)
      this.userConnections.set(user.userId, userConns)
    }

    logger.debug(
      { connectionId, userId: user?.userId },
      'WebSocket connection added'
    )

    return connection
  }

  /**
   * Remove a connection and clean up
   */
  removeConnection(connectionId: string): void {
    const connection = this.connections.get(connectionId)
    if (!connection) return

    // Remove from all rooms
    for (const room of connection.rooms) {
      this.unsubscribe(connectionId, room)
    }

    // Remove from user connections
    if (connection.user) {
      const userConns = this.userConnections.get(connection.user.userId)
      if (userConns) {
        userConns.delete(connectionId)
        if (userConns.size === 0) {
          this.userConnections.delete(connection.user.userId)
        }
      }
    }

    // Clean up rate limits
    this.rateLimits.delete(connectionId)

    // Remove connection
    this.connections.delete(connectionId)

    logger.debug(
      { connectionId, userId: connection.user?.userId },
      'WebSocket connection removed'
    )
  }

  /**
   * Get connection by ID
   */
  getConnection(connectionId: string): WSConnection | undefined {
    return this.connections.get(connectionId)
  }

  /**
   * Get all connections for a user
   */
  getUserConnections(userId: number): WSConnection[] {
    const connectionIds = this.userConnections.get(userId)
    if (!connectionIds) return []

    return Array.from(connectionIds)
      .map(id => this.connections.get(id))
      .filter((c): c is WSConnection => c !== undefined)
  }

  /**
   * Subscribe connection to a room
   */
  subscribe(connectionId: string, room: string): boolean {
    const connection = this.connections.get(connectionId)
    if (!connection) {
      this.sendError(connectionId, 'NOT_CONNECTED', 'Connection not found')
      return false
    }

    // Check room limit
    if (connection.rooms.size >= WS_LIMITS.maxRoomsPerConnection) {
      this.sendError(connectionId, 'ROOM_LIMIT', 'Maximum room subscriptions reached')
      return false
    }

    // Check room authorization
    const roomConfig = ROOM_CONFIG[room]
    if (roomConfig?.minRole) {
      if (!connection.user) {
        this.sendError(connectionId, 'AUTH_REQUIRED', 'Authentication required for this room')
        return false
      }
      if (!hasRole(connection.user.role, roomConfig.minRole)) {
        this.sendError(connectionId, 'FORBIDDEN', 'Insufficient permissions for this room')
        return false
      }
    }

    // Check room capacity
    if (roomConfig?.maxConnections) {
      const roomConns = this.rooms.get(room)
      if (roomConns && roomConns.size >= roomConfig.maxConnections) {
        this.sendError(connectionId, 'ROOM_FULL', 'Room is at capacity')
        return false
      }
    }

    // Add to room
    connection.rooms.add(room)
    const roomConnections = this.rooms.get(room) || new Set()
    roomConnections.add(connectionId)
    this.rooms.set(room, roomConnections)

    // Send confirmation
    this.send(connectionId, createWSMessage<WSSubscribedPayload>(
      WS_MESSAGE_TYPES.SUBSCRIBED,
      { room, success: true }
    ))

    logger.debug(
      { connectionId, room, userId: connection.user?.userId },
      'Subscribed to room'
    )

    return true
  }

  /**
   * Unsubscribe connection from a room
   */
  unsubscribe(connectionId: string, room: string): void {
    const connection = this.connections.get(connectionId)
    if (!connection) return

    connection.rooms.delete(room)

    const roomConnections = this.rooms.get(room)
    if (roomConnections) {
      roomConnections.delete(connectionId)
      if (roomConnections.size === 0) {
        this.rooms.delete(room)
      }
    }

    this.send(connectionId, createWSMessage<WSSubscribedPayload>(
      WS_MESSAGE_TYPES.UNSUBSCRIBED,
      { room, success: true }
    ))

    logger.debug(
      { connectionId, room, userId: connection.user?.userId },
      'Unsubscribed from room'
    )
  }

  /**
   * Send message to a specific connection
   */
  send(connectionId: string, message: WSMessage): boolean {
    const connection = this.connections.get(connectionId)
    if (!connection) return false

    try {
      connection.peer.send(JSON.stringify(message))
      return true
    } catch (error) {
      logger.error({ connectionId, error }, 'Failed to send WebSocket message')
      return false
    }
  }

  /**
   * Send error to a specific connection
   */
  sendError(connectionId: string, code: string, message: string): void {
    this.send(connectionId, createWSMessage<WSErrorPayload>(
      WS_MESSAGE_TYPES.ERROR,
      { code, message }
    ))
  }

  /**
   * Broadcast message to all connections in a room
   */
  broadcastToRoom(room: string, message: WSMessage, excludeConnectionId?: string): number {
    const roomConnections = this.rooms.get(room)
    if (!roomConnections) return 0

    let sent = 0
    for (const connectionId of roomConnections) {
      if (connectionId === excludeConnectionId) continue
      if (this.send(connectionId, message)) {
        sent++
      }
    }

    return sent
  }

  /**
   * Send message to a specific user (all their connections)
   */
  sendToUser(userId: number, message: WSMessage): number {
    const connections = this.getUserConnections(userId)
    let sent = 0

    for (const connection of connections) {
      if (this.send(connection.connectionId, message)) {
        sent++
      }
    }

    return sent
  }

  /**
   * Broadcast to all authenticated connections
   */
  broadcastToAuthenticated(message: WSMessage, minRole?: UserRole): number {
    let sent = 0

    for (const connection of this.connections.values()) {
      if (!connection.user) continue
      if (minRole && !hasRole(connection.user.role, minRole)) continue

      if (this.send(connection.connectionId, message)) {
        sent++
      }
    }

    return sent
  }

  /**
   * Check rate limit for a connection
   */
  checkRateLimit(connectionId: string, maxMessages: number, windowMs: number): boolean {
    const now = Date.now()
    const timestamps = this.rateLimits.get(connectionId) || []

    // Remove old timestamps outside window
    const validTimestamps = timestamps.filter(t => now - t < windowMs)

    if (validTimestamps.length >= maxMessages) {
      return false // Rate limited
    }

    validTimestamps.push(now)
    this.rateLimits.set(connectionId, validTimestamps)
    return true
  }

  /**
   * Get hub statistics
   */
  getStats(): {
    totalConnections: number
    authenticatedConnections: number
    totalRooms: number
    roomStats: Record<string, number>
  } {
    let authenticatedConnections = 0
    for (const connection of this.connections.values()) {
      if (connection.user) authenticatedConnections++
    }

    const roomStats: Record<string, number> = {}
    for (const [room, connections] of this.rooms) {
      roomStats[room] = connections.size
    }

    return {
      totalConnections: this.connections.size,
      authenticatedConnections,
      totalRooms: this.rooms.size,
      roomStats
    }
  }
}

// Singleton instance
export const wsHub = new WebSocketHub()
