/**
 * Account Lockout Utility
 *
 * Implements account-level protection against brute force attacks.
 * Works alongside IP-based rate limiting for defense in depth.
 *
 * Configuration:
 * - MAX_FAILED_ATTEMPTS: 5 failed attempts before lockout
 * - LOCKOUT_DURATION_MS: 30 minutes lockout
 * - ATTEMPT_RESET_MS: 1 hour - failed attempts reset after this period of no failures
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../database/client'

const MAX_FAILED_ATTEMPTS = 5
const LOCKOUT_DURATION_MS = 30 * 60 * 1000 // 30 minutes
const ATTEMPT_RESET_MS = 60 * 60 * 1000 // 1 hour

interface LockoutStatus {
  isLocked: boolean
  lockedUntil: Date | null
  attemptsRemaining: number
  minutesRemaining: number
}

/**
 * Check if an account is currently locked
 */
export async function checkAccountLocked(userId: number): Promise<LockoutStatus> {
  const db = useDatabase()

  const user = db
    .select({
      failedLoginAttempts: schema.users.failedLoginAttempts,
      lockedUntil: schema.users.lockedUntil,
      lastFailedLogin: schema.users.lastFailedLogin
    })
    .from(schema.users)
    .where(eq(schema.users.id, userId))
    .get()

  if (!user) {
    return {
      isLocked: false,
      lockedUntil: null,
      attemptsRemaining: MAX_FAILED_ATTEMPTS,
      minutesRemaining: 0
    }
  }

  const now = new Date()

  // Check if lockout has expired
  if (user.lockedUntil && now < user.lockedUntil) {
    const minutesRemaining = Math.ceil((user.lockedUntil.getTime() - now.getTime()) / 60000)
    return {
      isLocked: true,
      lockedUntil: user.lockedUntil,
      attemptsRemaining: 0,
      minutesRemaining
    }
  }

  // If lockout expired or never locked, check if we should reset attempts
  if (user.lockedUntil && now >= user.lockedUntil) {
    // Lockout expired, reset the account
    await resetFailedAttempts(userId)
    return {
      isLocked: false,
      lockedUntil: null,
      attemptsRemaining: MAX_FAILED_ATTEMPTS,
      minutesRemaining: 0
    }
  }

  // Check if attempts should be reset due to time passing
  if (user.lastFailedLogin) {
    const timeSinceLastFailure = now.getTime() - user.lastFailedLogin.getTime()
    if (timeSinceLastFailure > ATTEMPT_RESET_MS) {
      await resetFailedAttempts(userId)
      return {
        isLocked: false,
        lockedUntil: null,
        attemptsRemaining: MAX_FAILED_ATTEMPTS,
        minutesRemaining: 0
      }
    }
  }

  const attempts = user.failedLoginAttempts || 0
  return {
    isLocked: false,
    lockedUntil: null,
    attemptsRemaining: Math.max(0, MAX_FAILED_ATTEMPTS - attempts),
    minutesRemaining: 0
  }
}

/**
 * Record a failed login attempt
 * Returns whether the account is now locked
 */
export async function recordFailedAttempt(userId: number): Promise<{
  isLocked: boolean
  attemptsRemaining: number
  lockoutMinutes: number
}> {
  const db = useDatabase()

  const user = db
    .select({
      failedLoginAttempts: schema.users.failedLoginAttempts,
      lastFailedLogin: schema.users.lastFailedLogin
    })
    .from(schema.users)
    .where(eq(schema.users.id, userId))
    .get()

  if (!user) {
    return { isLocked: false, attemptsRemaining: MAX_FAILED_ATTEMPTS, lockoutMinutes: 0 }
  }

  const now = new Date()

  // Check if we should reset attempts due to time passing
  let currentAttempts = user.failedLoginAttempts || 0
  if (user.lastFailedLogin) {
    const timeSinceLastFailure = now.getTime() - user.lastFailedLogin.getTime()
    if (timeSinceLastFailure > ATTEMPT_RESET_MS) {
      currentAttempts = 0
    }
  }

  // Increment attempts
  const newAttempts = currentAttempts + 1

  // Prepare update
  const updateData: {
    failedLoginAttempts: number
    lastFailedLogin: Date
    lockedUntil?: Date
  } = {
    failedLoginAttempts: newAttempts,
    lastFailedLogin: now
  }

  // Check if we should lock the account
  if (newAttempts >= MAX_FAILED_ATTEMPTS) {
    updateData.lockedUntil = new Date(now.getTime() + LOCKOUT_DURATION_MS)
  }

  db.update(schema.users).set(updateData).where(eq(schema.users.id, userId)).run()

  const isLocked = newAttempts >= MAX_FAILED_ATTEMPTS
  return {
    isLocked,
    attemptsRemaining: Math.max(0, MAX_FAILED_ATTEMPTS - newAttempts),
    lockoutMinutes: isLocked ? Math.ceil(LOCKOUT_DURATION_MS / 60000) : 0
  }
}

/**
 * Reset failed login attempts (called on successful login or by admin)
 */
export async function resetFailedAttempts(userId: number): Promise<void> {
  const db = useDatabase()

  db.update(schema.users)
    .set({
      failedLoginAttempts: 0,
      lockedUntil: null,
      lastFailedLogin: null
    })
    .where(eq(schema.users.id, userId))
    .run()
}

/**
 * Admin function: Manually unlock an account
 */
export async function unlockAccount(userId: number): Promise<void> {
  await resetFailedAttempts(userId)
}

/**
 * Get lockout configuration for documentation
 */
export function getLockoutConfig() {
  return {
    maxFailedAttempts: MAX_FAILED_ATTEMPTS,
    lockoutDurationMinutes: LOCKOUT_DURATION_MS / 60000,
    attemptResetHours: ATTEMPT_RESET_MS / 3600000
  }
}
