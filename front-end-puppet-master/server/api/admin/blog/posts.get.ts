/**
 * GET /api/admin/blog/posts
 *
 * Get all blog posts with translations for admin.
 * Content stored in centralized translations table with key pattern: blog.post.{id}.{field}
 */
import { desc, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async () => {
  const db = useDatabase()

  // Get all posts
  const posts = db
    .select()
    .from(schema.blogPosts)
    .orderBy(desc(schema.blogPosts.createdAt))
    .all()

  // Get all translations from centralized table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'blog.post.%'))
    .all()

  // Get categories
  const categories = db.select().from(schema.blogCategories).all()

  // Get authors
  const users = db.select().from(schema.users).all()

  // Get tags for posts
  const postTags = db.select().from(schema.blogPostTags).all()
  const tags = db.select().from(schema.blogTags).all()

  const postsWithRelations = posts.map(post => {
    // Group translations by locale from centralized table
    const postTranslations: Record<string, { title: string; excerpt: string | null; content: string | null }> = {}

    const titleKey = `blog.post.${post.id}.title`
    const excerptKey = `blog.post.${post.id}.excerpt`
    const contentKey = `blog.post.${post.id}.content`

    // Get all locales for this post
    const postTrans = allTranslations.filter(t => t.key.startsWith(`blog.post.${post.id}.`))
    const locales = [...new Set(postTrans.map(t => t.locale))]

    for (const locale of locales) {
      const title = allTranslations.find(t => t.key === titleKey && t.locale === locale)
      const excerpt = allTranslations.find(t => t.key === excerptKey && t.locale === locale)
      const content = allTranslations.find(t => t.key === contentKey && t.locale === locale)

      if (title) {
        postTranslations[locale] = {
          title: title.value,
          excerpt: excerpt?.value || null,
          content: content?.value || null
        }
      }
    }

    // Get category
    const category = post.categoryId
      ? categories.find(c => c.id === post.categoryId)
      : null

    // Get author
    const author = post.authorId
      ? users.find(u => u.id === post.authorId)
      : null

    // Get tags
    const postTagIds = postTags.filter(pt => pt.postId === post.id).map(pt => pt.tagId)
    const postTagList = tags.filter(t => postTagIds.includes(t.id))

    return {
      ...post,
      translations: postTranslations,
      category: category ? { id: category.id, slug: category.slug } : null,
      author: author ? { id: author.id, name: author.name } : null,
      tags: postTagList.map(t => ({ id: t.id, slug: t.slug, name: t.name }))
    }
  })

  return {
    success: true,
    data: postsWithRelations,
    posts: postsWithRelations
  }
})
