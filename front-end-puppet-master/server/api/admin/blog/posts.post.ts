/**
 * POST /api/admin/blog/posts
 *
 * Create a new blog post.
 * Content stored in centralized translations table with key pattern: blog.post.{id}.{field}
 */
import { eq } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'
import { escapeHtml } from '../../../utils/sanitize'

const createPostSchema = z.object({
  slug: z.string().min(1).max(200).regex(/^[a-z0-9-]+$/),
  categoryId: z.number().int().optional().nullable(),
  coverImageUrl: z.string().max(500).optional().nullable(),
  coverImageAlt: z.string().max(200).optional().nullable(),
  published: z.boolean().default(false),
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
  const session = event.context.session
  const body = await readBody(event)

  const result = createPostSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, tagIds, ...data } = result.data

  // Check if slug exists
  const existing = db
    .select()
    .from(schema.blogPosts)
    .where(eq(schema.blogPosts.slug, data.slug))
    .get()

  if (existing) {
    throw createError({
      statusCode: 409,
      message: 'Slug already exists'
    })
  }

  // Insert post
  const [post] = db
    .insert(schema.blogPosts)
    .values({
      slug: data.slug,
      authorId: session?.userId || null,
      categoryId: data.categoryId || null,
      coverImageUrl: data.coverImageUrl || null,
      coverImageAlt: data.coverImageAlt || null,
      published: data.published,
      publishedAt: data.published ? new Date() : null,
      readingTimeMinutes: data.readingTimeMinutes || null
    })
    .returning()

  if (!post) {
    throw createError({ statusCode: 500, message: 'Failed to create post' })
  }

  // Insert translations into centralized table
  if (translations) {
    const translationValues = []

    for (const [locale, trans] of Object.entries(translations)) {
      translationValues.push({
        locale,
        key: `blog.post.${post.id}.title`,
        value: escapeHtml(trans.title)
      })

      if (trans.excerpt) {
        translationValues.push({
          locale,
          key: `blog.post.${post.id}.excerpt`,
          value: escapeHtml(trans.excerpt)
        })
      }

      if (trans.content) {
        translationValues.push({
          locale,
          key: `blog.post.${post.id}.content`,
          value: trans.content // Markdown content - sanitized on output
        })
      }
    }

    if (translationValues.length > 0) {
      db.insert(schema.translations)
        .values(translationValues)
        .run()
    }
  }

  // Insert tags
  if (tagIds && tagIds.length > 0) {
    for (const tagId of tagIds) {
      db.insert(schema.blogPostTags)
        .values({ postId: post.id, tagId })
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
