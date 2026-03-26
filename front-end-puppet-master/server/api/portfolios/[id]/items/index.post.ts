/**
 * POST /api/portfolios/[id]/items
 *
 * Creates a new item in a portfolio.
 * Requires admin authentication.
 * Supports multiple locales for translatable fields.
 *
 * Supports different item types:
 * - image: Uploaded image (mediaUrl, thumbnailUrl, caption)
 * - video: Uploaded video (mediaUrl, thumbnailUrl, caption)
 * - link: External URL (mediaUrl, caption)
 * - case_study: Rich content (slug, title, description, content, tags, category)
 */
import { z } from 'zod'
import { eq, and } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'
import type { NewPortfolioItem } from '../../../../database/schema'
import { requireAdminSectionAccess } from '../../../../utils/admin-rbac'
import { sanitizeHtml, escapeHtml } from '../../../../utils/sanitize'
import { safeJsonParse } from '../../../../utils/json'
import { logger } from '../../../../utils/logger'

// Translation schema for case study items
const itemTranslationSchema = z.object({
  title: z.string().max(200).nullable().optional(),
  description: z.string().max(1000).nullable().optional(),
  content: z.string().nullable().optional(),
  category: z.string().max(50).nullable().optional()
})

// Base schema for common fields
const baseSchema = z.object({
  itemType: z.enum(['image', 'video', 'link', 'case_study']),
  order: z.number().int().optional(),
  published: z.boolean().optional()
})

// Media item schema (image, video, link)
const mediaSchema = baseSchema.extend({
  itemType: z.enum(['image', 'video', 'link']),
  mediaUrl: z.string().min(1),
  thumbnailUrl: z.string().optional().nullable(),
  caption: z.string().max(500).optional().nullable()
})

// Case study schema
const caseStudySchema = baseSchema.extend({
  itemType: z.literal('case_study'),
  slug: z
    .string()
    .min(1)
    .max(100)
    .regex(/^[a-z0-9-]+$/, 'Slug must be lowercase alphanumeric with dashes'),
  title: z.string().min(1).max(200),
  description: z.string().max(1000).optional().nullable(),
  content: z.string().optional().nullable(),
  tags: z.array(z.string()).optional(),
  category: z.string().max(50).optional().nullable(),
  mediaUrl: z.string().optional().nullable(), // Cover image
  thumbnailUrl: z.string().optional().nullable(),
  translations: z.record(itemTranslationSchema).optional()
})

// Union schema
const createSchema = z.discriminatedUnion('itemType', [
  mediaSchema.extend({ itemType: z.literal('image') }),
  mediaSchema.extend({ itemType: z.literal('video') }),
  mediaSchema.extend({ itemType: z.literal('link') }),
  caseStudySchema
])

export default defineEventHandler(async event => {
  await requireAdminSectionAccess(event.context.user, 'portfolios')

  const portfolioId = getRouterParam(event, 'id')

  // Validate portfolio ID
  if (!portfolioId || !/^\d+$/.test(portfolioId)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid portfolio ID'
    })
  }

  const id = parseInt(portfolioId)
  const db = useDatabase()

  // Check if portfolio exists
  const portfolio = db
    .select()
    .from(schema.portfolios)
    .where(eq(schema.portfolios.id, id))
    .get()

  if (!portfolio) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Portfolio not found'
    })
  }

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

  const data = result.data

  // For case studies, check slug uniqueness within portfolio
  if (data.itemType === 'case_study') {
    const existingSlug = db
      .select()
      .from(schema.portfolioItems)
      .where(
        and(eq(schema.portfolioItems.portfolioId, id), eq(schema.portfolioItems.slug, data.slug))
      )
      .get()

    if (existingSlug) {
      throw createError({
        statusCode: 409,
        statusMessage: 'Slug already exists in this portfolio'
      })
    }
  }

  // Build insert values based on item type
  const baseValues = {
    portfolioId: id,
    itemType: data.itemType,
    order: data.order || 0,
    published: data.published ?? true
  }

  let insertValues: NewPortfolioItem
  let translations: Record<string, { title?: string | null; description?: string | null; content?: string | null; category?: string | null }> | undefined

  if (data.itemType === 'case_study') {
    translations = data.translations
    insertValues = {
      ...baseValues,
      slug: data.slug,
      title: escapeHtml(data.title),
      description: data.description ? escapeHtml(data.description) : null,
      content: data.content ? sanitizeHtml(data.content) : null,
      tags: data.tags ? JSON.stringify(data.tags.map(t => escapeHtml(t))) : null,
      category: data.category ? escapeHtml(data.category) : null,
      mediaUrl: data.mediaUrl || null,
      thumbnailUrl: data.thumbnailUrl || null,
      publishedAt: data.published ? new Date() : null
    }
  } else {
    insertValues = {
      ...baseValues,
      mediaUrl: data.mediaUrl,
      thumbnailUrl: data.thumbnailUrl || null,
      caption: data.caption ? escapeHtml(data.caption) : null
    }
  }

  // Insert new item
  try {
    const insertResult = db.insert(schema.portfolioItems).values(insertValues).run()
    const itemId = Number(insertResult.lastInsertRowid)

    // Store translations for case study items
    if (data.itemType === 'case_study' && translations) {
      const translationValues = []

      for (const [locale, trans] of Object.entries(translations)) {
        if (trans.title) {
          translationValues.push({
            locale,
            key: `portfolio_item.${itemId}.title`,
            value: escapeHtml(trans.title)
          })
        }

        if (trans.description) {
          translationValues.push({
            locale,
            key: `portfolio_item.${itemId}.description`,
            value: escapeHtml(trans.description)
          })
        }

        if (trans.content) {
          translationValues.push({
            locale,
            key: `portfolio_item.${itemId}.content`,
            value: sanitizeHtml(trans.content)
          })
        }

        if (trans.category) {
          translationValues.push({
            locale,
            key: `portfolio_item.${itemId}.category`,
            value: escapeHtml(trans.category)
          })
        }
      }

      if (translationValues.length > 0) {
        db.insert(schema.translations)
          .values(translationValues)
          .run()
      }
    }

    // Fetch the created item
    const newItem = db
      .select()
      .from(schema.portfolioItems)
      .where(eq(schema.portfolioItems.id, itemId))
      .get()

    return {
      success: true,
      data: {
        item: {
          ...newItem,
          tags: safeJsonParse(newItem?.tags, [])
        }
      }
    }
  } catch (dbError) {
    logger.error(
      {
        error: dbError instanceof Error ? dbError.message : String(dbError),
        portfolioId: id,
        itemType: insertValues.itemType
      },
      'Database insert error for portfolio item'
    )
    throw createError({
      statusCode: 500,
      statusMessage: 'Failed to create portfolio item'
    })
  }
})
