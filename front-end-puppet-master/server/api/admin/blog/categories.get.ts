/**
 * GET /api/admin/blog/categories
 *
 * Get all blog categories with translations for admin.
 * Content stored in centralized translations table with key pattern: blog.category.{id}.{field}
 */
import { asc, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async () => {
  const db = useDatabase()

  const categories = db
    .select()
    .from(schema.blogCategories)
    .orderBy(asc(schema.blogCategories.order))
    .all()

  // Get all translations from centralized table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'blog.category.%'))
    .all()

  const categoriesWithTranslations = categories.map(cat => {
    const nameKey = `blog.category.${cat.id}.name`
    const descKey = `blog.category.${cat.id}.description`

    // Group translations by locale
    const catTranslations: Record<string, { name: string; description: string | null }> = {}
    const catTrans = allTranslations.filter(t => t.key.startsWith(`blog.category.${cat.id}.`))
    const locales = [...new Set(catTrans.map(t => t.locale))]

    for (const locale of locales) {
      const name = allTranslations.find(t => t.key === nameKey && t.locale === locale)
      const desc = allTranslations.find(t => t.key === descKey && t.locale === locale)

      if (name) {
        catTranslations[locale] = {
          name: name.value,
          description: desc?.value || null
        }
      }
    }

    return {
      ...cat,
      translations: catTranslations
    }
  })

  return {
    success: true,
    data: categoriesWithTranslations,
    categories: categoriesWithTranslations
  }
})
