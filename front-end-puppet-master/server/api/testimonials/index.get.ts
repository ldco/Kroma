/**
 * GET /api/testimonials
 *
 * Returns list of published testimonials.
 * Quotes are stored in centralized translations table with key pattern: testimonial.{id}.quote
 */
import { eq, asc, and, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)
  const locale = (query.locale as string) || config.defaultLocale || 'en'
  const defaultLocale = config.defaultLocale || 'en'

  const conditions = [eq(schema.testimonials.published, true)]

  if (query.featured === 'true') {
    conditions.push(eq(schema.testimonials.featured, true))
  }

  const limit = query.limit ? parseInt(query.limit as string) : undefined

  let testimonials = db
    .select()
    .from(schema.testimonials)
    .where(and(...conditions))
    .orderBy(asc(schema.testimonials.order))
    .all()

  if (limit) {
    testimonials = testimonials.slice(0, limit)
  }

  if (testimonials.length === 0) return { success: true, data: [] }

  // Get all testimonial quotes from centralized translations table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'testimonial.%.quote'))
    .all()

  return {
    success: true,
    data: testimonials.map(t => {
      const testimonialKey = `testimonial.${t.id}.quote`
      const keyTranslations = allTranslations.filter(tr => tr.key === testimonialKey)

      // Try requested locale first, then fall back to default locale
      const trans =
        keyTranslations.find(tr => tr.locale === locale) ||
        keyTranslations.find(tr => tr.locale === defaultLocale)

      return {
        id: t.id,
        authorName: t.authorName,
        authorTitle: t.authorTitle,
        authorCompany: t.authorCompany,
        authorPhotoUrl: t.authorPhotoUrl,
        rating: t.rating,
        quote: trans?.value || ''
      }
    })
  }
})
