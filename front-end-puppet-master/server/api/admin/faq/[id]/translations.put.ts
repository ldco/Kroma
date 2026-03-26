/**
 * PUT /api/admin/faq/:id/translations
 *
 * Update or create translations for a FAQ item.
 * Uses centralized translations table with key pattern: faq.{id}.{field}
 */
import { eq, and } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../../database/client'
import { escapeHtml, sanitizeHtml } from '../../../../utils/sanitize'

const updateFaqTranslationSchema = z
  .object({
    locale: z.string().min(2).max(8),
    question: z.string().max(500).optional().nullable(),
    answer: z.string().max(5000).optional().nullable()
  })
  .refine(
    value => value.question !== undefined || value.answer !== undefined,
    { message: 'At least one of question or answer is required' }
  )

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)
  const result = updateFaqTranslationSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { locale, question, answer } = result.data

  // Check if FAQ item exists
  const existing = db
    .select()
    .from(schema.faqItems)
    .where(eq(schema.faqItems.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'FAQ item not found' })
  }

  // Upsert question translation
  if (question !== undefined) {
    const questionKey = `faq.${id}.question`
    const existingQuestion = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, questionKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingQuestion) {
      db.update(schema.translations)
        .set({ value: question ? escapeHtml(question) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingQuestion.id))
        .run()
    } else if (question) {
      db.insert(schema.translations)
        .values({ locale, key: questionKey, value: escapeHtml(question) })
        .run()
    }
  }

  // Upsert answer translation
  if (answer !== undefined) {
    const answerKey = `faq.${id}.answer`
    const existingAnswer = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, answerKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingAnswer) {
      db.update(schema.translations)
        .set({ value: answer ? sanitizeHtml(answer) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingAnswer.id))
        .run()
    } else if (answer) {
      db.insert(schema.translations)
        .values({ locale, key: answerKey, value: sanitizeHtml(answer) })
        .run()
    }
  }

  return {
    success: true,
    data: null
  }
})
