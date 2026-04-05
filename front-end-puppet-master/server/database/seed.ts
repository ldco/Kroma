/**
 * Database Seed Script
 *
 * Creates initial admin user, default settings, and translations.
 * Run with: npm run db:seed
 *
 * IMPORTANT: Settings are defined in puppet-master.config.ts
 * IMPORTANT: Translations are stored in database:
 *            - System translations: seeded from i18n/system-seed.json, editable by master
 *            - Content translations: seeded from i18n/content.ts, editable by admin+
 */
import Database from 'better-sqlite3'
import { drizzle } from 'drizzle-orm/better-sqlite3'
import { scryptSync, randomBytes } from 'crypto'
import * as schema from './schema'
import { mkdirSync, existsSync, readFileSync } from 'fs'
import { join, dirname } from 'path'
import { getContentSeedData } from '../../i18n/content'
import config from '../../app/puppet-master.config'
import { encryptSecretValue, isEncryptedSecretValue } from '../utils/secrets'

// Load system translations from JSON seed file
function getSystemSeedData(): Array<{ locale: string; key: string; value: string }> {
  const seedPath = join(process.cwd(), 'i18n/system-seed.json')
  try {
    if (existsSync(seedPath)) {
      const data = readFileSync(seedPath, 'utf-8')
      return JSON.parse(data)
    }
    console.warn('   ⚠️  system-seed.json not found at:', seedPath)
    return []
  } catch (e) {
    console.warn('   ⚠️  Failed to load system-seed.json:', e)
    return []
  }
}

const DB_PATH = process.env.DATABASE_URL || './data/sqlite.db'

// Ensure data directory exists
const dir = dirname(DB_PATH)
if (!existsSync(dir)) {
  mkdirSync(dir, { recursive: true })
}

// Create database connection
const sqlite = new Database(DB_PATH)
sqlite.pragma('journal_mode = WAL')
const db = drizzle(sqlite, { schema })

// Password hashing (same as server/utils/password.ts)
function hashPassword(password: string): string {
  const salt = randomBytes(16).toString('hex')
  const hash = scryptSync(password, salt, 64).toString('hex')
  return `${salt}:${hash}`
}

async function seed() {
  console.log('🌱 Seeding database...\n')

  // ── Built-in roles ──────────────────────────────────────────────
  console.log('  📋 Seeding built-in roles...')
  const builtInRoles = [
    { name: 'Master', slug: 'master', description: 'Developer/agency — full access', permissions: JSON.stringify({}), level: 100, isBuiltIn: true, color: 'danger', icon: 'crown' },
    { name: 'Admin', slug: 'admin', description: 'Site owner — manage content & users', permissions: JSON.stringify({}), level: 50, isBuiltIn: true, color: 'warning', icon: 'shield' },
    { name: 'Editor', slug: 'editor', description: 'Content editor', permissions: JSON.stringify({}), level: 10, isBuiltIn: true, color: 'primary', icon: 'pencil' }
  ]
  for (const role of builtInRoles) {
    db.insert(schema.roles)
      .values({ ...role, createdAt: new Date(), updatedAt: new Date() })
      .onConflictDoNothing()
      .run()
  }
  const masterRole = db.select().from(schema.roles).where((roles: any) => (roles.slug === 'master')).get()
  const adminRole = db.select().from(schema.roles).where((roles: any) => (roles.slug === 'admin')).get()
  const editorRole = db.select().from(schema.roles).where((roles: any) => (roles.slug === 'editor')).get()
  console.log(`     ✅ ${builtInRoles.length} roles seeded\n`)

  // ── Admin user ──────────────────────────────────────────────────
  console.log('  👤 Seeding admin user...')
  const adminEmail = config.admin?.defaultUser?.email || 'master@example.com'
  const adminPassword = config.admin?.defaultUser?.password || 'master123'
  const adminName = config.admin?.defaultUser?.name || 'Master'

  const existingUser = db.select().from(schema.users).where((users: any) => (users.email === adminEmail)).get()
  if (!existingUser) {
    const roleId = masterRole?.id ?? null
    db.insert(schema.users)
      .values({
        email: adminEmail,
        passwordHash: hashPassword(adminPassword),
        name: adminName,
        role: 'master',
        roleId,
        createdAt: new Date(),
        updatedAt: new Date()
      })
      .run()
    console.log(`     ✅ User created: ${adminEmail}\n`)
  } else {
    console.log(`     ℹ️  User already exists: ${adminEmail}\n`)
  }

  // ── Default settings ────────────────────────────────────────────
  console.log('  ⚙️  Seeding default settings...')
  const defaultSettings = [
    { key: 'site.name', value: 'Kroma', type: 'string', group: 'site', updatedAt: new Date() },
    { key: 'site.tagline', value: 'AI-powered comic & graphic novel production', type: 'string', group: 'site', updatedAt: new Date() },
    { key: 'contact.email', value: '', type: 'email', group: 'contact', updatedAt: new Date() },
    { key: 'contact.phone', value: '', type: 'tel', group: 'contact', updatedAt: new Date() },
    { key: 'contact.location', value: '', type: 'string', group: 'contact', updatedAt: new Date() },
    { key: 'contact.telegramBotToken', value: '', type: 'password', group: 'contact', updatedAt: new Date() }
  ]
  let settingsAdded = 0
  for (const s of defaultSettings) {
    try {
      db.insert(schema.settings).values(s).onConflictDoNothing().run()
      settingsAdded++
    } catch { /* ignore duplicates */ }
  }
  console.log(`     ✅ ${settingsAdded} settings seeded\n`)

  // ── Translations ────────────────────────────────────────────────
  console.log('  🌍 Seeding translations...')
  const systemTranslations = getSystemSeedData()
  const contentTranslations = getContentSeedData()

  let translationsAdded = 0
  const allTranslations = [...systemTranslations, ...contentTranslations]

  for (const t of allTranslations) {
    try {
      db.insert(schema.translations)
        .values({ locale: t.locale, key: t.key, value: t.value })
        .onConflictDoNothing()
        .run()
      translationsAdded++
    } catch { /* ignore duplicates */ }
  }
  console.log(`     ✅ ${translationsAdded} translations seeded (${systemTranslations.length} system + ${contentTranslations.length} content)\n`)

  console.log('✅ Database seeded successfully!\n')
}

seed().catch(error => {
  console.error('❌ Seed failed:', error)
  process.exit(1)
})
