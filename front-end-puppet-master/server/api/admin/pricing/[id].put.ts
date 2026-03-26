/**
 * PUT /api/admin/pricing/:id
 * Update a pricing tier with multi-locale support.
 * Content stored in centralized translations table.
 */
import { eq, and, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { escapeHtml } from '../../../utils/sanitize'
import { z } from 'zod'

const updateTierSchema = z.object({
  slug: z.string().min(1).max(50).regex(/^[a-z0-9-]+$/).optional(),
  price: z.number().min(0).nullable().optional(),
  currency: z.string().length(3).optional(),
  period: z.enum(['month', 'year', 'one-time']).optional(),
  featured: z.boolean().optional(),
  ctaUrl: z.string().max(200).optional(),
  order: z.number().int().min(0).optional(),
  published: z.boolean().optional(),
  translations: z.record(z.object({
    name: z.string().max(100),
    description: z.string().max(500),
    ctaText: z.string().max(50)
  })).optional(),
  features: z.array(z.object({
    id: z.number().optional(),
    included: z.boolean().default(true),
    order: z.number().int().min(0).default(0),
    translations: z.record(z.string().max(200)).optional()
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(getRouterParam(event, 'id') || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid tier ID' })
  }

  const body = await readBody(event)
  const result = updateTierSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, features, price, ...tierData } = result.data

  // Check if tier exists
  const [existing] = await db
    .select()
    .from(schema.pricingTiers)
    .where(eq(schema.pricingTiers.id, id))

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Tier not found' })
  }

  // Get first available name for fallback
  let fallbackName = existing.name
  if (translations) {
    const firstTrans = Object.values(translations)[0]
    if (firstTrans?.name) fallbackName = firstTrans.name
  }

  // Update tier
  const updateData: Record<string, unknown> = { ...tierData, name: fallbackName, updatedAt: new Date() }
  if (price !== undefined) {
    updateData.price = price !== null ? Math.round(price * 100) : null
  }

  const [tier] = await db
    .update(schema.pricingTiers)
    .set(updateData)
    .where(eq(schema.pricingTiers.id, id))
    .returning()

  // Update tier translations for all locales
  if (translations) {
    for (const [locale, trans] of Object.entries(translations)) {
      // Upsert name
      const nameKey = `pricing.tier.${id}.name`
      const existingName = await db.select().from(schema.translations)
        .where(and(eq(schema.translations.key, nameKey), eq(schema.translations.locale, locale))).get()

      if (existingName) {
        await db.update(schema.translations)
          .set({ value: trans.name ? escapeHtml(trans.name) : '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingName.id))
      } else if (trans.name) {
        await db.insert(schema.translations).values({ locale, key: nameKey, value: escapeHtml(trans.name) })
      }

      // Upsert description
      const descKey = `pricing.tier.${id}.description`
      const existingDesc = await db.select().from(schema.translations)
        .where(and(eq(schema.translations.key, descKey), eq(schema.translations.locale, locale))).get()

      if (existingDesc) {
        await db.update(schema.translations)
          .set({ value: trans.description ? escapeHtml(trans.description) : '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingDesc.id))
      } else if (trans.description) {
        await db.insert(schema.translations).values({ locale, key: descKey, value: escapeHtml(trans.description) })
      }

      // Upsert ctaText
      const ctaKey = `pricing.tier.${id}.ctaText`
      const existingCta = await db.select().from(schema.translations)
        .where(and(eq(schema.translations.key, ctaKey), eq(schema.translations.locale, locale))).get()

      if (existingCta) {
        await db.update(schema.translations)
          .set({ value: trans.ctaText ? escapeHtml(trans.ctaText) : '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingCta.id))
      } else if (trans.ctaText) {
        await db.insert(schema.translations).values({ locale, key: ctaKey, value: escapeHtml(trans.ctaText) })
      }
    }
  }

  // Update features if provided (replace all)
  if (features !== undefined) {
    // Get existing feature IDs to delete their translations
    const existingFeatures = await db
      .select()
      .from(schema.pricingFeatures)
      .where(eq(schema.pricingFeatures.tierId, id))

    // Delete existing feature translations from centralized table
    for (const f of existingFeatures) {
      await db.delete(schema.translations)
        .where(like(schema.translations.key, `pricing.feature.${f.id}.%`))
    }

    // Delete existing features
    await db.delete(schema.pricingFeatures).where(eq(schema.pricingFeatures.tierId, id))

    // Insert new features
    if (features.length > 0) {
      const insertedFeatures = await db.insert(schema.pricingFeatures).values(
        features.map((f, index) => {
          let fallbackText = ''
          if (f.translations) {
            const firstText = Object.values(f.translations)[0]
            if (firstText) fallbackText = firstText
          }
          return {
            tierId: id,
            text: fallbackText,
            included: f.included,
            order: f.order ?? index
          }
        })
      ).returning()

      // Store feature translations for all locales
      const translationValues = []
      for (let i = 0; i < insertedFeatures.length; i++) {
        const feature = insertedFeatures[i]
        const originalFeature = features[i]
        if (feature && originalFeature?.translations) {
          for (const [locale, text] of Object.entries(originalFeature.translations)) {
            if (text) {
              translationValues.push({
                locale,
                key: `pricing.feature.${feature.id}.text`,
                value: escapeHtml(text)
              })
            }
          }
        }
      }

      if (translationValues.length > 0) {
        await db.insert(schema.translations).values(translationValues)
      }
    }
  }

  return {
    success: true,
    data: {
      tier
    }
  }
})
