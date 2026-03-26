/**
 * Performance Analytics Endpoint
 *
 * POST /api/analytics/performance
 * Body: PerformanceReport
 *
 * Receives client-side performance metrics and stores/processes them.
 * In production, this would forward to a proper analytics backend.
 *
 * Security:
 * - Rate limited to prevent abuse
 * - Validates report structure
 * - Does not store PII
 */
import { z } from 'zod'

// Schema for validating performance reports
const webVitalSchema = z.object({
  name: z.string(),
  value: z.number(),
  rating: z.enum(['good', 'needs-improvement', 'poor']),
  delta: z.number()
})

const resourceTimingSchema = z.object({
  name: z.string(),
  type: z.string(),
  duration: z.number(),
  transferSize: z.number(),
  encodedBodySize: z.number(),
  decodedBodySize: z.number()
})

const performanceReportSchema = z.object({
  url: z.string().url(),
  timestamp: z.number(),
  webVitals: z.record(z.string(), webVitalSchema),
  resourceTimings: z.array(resourceTimingSchema),
  serverTimings: z.record(z.string(), z.number()),
  customMarks: z.record(z.string(), z.number()),
  navigationTiming: z.object({
    domContentLoaded: z.number(),
    loadComplete: z.number(),
    firstByte: z.number()
  })
})

// In-memory storage for development/demo
// In production, use proper time-series database or analytics service
const metricsStore: Array<{
  report: z.infer<typeof performanceReportSchema>
  receivedAt: Date
  userAgent: string
  ip: string
}> = []

// Keep only last 1000 reports in memory
const MAX_STORED_REPORTS = 1000

// Simple rate limiting
const rateLimitMap = new Map<string, { count: number; resetAt: number }>()
const RATE_LIMIT_WINDOW_MS = 60000 // 1 minute
const MAX_REQUESTS_PER_WINDOW = 10

function checkRateLimit(ip: string): boolean {
  const now = Date.now()
  const record = rateLimitMap.get(ip)

  if (!record || now > record.resetAt) {
    rateLimitMap.set(ip, { count: 1, resetAt: now + RATE_LIMIT_WINDOW_MS })
    return true
  }

  if (record.count >= MAX_REQUESTS_PER_WINDOW) {
    return false
  }

  record.count++
  return true
}

export default defineEventHandler(async event => {
  // Get client IP for rate limiting
  const ip = getRequestIP(event, { xForwardedFor: true }) || 'unknown'

  // Rate limit check
  if (!checkRateLimit(ip)) {
    throw createError({
      statusCode: 429,
      message: 'Too many performance reports. Please try again later.'
    })
  }

  const body = await readBody(event)

  // Validate report structure
  const result = performanceReportSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Invalid performance report format',
      data: result.error.flatten()
    })
  }

  const report = result.data

  // Get user agent for analytics
  const userAgent = getHeader(event, 'user-agent') || 'unknown'

  // Store the report
  metricsStore.push({
    report,
    receivedAt: new Date(),
    userAgent,
    ip: ip.substring(0, ip.lastIndexOf('.')) + '.x' // Partial IP for privacy
  })

  // Trim old reports
  if (metricsStore.length > MAX_STORED_REPORTS) {
    metricsStore.splice(0, metricsStore.length - MAX_STORED_REPORTS)
  }

  // Log summary (in production, forward to analytics service)
  const webVitals = Object.values(report.webVitals)
  const poorVitals = webVitals.filter(v => v.rating === 'poor')

  if (poorVitals.length > 0) {
    console.warn('[PERF] Poor Web Vitals reported:', {
      url: new URL(report.url).pathname,
      poorVitals: poorVitals.map(v => `${v.name}: ${v.value.toFixed(2)}`).join(', '),
      loadTime: `${report.navigationTiming.loadComplete.toFixed(0)}ms`
    })
  }

  // In production, you would:
  // 1. Forward to analytics service (e.g., BigQuery, ClickHouse)
  // 2. Send to Sentry Performance
  // 3. Store in time-series database for dashboards

  return {
    success: true,
    data: {
      message: 'Performance report received'
    }
  }
})

/**
 * Export for internal use (e.g., admin dashboard)
 */
export function getStoredMetrics() {
  return metricsStore
}

/**
 * Get aggregated metrics summary
 */
export function getMetricsSummary() {
  if (metricsStore.length === 0) {
    return { totalReports: 0, averages: {} }
  }

  const vitalSums: Record<string, { sum: number; count: number }> = {}

  for (const { report } of metricsStore) {
    for (const [name, vital] of Object.entries(report.webVitals)) {
      if (!vitalSums[name]) {
        vitalSums[name] = { sum: 0, count: 0 }
      }
      vitalSums[name].sum += vital.value
      vitalSums[name].count++
    }
  }

  const averages: Record<string, number> = {}
  for (const [name, data] of Object.entries(vitalSums)) {
    averages[name] = data.sum / data.count
  }

  return {
    totalReports: metricsStore.length,
    averages,
    timeRange: {
      oldest: metricsStore[0]?.receivedAt,
      newest: metricsStore[metricsStore.length - 1]?.receivedAt
    }
  }
}
