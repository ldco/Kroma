/**
 * Database Schema
 *
 * SQLite schema using Drizzle ORM.
 * Tables: users, sessions, settings, portfolios, portfolio_items, contact_submissions,
 *         pricing_tiers, pricing_features, translations, audit_logs,
 *         team_members, blog_posts, blog_categories, blog_tags, blog_media,
 *         clients, features, testimonials, faq_items
 */
import { sqliteTable, text, integer, unique, index } from 'drizzle-orm/sqlite-core'

// ═══════════════════════════════════════════════════════════════════════════
// USERS & AUTH
// ═══════════════════════════════════════════════════════════════════════════

/**
 * User roles hierarchy (legacy - for backward compatibility):
 * - master: Developer/agency who builds the site (full access)
 * - admin: Client who owns the site (can manage content + users except master)
 * - editor: Client's employees (can only edit content)
 */
export const USER_ROLES = ['master', 'admin', 'editor'] as const
export type UserRole = (typeof USER_ROLES)[number]

/**
 * Admin page IDs - each admin page is a permission
 * Simple model: if page is true, user sees it in nav
 */
export const ADMIN_PAGE_IDS = [
  // System pages
  'users',
  'roles',
  'translations',
  'settings',
  'health',
  'contacts'
] as const
export type AdminPageId = (typeof ADMIN_PAGE_IDS)[number]
export type RolePermissions = Partial<Record<AdminPageId, boolean>>

/**
 * Badge color options for roles
 */
export const ROLE_COLORS = ['primary', 'secondary', 'warning', 'success', 'danger'] as const
export type RoleColor = (typeof ROLE_COLORS)[number]

/**
 * Roles - Dynamic role management
 */
export const roles = sqliteTable('roles', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull().unique(),
  slug: text('slug').notNull().unique(),
  description: text('description'),
  permissions: text('permissions').notNull(), // JSON of RolePermissions
  level: integer('level').notNull().default(0), // Higher = more authority
  isBuiltIn: integer('is_built_in', { mode: 'boolean' }).default(false),
  color: text('color').default('secondary'),
  icon: text('icon').default('pencil'), // Icon name for custom roles
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

/**
 * Admin users
 */
export const users = sqliteTable('users', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  email: text('email').notNull().unique(),
  passwordHash: text('password_hash').notNull(),
  name: text('name'),
  // Legacy role column (for backward compatibility during migration)
  role: text('role', { enum: ['master', 'admin', 'editor'] })
    .default('editor')
    .notNull(),
  // New dynamic role reference
  roleId: integer('role_id').references(() => roles.id, { onDelete: 'set null' }),
  // Account lockout fields (CRIT-04)
  failedLoginAttempts: integer('failed_login_attempts').default(0),
  lockedUntil: integer('locked_until', { mode: 'timestamp' }),
  lastFailedLogin: integer('last_failed_login', { mode: 'timestamp' }),
  // Two-factor authentication
  twoFactorEnabled: integer('two_factor_enabled', { mode: 'boolean' }).default(false),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

/**
 * User sessions for authentication
 */
export const sessions = sqliteTable(
  'sessions',
  {
    id: text('id').primaryKey(), // UUID
    userId: integer('user_id')
      .notNull()
      .references(() => users.id, { onDelete: 'cascade' }),
    expiresAt: integer('expires_at', { mode: 'timestamp' }).notNull(),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    index('sessions_user_expires_idx').on(table.userId, table.expiresAt),
    index('sessions_expires_idx').on(table.expiresAt)
  ]
)

/**
 * Two-Factor Authentication secrets and backup codes
 * Stores encrypted TOTP secret and one-time backup codes
 */
export const user2fa = sqliteTable(
  'user_2fa',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    userId: integer('user_id')
      .notNull()
      .unique()
      .references(() => users.id, { onDelete: 'cascade' }),
    // Encrypted TOTP secret (base32 encoded)
    secret: text('secret').notNull(),
    // JSON array of hashed backup codes (10 codes, each usable once)
    backupCodes: text('backup_codes').notNull(),
    // Number of backup codes remaining
    backupCodesRemaining: integer('backup_codes_remaining').default(10),
    // Timestamps
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('user_2fa_user_idx').on(table.userId)]
)

// ═══════════════════════════════════════════════════════════════════════════
// SITE SETTINGS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Key-value settings store (editable via admin)
 */
export const settings = sqliteTable('settings', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  key: text('key').notNull().unique(),
  value: text('value'),
  type: text('type', { enum: ['string', 'number', 'boolean', 'json'] })
    .default('string')
    .notNull(),
  group: text('group').default('general'), // For organizing in admin UI
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

/**
 * Contact form submissions
 */
export const contactSubmissions = sqliteTable('contact_submissions', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
  email: text('email').notNull(),
  phone: text('phone'),
  subject: text('subject'),
  message: text('message').notNull(),
  read: integer('read', { mode: 'boolean' }).default(false),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

// ═══════════════════════════════════════════════════════════════════════════
// TRANSLATIONS (Database-driven i18n)
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Translation strings by locale
 */
export const translations = sqliteTable(
  'translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    locale: text('locale').notNull(), // 'en', 'ru', 'he'
    key: text('key').notNull(), // 'nav.home', 'common.submit'
    value: text('value').notNull(),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [unique().on(table.locale, table.key)]
)

// ═══════════════════════════════════════════════════════════════════════════
// AUDIT LOGGING (HIGH-04)
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Audit actions for security-relevant events
 */
export const AUDIT_ACTIONS = [
  'login',
  'login_failed',
  'logout',
  'password_change',
  'role_change',
  'user_create',
  'user_update',
  'user_delete',
  'account_locked',
  'account_unlocked',
  'session_expired',
  // Role management
  'role_create',
  'role_update',
  'role_delete',
  'settings_update',
  // Two-Factor Authentication
  '2fa_enabled',
  '2fa_disabled',
  '2fa_verified',
  '2fa_failed'
] as const
export type AuditAction = (typeof AUDIT_ACTIONS)[number]

/**
 * Audit log for security-relevant events
 */
export const auditLogs = sqliteTable(
  'audit_logs',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    action: text('action').notNull(), // AuditAction
    userId: integer('user_id').references(() => users.id, { onDelete: 'set null' }), // Actor (null for system)
    targetUserId: integer('target_user_id').references(() => users.id, { onDelete: 'set null' }), // Affected user
    ipAddress: text('ip_address'),
    userAgent: text('user_agent'),
    details: text('details'), // JSON with additional context
    success: integer('success', { mode: 'boolean' }).default(true),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    index('audit_logs_created_idx').on(table.createdAt),
    index('audit_logs_action_idx').on(table.action)
  ]
)

// ═══════════════════════════════════════════════════════════════════════════
// TYPE EXPORTS
// ═══════════════════════════════════════════════════════════════════════════

export type AuditLog = typeof auditLogs.$inferSelect
export type NewAuditLog = typeof auditLogs.$inferInsert
export type Role = typeof roles.$inferSelect
export type NewRole = typeof roles.$inferInsert
export type User = typeof users.$inferSelect
export type NewUser = typeof users.$inferInsert
export type Session = typeof sessions.$inferSelect
export type NewSession = typeof sessions.$inferInsert
export type User2fa = typeof user2fa.$inferSelect
export type NewUser2fa = typeof user2fa.$inferInsert
export type Setting = typeof settings.$inferSelect
export type NewSetting = typeof settings.$inferInsert
export type ContactSubmission = typeof contactSubmissions.$inferSelect
export type NewContactSubmission = typeof contactSubmissions.$inferInsert
export type Translation = typeof translations.$inferSelect
export type NewTranslation = typeof translations.$inferInsert
