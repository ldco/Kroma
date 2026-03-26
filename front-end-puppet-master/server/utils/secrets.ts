/**
 * Secret value helpers for settings persisted in the database.
 *
 * Storage envelope format:
 *   enc:v1:{ivHex}:{authTagHex}:{cipherHex}
 *
 * Security:
 * - In production, requires PM_SETTINGS_ENCRYPTION_KEY or SETTINGS_ENCRYPTION_KEY env var
 * - No fallback to hardcoded key in production (fails fast)
 * - Development/test mode allows fallback for local development
 */
import { createCipheriv, createDecipheriv, randomBytes, scryptSync } from 'node:crypto'

const SETTINGS_SECRET_PREFIX = 'enc:v1:'
const ENCRYPTION_ALGORITHM = 'aes-256-gcm'
const IV_LENGTH = 16
const AUTH_TAG_LENGTH = 16

function getSettingsEncryptionKey(): Buffer {
  const envKey = process.env.SETTINGS_ENCRYPTION_KEY || process.env.PM_SETTINGS_ENCRYPTION_KEY

  if (envKey) {
    return scryptSync(envKey, 'pm-settings-secret-salt', 32)
  }

  // Comment 3: In production, fail fast if encryption key is not provided
  // This prevents silent fallback to weak hardcoded key
  if (process.env.NODE_ENV === 'production') {
    throw new Error(
      'FATAL: PM_SETTINGS_ENCRYPTION_KEY or SETTINGS_ENCRYPTION_KEY environment variable is required in production. ' +
      'Set a strong, unique key (minimum 32 characters) before starting the server.'
    )
  }

  // Development/test fallback for local environments without explicit key provisioning.
  // WARNING: This key is not secure and should never be used in production.
  console.warn(
    '⚠️  Using development encryption key - DO NOT use in production. ' +
    'Set PM_SETTINGS_ENCRYPTION_KEY environment variable.'
  )
  return scryptSync('pm-development-settings-key', 'pm-settings-secret-salt', 32)
}

export function isEncryptedSecretValue(value: string | null | undefined): value is string {
  return typeof value === 'string' && value.startsWith(SETTINGS_SECRET_PREFIX)
}

export function encryptSecretValue(plainText: string): string {
  const key = getSettingsEncryptionKey()
  const iv = randomBytes(IV_LENGTH)
  const cipher = createCipheriv(ENCRYPTION_ALGORITHM, key, iv)

  let encrypted = cipher.update(plainText, 'utf8', 'hex')
  encrypted += cipher.final('hex')
  const authTag = cipher.getAuthTag()

  return `${SETTINGS_SECRET_PREFIX}${iv.toString('hex')}:${authTag.toString('hex')}:${encrypted}`
}

export function decryptSecretValue(value: string): string {
  if (!isEncryptedSecretValue(value)) {
    return value
  }

  const payload = value.slice(SETTINGS_SECRET_PREFIX.length)
  const parts = payload.split(':')
  if (parts.length !== 3) {
    throw new Error('Invalid encrypted secret payload format')
  }

  const [ivHex, authTagHex, cipherHex] = parts
  if (!ivHex || !authTagHex || !cipherHex) {
    throw new Error('Invalid encrypted secret payload')
  }

  const iv = Buffer.from(ivHex, 'hex')
  const authTag = Buffer.from(authTagHex, 'hex')
  if (iv.length !== IV_LENGTH || authTag.length !== AUTH_TAG_LENGTH) {
    throw new Error('Invalid encrypted secret payload length')
  }

  const key = getSettingsEncryptionKey()
  const decipher = createDecipheriv(ENCRYPTION_ALGORITHM, key, iv)
  decipher.setAuthTag(authTag)

  let decrypted = decipher.update(cipherHex, 'hex', 'utf8')
  decrypted += decipher.final('utf8')
  return decrypted
}
