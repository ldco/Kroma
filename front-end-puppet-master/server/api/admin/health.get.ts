/**
 * Admin Health Endpoint
 *
 * GET /api/admin/health
 *
 * Returns detailed server health information for the admin health page.
 * Requires master role (enforced here, auth already checked by middleware).
 *
 * Returns more details than the public /api/health endpoint:
 * - Full memory breakdown
 * - Disk usage (if available)
 * - Log buffer stats
 * - Environment info
 */
import { useDatabase, schema } from '../../database/client'
import { requireMaster } from '../../utils/roles'
import { getBufferStats } from '../../utils/logBuffer'
import type { UserRole } from '../../database/schema'
import { sql } from 'drizzle-orm'

interface HealthCheck {
  name: string
  status: 'ok' | 'error' | 'warning'
  latency?: number
  message?: string
  details?: Record<string, unknown>
}

export default defineEventHandler(async event => {
  // Require master role for detailed health info
  const user = event.context.user as { role: UserRole } | undefined
  requireMaster(user?.role)

  const startTime = Date.now()
  const checks: HealthCheck[] = []
  let overallStatus: 'ok' | 'degraded' | 'unhealthy' = 'ok'

  // Database check with more details
  const dbStart = Date.now()
  try {
    const db = useDatabase()

    // Check connection using COUNT(*) instead of loading all rows
    const userCountResult = db.select({ count: sql<number>`count(*)` }).from(schema.users).get()
    const userCount = userCountResult?.count ?? 0

    // Get audit log count using COUNT(*)
    const auditCountResult = db.select({ count: sql<number>`count(*)` }).from(schema.auditLogs).get()
    const auditCount = auditCountResult?.count ?? 0

    checks.push({
      name: 'database',
      status: 'ok',
      latency: Date.now() - dbStart,
      details: {
        users: userCount,
        auditLogs: auditCount
      }
    })
  } catch (error: unknown) {
    overallStatus = 'unhealthy'
    checks.push({
      name: 'database',
      status: 'error',
      message: error instanceof Error ? error.message : 'Database connection failed'
    })
  }

  // Memory check with full breakdown
  const memUsage = process.memoryUsage()
  const heapUsedMB = Math.round(memUsage.heapUsed / 1024 / 1024)
  const heapTotalMB = Math.round(memUsage.heapTotal / 1024 / 1024)
  const rssMB = Math.round(memUsage.rss / 1024 / 1024)
  const externalMB = Math.round(memUsage.external / 1024 / 1024)
  const memoryThreshold = 500 // MB

  if (heapUsedMB > memoryThreshold) {
    if (overallStatus === 'ok') overallStatus = 'degraded'
    checks.push({
      name: 'memory',
      status: 'warning',
      message: `High memory usage: ${heapUsedMB}MB`,
      details: {
        heapUsed: heapUsedMB,
        heapTotal: heapTotalMB,
        rss: rssMB,
        external: externalMB,
        threshold: memoryThreshold
      }
    })
  } else {
    checks.push({
      name: 'memory',
      status: 'ok',
      details: {
        heapUsed: heapUsedMB,
        heapTotal: heapTotalMB,
        rss: rssMB,
        external: externalMB,
        percentage: Math.round((heapUsedMB / heapTotalMB) * 100)
      }
    })
  }

  // Log buffer check
  const bufferStats = getBufferStats()
  checks.push({
    name: 'logBuffer',
    status: 'ok',
    details: {
      entries: bufferStats.size,
      maxEntries: bufferStats.maxSize,
      oldestEntry: bufferStats.oldestEntry
    }
  })

  // CPU usage (approximate via event loop)
  const cpuUsage = process.cpuUsage()
  checks.push({
    name: 'cpu',
    status: 'ok',
    details: {
      user: Math.round(cpuUsage.user / 1000), // microseconds to ms
      system: Math.round(cpuUsage.system / 1000)
    }
  })

  // Build response
  return {
    success: true,
    data: {
      status: overallStatus,
      timestamp: new Date().toISOString(),
      uptime: Math.floor(process.uptime()),
      version: process.env.npm_package_version || '1.0.0',
      nodeVersion: process.version,
      environment: process.env.NODE_ENV || 'development',
      checks,
      responseTime: Date.now() - startTime
    }
  }
})
