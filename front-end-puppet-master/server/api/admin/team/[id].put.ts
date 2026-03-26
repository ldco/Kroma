/**
 * PUT /api/admin/team/:id
 *
 * Update a team member.
 * Content stored in centralized translations table.
 * Supports multiple locales in one request.
 */
import { eq, and, ne } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'
import { escapeHtml } from '../../../utils/sanitize'

const updateMemberSchema = z.object({
  slug: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/).optional(),
  name: z.string().min(1).max(100).optional(),
  photoUrl: z.string().max(500).nullish(),
  hoverPhotoUrl: z.string().max(500).nullish(),
  email: z.string().email().max(255).nullish(),
  phone: z.string().max(30).nullish(),
  department: z.string().max(50).nullish(),
  socialLinks: z.record(z.string()).nullish(),
  order: z.number().int().min(0).optional(),
  published: z.boolean().optional(),
  translations: z.record(z.object({
    position: z.string().max(100),
    bio: z.string().max(2000)
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)

  const result = updateMemberSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  // Check if member exists
  const existing = db
    .select()
    .from(schema.teamMembers)
    .where(eq(schema.teamMembers.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Team member not found' })
  }

  const { translations, ...data } = result.data

  // Check slug uniqueness (excluding current)
  if (data.slug && data.slug !== existing.slug) {
    const slugExists = db
      .select()
      .from(schema.teamMembers)
      .where(and(eq(schema.teamMembers.slug, data.slug), ne(schema.teamMembers.id, id)))
      .get()

    if (slugExists) {
      throw createError({ statusCode: 409, message: 'Slug already exists' })
    }
  }

  // Build update values
  const updateValues: Record<string, unknown> = {}

  if (data.slug !== undefined) updateValues.slug = data.slug
  if (data.name !== undefined) updateValues.name = escapeHtml(data.name)
  if (data.photoUrl !== undefined) updateValues.photoUrl = data.photoUrl
  if (data.hoverPhotoUrl !== undefined) updateValues.hoverPhotoUrl = data.hoverPhotoUrl
  if (data.email !== undefined) updateValues.email = data.email
  if (data.phone !== undefined) updateValues.phone = data.phone
  if (data.department !== undefined) updateValues.department = data.department
  if (data.socialLinks !== undefined) updateValues.socialLinks = data.socialLinks ? JSON.stringify(data.socialLinks) : null
  if (data.order !== undefined) updateValues.order = data.order
  if (data.published !== undefined) updateValues.published = data.published

  if (Object.keys(updateValues).length > 0) {
    db.update(schema.teamMembers)
      .set(updateValues)
      .where(eq(schema.teamMembers.id, id))
      .run()
  }

  // Update translations in centralized table
  if (translations) {
    for (const [locale, trans] of Object.entries(translations)) {
      const positionKey = `team.${id}.position`
      const bioKey = `team.${id}.bio`

      // Upsert position
      const existingPosition = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, positionKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingPosition) {
        db.update(schema.translations)
          .set({ value: trans.position ? escapeHtml(trans.position) : '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingPosition.id))
          .run()
      } else if (trans.position) {
        db.insert(schema.translations)
          .values({ locale, key: positionKey, value: escapeHtml(trans.position) })
          .run()
      }

      // Upsert bio
      const existingBio = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, bioKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingBio) {
        db.update(schema.translations)
          .set({ value: trans.bio ? escapeHtml(trans.bio) : '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingBio.id))
          .run()
      } else if (trans.bio) {
        db.insert(schema.translations)
          .values({ locale, key: bioKey, value: escapeHtml(trans.bio) })
          .run()
      }
    }
  }

  return {
    success: true,
    data: {}
  }
})
