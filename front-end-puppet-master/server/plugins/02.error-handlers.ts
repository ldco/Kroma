/**
 * Global Error Handlers Plugin
 *
 * Catches unhandled promise rejections and uncaught exceptions
 * to prevent silent failures and ensure proper logging.
 *
 * Behavior:
 * - unhandledRejection: Logged, process continues (may be recoverable)
 * - uncaughtException: Logged, process exits in production (enables supervisor restart)
 */
import { logger } from '../utils/logger'

export default defineNitroPlugin(() => {
  // Handle unhandled promise rejections
  process.on('unhandledRejection', (reason, _promise) => {
    logger.error(
      {
        reason: reason instanceof Error ? reason.message : String(reason),
        stack: reason instanceof Error ? reason.stack : undefined
      },
      'Unhandled promise rejection'
    )
    // Don't exit - many rejections are recoverable
  })

  // Handle uncaught exceptions (fatal in production)
  process.on('uncaughtException', (error) => {
    const isProduction = process.env.NODE_ENV === 'production'

    logger.error(
      {
        error: error.message,
        stack: error.stack
      },
      'Uncaught exception'
    )

    // GAP-010: Exit process in production to enable supervisor-based restarts
    // In development, log only to allow debugging
    if (isProduction) {
      logger.error('Uncaught exception in production - exiting process for clean restart')
      // Give logger time to flush, then exit
      setTimeout(() => {
        process.exit(1)
      }, 100)
    }
    // In development: don't exit, allow debugging
  })

  logger.info('Global error handlers registered')
})
