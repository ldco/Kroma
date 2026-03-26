/**
 * WebSocket Authentication
 *
 * Validates session cookies on WebSocket upgrade requests.
 * Reuses the same session lookup logic as the HTTP auth middleware.
 *
 * Usage:
 *   import { authenticateWebSocket } from './auth'
 *
 *   // In WebSocket handler
 *   const user = await authenticateWebSocket(request)
 *   if (!user) {
 *     // Anonymous connection (allowed for public rooms)
 *   }
 */
import { eq, and, gt } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import type { WSUserContext } from './types'
import { logger } from '../../../utils/logger'
import { ROLE_HIERARCHY } from '../../../utils/roles'

/**
 * Parse cookies from request headers
 */
function parseCookies(cookieHeader: string | null): Record<string, string> {
  if (!cookieHeader) return {}

  const cookies: Record<string, string> = {}
  const pairs = cookieHeader.split(';')

  for (const pair of pairs) {
    const [name, ...rest] = pair.trim().split('=')
    if (name && rest.length > 0) {
      cookies[name] = decodeURIComponent(rest.join('='))
    }
  }

  return cookies
}

/**
 * Authenticate WebSocket connection from upgrade request
 *
 * @param request - The HTTP upgrade request
 * @returns User context if authenticated, null otherwise
 */
export async function authenticateWebSocket(
  request: Request
): Promise<WSUserContext | null> {
  try {
    // Extract session cookie from request headers
    const cookieHeader = request.headers.get('cookie')
    const cookies = parseCookies(cookieHeader)
    const sessionId = cookies['pm-session']

    if (!sessionId) {
      logger.debug('WebSocket auth: No session cookie')
      return null
    }

    const db = useDatabase()

    // Find valid session (not expired)
    const session = db
      .select()
      .from(schema.sessions)
      .where(
        and(
          eq(schema.sessions.id, sessionId),
          gt(schema.sessions.expiresAt, new Date())
        )
      )
      .get()

    if (!session) {
      logger.debug('WebSocket auth: Session not found or expired')
      return null
    }

    // Get user
    const user = db
      .select({
        id: schema.users.id,
        email: schema.users.email,
        name: schema.users.name,
        role: schema.users.role
      })
      .from(schema.users)
      .where(eq(schema.users.id, session.userId))
      .get()

    if (!user) {
      logger.debug({ userId: session.userId }, 'WebSocket auth: User not found')
      return null
    }

    logger.debug({ userId: user.id }, 'WebSocket auth: Authenticated')

    return {
      userId: user.id,
      email: user.email,
      name: user.name,
      role: user.role as WSUserContext['role']
    }
  } catch (error) {
    logger.error({ error }, 'WebSocket auth error')
    return null
  }
}

/**
 * Validate that a user has the required role
 */
export function requireWSRole(
  user: WSUserContext | null,
  minRole: WSUserContext['role']
): boolean {
  if (!user) return false

  const userLevel = ROLE_HIERARCHY[user.role] ?? -1
  const requiredLevel = ROLE_HIERARCHY[minRole] ?? 999

  return userLevel >= requiredLevel
}
