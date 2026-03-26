/**
 * GET /api/blog/:slug
 *
 * Returns a single blog post with full content.
 * Content stored in centralized translations table with key pattern: blog.post.{id}.{field}
 */
import { eq, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { hasAdminSectionAccess } from '../../utils/admin-rbac'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const slug = event.context.params?.slug
  const query = getQuery(event)
  const locale = (query.locale as string) || config.defaultLocale || 'en'
  const defaultLocale = config.defaultLocale || 'en'

  if (!slug) {
    throw createError({
      statusCode: 400,
      message: 'Slug is required'
    })
  }

  // Get post
  const post = db
    .select()
    .from(schema.blogPosts)
    .where(eq(schema.blogPosts.slug, slug))
    .get()

  if (!post) {
    throw createError({
      statusCode: 404,
      message: 'Post not found'
    })
  }

  // Check if published (or authorized blog editor/admin)
  const user = event.context.user
  const canReadUnpublished = user ? await hasAdminSectionAccess(user, 'blog') : false
  if (!post.published && !canReadUnpublished) {
    if (user) {
      throw createError({
        statusCode: 403,
        message: 'Access denied: unpublished posts require blog section access'
      })
    }
    throw createError({
      statusCode: 404,
      message: 'Post not found'
    })
  }

  // Increment view count
  if (post.published) {
    db.update(schema.blogPosts)
      .set({ viewCount: (post.viewCount || 0) + 1 })
      .where(eq(schema.blogPosts.id, post.id))
      .run()
  }

  // Get translations from centralized table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, `blog.post.${post.id}.%`))
    .all()

  const titleKey = `blog.post.${post.id}.title`
  const excerptKey = `blog.post.${post.id}.excerpt`
  const contentKey = `blog.post.${post.id}.content`

  const titleTrans =
    allTranslations.find(t => t.key === titleKey && t.locale === locale) ||
    allTranslations.find(t => t.key === titleKey && t.locale === defaultLocale)

  const excerptTrans =
    allTranslations.find(t => t.key === excerptKey && t.locale === locale) ||
    allTranslations.find(t => t.key === excerptKey && t.locale === defaultLocale)

  const contentTrans =
    allTranslations.find(t => t.key === contentKey && t.locale === locale) ||
    allTranslations.find(t => t.key === contentKey && t.locale === defaultLocale)

  // Get category
  let category = null
  if (post.categoryId) {
    const cat = db
      .select()
      .from(schema.blogCategories)
      .where(eq(schema.blogCategories.id, post.categoryId))
      .get()

    if (cat) {
      const catNameKey = `blog.category.${cat.id}.name`
      const catTrans = db
        .select()
        .from(schema.translations)
        .where(like(schema.translations.key, catNameKey))
        .all()

      const catNameTrans =
        catTrans.find(t => t.locale === locale) ||
        catTrans.find(t => t.locale === defaultLocale)

      category = {
        slug: cat.slug,
        name: catNameTrans?.value || cat.slug
      }
    }
  }

  // Get tags
  const postTags = db
    .select()
    .from(schema.blogPostTags)
    .where(eq(schema.blogPostTags.postId, post.id))
    .all()

  const tagIds = postTags.map(pt => pt.tagId)
  const tags = tagIds.length > 0
    ? db.select().from(schema.blogTags).all().filter(t => tagIds.includes(t.id))
    : []

  // Get author
  let author = null
  if (post.authorId) {
    const user = db
      .select()
      .from(schema.users)
      .where(eq(schema.users.id, post.authorId))
      .get()

    if (user) {
      author = { name: user.name || 'Anonymous' }
    }
  }

  // Get media
  const media = db
    .select()
    .from(schema.blogMedia)
    .where(eq(schema.blogMedia.postId, post.id))
    .all()

  return {
    success: true,
    data: {
      id: post.id,
      slug: post.slug,
      title: titleTrans?.value || 'Untitled',
      excerpt: excerptTrans?.value || null,
      content: contentTrans?.value || null,
      coverImageUrl: post.coverImageUrl,
      coverImageAlt: post.coverImageAlt,
      publishedAt: post.publishedAt,
      readingTimeMinutes: post.readingTimeMinutes,
      viewCount: post.viewCount,
      category,
      tags: tags.map(t => ({ slug: t.slug, name: t.name })),
      author,
      media: media.map(m => ({
        type: m.type,
        url: m.url,
        thumbnailUrl: m.thumbnailUrl,
        alt: m.alt
      }))
    }
  }
})
