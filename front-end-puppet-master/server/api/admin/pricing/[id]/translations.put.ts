/**
 * PUT /api/admin/pricing/:id/translations
 * Update translations for a pricing tier and its features
 * Uses centralized translations table with key patterns:
 *   - pricing.tier.{id}.{field}
 *   - pricing.feature.{id}.text
 */
import { eq, and } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'
import { escapeHtml } from '../../../../utils/sanitize'
import { z } from 'zod'

const translationsSchema = z.object({
  locale: z.string().min(2).max(5),
  tier: z.object({
    name: z.string().max(100).nullable().optional(),
    description: z.string().max(500).nullable().optional(),
    ctaText: z.string().max(50).nullable().optional()
  }),
  features: z
    .array(
      z.object({
        featureId: z.number(),
        text: z.string().max(200).nullable()
      })
    )
    .optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(getRouterParam(event, 'id') || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid tier ID' })
  }

  const body = await readBody(event)
  const result = translationsSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { locale, tier: tierTranslation, features: featureTranslations } = result.data

  // Check if tier exists
  const [existing] = await db
    .select()
    .from(schema.pricingTiers)
    .where(eq(schema.pricingTiers.id, id))

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Tier not found' })
  }

  // Upsert tier name translation
  if (tierTranslation.name !== undefined) {
    const nameKey = `pricing.tier.${id}.name`
    const existingName = await db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, nameKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingName) {
      await db.update(schema.translations)
        .set({ value: tierTranslation.name ? escapeHtml(tierTranslation.name) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingName.id))
    } else if (tierTranslation.name) {
      await db.insert(schema.translations)
        .values({ locale, key: nameKey, value: escapeHtml(tierTranslation.name) })
    }
  }

  // Upsert tier description translation
  if (tierTranslation.description !== undefined) {
    const descKey = `pricing.tier.${id}.description`
    const existingDesc = await db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, descKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingDesc) {
      await db.update(schema.translations)
        .set({ value: tierTranslation.description ? escapeHtml(tierTranslation.description) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingDesc.id))
    } else if (tierTranslation.description) {
      await db.insert(schema.translations)
        .values({ locale, key: descKey, value: escapeHtml(tierTranslation.description) })
    }
  }

  // Upsert tier ctaText translation
  if (tierTranslation.ctaText !== undefined) {
    const ctaKey = `pricing.tier.${id}.ctaText`
    const existingCta = await db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, ctaKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingCta) {
      await db.update(schema.translations)
        .set({ value: tierTranslation.ctaText ? escapeHtml(tierTranslation.ctaText) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingCta.id))
    } else if (tierTranslation.ctaText) {
      await db.insert(schema.translations)
        .values({ locale, key: ctaKey, value: escapeHtml(tierTranslation.ctaText) })
    }
  }

  // Upsert feature translations
  if (featureTranslations && featureTranslations.length > 0) {
    for (const ft of featureTranslations) {
      const textKey = `pricing.feature.${ft.featureId}.text`
      const existingText = await db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, textKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingText) {
        await db.update(schema.translations)
          .set({ value: ft.text ? escapeHtml(ft.text) : '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingText.id))
      } else if (ft.text) {
        await db.insert(schema.translations)
          .values({ locale, key: textKey, value: escapeHtml(ft.text) })
      }
    }
  }

  return {
    success: true,
    data: {}
  }
})
