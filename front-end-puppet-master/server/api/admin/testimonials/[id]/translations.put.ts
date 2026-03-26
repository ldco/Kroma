/**
 * PUT /api/admin/testimonials/:id/translations
 *
 * Update or create translations for a testimonial.
 * Uses centralized translations table with key pattern: testimonial.{id}.quote
 */
import { eq, and } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)
  const { locale, quote, authorTitle } = body

  if (!locale) {
    throw createError({ statusCode: 400, message: 'Locale is required' })
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

  // Upsert quote translation
  if (quote) {
    const quoteKey = `testimonial.${id}.quote`
    const existingQuote = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, quoteKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingQuote) {
      db.update(schema.translations)
        .set({ value: quote, updatedAt: new Date() })
        .where(eq(schema.translations.id, existingQuote.id))
        .run()
    } else {
      db.insert(schema.translations)
        .values({ locale, key: quoteKey, value: quote })
        .run()
    }
  }

  // Upsert authorTitle translation
  if (authorTitle) {
    const titleKey = `testimonial.${id}.authorTitle`
    const existingTitle = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, titleKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingTitle) {
      db.update(schema.translations)
        .set({ value: authorTitle, updatedAt: new Date() })
        .where(eq(schema.translations.id, existingTitle.id))
        .run()
    } else {
      db.insert(schema.translations)
        .values({ locale, key: titleKey, value: authorTitle })
        .run()
    }
  }

  return {
    success: true,
    data: {}
  }
})
