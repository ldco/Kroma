/**
 * Portfolio Migration Script
 *
 * Migrates from old portfolio_items structure to new portfolios + portfolio_items.
 *
 * Steps:
 * 1. Create portfolios table
 * 2. Create default portfolio from existing items
 * 3. Add new columns to portfolio_items
 * 4. Migrate existing items to reference the default portfolio
 *
 * Run with: npx tsx scripts/migrate-portfolio.ts
 */
import Database from 'better-sqlite3'
import { resolve } from 'path'

const dbPath = resolve('./data/sqlite.db')
const db = new Database(dbPath)

console.log('Starting portfolio migration...')

try {
  db.exec('BEGIN TRANSACTION')

  // 1. Create portfolios table if it doesn't exist
  console.log('1. Creating portfolios table...')
  db.exec(`
    CREATE TABLE IF NOT EXISTS portfolios (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      slug TEXT NOT NULL UNIQUE,
      name TEXT NOT NULL,
      description TEXT,
      type TEXT NOT NULL CHECK(type IN ('gallery', 'case_study')),
      cover_image_url TEXT,
      cover_thumbnail_url TEXT,
      "order" INTEGER DEFAULT 0,
      published INTEGER DEFAULT 0,
      created_at INTEGER,
      updated_at INTEGER
    )
  `)

  // 2. Check if there are existing portfolio_items to migrate
  const existingItems = db.prepare('SELECT COUNT(*) as count FROM portfolio_items').get() as { count: number }
  console.log(`Found ${existingItems.count} existing portfolio items`)

  let defaultPortfolioId: number | null = null

  if (existingItems.count > 0) {
    // Create a default portfolio for existing items
    console.log('2. Creating default portfolio for existing items...')
    const now = Date.now()
    const result = db.prepare(`
      INSERT INTO portfolios (slug, name, description, type, published, created_at, updated_at)
      VALUES ('portfolio', 'Portfolio', 'Migrated portfolio', 'case_study', 1, ?, ?)
    `).run(now, now)
    defaultPortfolioId = result.lastInsertRowid as number
    console.log(`Created default portfolio with ID: ${defaultPortfolioId}`)
  }

  // 3. Add new columns to portfolio_items if they don't exist
  console.log('3. Adding new columns to portfolio_items...')

  // Check existing columns
  const columns = db.prepare("PRAGMA table_info(portfolio_items)").all() as { name: string }[]
  const existingColumns = new Set(columns.map(c => c.name))

  // Add portfolio_id if not exists
  if (!existingColumns.has('portfolio_id')) {
    db.exec(`ALTER TABLE portfolio_items ADD COLUMN portfolio_id INTEGER`)
  }

  // Add item_type if not exists
  if (!existingColumns.has('item_type')) {
    db.exec(`ALTER TABLE portfolio_items ADD COLUMN item_type TEXT DEFAULT 'case_study'`)
  }

  // Add media_url if not exists (rename image_url)
  if (!existingColumns.has('media_url')) {
    db.exec(`ALTER TABLE portfolio_items ADD COLUMN media_url TEXT`)
    // Copy image_url to media_url
    db.exec(`UPDATE portfolio_items SET media_url = image_url WHERE image_url IS NOT NULL`)
  }

  // Add caption if not exists
  if (!existingColumns.has('caption')) {
    db.exec(`ALTER TABLE portfolio_items ADD COLUMN caption TEXT`)
  }

  // 4. Update existing items to reference the default portfolio
  if (defaultPortfolioId !== null) {
    console.log('4. Updating existing items to reference default portfolio...')
    db.prepare(`
      UPDATE portfolio_items
      SET portfolio_id = ?, item_type = 'case_study'
      WHERE portfolio_id IS NULL
    `).run(defaultPortfolioId)
  }

  db.exec('COMMIT')
  console.log('Migration completed successfully!')

  // Show summary
  const portfolios = db.prepare('SELECT COUNT(*) as count FROM portfolios').get() as { count: number }
  const items = db.prepare('SELECT COUNT(*) as count FROM portfolio_items').get() as { count: number }
  console.log(`\nSummary:`)
  console.log(`- Portfolios: ${portfolios.count}`)
  console.log(`- Portfolio Items: ${items.count}`)

} catch (error) {
  db.exec('ROLLBACK')
  console.error('Migration failed:', error)
  process.exit(1)
} finally {
  db.close()
}
