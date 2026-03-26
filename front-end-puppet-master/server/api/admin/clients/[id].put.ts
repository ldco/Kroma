/**
 * PUT /api/admin/clients/:id
 *
 * Update a client.
 */
import { eq, and, ne } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)
  const { slug, name, logoUrl, websiteUrl, category, featured, order } = body

  if (!slug || !name) {
    throw createError({ statusCode: 400, message: 'Slug and name are required' })
  }

  // Check if client exists
  const existing = db
    .select()
    .from(schema.clients)
    .where(eq(schema.clients.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Client not found' })
  }

  // Check slug uniqueness (excluding current)
  const slugExists = db
    .select()
    .from(schema.clients)
    .where(
      and(
        eq(schema.clients.slug, slug),
        ne(schema.clients.id, id)
      )
    )
    .get()

  if (slugExists) {
    throw createError({ statusCode: 409, message: 'Slug already exists' })
  }

  // Update client
  db.update(schema.clients)
    .set({
      slug,
      name,
      logoUrl: logoUrl || null,
      websiteUrl: websiteUrl || null,
      category: category || null,
      featured: featured ?? false,
      order: order ?? 0,
      updatedAt: new Date().toISOString()
    })
    .where(eq(schema.clients.id, id))
    .run()

  return {
    success: true,
    data: null
  }
})
