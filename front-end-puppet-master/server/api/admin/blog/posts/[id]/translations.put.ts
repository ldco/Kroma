/**
 * PUT /api/admin/blog/posts/:id/translations
 *
 * Update or create translations for a blog post.
 * Uses centralized translations table with key pattern: blog.post.{id}.{field}
 */
import { eq, and } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../../../database/client'
import { escapeHtml } from '../../../../../utils/sanitize'

const updateTranslationSchema = z.object({
  locale: z.string().min(2).max(5),
  title: z.string().min(1).max(300).optional(),
  excerpt: z.string().max(500).optional().nullable(),
  content: z.string().optional().nullable()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)
  const result = updateTranslationSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { locale, title, excerpt, content } = result.data

  // Check if post exists
  const existing = db
    .select()
    .from(schema.blogPosts)
    .where(eq(schema.blogPosts.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Post not found' })
  }

  // Upsert title
  if (title !== undefined) {
    const titleKey = `blog.post.${id}.title`
    const existingTitle = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, titleKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingTitle) {
      db.update(schema.translations)
        .set({ value: escapeHtml(title), updatedAt: new Date() })
        .where(eq(schema.translations.id, existingTitle.id))
        .run()
    } else {
      db.insert(schema.translations)
        .values({ locale, key: titleKey, value: escapeHtml(title) })
        .run()
    }
  }

  // Upsert excerpt
  if (excerpt !== undefined) {
    const excerptKey = `blog.post.${id}.excerpt`
    const existingExcerpt = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, excerptKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingExcerpt) {
      db.update(schema.translations)
        .set({ value: excerpt ? escapeHtml(excerpt) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingExcerpt.id))
        .run()
    } else if (excerpt) {
      db.insert(schema.translations)
        .values({ locale, key: excerptKey, value: escapeHtml(excerpt) })
        .run()
    }
  }

  // Upsert content
  if (content !== undefined) {
    const contentKey = `blog.post.${id}.content`
    const existingContent = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, contentKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingContent) {
      db.update(schema.translations)
        .set({ value: content || '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingContent.id))
        .run()
    } else if (content) {
      db.insert(schema.translations)
        .values({ locale, key: contentKey, value: content })
        .run()
    }
  }

  return {
    success: true,
    data: null
  }
})
