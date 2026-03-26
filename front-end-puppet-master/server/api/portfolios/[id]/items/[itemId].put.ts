/**
 * PUT /api/portfolios/[id]/items/[itemId]
 *
 * Updates a portfolio item.
 * Requires admin authentication.
 * Supports multiple locales for translatable fields.
 */
import { z } from 'zod'
import { eq, and, ne } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'
import { requireAdminSectionAccess } from '../../../../utils/admin-rbac'
import { sanitizeHtml, escapeHtml } from '../../../../utils/sanitize'
import { safeJsonParse } from '../../../../utils/json'

// Translation schema for case study items
const itemTranslationSchema = z.object({
  title: z.string().max(200).nullable().optional(),
  description: z.string().max(1000).nullable().optional(),
  content: z.string().nullable().optional(),
  category: z.string().max(50).nullable().optional()
})

// Update schema - all fields optional for partial update
const updateSchema = z.object({
  order: z.number().int().optional(),
  published: z.boolean().optional(),
  // Media fields
  mediaUrl: z.string().optional().nullable(),
  thumbnailUrl: z.string().optional().nullable(),
  caption: z.string().max(500).optional().nullable(),
  // Case study fields
  slug: z
    .string()
    .min(1)
    .max(100)
    .regex(/^[a-z0-9-]+$/, 'Slug must be lowercase alphanumeric with dashes')
    .optional(),
  title: z.string().min(1).max(200).optional(),
  description: z.string().max(1000).optional().nullable(),
  content: z.string().optional().nullable(),
  tags: z.array(z.string()).optional(),
  category: z.string().max(50).optional().nullable(),
  translations: z.record(itemTranslationSchema).optional()
})

export default defineEventHandler(async event => {
  await requireAdminSectionAccess(event.context.user, 'portfolios')

  const portfolioId = getRouterParam(event, 'id')
  const itemId = getRouterParam(event, 'itemId')

  // Validate IDs
  if (!portfolioId || !/^\d+$/.test(portfolioId)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid portfolio ID'
    })
  }

  if (!itemId || !/^\d+$/.test(itemId)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid item ID'
    })
  }

  const pId = parseInt(portfolioId)
  const iId = parseInt(itemId)
  const db = useDatabase()

  // Check if item exists and belongs to portfolio
  const existing = db
    .select()
    .from(schema.portfolioItems)
    .where(and(eq(schema.portfolioItems.id, iId), eq(schema.portfolioItems.portfolioId, pId)))
    .get()

  if (!existing) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Item not found'
    })
  }

  // Parse and validate request body
  const body = await readBody(event)
  const result = updateSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, ...data } = result.data

  // For case studies, check slug uniqueness within portfolio if changing
  if (data.slug && data.slug !== existing.slug) {
    const slugConflict = db
      .select()
      .from(schema.portfolioItems)
      .where(
        and(
          eq(schema.portfolioItems.portfolioId, pId),
          eq(schema.portfolioItems.slug, data.slug),
          ne(schema.portfolioItems.id, iId)
        )
      )
      .get()

    if (slugConflict) {
      throw createError({
        statusCode: 409,
        statusMessage: 'Slug already exists in this portfolio'
      })
    }
  }

  // Build update object
  const updateData: Record<string, unknown> = {
    updatedAt: new Date()
  }

  // Common fields
  if (data.order !== undefined) updateData.order = data.order
  if (data.published !== undefined) {
    updateData.published = data.published
    // Set publishedAt for case studies
    if (existing.itemType === 'case_study' && data.published && !existing.publishedAt) {
      updateData.publishedAt = new Date()
    }
  }

  // Media fields
  if (data.mediaUrl !== undefined) updateData.mediaUrl = data.mediaUrl
  if (data.thumbnailUrl !== undefined) updateData.thumbnailUrl = data.thumbnailUrl
  if (data.caption !== undefined)
    updateData.caption = data.caption ? escapeHtml(data.caption) : null

  // Case study fields
  if (data.slug !== undefined) updateData.slug = data.slug
  if (data.title !== undefined) updateData.title = escapeHtml(data.title)
  if (data.description !== undefined)
    updateData.description = data.description ? escapeHtml(data.description) : null
  if (data.content !== undefined)
    updateData.content = data.content ? sanitizeHtml(data.content) : null
  if (data.tags !== undefined)
    updateData.tags = data.tags ? JSON.stringify(data.tags.map(t => escapeHtml(t))) : null
  if (data.category !== undefined)
    updateData.category = data.category ? escapeHtml(data.category) : null

  // Update item
  const updated = db
    .update(schema.portfolioItems)
    .set(updateData)
    .where(eq(schema.portfolioItems.id, iId))
    .returning()
    .get()

  // Update translations for case study items
  if (existing.itemType === 'case_study' && translations) {
    for (const [locale, trans] of Object.entries(translations)) {
      const titleKey = `portfolio_item.${iId}.title`
      const descKey = `portfolio_item.${iId}.description`
      const contentKey = `portfolio_item.${iId}.content`
      const categoryKey = `portfolio_item.${iId}.category`

      // Upsert title
      if (trans.title !== undefined) {
        const existingTitle = db
          .select()
          .from(schema.translations)
          .where(and(eq(schema.translations.key, titleKey), eq(schema.translations.locale, locale)))
          .get()

        if (existingTitle) {
          db.update(schema.translations)
            .set({ value: trans.title ? escapeHtml(trans.title) : '', updatedAt: new Date() })
            .where(eq(schema.translations.id, existingTitle.id))
            .run()
        } else if (trans.title) {
          db.insert(schema.translations)
            .values({ locale, key: titleKey, value: escapeHtml(trans.title) })
            .run()
        }
      }

      // Upsert description
      if (trans.description !== undefined) {
        const existingDesc = db
          .select()
          .from(schema.translations)
          .where(and(eq(schema.translations.key, descKey), eq(schema.translations.locale, locale)))
          .get()

        if (existingDesc) {
          db.update(schema.translations)
            .set({ value: trans.description ? escapeHtml(trans.description) : '', updatedAt: new Date() })
            .where(eq(schema.translations.id, existingDesc.id))
            .run()
        } else if (trans.description) {
          db.insert(schema.translations)
            .values({ locale, key: descKey, value: escapeHtml(trans.description) })
            .run()
        }
      }

      // Upsert content
      if (trans.content !== undefined) {
        const existingContent = db
          .select()
          .from(schema.translations)
          .where(and(eq(schema.translations.key, contentKey), eq(schema.translations.locale, locale)))
          .get()

        if (existingContent) {
          db.update(schema.translations)
            .set({ value: trans.content ? sanitizeHtml(trans.content) : '', updatedAt: new Date() })
            .where(eq(schema.translations.id, existingContent.id))
            .run()
        } else if (trans.content) {
          db.insert(schema.translations)
            .values({ locale, key: contentKey, value: sanitizeHtml(trans.content) })
            .run()
        }
      }

      // Upsert category
      if (trans.category !== undefined) {
        const existingCategory = db
          .select()
          .from(schema.translations)
          .where(and(eq(schema.translations.key, categoryKey), eq(schema.translations.locale, locale)))
          .get()

        if (existingCategory) {
          db.update(schema.translations)
            .set({ value: trans.category ? escapeHtml(trans.category) : '', updatedAt: new Date() })
            .where(eq(schema.translations.id, existingCategory.id))
            .run()
        } else if (trans.category) {
          db.insert(schema.translations)
            .values({ locale, key: categoryKey, value: escapeHtml(trans.category) })
            .run()
        }
      }
    }
  }

  return {
    success: true,
    data: {
      item: {
        ...updated,
        tags: safeJsonParse(updated.tags, [])
      }
    }
  }
})
