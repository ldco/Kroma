/**
 * Password Utilities
 *
 * Secure password hashing using Node.js native scrypt.
 * Includes password policy enforcement and strength calculation.
 *
 * Password Policy:
 * - Minimum 8 characters
 * - At least one uppercase letter
 * - At least one lowercase letter
 * - At least one number
 * - At least one special character
 * - Not in common passwords list
 */
import { scryptSync, randomBytes, timingSafeEqual } from 'crypto'

const SALT_LENGTH = 16
const KEY_LENGTH = 64

// Common passwords to reject (subset of common list)
const COMMON_PASSWORDS = new Set([
  'password', 'password1', 'password123', '123456', '12345678', '123456789',
  'qwerty', 'abc123', 'letmein', 'welcome', 'monkey', 'dragon', 'master',
  'admin', 'login', 'passw0rd', 'hello', 'sunshine', 'princess', 'football',
  'baseball', 'iloveyou', 'trustno1', 'superman', 'batman', 'shadow', 'michael',
  'ashley', 'daniel', 'jessica', 'charlie', 'thomas', 'robert', 'jennifer'
])

/**
 * Password policy configuration
 */
export interface PasswordPolicy {
  minLength: number
  requireUppercase: boolean
  requireLowercase: boolean
  requireNumbers: boolean
  requireSpecial: boolean
  rejectCommon: boolean
}

export const DEFAULT_PASSWORD_POLICY: PasswordPolicy = {
  minLength: 8,
  requireUppercase: true,
  requireLowercase: true,
  requireNumbers: true,
  requireSpecial: true,
  rejectCommon: true
}

/**
 * Password validation result
 */
export interface PasswordValidationResult {
  valid: boolean
  errors: string[]
  strength: 'weak' | 'fair' | 'good' | 'strong'
  score: number
}

/**
 * Validate password against policy
 */
export function validatePassword(
  password: string,
  policy: PasswordPolicy = DEFAULT_PASSWORD_POLICY
): PasswordValidationResult {
  const errors: string[] = []
  let score = 0
  const specialCharRegex = /[^A-Za-z0-9]/

  // Length check
  if (password.length < policy.minLength) {
    errors.push(`Password must be at least ${policy.minLength} characters`)
  } else {
    score += 1
    if (password.length >= 12) score += 1
    if (password.length >= 16) score += 1
  }

  // Uppercase check
  if (policy.requireUppercase && !/[A-Z]/.test(password)) {
    errors.push('Password must contain at least one uppercase letter')
  } else if (/[A-Z]/.test(password)) {
    score += 1
  }

  // Lowercase check
  if (policy.requireLowercase && !/[a-z]/.test(password)) {
    errors.push('Password must contain at least one lowercase letter')
  } else if (/[a-z]/.test(password)) {
    score += 1
  }

  // Number check
  if (policy.requireNumbers && !/\d/.test(password)) {
    errors.push('Password must contain at least one number')
  } else if (/\d/.test(password)) {
    score += 1
  }

  // Special character check
  if (policy.requireSpecial && !specialCharRegex.test(password)) {
    errors.push('Password must contain at least one special character')
  } else if (specialCharRegex.test(password)) {
    score += 1
  }

  // Common password check
  if (policy.rejectCommon) {
    const lowerPassword = password.toLowerCase()
    if (COMMON_PASSWORDS.has(lowerPassword)) {
      errors.push('Password is too common')
      score = Math.max(0, score - 2)
    }
  }

  // Additional checks for bonus points
  // Variety of character types
  const charTypes = [
    /[a-z]/.test(password),
    /[A-Z]/.test(password),
    /\d/.test(password),
    specialCharRegex.test(password)
  ].filter(Boolean).length

  if (charTypes === 4) score += 1

  // No repeated characters (3 in a row)
  if (!/(.)\1{2,}/.test(password)) score += 1

  // Determine strength
  let strength: 'weak' | 'fair' | 'good' | 'strong'
  if (score <= 3) {
    strength = 'weak'
  } else if (score <= 5) {
    strength = 'fair'
  } else if (score <= 7) {
    strength = 'good'
  } else {
    strength = 'strong'
  }

  return {
    valid: errors.length === 0,
    errors,
    strength,
    score: Math.min(10, score)
  }
}

/**
 * Calculate password strength (0-100)
 */
export function calculatePasswordStrength(password: string): number {
  const result = validatePassword(password)
  return Math.min(100, result.score * 10)
}

/**
 * Generate a secure random password
 */
export function generateSecurePassword(length: number = 16): string {
  const uppercase = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'
  const lowercase = 'abcdefghijklmnopqrstuvwxyz'
  const numbers = '0123456789'
  const special = '!@#$%^&*()_+-='
  const all = uppercase + lowercase + numbers + special

  let password = ''

  // Ensure at least one of each required type
  password += uppercase[Math.floor(Math.random() * uppercase.length)]
  password += lowercase[Math.floor(Math.random() * lowercase.length)]
  password += numbers[Math.floor(Math.random() * numbers.length)]
  password += special[Math.floor(Math.random() * special.length)]

  // Fill the rest randomly
  for (let i = password.length; i < length; i++) {
    const randomIndex = randomBytes(1)[0] % all.length
    password += all[randomIndex]
  }

  // Shuffle the password
  const chars = password.split('')
  for (let i = chars.length - 1; i > 0; i--) {
    const j = randomBytes(1)[0] % (i + 1)
    ;[chars[i], chars[j]] = [chars[j], chars[i]]
  }

  return chars.join('')
}

/**
 * Hash a password using scrypt
 * Returns format: salt:hash (both hex encoded)
 */
export function hashPassword(password: string): string {
  const salt = randomBytes(SALT_LENGTH).toString('hex')
  const hash = scryptSync(password, salt, KEY_LENGTH).toString('hex')
  return `${salt}:${hash}`
}

/**
 * Verify a password against a stored hash
 * Uses timing-safe comparison to prevent timing attacks
 */
export function verifyPassword(password: string, storedHash: string): boolean {
  try {
    const [salt, hash] = storedHash.split(':')
    if (!salt || !hash) return false

    const hashBuffer = Buffer.from(hash, 'hex')
    const suppliedHashBuffer = scryptSync(password, salt, KEY_LENGTH)

    return timingSafeEqual(hashBuffer, suppliedHashBuffer)
  } catch {
    return false
  }
}

/**
 * Generate a secure random session ID
 */
export function generateSessionId(): string {
  return randomBytes(32).toString('hex')
}

/**
 * Check if password has been seen in a breach (placeholder)
 * In production, integrate with Have I Been Pwned API
 */
export async function checkPasswordBreach(password: string): Promise<boolean> {
  // This is a placeholder - in production, integrate with HIBP API
  // using k-anonymity to not expose the actual password
  // https://haveibeenpwned.com/API/v3#PwnedPasswords
  return COMMON_PASSWORDS.has(password.toLowerCase())
}
