/**
 * i18n Translations API - GET
 *
 * Returns all translations for a specific locale from the database.
 * Uses in-memory cache for performance.
 */
import { translationCache } from '../../utils/translationCache'

export default defineEventHandler(async event => {
  const locale = getRouterParam(event, 'locale')

  if (!locale) {
    throw createError({ statusCode: 400, message: 'Locale is required' })
  }

  // Get translations from cache (or DB if not cached)
  const flatTranslations = translationCache.get(locale)

  // Convert to nested object format for i18n
  // e.g., { 'nav.home': 'Home' } -> { nav: { home: 'Home' } }
  type NestedRecord = { [key: string]: string | NestedRecord }
  const result: NestedRecord = {}

  for (const [key, value] of Object.entries(flatTranslations)) {
    const keys = key.split('.')
    let current: NestedRecord = result

    for (let i = 0; i < keys.length - 1; i++) {
      const k = keys[i]!
      if (!current[k] || typeof current[k] === 'string') {
        current[k] = {}
      }
      current = current[k] as NestedRecord
    }

    const lastKey = keys[keys.length - 1]!
    current[lastKey] = value
  }

  return {
    success: true,
    data: result
  }
})
