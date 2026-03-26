/**
 * Admin Types
 *
 * Types for admin panel pages and API responses.
 * These are database entities and API response types that should be shared.
 */

// ═══════════════════════════════════════════════════════════════════════════
// TRANSLATIONS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Translation entry entity
 */
export interface Translation {
  id: number
  key: string
  value: string
}

/**
 * Translations API response
 */
export interface TranslationsData {
  locales: string[]
  localeNames: Record<string, string>
  content: Record<string, Translation[]>
  system: Record<string, Translation[]> | null
  canEditSystem: boolean
}

// ═══════════════════════════════════════════════════════════════════════════
// HEALTH & MONITORING
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Health status values
 */
export type HealthStatus = 'ok' | 'degraded' | 'unhealthy'
export type CheckStatus = 'ok' | 'error' | 'warning'

/**
 * Individual health check result
 */
export interface HealthCheck {
  name: string
  status: CheckStatus
  latency?: number
  message?: string
  details?: Record<string, unknown>
}

/**
 * Overall system health data
 */
export interface HealthData {
  status: HealthStatus
  timestamp: string
  uptime: number
  version: string
  nodeVersion: string
  environment: string
  checks: HealthCheck[]
  responseTime: number
}

/**
 * Application log entry
 */
export interface LogEntry {
  time: string
  level: number
  levelLabel: string
  msg: string
  context?: Record<string, unknown>
}

/**
 * Audit trail log entry
 */
export interface AuditLogEntry {
  id: number
  action: string
  userId: number | null
  targetUserId: number | null
  actorName: string | null
  actorEmail: string | null
  ipAddress: string | null
  userAgent: string | null
  details: Record<string, unknown> | null
  success: boolean
  createdAt: string
}
