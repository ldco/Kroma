/**
 * WebSocket Utilities
 *
 * Export all WebSocket-related utilities for easy import.
 *
 * Usage:
 *   import { wsHub, authenticateWebSocket } from '../utils/websocket'
 */

export { wsHub } from './_internal/hub'
export { authenticateWebSocket, requireWSRole } from './_internal/auth'
export * from './_internal/types'
