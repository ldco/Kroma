/**
 * GET /api/admin/pricing
 * Get all pricing tiers with translations (including unpublished) for admin
 * Content stored in centralized translations table with key patterns:
 *   - pricing.tier.{id}.{field}
 *   - pricing.feature.{id}.text
 */
import { asc, eq, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async () => {
  const db = useDatabase()

  // Get all tiers ordered by order
  const tiers = await db
    .select()
    .from(schema.pricingTiers)
    .orderBy(asc(schema.pricingTiers.order))

  // Get all translations from centralized table
  const allTranslations = await db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'pricing.%'))

  // Get features and feature translations for each tier
  const tiersWithFeatures = await Promise.all(
    tiers.map(async tier => {
      const features = await db
        .select()
        .from(schema.pricingFeatures)
        .where(eq(schema.pricingFeatures.tierId, tier.id))
        .orderBy(asc(schema.pricingFeatures.order))

      // Get tier translations grouped by locale
      const tierTranslations: Record<string, { name: string | null; description: string | null; ctaText: string | null }> = {}

      const nameKey = `pricing.tier.${tier.id}.name`
      const descKey = `pricing.tier.${tier.id}.description`
      const ctaKey = `pricing.tier.${tier.id}.ctaText`

      // Find all locales that have tier translations
      const tierTrans = allTranslations.filter(t => t.key.startsWith(`pricing.tier.${tier.id}.`))
      const tierLocales = [...new Set(tierTrans.map(t => t.locale))]

      for (const locale of tierLocales) {
        const name = allTranslations.find(t => t.key === nameKey && t.locale === locale)
        const desc = allTranslations.find(t => t.key === descKey && t.locale === locale)
        const cta = allTranslations.find(t => t.key === ctaKey && t.locale === locale)

        tierTranslations[locale] = {
          name: name?.value || null,
          description: desc?.value || null,
          ctaText: cta?.value || null
        }
      }

      // Get feature translations grouped by featureId and locale
      const featuresWithTranslations = features.map(f => {
        const featureTextKey = `pricing.feature.${f.id}.text`
        const featureTrans = allTranslations.filter(t => t.key === featureTextKey)

        const translations: Record<string, string | null> = {}
        for (const ft of featureTrans) {
          translations[ft.locale] = ft.value
        }

        return {
          ...f,
          translations
        }
      })

      return {
        ...tier,
        // Convert price from cents to dollars for display
        price: tier.price !== null ? tier.price / 100 : null,
        translations: tierTranslations,
        features: featuresWithTranslations
      }
    })
  )

  return {
    success: true,
    data: tiersWithFeatures
  }
})
