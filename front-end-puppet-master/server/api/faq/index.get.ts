/**
 * GET /api/faq
 *
 * Returns list of published FAQ items.
 * Content stored in centralized translations table with key pattern: faq.{id}.{field}
 */
import { eq, asc, and, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { sanitizeHtml } from '../../utils/sanitize'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)
  const locale = (query.locale as string) || config.defaultLocale || 'en'
  const defaultLocale = config.defaultLocale || 'en'

  const conditions = [eq(schema.faqItems.published, true)]

  if (query.category && typeof query.category === 'string') {
    conditions.push(eq(schema.faqItems.category, query.category))
  }

  const items = db
    .select()
    .from(schema.faqItems)
    .where(and(...conditions))
    .orderBy(asc(schema.faqItems.order))
    .all()

  if (items.length === 0) return { success: true, data: [] }

  // Get all FAQ translations from centralized translations table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'faq.%'))
    .all()

  return {
    success: true,
    data: items.map(item => {
      const questionKey = `faq.${item.id}.question`
      const answerKey = `faq.${item.id}.answer`

      // Try requested locale first, then fall back to default locale
      const questionTrans =
        allTranslations.find(t => t.key === questionKey && t.locale === locale) ||
        allTranslations.find(t => t.key === questionKey && t.locale === defaultLocale)

      const answerTrans =
        allTranslations.find(t => t.key === answerKey && t.locale === locale) ||
        allTranslations.find(t => t.key === answerKey && t.locale === defaultLocale)

      return {
        id: item.id,
        slug: item.slug,
        category: item.category,
        question: questionTrans?.value || '',
        // Public FAQ payload is sanitized before UI v-html rendering.
        answer: sanitizeHtml(answerTrans?.value || '')
      }
    })
  }
})
