/**
 * PUT /api/admin/testimonials/:id
 *
 * Update a testimonial.
 * Quote and authorTitle stored in centralized translations table.
 * Supports multiple locales in one request.
 */
import { eq, and } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'

const updateTestimonialSchema = z.object({
  authorName: z.string().min(1).max(100).optional(),
  authorPhoto: z.string().max(500).nullish(),
  rating: z.number().int().min(1).max(5).optional(),
  order: z.number().int().min(0).optional(),
  published: z.boolean().optional(),
  translations: z.record(z.object({
    quote: z.string().max(2000),
    authorTitle: z.string().max(200)
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)

  const result = updateTestimonialSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  // Check if testimonial exists
  const existing = db
    .select()
    .from(schema.testimonials)
    .where(eq(schema.testimonials.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Testimonial not found' })
  }

  const { translations, ...data } = result.data

  // Build update values
  const updateValues: Record<string, unknown> = {}

  if (data.authorName !== undefined) updateValues.authorName = data.authorName
  if (data.authorPhoto !== undefined) updateValues.authorPhotoUrl = data.authorPhoto
  if (data.rating !== undefined) updateValues.rating = data.rating
  if (data.order !== undefined) updateValues.order = data.order
  if (data.published !== undefined) updateValues.published = data.published

  if (Object.keys(updateValues).length > 0) {
    db.update(schema.testimonials)
      .set(updateValues)
      .where(eq(schema.testimonials.id, id))
      .run()
  }

  // Update translations in centralized table
  if (translations) {
    for (const [locale, trans] of Object.entries(translations)) {
      const quoteKey = `testimonial.${id}.quote`
      const titleKey = `testimonial.${id}.authorTitle`

      // Upsert quote
      const existingQuote = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, quoteKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingQuote) {
        db.update(schema.translations)
          .set({ value: trans.quote || '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingQuote.id))
          .run()
      } else if (trans.quote) {
        db.insert(schema.translations)
          .values({ locale, key: quoteKey, value: trans.quote })
          .run()
      }

      // Upsert authorTitle
      const existingTitle = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, titleKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingTitle) {
        db.update(schema.translations)
          .set({ value: trans.authorTitle || '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingTitle.id))
          .run()
      } else if (trans.authorTitle) {
        db.insert(schema.translations)
          .values({ locale, key: titleKey, value: trans.authorTitle })
          .run()
      }
    }
  }

  return {
    success: true,
    data: {}
  }
})
