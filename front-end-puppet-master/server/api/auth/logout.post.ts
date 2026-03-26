/**
 * Logout API Endpoint
 *
 * POST /api/auth/logout
 * Clears session cookie, CSRF cookie, and deletes session from database
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { deleteCsrfCookie } from '../../utils/csrf'
import { audit } from '../../utils/audit'

export default defineEventHandler(async event => {
  const sessionId = getCookie(event, 'pm-session')
  let userId: number | undefined

  if (sessionId) {
    const db = useDatabase()

    // Get user ID before deleting session for audit
    const session = db
      .select({ userId: schema.sessions.userId })
      .from(schema.sessions)
      .where(eq(schema.sessions.id, sessionId))
      .get()

    if (session) {
      userId = session.userId
    }

    // Delete session from database
    db.delete(schema.sessions).where(eq(schema.sessions.id, sessionId)).run()
  }

  // Log logout
  if (userId) {
    await audit.logout(event, userId)
  }

  // Clear session cookie
  deleteCookie(event, 'pm-session', {
    path: '/'
  })

  // Clear CSRF cookie
  deleteCsrfCookie(event)

  return {
    success: true,
    data: null
  }
})
