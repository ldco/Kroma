/**
 * Structured Logging Utility (HIGH-03)
 *
 * Uses Pino for high-performance, structured JSON logging.
 * - Production: JSON output for log aggregators (Datadog, Loki, CloudWatch)
 * - Development: Pretty colored output for readability
 * - All environments: Logs also captured in memory buffer for admin health page
 *
 * Usage:
 * import { logger } from '../utils/logger'
 * logger.info('User logged in', { userId: 123 })
 * logger.error('Database error', { error: err.message, stack: err.stack })
 *
 * Log levels (in order of severity):
 * - trace: Very detailed debugging (usually disabled)
 * - debug: Debugging information
 * - info: Normal operations
 * - warn: Warning conditions
 * - error: Error conditions
 * - fatal: Critical errors
 */
import pino from 'pino'
import { createBufferDestination } from './logBuffer'

// Check environment
const isProduction = process.env.NODE_ENV === 'production'
const logLevel = process.env.LOG_LEVEL || (isProduction ? 'info' : 'debug')

// Create multistream destination for production
// In production, write to both stdout and the in-memory buffer
const destinations = isProduction
  ? pino.multistream([
      { stream: process.stdout },
      { stream: createBufferDestination() }
    ])
  : undefined

// Create logger instance
export const logger = pino(
  {
    level: logLevel,

    // Base context added to all logs
    base: {
      service: 'puppetmaster2',
      version: process.env.npm_package_version || '1.0.0',
      environment: process.env.NODE_ENV || 'development'
    },

    // Timestamp format
    timestamp: pino.stdTimeFunctions.isoTime,

    // Development: Pretty output with pino-pretty (can't use multistream with transport)
    ...(!isProduction && {
      transport: {
        target: 'pino-pretty',
        options: {
          colorize: true,
          translateTime: 'HH:MM:ss',
          ignore: 'pid,hostname,service,version,environment'
        }
      }
    }),

    // Production: use hooks to capture logs to buffer
    ...(isProduction && {
      hooks: {
        logMethod(inputArgs, method, _level) {
          // Call original method
          return method.apply(this, inputArgs)
        }
      }
    })
  },
  // Pass multistream destination for production
  destinations
)

/**
 * Create a child logger with pre-set context
 * Useful for adding request-specific context
 */
export function createChildLogger(context: Record<string, unknown>) {
  return logger.child(context)
}

/**
 * Request logger middleware helper
 * Creates a child logger with request context
 */
export function createRequestLogger(event: { node: { req: { method?: string; url?: string } } }) {
  return logger.child({
    method: event.node.req.method,
    url: event.node.req.url
  })
}
