/**
 * Environment Validation Plugin
 *
 * Runs at server startup to validate all required
 * environment variables before accepting requests.
 *
 * Comment 3: Enforces PM_SETTINGS_ENCRYPTION_KEY in production
 */
import { validateEnv } from '../utils/env'

export default defineNitroPlugin(() => {
  validateEnv()

  // Comment 3: Additional production check for encryption key
  if (process.env.NODE_ENV === 'production') {
    const encryptionKey = process.env.PM_SETTINGS_ENCRYPTION_KEY || process.env.SETTINGS_ENCRYPTION_KEY

    if (!encryptionKey) {
      console.error(
        'FATAL: PM_SETTINGS_ENCRYPTION_KEY or SETTINGS_ENCRYPTION_KEY is required in production.\n' +
        'This key encrypts sensitive settings (API keys, tokens) stored in the database.\n' +
        'Set a strong, unique key (minimum 32 characters) in your environment.\n' +
        'Example: PM_SETTINGS_ENCRYPTION_KEY=$(openssl rand -hex 32)'
      )
      process.exit(1)
    }

    // Warn if key is too short
    if (encryptionKey.length < 32) {
      console.warn(
        'WARNING: PM_SETTINGS_ENCRYPTION_KEY is less than 32 characters. ' +
        'For production security, use a key of at least 32 characters.'
      )
    }
  }
})
