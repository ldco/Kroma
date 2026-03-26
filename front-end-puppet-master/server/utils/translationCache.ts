/**
 * Translation Cache
 *
 * In-memory cache for translations to avoid DB queries on every request.
 * Cache is invalidated when translations are updated via admin panel.
 *
 * Usage:
 *   import { translationCache } from '../utils/translationCache'
 *
 *   // Get cached translations (or fetch from DB)
 *   const translations = translationCache.get('en')
 *
 *   // Invalidate after update
 *   translationCache.invalidate('en')
 *   translationCache.invalidateAll()
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../database/client'
import { logger } from './logger'

interface CacheEntry {
  data: Record<string, string>
  timestamp: number
}

// Cache TTL in milliseconds (5 minutes)
const CACHE_TTL = 5 * 60 * 1000

class TranslationCache {
  private cache = new Map<string, CacheEntry>()

  /**
   * Get translations for a locale (from cache or DB)
   */
  get(locale: string): Record<string, string> {
    const cached = this.cache.get(locale)
    const now = Date.now()

    // Return cached if valid
    if (cached && now - cached.timestamp < CACHE_TTL) {
      return cached.data
    }

    // Fetch from database
    const translations = this.fetchFromDb(locale)

    // Cache the result
    this.cache.set(locale, {
      data: translations,
      timestamp: now
    })

    logger.debug({ locale, count: Object.keys(translations).length }, 'Translation cache refreshed')

    return translations
  }

  /**
   * Fetch translations from database
   */
  private fetchFromDb(locale: string): Record<string, string> {
    const db = useDatabase()

    const rows = db
      .select({
        key: schema.translations.key,
        value: schema.translations.value
      })
      .from(schema.translations)
      .where(eq(schema.translations.locale, locale))
      .all()

    // Convert to flat object
    const result: Record<string, string> = {}
    for (const row of rows) {
      if (row.value) {
        result[row.key] = row.value
      }
    }

    return result
  }

  /**
   * Invalidate cache for a specific locale
   */
  invalidate(locale: string): void {
    this.cache.delete(locale)
    logger.debug({ locale }, 'Translation cache invalidated')
  }

  /**
   * Invalidate entire cache
   */
  invalidateAll(): void {
    this.cache.clear()
    logger.debug('Translation cache fully invalidated')
  }

  /**
   * Get cache statistics
   */
  getStats(): { locales: string[]; size: number } {
    return {
      locales: Array.from(this.cache.keys()),
      size: this.cache.size
    }
  }
}

// Singleton instance
export const translationCache = new TranslationCache()
