/**
 * Server-side WebSocket Types
 *
 * Types specific to server-side WebSocket handling.
 */
import type { Peer } from 'crossws'
import type { UserRole } from '../../../database/schema'

/**
 * Authenticated user context attached to WebSocket connection
 */
export interface WSUserContext {
  userId: number
  email: string
  name: string | null
  role: UserRole
}

/**
 * Extended peer with user context and metadata
 */
export interface WSConnection {
  peer: Peer
  user: WSUserContext | null
  connectionId: string
  connectedAt: number
  rooms: Set<string>
}

/**
 * Room subscription options
 */
export interface WSRoomOptions {
  /** Minimum role required to join this room */
  minRole?: UserRole
  /** Maximum connections allowed in this room */
  maxConnections?: number
}

/**
 * Predefined room configurations
 */
export const ROOM_CONFIG: Record<string, WSRoomOptions> = {
  // Support room - admin+ only
  support: { minRole: 'editor' },
  // Admin notifications - admin+ only
  'admin:notifications': { minRole: 'admin' },
  // System-wide broadcasts - master only
  'system:broadcast': { minRole: 'master' }
}

/**
 * Rate limit configuration for WebSocket messages
 */
export interface WSRateLimitConfig {
  /** Maximum messages per window */
  maxMessages: number
  /** Time window in milliseconds */
  windowMs: number
}

export const WS_RATE_LIMIT: WSRateLimitConfig = {
  maxMessages: 60,
  windowMs: 60_000 // 1 minute
}

/**
 * Connection limits
 */
export const WS_LIMITS = {
  /** Maximum connections per user */
  maxConnectionsPerUser: 5,
  /** Maximum rooms per connection */
  maxRoomsPerConnection: 10,
  /** Heartbeat interval in ms */
  heartbeatIntervalMs: 30_000,
  /** Connection timeout if no heartbeat response */
  heartbeatTimeoutMs: 10_000
}
