/**
 * Auth Middleware
 *
 * Functions:
 * 1. Populates event.context.session and event.context.user for ALL /api/* routes (if logged in)
 * 2. ENFORCES auth for /api/admin/* routes (throws 401 if not logged in)
 * 3. ENFORCES section-level RBAC for /api/admin/* routes via shared permission helpers.
 *
 * This allows endpoints to optionally check auth (event.context.session)
 * while admin routes are always protected with section-aware authorization.
 */
import { eq, and, gt } from 'drizzle-orm'
import { useDatabase, schema } from '../database/client'
import { requireAdminRouteAccessByPath } from '../utils/admin-rbac'

export default defineEventHandler(async event => {
  const path = getRequestURL(event).pathname

  // Only process /api/* routes
  if (!path.startsWith('/api')) {
    return
  }

  const sessionId = getCookie(event, 'pm-session')
  const isAdminRoute = path.startsWith('/api/admin')

  // No session cookie
  if (!sessionId) {
    // For admin routes, require auth
    if (isAdminRoute) {
      throw createError({
        statusCode: 401,
        message: 'Authentication required'
      })
    }
    // For other routes, just don't set session context
    return
  }

  const db = useDatabase()

  // Find valid session
  const session = db
    .select()
    .from(schema.sessions)
    .where(and(eq(schema.sessions.id, sessionId), gt(schema.sessions.expiresAt, new Date())))
    .get()

  if (!session) {
    deleteCookie(event, 'pm-session', { path: '/' })
    if (isAdminRoute) {
      throw createError({
        statusCode: 401,
        message: 'Session expired'
      })
    }
    return
  }

  // Get user (including roleId for permission checks)
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
    if (isAdminRoute) {
      throw createError({
        statusCode: 401,
        message: 'User not found'
      })
    }
    return
  }

  // Attach session and user to event context for use in handlers
  event.context.session = { userId: user.id }
  event.context.user = user

  // Section-level route protection for all /api/admin/* endpoints.
  if (isAdminRoute) {
    await requireAdminRouteAccessByPath(user, path)
  }
})
