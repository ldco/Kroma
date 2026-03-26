/**
 * PUT /api/admin/team/:id/translations
 *
 * Update translations for a team member.
 * Uses centralized translations table with key pattern: team.{id}.{field}
 */
import { eq, and } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../../database/client'
import { escapeHtml } from '../../../../utils/sanitize'

const updateTranslationSchema = z.object({
  locale: z.string().min(2).max(5),
  position: z.string().max(100).optional().nullable(),
  bio: z.string().max(2000).optional().nullable()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({
      statusCode: 400,
      message: 'Invalid ID'
    })
  }

  const body = await readBody(event)
  const result = updateTranslationSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { locale, position, bio } = result.data

  // Check if member exists
  const member = db
    .select()
    .from(schema.teamMembers)
    .where(eq(schema.teamMembers.id, id))
    .get()

  if (!member) {
    throw createError({
      statusCode: 404,
      message: 'Team member not found'
    })
  }

  // Upsert position translation
  if (position !== undefined) {
    const positionKey = `team.${id}.position`
    const existingPosition = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, positionKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingPosition) {
      db.update(schema.translations)
        .set({ value: position ? escapeHtml(position) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingPosition.id))
        .run()
    } else if (position) {
      db.insert(schema.translations)
        .values({ locale, key: positionKey, value: escapeHtml(position) })
        .run()
    }
  }

  // Upsert bio translation
  if (bio !== undefined) {
    const bioKey = `team.${id}.bio`
    const existingBio = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, bioKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingBio) {
      db.update(schema.translations)
        .set({ value: bio ? escapeHtml(bio) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingBio.id))
        .run()
    } else if (bio) {
      db.insert(schema.translations)
        .values({ locale, key: bioKey, value: escapeHtml(bio) })
        .run()
    }
  }

  return {
    success: true,
    data: {}
  }
})
