/**
 * Audit Logging Utility (HIGH-04)
 *
 * Logs security-relevant events for:
 * - Authentication (login, logout, failed attempts)
 * - User management (create, update, delete, role changes)
 * - Account security (lockouts, password changes)
 *
 * Usage:
 * await audit.login(event, userId)
 * await audit.loginFailed(event, email, reason)
 * await audit.roleChange(event, actorId, targetId, oldRole, newRole)
 */
import type { H3Event } from 'h3'
import { useDatabase, schema } from '../database/client'
import type { AuditAction } from '../database/schema'
import { logger } from './logger'

interface AuditOptions {
  userId?: number | null // Actor performing the action
  targetUserId?: number | null // User affected by the action
  details?: Record<string, unknown>
  success?: boolean
}

/**
 * Get client IP from request, handling proxies
 */
function getClientIp(event: H3Event): string | null {
  const forwarded = getHeader(event, 'x-forwarded-for')
  if (forwarded) {
    const firstIp = forwarded.split(',')[0]
    return firstIp ? firstIp.trim() : null
  }
  const realIp = getHeader(event, 'x-real-ip')
  if (realIp) {
    return realIp
  }
  return event.node.req.socket?.remoteAddress || null
}

/**
 * Get user agent from request
 */
function getUserAgent(event: H3Event): string | null {
  return getHeader(event, 'user-agent') || null
}

/**
 * Log an audit event
 */
async function logAudit(
  event: H3Event,
  action: AuditAction,
  options: AuditOptions = {}
): Promise<void> {
  const db = useDatabase()

  try {
    await db.insert(schema.auditLogs).values({
      action,
      userId: options.userId ?? null,
      targetUserId: options.targetUserId ?? null,
      ipAddress: getClientIp(event),
      userAgent: getUserAgent(event),
      details: options.details ? JSON.stringify(options.details) : null,
      success: options.success ?? true
    })
  } catch (error: unknown) {
    // Don't throw on audit failures - log and continue
    const message = error instanceof Error ? error.message : String(error)
    logger.error(
      { action, error: message },
      'Failed to log audit event'
    )
  }
}

/**
 * Audit logging interface
 */
export const audit = {
  /**
   * Log successful login
   */
  async login(event: H3Event, userId: number): Promise<void> {
    await logAudit(event, 'login', { userId, targetUserId: userId })
  },

  /**
   * Log failed login attempt
   */
  async loginFailed(event: H3Event, email: string, reason: string, userId?: number): Promise<void> {
    await logAudit(event, 'login_failed', {
      targetUserId: userId,
      details: { email, reason },
      success: false
    })
  },

  /**
   * Log logout
   */
  async logout(event: H3Event, userId: number): Promise<void> {
    await logAudit(event, 'logout', { userId, targetUserId: userId })
  },

  /**
   * Log password change
   */
  async passwordChange(event: H3Event, actorId: number, targetUserId: number): Promise<void> {
    await logAudit(event, 'password_change', {
      userId: actorId,
      targetUserId,
      details: { selfChange: actorId === targetUserId }
    })
  },

  /**
   * Log role change
   */
  async roleChange(
    event: H3Event,
    actorId: number,
    targetUserId: number,
    oldRole: string,
    newRole: string
  ): Promise<void> {
    await logAudit(event, 'role_change', {
      userId: actorId,
      targetUserId,
      details: { oldRole, newRole }
    })
  },

  /**
   * Log user creation
   */
  async userCreate(
    event: H3Event,
    actorId: number,
    newUserId: number,
    email: string,
    role: string
  ): Promise<void> {
    await logAudit(event, 'user_create', {
      userId: actorId,
      targetUserId: newUserId,
      details: { email, role }
    })
  },

  /**
   * Log user update
   */
  async userUpdate(
    event: H3Event,
    actorId: number,
    targetUserId: number,
    changes: Record<string, unknown>
  ): Promise<void> {
    await logAudit(event, 'user_update', {
      userId: actorId,
      targetUserId,
      details: { changes }
    })
  },

  /**
   * Log user deletion
   */
  async userDelete(
    event: H3Event,
    actorId: number,
    targetUserId: number,
    email: string
  ): Promise<void> {
    await logAudit(event, 'user_delete', {
      userId: actorId,
      targetUserId,
      details: { email }
    })
  },

  /**
   * Log account lockout
   */
  async accountLocked(event: H3Event, userId: number, reason: string): Promise<void> {
    await logAudit(event, 'account_locked', {
      targetUserId: userId,
      details: { reason }
    })
  },

  /**
   * Log account unlock
   */
  async accountUnlocked(
    event: H3Event,
    actorId: number | null,
    targetUserId: number,
    reason: string
  ): Promise<void> {
    await logAudit(event, 'account_unlocked', {
      userId: actorId,
      targetUserId,
      details: { reason }
    })
  },

  /**
   * Log session expiration
   */
  async sessionExpired(event: H3Event, userId: number): Promise<void> {
    await logAudit(event, 'session_expired', {
      targetUserId: userId
    })
  },

  /**
   * Log role creation
   */
  async roleCreate(
    event: H3Event,
    actorId: number,
    roleId: number,
    roleName: string,
    roleSlug: string
  ): Promise<void> {
    await logAudit(event, 'role_create', {
      userId: actorId,
      details: { roleId, roleName, roleSlug }
    })
  },

  /**
   * Log role update
   */
  async roleUpdate(
    event: H3Event,
    actorId: number,
    roleId: number,
    roleName: string,
    changes: Record<string, unknown>
  ): Promise<void> {
    await logAudit(event, 'role_update', {
      userId: actorId,
      details: { roleId, roleName, changes }
    })
  },

  /**
   * Log role deletion
   */
  async roleDelete(
    event: H3Event,
    actorId: number,
    roleId: number,
    roleName: string
  ): Promise<void> {
    await logAudit(event, 'role_delete', {
      userId: actorId,
      details: { roleId, roleName }
    })
  },

  /**
   * Log settings update with secret-safe metadata.
   */
  async settingsUpdate(
    event: H3Event,
    actorId: number,
    details: {
      updatedKeys: string[]
      createdKeys: string[]
      skippedKeys: string[]
      secretPolicy: Record<string, { configured: boolean }>
    }
  ): Promise<void> {
    await logAudit(event, 'settings_update', {
      userId: actorId,
      details
    })
  },

  /**
   * Log 2FA enabled
   */
  async twoFactorEnabled(event: H3Event, userId: number): Promise<void> {
    await logAudit(event, '2fa_enabled', {
      userId,
      targetUserId: userId,
      details: { action: '2FA enabled for account' }
    })
  },

  /**
   * Log 2FA disabled
   */
  async twoFactorDisabled(
    event: H3Event,
    actorId: number,
    targetUserId: number,
    reason?: string
  ): Promise<void> {
    await logAudit(event, '2fa_disabled', {
      userId: actorId,
      targetUserId,
      details: { reason: reason || 'User disabled 2FA', selfAction: actorId === targetUserId }
    })
  },

  /**
   * Log successful 2FA verification
   */
  async twoFactorVerified(
    event: H3Event,
    userId: number,
    method: 'totp' | 'backup'
  ): Promise<void> {
    await logAudit(event, '2fa_verified', {
      userId,
      targetUserId: userId,
      details: { method }
    })
  },

  /**
   * Log failed 2FA verification attempt
   */
  async twoFactorFailed(event: H3Event, userId: number, reason: string): Promise<void> {
    await logAudit(event, '2fa_failed', {
      targetUserId: userId,
      details: { reason },
      success: false
    })
  }
}
