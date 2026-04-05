/**
 * Admin Stats API
 *
 * GET /api/admin/stats
 * Returns dashboard statistics for the admin panel.
 * Requires authentication (enforced by middleware for /api/admin/*).
 */
import { useDatabase, schema } from '../../database/client'
import { count, eq } from 'drizzle-orm'

export default defineEventHandler(async _event => {
  const db = useDatabase()

  // Get total contact submissions
  const contactsResult = db.select({ count: count() }).from(schema.contactSubmissions).get()

  // Get unread messages count
  const unreadResult = db
    .select({ count: count() })
    .from(schema.contactSubmissions)
    .where(eq(schema.contactSubmissions.read, false))
    .get()

  return {
    success: true,
    data: {
      contactSubmissions: contactsResult?.count ?? 0,
      unreadMessages: unreadResult?.count ?? 0
    }
  }
})
