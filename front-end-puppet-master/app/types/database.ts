/**
 * Database Types
 *
 * Shared types derived from database schema.
 * These are the single source of truth for entity shapes.
 *
 * Database types use `Date` objects, but JSON responses serialize dates
 * as ISO strings or timestamps. The `APIResponse<T>` utility converts
 * Date fields to DateLike for API consumption.
 */
import type * as schema from '~~/server/database/schema'

/**
 * Date type that handles JSON serialization
 * API responses serialize Date objects as strings or numbers
 */
export type DateLike = Date | string | number

/**
 * Utility type to convert Date fields to DateLike for API responses
 */
type DateToDateLike<T> = {
  [K in keyof T]: T[K] extends Date | null | undefined
    ? DateLike | null | undefined
    : T[K] extends Date
      ? DateLike
      : T[K]
}

// ═══════════════════════════════════════════════════════════════════════════
// DATABASE SCHEMA TYPES (Direct exports)
// ═══════════════════════════════════════════════════════════════════════════

export type User = schema.User
export type Session = schema.Session
export type Setting = schema.Setting
export type Translation = schema.Translation
export type AuditLog = schema.AuditLog

// ═══════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES (Date → DateLike conversion)
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Portfolio (API response type)
 */
export type Portfolio = DateToDateLike<schema.Portfolio>

/**
 * Portfolio item (API response type)
 */
export type PortfolioItem = DateToDateLike<schema.PortfolioItem>

/**
 * Portfolio with items (API response when includeItems=true)
 */
export type PortfolioWithItems = Portfolio & {
  items: PortfolioItem[]
}

/**
 * Contact submission (API response type)
 */
export type ContactSubmission = DateToDateLike<schema.ContactSubmission>

// ═══════════════════════════════════════════════════════════════════════════
// TYPE CONSTANTS (Re-export from schema)
// ═══════════════════════════════════════════════════════════════════════════

export { PORTFOLIO_TYPES, PORTFOLIO_ITEM_TYPES } from '~~/server/database/schema'
export type { PortfolioType, PortfolioItemType } from '~~/server/database/schema'

// User roles
export { USER_ROLES } from '~~/server/database/schema'
export type { UserRole } from '~~/server/database/schema'

// Audit actions
export { AUDIT_ACTIONS } from '~~/server/database/schema'
export type { AuditAction } from '~~/server/database/schema'
