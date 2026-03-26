/**
 * PUT /api/admin/blog/posts/:id
 *
 * Update a blog post.
 * Content stored in centralized translations table with key pattern: blog.post.{id}.{field}
 */
import { eq, and, ne } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../../database/client'
import { escapeHtml } from '../../../../utils/sanitize'

const updatePostSchema = z.object({
  slug: z.string().min(1).max(200).regex(/^[a-z0-9-]+$/).optional(),
  categoryId: z.number().int().optional().nullable(),
  coverImageUrl: z.string().max(500).optional().nullable(),
  coverImageAlt: z.string().max(200).optional().nullable(),
  published: z.boolean().optional(),
  readingTimeMinutes: z.number().int().min(0).optional().nullable(),
  translations: z.record(z.object({
    title: z.string().max(300),
    excerpt: z.string().max(500),
    content: z.string()
  })).optional(),
  tagIds: z.array(z.number().int()).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)
  const result = updatePostSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  // Check if post exists
  const existing = db
    .select()
    .from(schema.blogPosts)
    .where(eq(schema.blogPosts.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Post not found' })
  }

  const { translations, tagIds, ...data } = result.data

  // Check slug uniqueness
  if (data.slug && data.slug !== existing.slug) {
    const slugExists = db
      .select()
      .from(schema.blogPosts)
      .where(and(eq(schema.blogPosts.slug, data.slug), ne(schema.blogPosts.id, id)))
      .get()

    if (slugExists) {
      throw createError({ statusCode: 409, message: 'Slug already exists' })
    }
  }

  // Build update values
  const updateValues: Record<string, unknown> = { updatedAt: new Date() }

  if (data.slug !== undefined) updateValues.slug = data.slug
  if (data.categoryId !== undefined) updateValues.categoryId = data.categoryId
  if (data.coverImageUrl !== undefined) updateValues.coverImageUrl = data.coverImageUrl
  if (data.coverImageAlt !== undefined) updateValues.coverImageAlt = data.coverImageAlt
  if (data.readingTimeMinutes !== undefined) updateValues.readingTimeMinutes = data.readingTimeMinutes

  if (data.published !== undefined) {
    updateValues.published = data.published
    if (data.published && !existing.publishedAt) {
      updateValues.publishedAt = new Date()
    }
  }

  // Update post
  const [post] = db
    .update(schema.blogPosts)
    .set(updateValues)
    .where(eq(schema.blogPosts.id, id))
    .returning()

  // Update translations in centralized table
  if (translations) {
    for (const [locale, trans] of Object.entries(translations)) {
      const titleKey = `blog.post.${id}.title`
      const excerptKey = `blog.post.${id}.excerpt`
      const contentKey = `blog.post.${id}.content`

      // Upsert title
      const existingTitle = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, titleKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingTitle) {
        db.update(schema.translations)
          .set({ value: escapeHtml(trans.title), updatedAt: new Date() })
          .where(eq(schema.translations.id, existingTitle.id))
          .run()
      } else {
        db.insert(schema.translations)
          .values({ locale, key: titleKey, value: escapeHtml(trans.title) })
          .run()
      }

      // Upsert excerpt
      const existingExcerpt = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, excerptKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingExcerpt) {
        db.update(schema.translations)
          .set({ value: trans.excerpt ? escapeHtml(trans.excerpt) : '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingExcerpt.id))
          .run()
      } else if (trans.excerpt) {
        db.insert(schema.translations)
          .values({ locale, key: excerptKey, value: escapeHtml(trans.excerpt) })
          .run()
      }

      // Upsert content
      const existingContent = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, contentKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingContent) {
        db.update(schema.translations)
          .set({ value: trans.content || '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingContent.id))
          .run()
      } else if (trans.content) {
        db.insert(schema.translations)
          .values({ locale, key: contentKey, value: trans.content })
          .run()
      }
    }
  }

  // Update tags
  if (tagIds !== undefined) {
    // Remove existing tags
    db.delete(schema.blogPostTags)
      .where(eq(schema.blogPostTags.postId, id))
      .run()

    // Add new tags
    for (const tagId of tagIds) {
      db.insert(schema.blogPostTags)
        .values({ postId: id, tagId })
        .run()
    }
  }

  return {
    success: true,
    data: {
      post
    },
    post
  }
})
