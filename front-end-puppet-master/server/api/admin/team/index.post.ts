/**
 * POST /api/admin/team
 *
 * Create a new team member.
 * Content stored in centralized translations table.
 * Supports multiple locales in one request.
 */
import { eq } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'
import { escapeHtml } from '../../../utils/sanitize'
import { safeJsonParseOrNull } from '../../../utils/json'

const createMemberSchema = z.object({
  slug: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/),
  name: z.string().min(1).max(100),
  photoUrl: z.string().max(500).nullish(),
  hoverPhotoUrl: z.string().max(500).nullish(),
  email: z.string().email().max(255).nullish(),
  phone: z.string().max(30).nullish(),
  department: z.string().max(50).nullish(),
  socialLinks: z.record(z.string()).nullish(),
  order: z.number().int().min(0).default(0),
  published: z.boolean().default(true),
  translations: z.record(z.object({
    position: z.string().max(100),
    bio: z.string().max(2000)
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const body = await readBody(event)

  const result = createMemberSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, ...data } = result.data

  // Check if slug exists
  const existing = db
    .select()
    .from(schema.teamMembers)
    .where(eq(schema.teamMembers.slug, data.slug))
    .get()

  if (existing) {
    throw createError({
      statusCode: 409,
      message: 'Slug already exists'
    })
  }

  // Insert team member metadata only
  const [member] = db
    .insert(schema.teamMembers)
    .values({
      slug: data.slug,
      name: escapeHtml(data.name),
      photoUrl: data.photoUrl || null,
      hoverPhotoUrl: data.hoverPhotoUrl || null,
      email: data.email || null,
      phone: data.phone || null,
      department: data.department || null,
      socialLinks: data.socialLinks ? JSON.stringify(data.socialLinks) : null,
      order: data.order,
      published: data.published
    })
    .returning()

  if (!member) {
    throw createError({ statusCode: 500, message: 'Failed to create team member' })
  }

  // Store translations in centralized table
  if (translations) {
    const translationValues = []

    for (const [locale, trans] of Object.entries(translations)) {
      if (trans.position) {
        translationValues.push({
          locale,
          key: `team.${member.id}.position`,
          value: escapeHtml(trans.position)
        })
      }

      if (trans.bio) {
        translationValues.push({
          locale,
          key: `team.${member.id}.bio`,
          value: escapeHtml(trans.bio)
        })
      }
    }

    if (translationValues.length > 0) {
      db.insert(schema.translations)
        .values(translationValues)
        .run()
    }
  }

  return {
    success: true,
    data: {
      member: {
        ...member,
        socialLinks: safeJsonParseOrNull(member.socialLinks)
      }
    }
  }
})
