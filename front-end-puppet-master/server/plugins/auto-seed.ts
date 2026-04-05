/**
 * Auto-Seed Plugin
 *
 * Automatically seeds the database on first server start.
 * Checks if translations table is empty and seeds if needed.
 */
import { existsSync, readFileSync } from 'fs'
import { join } from 'path'
import { useDatabase, schema } from '../database/client'
import { getContentSeedData } from '../../i18n/content'
import { count } from 'drizzle-orm'

// Flatten nested object into dot-notation key-value pairs
function flattenTranslations(obj: Record<string, unknown>, prefix = ''): Array<{ key: string; value: string }> {
  const result: Array<{ key: string; value: string }> = []
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      result.push(...flattenTranslations(value as Record<string, unknown>, fullKey))
    } else if (typeof value === 'string') {
      result.push({ key: fullKey, value })
    }
  }
  return result
}

// Load Kroma app translations from kroma-seed.json
function getKromaSeedData(): Array<{ locale: string; key: string; value: string }> {
  try {
    const kromaPath = join(process.cwd(), 'i18n/kroma-seed.json')
    if (!existsSync(kromaPath)) return []
    const data = readFileSync(kromaPath, 'utf-8')
    return JSON.parse(data)
  } catch (e) {
    console.error('[auto-seed] Failed to load kroma-seed.json:', e)
    return []
  }
}

export default defineNitroPlugin(async () => {
  const db = useDatabase()

  // Check if translations exist
  const result = db.select({ count: count() }).from(schema.translations).get()
  const translationCount = result?.count ?? 0

  if (translationCount > 0) {
    console.log(`[auto-seed] Database has ${translationCount} translations, skipping seed`)
    return
  }

  console.log('[auto-seed] Empty database detected, seeding translations...')

  // Load system translations from JSON
  const seedPath = join(process.cwd(), 'i18n/system-seed.json')
  let systemTranslations: Array<{ locale: string; key: string; value: string }> = []

  if (existsSync(seedPath)) {
    try {
      const data = readFileSync(seedPath, 'utf-8')
      systemTranslations = JSON.parse(data)
    } catch (e) {
      console.error('[auto-seed] Failed to load system-seed.json:', e)
    }
  }

  // Get content translations
  const contentTranslations = getContentSeedData()

  // Get Kroma app translations
  const kromaTranslations = getKromaSeedData()

  // Seed all translations
  const allTranslations = [...systemTranslations, ...contentTranslations, ...kromaTranslations]
  let added = 0

  for (const t of allTranslations) {
    try {
      db.insert(schema.translations)
        .values({
          locale: t.locale,
          key: t.key,
          value: t.value
        })
        .onConflictDoNothing()
        .run()
      added++
    } catch {
      // Ignore duplicates
    }
  }

  console.log(`[auto-seed] Seeded ${added} translations (${systemTranslations.length} system + ${contentTranslations.length} content + ${kromaTranslations.length} kroma)`)
})
