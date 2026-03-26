/**
 * GET /api/clients
 *
 * Returns list of published clients/sponsors/partners.
 */
import { eq, asc, and } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)

  const conditions = [eq(schema.clients.published, true)]

  if (query.category && typeof query.category === 'string') {
    conditions.push(eq(schema.clients.category, query.category as 'client' | 'sponsor' | 'partner'))
  }

  if (query.featured === 'true') {
    conditions.push(eq(schema.clients.featured, true))
  }

  const limit = query.limit ? parseInt(query.limit as string) : undefined

  let clients = db
    .select()
    .from(schema.clients)
    .where(and(...conditions))
    .orderBy(asc(schema.clients.order))
    .all()

  if (limit) {
    clients = clients.slice(0, limit)
  }

  return {
    success: true,
    data: clients
  }
})
