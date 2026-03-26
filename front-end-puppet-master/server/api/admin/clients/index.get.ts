/**
 * GET /api/admin/clients
 *
 * List all clients for admin.
 */
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async () => {
  const db = useDatabase()

  const clients = db
    .select()
    .from(schema.clients)
    .orderBy(schema.clients.order)
    .all()

  return {
    success: true,
    data: clients,
    clients
  }
})
