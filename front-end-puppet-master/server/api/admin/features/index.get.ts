/**
 * GET /api/admin/features
 *
 * List all features for admin.
 * Content stored in centralized translations table with key pattern: feature.{id}.{field}
 */
import { like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async () => {
  const db = useDatabase()
  const defaultLocale = config.defaultLocale || 'en'

  const features = db
    .select()
    .from(schema.features)
    .orderBy(schema.features.order)
    .all()

  // Get all feature translations from centralized table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'feature.%'))
    .all()

  const result = features.map(feature => {
    // Build translations map for this feature
    const translationsMap: Record<string, { title: string | null; description: string | null }> = {}
    const titleKey = `feature.${feature.id}.title`
    const descKey = `feature.${feature.id}.description`

    // Group translations by locale
    const locales = new Set(
      allTranslations
        .filter(t => t.key === titleKey || t.key === descKey)
        .map(t => t.locale)
    )

    locales.forEach(locale => {
      const title = allTranslations.find(t => t.key === titleKey && t.locale === locale)?.value || null
      const description = allTranslations.find(t => t.key === descKey && t.locale === locale)?.value || null
      translationsMap[locale] = { title, description }
    })

    // Get title/description from default locale for the form
    const defaultTitle = translationsMap[defaultLocale]?.title || ''
    const defaultDescription = translationsMap[defaultLocale]?.description || ''

    return {
      ...feature,
      title: defaultTitle,
      description: defaultDescription,
      translations: translationsMap
    }
  })

  return {
    success: true,
    data: result,
    features: result
  }
})
