/**
 * GET /api/blog/posts
 *
 * Returns list of published blog posts with translations.
 * Content stored in centralized translations table with key pattern: blog.post.{id}.{field}
 *
 * Query params:
 *   - locale: string - Get translations for this locale
 *   - category: string - Filter by category slug
 *   - tag: string - Filter by tag slug
 *   - limit: number - Limit results
 *   - offset: number - Pagination offset
 */
import { eq, desc, and, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)
  const locale = (query.locale as string) || config.defaultLocale || 'en'
  const defaultLocale = config.defaultLocale || 'en'

  // Build query conditions
  const conditions = [eq(schema.blogPosts.published, true)]

  // Filter by category
  if (query.category && typeof query.category === 'string') {
    const category = db
      .select()
      .from(schema.blogCategories)
      .where(eq(schema.blogCategories.slug, query.category))
      .get()

    if (category) {
      conditions.push(eq(schema.blogPosts.categoryId, category.id))
    }
  }

  // Parse pagination
  const limit = query.limit ? parseInt(query.limit as string) : 10
  const offset = query.offset ? parseInt(query.offset as string) : 0

  // Get posts
  const posts = db
    .select()
    .from(schema.blogPosts)
    .where(and(...conditions))
    .orderBy(desc(schema.blogPosts.publishedAt), desc(schema.blogPosts.createdAt))
    .limit(limit)
    .offset(offset)
    .all()

  if (posts.length === 0) {
    return { success: true, data: [] }
  }

  // Get translations from centralized table for all posts
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'blog.post.%'))
    .all()

  // Get categories
  const categories = db.select().from(schema.blogCategories).all()

  // Get category translations from centralized table
  const categoryTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'blog.category.%'))
    .all()

  // Get tags for posts
  const postTags = db.select().from(schema.blogPostTags).all()
  const tags = db.select().from(schema.blogTags).all()

  // Get authors
  const authorIds = [...new Set(posts.map(p => p.authorId).filter(Boolean))] as number[]
  const authors = authorIds.length > 0
    ? db.select().from(schema.users).all().filter(u => authorIds.includes(u.id))
    : []

  // Build response
  return {
    success: true,
    data: posts.map(post => {
      const titleKey = `blog.post.${post.id}.title`
      const excerptKey = `blog.post.${post.id}.excerpt`

      // Get translation for locale with fallback to default
      const titleTrans =
        allTranslations.find(t => t.key === titleKey && t.locale === locale) ||
        allTranslations.find(t => t.key === titleKey && t.locale === defaultLocale)

      const excerptTrans =
        allTranslations.find(t => t.key === excerptKey && t.locale === locale) ||
        allTranslations.find(t => t.key === excerptKey && t.locale === defaultLocale)

      // Get category
      const category = post.categoryId
        ? categories.find(c => c.id === post.categoryId)
        : null

      let categoryName = category?.slug || null
      if (category) {
        const catNameKey = `blog.category.${category.id}.name`
        const catTrans =
          categoryTranslations.find(t => t.key === catNameKey && t.locale === locale) ||
          categoryTranslations.find(t => t.key === catNameKey && t.locale === defaultLocale)
        if (catTrans) categoryName = catTrans.value
      }

      // Get tags
      const postTagIds = postTags.filter(pt => pt.postId === post.id).map(pt => pt.tagId)
      const postTagList = tags.filter(t => postTagIds.includes(t.id))

      // Get author
      const author = post.authorId
        ? authors.find(u => u.id === post.authorId)
        : null

      return {
        id: post.id,
        slug: post.slug,
        title: titleTrans?.value || 'Untitled',
        excerpt: excerptTrans?.value || null,
        coverImageUrl: post.coverImageUrl,
        coverImageAlt: post.coverImageAlt,
        publishedAt: post.publishedAt,
        readingTimeMinutes: post.readingTimeMinutes,
        viewCount: post.viewCount,
        category: category ? {
          slug: category.slug,
          name: categoryName
        } : null,
        tags: postTagList.map(t => ({ slug: t.slug, name: t.name })),
        author: author ? {
          name: author.name || 'Anonymous'
        } : null
      }
    })
  }
})
