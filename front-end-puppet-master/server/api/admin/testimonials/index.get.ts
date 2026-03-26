/**
 * GET /api/admin/testimonials
 *
 * List all testimonials for admin.
 * Content stored in centralized translations table with key pattern: testimonial.{id}.{field}
 */
import { like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async () => {
  const db = useDatabase()
  const defaultLocale = config.defaultLocale || 'en'

  const testimonials = db
    .select()
    .from(schema.testimonials)
    .orderBy(schema.testimonials.order)
    .all()

  // Get all testimonial translations from centralized table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'testimonial.%'))
    .all()

  const result = testimonials.map(testimonial => {
    // Build translations map for this testimonial
    const translationsMap: Record<string, { quote: string | null; authorTitle: string | null }> = {}
    const quoteKey = `testimonial.${testimonial.id}.quote`
    const titleKey = `testimonial.${testimonial.id}.authorTitle`

    // Group translations by locale
    const locales = new Set(allTranslations.filter(t => t.key === quoteKey || t.key === titleKey).map(t => t.locale))

    locales.forEach(locale => {
      const quote = allTranslations.find(t => t.key === quoteKey && t.locale === locale)?.value || null
      const authorTitle = allTranslations.find(t => t.key === titleKey && t.locale === locale)?.value || null
      translationsMap[locale] = { quote, authorTitle }
    })

    // Get quote from default locale for the form
    const defaultQuote = translationsMap[defaultLocale]?.quote || ''

    return {
      ...testimonial,
      quote: defaultQuote,
      translations: translationsMap
    }
  })

  return {
    success: true,
    data: result
  }
})
