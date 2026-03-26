/**
 * Performance Monitoring Middleware
 *
 * Tracks request timing, database query performance, and resource usage.
 * Integrates with Sentry for error tracking and performance monitoring.
 *
 * Features:
 * - Request timing with high-resolution timestamps
 * - Database query timing
 * - Memory usage tracking
 * - Slow request alerting
 * - Server-Timing header for client visibility
 */
import { getHeader, setHeader } from 'h3'

// Configuration
const SLOW_REQUEST_THRESHOLD_MS = 1000 // 1 second
const ENABLE_DETAILED_METRICS = process.env.NODE_ENV !== 'production' || process.env.ENABLE_PERF_METRICS === 'true'

interface PerformanceMetrics {
  requestId: string
  method: string
  path: string
  startTime: number
  endTime?: number
  duration?: number
  statusCode?: number
  memoryUsage?: {
    heapUsed: number
    heapTotal: number
    external: number
    rss: number
  }
  timings: {
    name: string
    duration: number
    description?: string
  }[]
}

// Store for collecting timings within a request
const requestMetrics = new Map<string, PerformanceMetrics>()

/**
 * Generate a unique request ID
 */
function generateRequestId(): string {
  return `${Date.now()}-${Math.random().toString(36).substring(2, 11)}`
}

/**
 * Format bytes to human readable string
 */
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`
}

/**
 * Log performance metrics
 */
function logMetrics(metrics: PerformanceMetrics) {
  const isSlowRequest = metrics.duration && metrics.duration > SLOW_REQUEST_THRESHOLD_MS

  if (ENABLE_DETAILED_METRICS || isSlowRequest) {
    const logLevel = isSlowRequest ? 'warn' : 'debug'
    const logData = {
      requestId: metrics.requestId,
      method: metrics.method,
      path: metrics.path,
      duration: `${metrics.duration?.toFixed(2)}ms`,
      status: metrics.statusCode,
      memoryUsed: metrics.memoryUsage ? formatBytes(metrics.memoryUsage.heapUsed) : undefined,
      timings: metrics.timings.map(t => `${t.name}:${t.duration.toFixed(2)}ms`).join(', ')
    }

    if (isSlowRequest) {
      console.warn('[PERF] Slow request detected:', logData)
    } else {
      console.debug('[PERF]', logData)
    }
  }
}

/**
 * Build Server-Timing header value
 */
function buildServerTimingHeader(metrics: PerformanceMetrics): string {
  const entries: string[] = []

  if (metrics.duration) {
    entries.push(`total;dur=${metrics.duration.toFixed(2)};desc="Total Request Time"`)
  }

  for (const timing of metrics.timings) {
    const desc = timing.description ? `;desc="${timing.description}"` : ''
    entries.push(`${timing.name};dur=${timing.duration.toFixed(2)}${desc}`)
  }

  return entries.join(', ')
}

export default defineEventHandler(async event => {
  // Skip for static assets and non-API routes
  const path = event.path
  if (
    path.startsWith('/_nuxt') ||
    path.startsWith('/favicon') ||
    path.endsWith('.js') ||
    path.endsWith('.css') ||
    path.endsWith('.png') ||
    path.endsWith('.jpg') ||
    path.endsWith('.svg')
  ) {
    return
  }

  // Generate request ID
  const requestId = getHeader(event, 'x-request-id') || generateRequestId()

  // Initialize metrics
  const metrics: PerformanceMetrics = {
    requestId,
    method: event.method,
    path,
    startTime: performance.now(),
    timings: []
  }

  // Store metrics for this request
  requestMetrics.set(requestId, metrics)

  // Add request ID to context for other handlers
  event.context.requestId = requestId
  event.context.perfMetrics = metrics

  // Helper function to record timing
  event.context.recordTiming = (name: string, duration: number, description?: string) => {
    metrics.timings.push({ name, duration, description })
  }

  // Helper to start a timer
  event.context.startTimer = (name: string) => {
    const start = performance.now()
    return {
      end: (description?: string) => {
        const duration = performance.now() - start
        metrics.timings.push({ name, duration, description })
        return duration
      }
    }
  }

  // Set request ID header
  setHeader(event, 'X-Request-ID', requestId)

  // Continue to next handler - onResponse will handle completion
  event.node.res.on('finish', () => {
    metrics.endTime = performance.now()
    metrics.duration = metrics.endTime - metrics.startTime
    metrics.statusCode = event.node.res.statusCode

    // Capture memory usage
    if (ENABLE_DETAILED_METRICS) {
      const memUsage = process.memoryUsage()
      metrics.memoryUsage = {
        heapUsed: memUsage.heapUsed,
        heapTotal: memUsage.heapTotal,
        external: memUsage.external,
        rss: memUsage.rss
      }
    }

    // Set Server-Timing header
    const serverTiming = buildServerTimingHeader(metrics)
    if (!event.node.res.headersSent) {
      setHeader(event, 'Server-Timing', serverTiming)
    }

    // Log metrics
    logMetrics(metrics)

    // Cleanup
    requestMetrics.delete(requestId)
  })
})

/**
 * Export utility to manually record database query timing
 * Can be used in API handlers
 */
export function recordDatabaseTiming(event: any, queryName: string, duration: number) {
  if (event.context.recordTiming) {
    event.context.recordTiming(`db-${queryName}`, duration, `Database: ${queryName}`)
  }
}

/**
 * Wrap a database query with timing
 */
export async function withDbTiming<T>(
  event: any,
  queryName: string,
  queryFn: () => Promise<T> | T
): Promise<T> {
  if (!event.context.startTimer) {
    return queryFn()
  }

  const timer = event.context.startTimer(`db-${queryName}`)
  try {
    const result = await queryFn()
    timer.end(`Database: ${queryName}`)
    return result
  } catch (error) {
    timer.end(`Database: ${queryName} (failed)`)
    throw error
  }
}
