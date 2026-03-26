/**
 * POST /api/admin/clients
 *
 * Create a new client.
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const body = await readBody(event)

  const { slug, name, logoUrl, websiteUrl, category, featured, order } = body

  if (!slug || !name) {
    throw createError({ statusCode: 400, message: 'Slug and name are required' })
  }

  // Check slug uniqueness
  const existing = db
    .select()
    .from(schema.clients)
    .where(eq(schema.clients.slug, slug))
    .get()

  if (existing) {
    throw createError({ statusCode: 409, message: 'Slug already exists' })
  }

  // Insert client
  const result = db
    .insert(schema.clients)
    .values({
      slug,
      name,
      logoUrl: logoUrl || null,
      websiteUrl: websiteUrl || null,
      category: category || null,
      featured: featured ?? false,
      order: order ?? 0
    })
    .run()

  return {
    success: true,
    data: {
      id: result.lastInsertRowid
    },
    id: result.lastInsertRowid
  }
})
