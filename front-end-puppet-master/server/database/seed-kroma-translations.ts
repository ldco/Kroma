/**
 * Seed Kroma App Translations
 *
 * Adds Kroma-specific translations (app.*) to the database.
 * Run with: npx tsx server/database/seed-kroma-translations.ts
 */
import Database from 'better-sqlite3'
import { drizzle } from 'drizzle-orm/better-sqlite3'
import { existsSync, readFileSync } from 'fs'
import { join, dirname } from 'path'
import * as schema from './schema'

const DB_PATH = process.env.DATABASE_URL || './data/sqlite.db'
const KROMA_SEED_PATH = join(process.cwd(), 'i18n/kroma-seed.json')

// Ensure data directory exists
const dir = dirname(DB_PATH)
if (!existsSync(dir)) {
  console.error('❌ Database directory not found:', dir)
  process.exit(1)
}

if (!existsSync(DB_PATH)) {
  console.error('❌ Database not found. Run npm run db:migrate first.')
  process.exit(1)
}

const sqlite = new Database(DB_PATH)
const db = drizzle(sqlite, { schema })

async function seedKromaTranslations() {
  if (!existsSync(KROMA_SEED_PATH)) {
    console.error('❌ kroma-seed.json not found at:', KROMA_SEED_PATH)
    process.exit(1)
  }

  const data = readFileSync(KROMA_SEED_PATH, 'utf-8')
  const translations: Array<{ locale: string; key: string; value: string }> = JSON.parse(data)

  console.log(`🌍 Seeding ${translations.length} Kroma translations...\n`)

  let added = 0
  let skipped = 0

  for (const t of translations) {
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
      skipped++
    }
  }

  console.log(`✅ Seeded ${added} Kroma translations (${skipped} already existed)\n`)
}

seedKromaTranslations().catch(error => {
  console.error('❌ Seed failed:', error)
  process.exit(1)
})
