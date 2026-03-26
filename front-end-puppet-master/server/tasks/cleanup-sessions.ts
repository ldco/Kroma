/**
 * Session Cleanup Task (MED-02)
 *
 * Scheduled task that removes expired sessions from the database.
 * Runs every hour to prevent table bloat.
 *
 * Usage:
 * - Automatically runs via Nitro scheduler
 * - Can be triggered manually via API if needed
 */
import { lt } from 'drizzle-orm'
import { useDatabase, schema } from '../database/client'
import { logger } from '../utils/logger'

export default defineTask({
  meta: {
    name: 'cleanup-sessions',
    description: 'Remove expired sessions from database'
  },

  async run() {
    const startTime = Date.now()
    const db = useDatabase()

    try {
      // Delete all sessions where expiresAt is in the past
      const result = db
        .delete(schema.sessions)
        .where(lt(schema.sessions.expiresAt, new Date()))
        .run()

      const duration = Date.now() - startTime

      logger.info(
        { deleted: result.changes, durationMs: duration },
        'Session cleanup completed'
      )

      return {
        result: 'success',
        deleted: result.changes,
        durationMs: duration
      }
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error)
      logger.error(
        { error: message },
        'Session cleanup failed'
      )

      return {
        result: 'error',
        error: message
      }
    }
  }
})
