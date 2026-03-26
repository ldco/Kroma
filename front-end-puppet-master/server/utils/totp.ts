/**
 * TOTP (Time-based One-Time Password) Utilities
 *
 * Provides two-factor authentication using TOTP (RFC 6238).
 * Uses the otpauth library for TOTP generation and verification.
 *
 * Features:
 * - TOTP secret generation and verification
 * - QR code generation for authenticator apps
 * - Backup codes generation and validation
 * - Secure secret encryption for storage
 */
import { TOTP, Secret } from 'otpauth'
import QRCode from 'qrcode'
import { randomBytes, createCipheriv, createDecipheriv, scryptSync } from 'crypto'
import config from '~~/app/puppet-master.config'

// Encryption configuration for TOTP secrets
const ENCRYPTION_ALGORITHM = 'aes-256-gcm'
const IV_LENGTH = 16
const AUTH_TAG_LENGTH = 16
const SALT_LENGTH = 16

/**
 * Get encryption key from environment or derive from a secret
 * In production, this should be a secure, randomly generated key stored in env
 */
function getEncryptionKey(): Buffer {
  const envKey = process.env.TOTP_ENCRYPTION_KEY
  if (envKey) {
    // If provided, derive a key from the env variable
    return scryptSync(envKey, 'totp-salt', 32)
  }
  // Development fallback (not secure for production)
  return scryptSync('development-totp-key', 'totp-salt', 32)
}

/**
 * Generate a new TOTP secret
 * Returns a base32-encoded secret string
 */
export function generateTotpSecret(): string {
  const secret = new Secret({ size: 20 })
  return secret.base32
}

/**
 * Create a TOTP instance for a user
 */
function createTotpInstance(secret: string, userEmail: string): TOTP {
  const projectName = config.meta?.title || 'Puppet Master'
  return new TOTP({
    issuer: projectName,
    label: userEmail,
    algorithm: 'SHA1',
    digits: 6,
    period: 30,
    secret: Secret.fromBase32(secret)
  })
}

/**
 * Generate a QR code data URL for authenticator app setup
 */
export async function generateTotpQRCode(
  secret: string,
  userEmail: string
): Promise<string> {
  const totp = createTotpInstance(secret, userEmail)
  const uri = totp.toString()
  return QRCode.toDataURL(uri, {
    errorCorrectionLevel: 'M',
    margin: 2,
    width: 256
  })
}

/**
 * Get the TOTP URI for manual entry in authenticator apps
 */
export function getTotpUri(secret: string, userEmail: string): string {
  const totp = createTotpInstance(secret, userEmail)
  return totp.toString()
}

/**
 * Verify a TOTP code
 * Allows for a window of +/- 1 period (30 seconds) to account for time drift
 */
export function verifyTotpCode(secret: string, code: string, userEmail: string): boolean {
  const totp = createTotpInstance(secret, userEmail)

  // Validate code with a window of 1 (allows codes from -30s to +30s)
  const delta = totp.validate({ token: code, window: 1 })

  // delta is null if invalid, otherwise the time step difference
  return delta !== null
}

/**
 * Generate backup codes (10 codes, 8 characters each)
 * Returns both the plain codes (to show user once) and hashed codes (to store)
 */
export function generateBackupCodes(): { plain: string[]; hashed: string[] } {
  const codes: string[] = []
  const hashedCodes: string[] = []

  for (let i = 0; i < 10; i++) {
    // Generate a random 8-character alphanumeric code
    const code = randomBytes(4).toString('hex').toUpperCase()
    codes.push(code)
    // Hash the code for storage (using simple hash since these are one-time use)
    const hash = scryptSync(code, 'backup-code-salt', 32).toString('hex')
    hashedCodes.push(hash)
  }

  return { plain: codes, hashed: hashedCodes }
}

/**
 * Verify a backup code against stored hashed codes
 * Returns the index of the matched code, or -1 if not found
 */
export function verifyBackupCode(code: string, hashedCodes: string[]): number {
  const normalizedCode = code.toUpperCase().replace(/[\s-]/g, '')
  const codeHash = scryptSync(normalizedCode, 'backup-code-salt', 32).toString('hex')

  return hashedCodes.findIndex(hash => hash === codeHash)
}

/**
 * Encrypt a TOTP secret for database storage
 */
export function encryptTotpSecret(secret: string): string {
  const key = getEncryptionKey()
  const iv = randomBytes(IV_LENGTH)
  const cipher = createCipheriv(ENCRYPTION_ALGORITHM, key, iv)

  let encrypted = cipher.update(secret, 'utf8', 'hex')
  encrypted += cipher.final('hex')

  const authTag = cipher.getAuthTag()

  // Return format: iv:authTag:encryptedData (all hex)
  return `${iv.toString('hex')}:${authTag.toString('hex')}:${encrypted}`
}

/**
 * Decrypt a TOTP secret from database storage
 */
export function decryptTotpSecret(encryptedSecret: string): string {
  const key = getEncryptionKey()
  const parts = encryptedSecret.split(':')

  if (parts.length !== 3) {
    throw new Error('Invalid encrypted secret format')
  }

  const [ivHex, authTagHex, encrypted] = parts
  const iv = Buffer.from(ivHex, 'hex')
  const authTag = Buffer.from(authTagHex, 'hex')

  const decipher = createDecipheriv(ENCRYPTION_ALGORITHM, key, iv)
  decipher.setAuthTag(authTag)

  let decrypted = decipher.update(encrypted, 'hex', 'utf8')
  decrypted += decipher.final('utf8')

  return decrypted
}

/**
 * 2FA verification result type
 */
export interface TwoFactorVerifyResult {
  success: boolean
  method?: 'totp' | 'backup'
  backupCodeIndex?: number
  error?: string
}

/**
 * Verify a 2FA code (either TOTP or backup code)
 * Automatically detects if it's a TOTP code (6 digits) or backup code (8 chars)
 */
export function verify2faCode(
  code: string,
  encryptedSecret: string,
  hashedBackupCodes: string[],
  userEmail: string
): TwoFactorVerifyResult {
  const normalizedCode = code.replace(/[\s-]/g, '')

  // Check if it looks like a TOTP code (6 digits)
  if (/^\d{6}$/.test(normalizedCode)) {
    const secret = decryptTotpSecret(encryptedSecret)
    if (verifyTotpCode(secret, normalizedCode, userEmail)) {
      return { success: true, method: 'totp' }
    }
    return { success: false, error: 'Invalid verification code' }
  }

  // Otherwise, try as backup code
  const backupIndex = verifyBackupCode(normalizedCode, hashedBackupCodes)
  if (backupIndex >= 0) {
    return { success: true, method: 'backup', backupCodeIndex: backupIndex }
  }

  return { success: false, error: 'Invalid verification code' }
}
