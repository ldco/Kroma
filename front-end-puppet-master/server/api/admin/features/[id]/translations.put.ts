/**
 * PUT /api/admin/features/:id/translations
 *
 * Update or create translations for a feature.
 * Uses centralized translations table with key pattern: feature.{id}.{field}
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
  const { locale, title, description } = body

  if (!locale) {
    throw createError({ statusCode: 400, message: 'Locale is required' })
  }

  // Check if feature exists
  const existing = db
    .select()
    .from(schema.features)
    .where(eq(schema.features.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Feature not found' })
  }

  // Upsert title translation
  if (title !== undefined) {
    const titleKey = `feature.${id}.title`
    const existingTitle = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, titleKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingTitle) {
      db.update(schema.translations)
        .set({ value: title, updatedAt: new Date() })
        .where(eq(schema.translations.id, existingTitle.id))
        .run()
    } else {
      db.insert(schema.translations)
        .values({ locale, key: titleKey, value: title })
        .run()
    }
  }

  // Upsert description translation
  if (description !== undefined) {
    const descKey = `feature.${id}.description`
    const existingDesc = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, descKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingDesc) {
      db.update(schema.translations)
        .set({ value: description, updatedAt: new Date() })
        .where(eq(schema.translations.id, existingDesc.id))
        .run()
    } else {
      db.insert(schema.translations)
        .values({ locale, key: descKey, value: description })
        .run()
    }
  }

  return {
    success: true,
    data: {}
  }
})
