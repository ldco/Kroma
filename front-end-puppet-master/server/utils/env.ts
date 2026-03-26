/**
 * Environment Variable Validation (HIGH-08)
 *
 * Validates required environment variables at startup.
 * Fails fast if critical variables are missing.
 *
 * Usage:
 * import { validateEnv, env } from '../utils/env'
 * validateEnv() // Call once at startup
 * console.log(env.DATABASE_URL) // Type-safe access
 */
import { z } from 'zod'
import { logger } from './logger'

// Helper: treat empty strings and undefined as "not set" (for optional fields)
const optionalString = z.preprocess(
  val => (val === '' || val === undefined ? undefined : val),
  z.string().optional()
)
const optionalEmail = z.preprocess(
  val => (val === '' || val === undefined ? undefined : val),
  z.string().email().optional()
)
const optionalUrl = z.preprocess(
  val => (val === '' || val === undefined ? undefined : val),
  z.string().url().optional()
)
const optionalNumber = z.preprocess(
  val => (val === '' || val === undefined ? undefined : Number(val)),
  z.number().optional()
)

// Define environment variable schema
const envSchema = z.object({
  // Node environment
  NODE_ENV: z.enum(['development', 'production', 'test']).default('development'),

  // Server
  HOST: z.string().default('0.0.0.0'),
  PORT: z.string().default('3000').transform(Number).pipe(z.number().min(1).max(65535)),

  // Database
  DATABASE_URL: z.string().min(1).default('/app/data/sqlite.db'),

  // Logging
  LOG_LEVEL: z.enum(['trace', 'debug', 'info', 'warn', 'error', 'fatal']).default('info'),

  // Email (optional - empty strings treated as not set)
  SMTP_HOST: optionalString,
  SMTP_PORT: optionalNumber,
  SMTP_USER: optionalString,
  SMTP_PASS: optionalString,
  SMTP_FROM: optionalEmail,

  // Telegram (optional)
  // Bot token is expected in settings (contact.telegramBotToken); env token remains as legacy fallback.
  TELEGRAM_BOT_TOKEN: optionalString,
  TELEGRAM_CHAT_ID: optionalString,

  // External API (optional)
  API_BASE_URL: optionalUrl,
  API_CLIENT_ID: optionalString,
  API_CLIENT_SECRET: optionalString
})

// Parsed environment type
export type Env = z.infer<typeof envSchema>

// Validated environment (populated after validation)
let _env: Env | null = null

/**
 * Get validated environment variables
 * Throws if validateEnv() hasn't been called
 */
export function getEnv(): Env {
  if (!_env) {
    // Auto-validate on first access
    validateEnv()
  }
  return _env!
}

/**
 * Validate environment variables at startup
 * Call this early in application initialization
 */
export function validateEnv(): void {
  if (_env) return // Already validated

  try {
    _env = envSchema.parse(process.env)

    logger.info(
      {
        nodeEnv: _env.NODE_ENV,
        hasSmtp: !!_env.SMTP_HOST,
        hasTelegramDestination: !!_env.TELEGRAM_CHAT_ID,
        hasExternalApi: !!_env.API_BASE_URL
      },
      'Environment validated'
    )

    // Warn about missing optional but recommended variables
    if (_env.NODE_ENV === 'production') {
      const warnings: string[] = []

      if (!_env.SMTP_HOST) {
        warnings.push('SMTP not configured - email notifications disabled')
      }
      if (!_env.TELEGRAM_CHAT_ID) {
        warnings.push('Telegram chat id not configured - Telegram notifications disabled')
      }

      warnings.forEach(w => logger.warn(w))
    }
  } catch (error) {
    if (error instanceof z.ZodError) {
      const issues = error.issues.map(i => `${i.path.join('.')}: ${i.message}`)
      logger.fatal({ issues }, 'Environment validation failed')

      // In production, exit immediately
      if (process.env.NODE_ENV === 'production') {
        console.error('FATAL: Environment validation failed:')
        issues.forEach(i => console.error(`  - ${i}`))
        process.exit(1)
      }

      throw new Error(`Environment validation failed: ${issues.join(', ')}`)
    }
    throw error
  }
}

/**
 * Check if a feature is enabled based on environment
 */
export const features = {
  get email(): boolean {
    const e = getEnv()
    return !!(e.SMTP_HOST && e.SMTP_USER && e.SMTP_PASS && e.SMTP_FROM)
  },

  get telegram(): boolean {
    const e = getEnv()
    return !!e.TELEGRAM_CHAT_ID
  },

  get externalApi(): boolean {
    const e = getEnv()
    return !!(e.API_BASE_URL && e.API_CLIENT_ID && e.API_CLIENT_SECRET)
  }
}

/**
 * Convenience export for direct env access
 * Usage: import { env } from '../utils/env'
 */
export const env = new Proxy({} as Env, {
  get(_, prop: string) {
    return getEnv()[prop as keyof Env]
  }
})
