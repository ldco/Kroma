/**
 * Health Check Endpoint
 *
 * GET /api/health
 *
 * Public endpoint for:
 * - Load balancer health checks
 * - Kubernetes liveness/readiness probes
 * - Docker health checks
 * - Monitoring systems
 *
 * Returns service health status and basic diagnostics.
 */
import { useDatabase, schema } from '../database/client'

interface HealthCheck {
  name: string
  status: 'ok' | 'error' | 'warning'
  latency?: number
  message?: string
}

export default defineEventHandler(async event => {
  const startTime = Date.now()
  const checks: HealthCheck[] = []
  let overallStatus: 'ok' | 'degraded' | 'unhealthy' = 'ok'

  // Database check
  const dbStart = Date.now()
  try {
    const db = useDatabase()
    // Simple query to verify database connection
    db.select().from(schema.users).limit(1).get()
    checks.push({
      name: 'database',
      status: 'ok',
      latency: Date.now() - dbStart
    })
  } catch (error: unknown) {
    overallStatus = 'unhealthy'
    const message = error instanceof Error ? error.message : 'Database connection failed'
    checks.push({
      name: 'database',
      status: 'error',
      message
    })
  }

  // Memory check
  const memUsage = process.memoryUsage()
  const heapUsedMB = Math.round(memUsage.heapUsed / 1024 / 1024)
  const heapTotalMB = Math.round(memUsage.heapTotal / 1024 / 1024)
  const memoryThreshold = 500 // MB

  if (heapUsedMB > memoryThreshold) {
    if (overallStatus === 'ok') overallStatus = 'degraded'
    checks.push({
      name: 'memory',
      status: 'warning',
      message: `High memory usage: ${heapUsedMB}MB / ${heapTotalMB}MB`
    })
  } else {
    checks.push({
      name: 'memory',
      status: 'ok',
      message: `${heapUsedMB}MB / ${heapTotalMB}MB`
    })
  }

  // Build response
  const response = {
    success: true,
    data: {
      status: overallStatus,
      timestamp: new Date().toISOString(),
      uptime: Math.floor(process.uptime()),
      version: process.env.npm_package_version || '1.0.0',
      checks,
      responseTime: Date.now() - startTime
    }
  }

  // Set appropriate status code
  if (overallStatus === 'unhealthy') {
    setResponseStatus(event, 503)
  } else if (overallStatus === 'degraded') {
    setResponseStatus(event, 200) // Still return 200 for degraded but functional
  }

  return response
})
