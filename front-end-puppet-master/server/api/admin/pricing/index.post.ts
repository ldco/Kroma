/**
 * POST /api/admin/pricing
 * Create a new pricing tier with multi-locale support.
 * Content stored in centralized translations table.
 */
import { useDatabase, schema } from '../../../database/client'
import { escapeHtml } from '../../../utils/sanitize'
import { z } from 'zod'

const createTierSchema = z.object({
  slug: z.string().min(1).max(50).regex(/^[a-z0-9-]+$/),
  price: z.number().min(0).nullable().optional(),
  currency: z.string().length(3).default('USD'),
  period: z.enum(['month', 'year', 'one-time']).default('month'),
  featured: z.boolean().default(false),
  ctaUrl: z.string().max(200).default('/contact'),
  order: z.number().int().min(0).default(0),
  published: z.boolean().default(true),
  translations: z.record(z.object({
    name: z.string().max(100),
    description: z.string().max(500),
    ctaText: z.string().max(50)
  })).optional(),
  features: z.array(z.object({
    included: z.boolean().default(true),
    order: z.number().int().min(0).default(0),
    translations: z.record(z.string().max(200)).optional()
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const body = await readBody(event)

  const result = createTierSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, features, price, ...tierData } = result.data

  // Get first available name for the table (fallback)
  let fallbackName = 'New Tier'
  if (translations) {
    const firstTrans = Object.values(translations)[0]
    if (firstTrans?.name) fallbackName = firstTrans.name
  }

  // Insert tier (convert price to cents)
  const [tier] = await db
    .insert(schema.pricingTiers)
    .values({
      ...tierData,
      name: fallbackName,
      description: null,
      ctaText: null,
      price: price !== null && price !== undefined ? Math.round(price * 100) : null
    })
    .returning()

  if (!tier) {
    throw createError({ statusCode: 500, message: 'Failed to create tier' })
  }

  // Store tier translations for all locales
  const translationValues = []
  if (translations) {
    for (const [locale, trans] of Object.entries(translations)) {
      if (trans.name) {
        translationValues.push({
          locale,
          key: `pricing.tier.${tier.id}.name`,
          value: escapeHtml(trans.name)
        })
      }
      if (trans.description) {
        translationValues.push({
          locale,
          key: `pricing.tier.${tier.id}.description`,
          value: escapeHtml(trans.description)
        })
      }
      if (trans.ctaText) {
        translationValues.push({
          locale,
          key: `pricing.tier.${tier.id}.ctaText`,
          value: escapeHtml(trans.ctaText)
        })
      }
    }
  }

  // Insert features if provided
  if (features && features.length > 0) {
    const insertedFeatures = await db.insert(schema.pricingFeatures).values(
      features.map((f, index) => {
        // Get first available text for fallback
        let fallbackText = ''
        if (f.translations) {
          const firstText = Object.values(f.translations)[0]
          if (firstText) fallbackText = firstText
        }
        return {
          tierId: tier.id,
          text: fallbackText,
          included: f.included,
          order: f.order ?? index
        }
      })
    ).returning()

    // Store feature translations for all locales
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
  }

  // Insert all translations
  if (translationValues.length > 0) {
    await db.insert(schema.translations).values(translationValues)
  }

  return {
    success: true,
    data: {
      tier
    }
  }
})
