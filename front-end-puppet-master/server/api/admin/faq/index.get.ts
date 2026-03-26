/**
 * GET /api/admin/faq
 *
 * List all FAQ items for admin.
 * Content stored in centralized translations table with key pattern: faq.{id}.{field}
 */
import { like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async () => {
  const db = useDatabase()
  const defaultLocale = config.defaultLocale || 'en'

  const faqItems = db
    .select()
    .from(schema.faqItems)
    .orderBy(schema.faqItems.order)
    .all()

  // Get all FAQ translations from centralized table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'faq.%'))
    .all()

  const result = faqItems.map(item => {
    // Build translations map for this FAQ item
    const translationsMap: Record<string, { question: string | null; answer: string | null }> = {}
    const questionKey = `faq.${item.id}.question`
    const answerKey = `faq.${item.id}.answer`

    // Group translations by locale
    const locales = new Set(
      allTranslations
        .filter(t => t.key === questionKey || t.key === answerKey)
        .map(t => t.locale)
    )

    locales.forEach(locale => {
      const question = allTranslations.find(t => t.key === questionKey && t.locale === locale)?.value || null
      const answer = allTranslations.find(t => t.key === answerKey && t.locale === locale)?.value || null
      translationsMap[locale] = { question, answer }
    })

    // Get question/answer from default locale for the form
    const defaultQuestion = translationsMap[defaultLocale]?.question || ''
    const defaultAnswer = translationsMap[defaultLocale]?.answer || ''

    return {
      ...item,
      question: defaultQuestion,
      answer: defaultAnswer,
      translations: translationsMap
    }
  })

  return {
    success: true,
    data: result,
    faq: result
  }
})
