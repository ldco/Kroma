/**
 * Database Client (HIGH-01: Transaction Support)
 *
 * Singleton Drizzle client with better-sqlite3 driver.
 * Uses lazy initialization to avoid issues during build.
 * Provides transaction support for atomic operations.
 */
import { drizzle } from 'drizzle-orm/better-sqlite3'
import Database from 'better-sqlite3'
import { mkdirSync, existsSync } from 'node:fs'
import { dirname } from 'node:path'
import * as schema from './schema'
import { logger } from '../utils/logger'

// Type for the drizzle instance
type DrizzleDB = ReturnType<typeof drizzle<typeof schema>>

// Lazy singleton
let _db: DrizzleDB | null = null
let _sqlite: Database.Database | null = null

/**
 * Get the database instance (lazy initialization)
 */
export function useDatabase(): DrizzleDB {
  if (!_db) {
    const config = useRuntimeConfig()
    const dbPath = config.databaseUrl || './data/sqlite.db'

    // Ensure data directory exists
    const dir = dirname(dbPath)
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true })
    }

    _sqlite = new Database(dbPath)

    // Enable WAL mode for better concurrency
    _sqlite.pragma('journal_mode = WAL')

    // Set busy timeout to prevent "database is locked" errors (5 seconds)
    _sqlite.pragma('busy_timeout = 5000')

    _db = drizzle(_sqlite, { schema })
  }
  return _db
}

/**
 * Execute a synchronous function within a database transaction
 *
 * IMPORTANT: better-sqlite3 is synchronous, so this function only supports
 * synchronous operations. Use transactionSync() for the same functionality.
 *
 * @deprecated Use transactionSync() instead for clarity
 *
 * Usage:
 * const result = withTransaction((db) => {
 *   db.insert(schema.users).values({ ... }).run()
 *   db.insert(schema.sessions).values({ ... }).run()
 *   return { success: true }
 * })
 *
 * If any operation fails, all changes are rolled back.
 */
export function withTransaction<T>(fn: (db: DrizzleDB) => T): T {
  // Delegate to transactionSync for consistency
  return transactionSync(fn)
}

/**
 * Synchronous transaction for better-sqlite3
 * Preferred method for atomic operations
 *
 * Usage:
 * const result = transactionSync((db) => {
 *   db.insert(schema.users).values({ ... }).run()
 *   db.insert(schema.sessions).values({ ... }).run()
 *   return { success: true }
 * })
 */
export function transactionSync<T>(fn: (db: DrizzleDB) => T): T {
  const db = useDatabase()

  if (!_sqlite) {
    throw new Error('Database not initialized')
  }

  const transaction = _sqlite.transaction(() => {
    return fn(db)
  })

  try {
    return transaction()
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error)
    logger.error(
      { error: message },
      'Transaction failed'
    )
    throw error
  }
}

// Export for direct import in API routes
export const db = {
  get instance() {
    return useDatabase()
  }
}

// Re-export schema for convenience
export { schema }
