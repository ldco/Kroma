/**
 * POST /api/admin/faq
 *
 * Create a new FAQ item.
 * Content stored in centralized translations table.
 * Supports multiple locales in one request.
 */
import { eq } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'
import { escapeHtml, sanitizeHtml } from '../../../utils/sanitize'

const createFaqSchema = z.object({
  category: z.string().max(100).nullish(),
  order: z.number().int().min(0).default(0),
  published: z.boolean().default(true),
  translations: z.record(z.object({
    question: z.string().min(1).max(500),
    answer: z.string().min(1).max(5000)
  })).optional()
})

function slugify(value: string): string {
  return value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

function createUniqueSlug(
  db: ReturnType<typeof useDatabase>,
  baseSlug: string
): string {
  const normalized = baseSlug || 'faq-item'
  let candidate = normalized
  let index = 1

  while (db.select().from(schema.faqItems).where(eq(schema.faqItems.slug, candidate)).get()) {
    candidate = `${normalized}-${index}`
    index += 1
  }

  return candidate
}

export default defineEventHandler(async event => {
  const db = useDatabase()
  const body = await readBody(event)

  const result = createFaqSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, ...data } = result.data

  // FAQ slug is required by schema.
  // Derive it from the first translation question and ensure uniqueness.
  const firstQuestion = translations
    ? Object.values(translations).find(translation => translation.question.trim().length > 0)?.question
    : ''

  const baseSlug = slugify(firstQuestion || `faq-${Date.now()}`)
  const uniqueSlug = createUniqueSlug(db, baseSlug)

  // Insert FAQ item metadata only
  const insertResult = db
    .insert(schema.faqItems)
    .values({
      slug: uniqueSlug,
      category: data.category ? escapeHtml(data.category) : null,
      order: data.order,
      published: data.published
    })
    .run()

  const faqId = Number(insertResult.lastInsertRowid)

  // Store translations in centralized table
  if (translations) {
    const translationValues = []

    for (const [locale, trans] of Object.entries(translations)) {
      if (trans.question) {
        translationValues.push({
          locale,
          key: `faq.${faqId}.question`,
          value: escapeHtml(trans.question)
        })
      }

      if (trans.answer) {
        translationValues.push({
          locale,
          key: `faq.${faqId}.answer`,
          value: sanitizeHtml(trans.answer)
        })
      }
    }

    if (translationValues.length > 0) {
      db.insert(schema.translations)
        .values(translationValues)
        .run()
    }
  }

  return {
    success: true,
    data: {
      id: faqId
    },
    id: faqId
  }
})
