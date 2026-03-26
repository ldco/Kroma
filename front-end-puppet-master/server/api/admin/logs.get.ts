/**
 * Admin Logs Endpoint
 *
 * GET /api/admin/logs
 *
 * Returns recent application logs from the in-memory buffer.
 * Requires master role.
 *
 * Query params:
 * - limit: Number of entries to return (default: 100, max: 500)
 * - level: Minimum log level (trace=10, debug=20, info=30, warn=40, error=50, fatal=60)
 */
import { getRecentLogs, getBufferStats } from '../../utils/logBuffer'
import { requireMaster } from '../../utils/roles'
import type { UserRole } from '../../database/schema'

// Log level name to number mapping
const LEVEL_VALUES: Record<string, number> = {
  trace: 10,
  debug: 20,
  info: 30,
  warn: 40,
  error: 50,
  fatal: 60
}

export default defineEventHandler(async event => {
  // Require master role
  const user = event.context.user as { role: UserRole } | undefined
  requireMaster(user?.role)

  // Parse query params
  const query = getQuery(event)

  // Limit (default 100, max 500)
  let limit = parseInt(String(query.limit || '100'), 10)
  if (isNaN(limit) || limit < 1) limit = 100
  if (limit > 500) limit = 500

  // Minimum level
  let minLevel = 0
  if (query.level) {
    const levelStr = String(query.level).toLowerCase()
    const levelValue = LEVEL_VALUES[levelStr]
    if (levelValue !== undefined) {
      minLevel = levelValue
    } else {
      const numLevel = parseInt(levelStr, 10)
      if (!isNaN(numLevel)) {
        minLevel = numLevel
      }
    }
  }

  // Get logs from buffer
  const logs = getRecentLogs(limit, minLevel)
  const stats = getBufferStats()

  return {
    success: true,
    data: {
      logs,
      meta: {
        returned: logs.length,
        limit,
        minLevel,
        bufferSize: stats.size,
        bufferMaxSize: stats.maxSize,
        oldestEntry: stats.oldestEntry
      }
    }
  }
})
