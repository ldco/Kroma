/**
 * System Translation Sync
 *
 * Automatically syncs system translations from database to JSON file.
 * This ensures new deployments have up-to-date system translations.
 */
import { writeFileSync } from 'fs'
import { join } from 'path'
import { useDatabase, schema } from '../database/client'
import { asc } from 'drizzle-orm'
import { isSystemKey } from '../../i18n/system'
import { logger } from './logger'

/**
 * Export all system translations from DB to JSON file
 * Called automatically when system translations are modified
 */
export function syncSystemTranslationsToFile(): void {
  try {
    const db = useDatabase()

    // Get all system translations
    const rows = db
      .select({
        locale: schema.translations.locale,
        key: schema.translations.key,
        value: schema.translations.value
      })
      .from(schema.translations)
      .orderBy(asc(schema.translations.locale), asc(schema.translations.key))
      .all()

    // Filter to system keys only
    const systemRows = rows
      .filter(row => isSystemKey(row.key))
      .map(row => ({
        locale: row.locale,
        key: row.key,
        value: row.value ?? ''
      }))

    // Write to JSON file
    const seedPath = join(process.cwd(), 'i18n/system-seed.json')
    writeFileSync(seedPath, JSON.stringify(systemRows, null, 2), 'utf-8')

    logger.info({ count: systemRows.length }, 'Synced system translations to system-seed.json')
  } catch (error) {
    logger.error({ error }, 'Failed to sync system translations')
  }
}
