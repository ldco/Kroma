/**
 * GET /api/blog/categories
 *
 * Returns list of blog categories with translations.
 * Content stored in centralized translations table with key pattern: blog.category.{id}.{field}
 */
import { asc, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)
  const locale = (query.locale as string) || config.defaultLocale || 'en'
  const defaultLocale = config.defaultLocale || 'en'

  // Get categories
  const categories = db
    .select()
    .from(schema.blogCategories)
    .orderBy(asc(schema.blogCategories.order))
    .all()

  // Get translations from centralized table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'blog.category.%'))
    .all()

  return {
    success: true,
    data: categories.map(cat => {
      const nameKey = `blog.category.${cat.id}.name`
      const descKey = `blog.category.${cat.id}.description`

      const nameTrans =
        allTranslations.find(t => t.key === nameKey && t.locale === locale) ||
        allTranslations.find(t => t.key === nameKey && t.locale === defaultLocale)

      const descTrans =
        allTranslations.find(t => t.key === descKey && t.locale === locale) ||
        allTranslations.find(t => t.key === descKey && t.locale === defaultLocale)

      return {
        id: cat.id,
        slug: cat.slug,
        name: nameTrans?.value || cat.slug,
        description: descTrans?.value || null
      }
    })
  }
})
