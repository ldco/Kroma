/**
 * POST /api/portfolios
 *
 * Creates a new portfolio.
 * Requires admin authentication.
 * Supports multiple locales for name/description.
 */
import { z } from 'zod'
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { requireAdminSectionAccess } from '../../utils/admin-rbac'
import { escapeHtml } from '../../utils/sanitize'

// Validation schema
const createSchema = z.object({
  slug: z
    .string()
    .min(1)
    .max(100)
    .regex(/^[a-z0-9-]+$/, 'Slug must be lowercase alphanumeric with dashes'),
  name: z.string().min(1).max(200),
  description: z.string().max(1000).optional(),
  type: z.enum(['gallery', 'case_study']),
  coverImageUrl: z.string().optional().nullable(),
  coverThumbnailUrl: z.string().optional().nullable(),
  order: z.number().int().optional(),
  published: z.boolean().optional(),
  translations: z.record(z.object({
    name: z.string().max(200).nullable().optional(),
    description: z.string().max(1000).nullable().optional()
  })).optional()
})

export default defineEventHandler(async event => {
  await requireAdminSectionAccess(event.context.user, 'portfolios')

  // Parse and validate request body
  const body = await readBody(event)
  const result = createSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, ...data } = result.data
  const db = useDatabase()

  // Check if slug already exists
  const existing = db
    .select()
    .from(schema.portfolios)
    .where(eq(schema.portfolios.slug, data.slug))
    .get()

  if (existing) {
    throw createError({
      statusCode: 409,
      statusMessage: 'Slug already exists'
    })
  }

  // Insert new portfolio
  const insertResult = db
    .insert(schema.portfolios)
    .values({
      slug: data.slug,
      name: escapeHtml(data.name),
      description: data.description ? escapeHtml(data.description) : null,
      type: data.type,
      coverImageUrl: data.coverImageUrl || null,
      coverThumbnailUrl: data.coverThumbnailUrl || null,
      order: data.order || 0,
      published: data.published || false
    })
    .run()

  const portfolioId = Number(insertResult.lastInsertRowid)

  // Store translations in centralized table
  if (translations) {
    const translationValues = []

    for (const [locale, trans] of Object.entries(translations)) {
      if (trans.name) {
        translationValues.push({
          locale,
          key: `portfolio.${portfolioId}.name`,
          value: escapeHtml(trans.name)
        })
      }

      if (trans.description) {
        translationValues.push({
          locale,
          key: `portfolio.${portfolioId}.description`,
          value: escapeHtml(trans.description)
        })
      }
    }

    if (translationValues.length > 0) {
      db.insert(schema.translations)
        .values(translationValues)
        .run()
    }
  }

  // Fetch the created portfolio
  const newPortfolio = db
    .select()
    .from(schema.portfolios)
    .where(eq(schema.portfolios.id, portfolioId))
    .get()

  return {
    success: true,
    data: {
      portfolio: newPortfolio
    }
  }
})
