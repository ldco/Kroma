/**
 * Session API Endpoint
 *
 * GET /api/auth/session
 * Returns current user session if valid, null otherwise.
 * Also returns CSRF token for client-side storage.
 * Also returns user permissions for role-based UI.
 */
import { eq, and, gt } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { getCsrfToken, generateCsrfToken, setCsrfCookie, deleteCsrfCookie } from '../../utils/csrf'
import { getUserPermissions } from '../../utils/permissions'

export default defineEventHandler(async event => {
  const sessionId = getCookie(event, 'pm-session')

  if (!sessionId) {
    // No session, ensure CSRF cookie is cleared
    deleteCsrfCookie(event)
    return {
      success: true,
      data: { user: null, csrfToken: null },
      user: null,
      csrfToken: null
    }
  }

  const db = useDatabase()

  // Find valid session (not expired)
  const session = db
    .select()
    .from(schema.sessions)
    .where(and(eq(schema.sessions.id, sessionId), gt(schema.sessions.expiresAt, new Date())))
    .get()

  if (!session) {
    // Clear invalid cookies
    deleteCookie(event, 'pm-session', { path: '/' })
    deleteCsrfCookie(event)
    return {
      success: true,
      data: { user: null, csrfToken: null },
      user: null,
      csrfToken: null
    }
  }

  // Get user with roleId for permission lookup
  const user = db
    .select({
      id: schema.users.id,
      email: schema.users.email,
      name: schema.users.name,
      role: schema.users.role,
      roleId: schema.users.roleId
    })
    .from(schema.users)
    .where(eq(schema.users.id, session.userId))
    .get()

  if (!user) {
    // User deleted, clear session
    db.delete(schema.sessions).where(eq(schema.sessions.id, sessionId)).run()
    deleteCookie(event, 'pm-session', { path: '/' })
    deleteCsrfCookie(event)
    return {
      success: true,
      data: { user: null, csrfToken: null },
      user: null,
      csrfToken: null
    }
  }

  // Get or regenerate CSRF token
  let csrfToken = getCsrfToken(event)
  if (!csrfToken) {
    // No CSRF token exists, generate new one
    csrfToken = generateCsrfToken()
    setCsrfCookie(event, csrfToken)
  }

  // Get user permissions from their role
  const permissions = await getUserPermissions(user)

  const userData = {
    id: user.id,
    email: user.email,
    name: user.name,
    role: user.role
  }

  return {
    success: true,
    data: {
      user: userData,
      permissions,
      csrfToken
    },
    user: userData,
    permissions,
    csrfToken
  }
})
