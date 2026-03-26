/**
 * GET /api/pricing
 * Get all pricing tiers with features (public endpoint)
 * Content stored in centralized translations table with key patterns:
 *   - pricing.tier.{id}.{field}
 *   - pricing.feature.{id}.text
 * Supports locale via ?locale=ru query param
 */
import { eq, asc, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)
  const locale = (query.locale as string) || config.defaultLocale || 'en'
  const defaultLocale = config.defaultLocale || 'en'

  // Get all published tiers ordered by order
  const tiers = await db
    .select()
    .from(schema.pricingTiers)
    .where(eq(schema.pricingTiers.published, true))
    .orderBy(asc(schema.pricingTiers.order))

  // Get all translations from centralized table
  const allTranslations = await db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'pricing.%'))

  // Get features for each tier with translations
  const tiersWithFeatures = await Promise.all(
    tiers.map(async tier => {
      const features = await db
        .select()
        .from(schema.pricingFeatures)
        .where(eq(schema.pricingFeatures.tierId, tier.id))
        .orderBy(asc(schema.pricingFeatures.order))

      // Get tier translations
      const nameKey = `pricing.tier.${tier.id}.name`
      const descKey = `pricing.tier.${tier.id}.description`
      const ctaKey = `pricing.tier.${tier.id}.ctaText`

      const nameTrans =
        allTranslations.find(t => t.key === nameKey && t.locale === locale) ||
        allTranslations.find(t => t.key === nameKey && t.locale === defaultLocale)

      const descTrans =
        allTranslations.find(t => t.key === descKey && t.locale === locale) ||
        allTranslations.find(t => t.key === descKey && t.locale === defaultLocale)

      const ctaTrans =
        allTranslations.find(t => t.key === ctaKey && t.locale === locale) ||
        allTranslations.find(t => t.key === ctaKey && t.locale === defaultLocale)

      return {
        ...tier,
        name: nameTrans?.value || tier.name,
        description: descTrans?.value || tier.description,
        ctaText: ctaTrans?.value || tier.ctaText,
        // Convert price from cents to dollars for display
        price: tier.price !== null ? tier.price / 100 : null,
        features: features.map(f => {
          const featureTextKey = `pricing.feature.${f.id}.text`
          const fTrans =
            allTranslations.find(t => t.key === featureTextKey && t.locale === locale) ||
            allTranslations.find(t => t.key === featureTextKey && t.locale === defaultLocale)

          return {
            ...f,
            text: fTrans?.value || f.text
          }
        })
      }
    })
  )

  return {
    success: true,
    data: tiersWithFeatures
  }
})
