/**
 * GET /api/features
 *
 * Returns list of published features.
 * Content stored in centralized translations table with key pattern: feature.{id}.{field}
 */
import { eq, asc, and, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)
  const locale = (query.locale as string) || config.defaultLocale || 'en'
  const defaultLocale = config.defaultLocale || 'en'

  const conditions = [eq(schema.features.published, true)]

  if (query.category && typeof query.category === 'string') {
    conditions.push(eq(schema.features.category, query.category))
  }

  const features = db
    .select()
    .from(schema.features)
    .where(and(...conditions))
    .orderBy(asc(schema.features.order))
    .all()

  if (features.length === 0) return { success: true, data: [] }

  // Get all feature translations from centralized translations table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'feature.%'))
    .all()

  return {
    success: true,
    data: features.map(feature => {
      const titleKey = `feature.${feature.id}.title`
      const descKey = `feature.${feature.id}.description`

      // Try requested locale first, then fall back to default locale
      const titleTrans =
        allTranslations.find(t => t.key === titleKey && t.locale === locale) ||
        allTranslations.find(t => t.key === titleKey && t.locale === defaultLocale)

      const descTrans =
        allTranslations.find(t => t.key === descKey && t.locale === locale) ||
        allTranslations.find(t => t.key === descKey && t.locale === defaultLocale)

      return {
        ...feature,
        title: titleTrans?.value || feature.slug,
        description: descTrans?.value || null
      }
    })
  }
})
