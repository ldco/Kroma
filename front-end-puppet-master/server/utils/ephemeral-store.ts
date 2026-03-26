/**
 * Ephemeral Store Utility
 *
 * Provides TTL-based key-value storage for temporary data like 2FA pending sessions.
 * Solves cross-process contamination issues in multi-worker Nitro deployments.
 *
 * Features:
 * - Automatic TTL expiration
 * - Redis-backed storage when available (multi-process safe)
 * - In-memory fallback with periodic cleanup
 * - Type-safe interface
 *
 * Usage:
 *   const store = useEphemeralStore('2fa-pending', { ttlSeconds: 300 })
 *   await store.set('user-id', { secret: '...', qrCode: '...' })
 *   const data = await store.get('user-id')
 *   await store.delete('user-id')
 */
import { logger } from './logger'

interface EphemeralEntry<T> {
  value: T
  expiresAt: number
}

/**
 * Redis client interface for ephemeral store
 */
interface RedisClient {
  get: (key: string) => Promise<string | null>
  setex: (key: string, seconds: number, value: string) => Promise<void>
  del: (key: string) => Promise<void>
  keys: (pattern: string) => Promise<string[]>
}

/**
 * Ephemeral store instance
 */
export class EphemeralStore<T extends Record<string, unknown>> {
  private prefix: string
  private ttlSeconds: number
  private memoryStore: Map<string, EphemeralEntry<T>>
  private redisClient: RedisClient | null
  private cleanupInterval: NodeJS.Timeout | null

  constructor(prefix: string, options: { ttlSeconds: number; redisClient?: RedisClient | null } = { ttlSeconds: 300 }) {
    this.prefix = `ephemeral:${prefix}`
    this.ttlSeconds = options.ttlSeconds
    this.memoryStore = new Map()
    this.redisClient = options.redisClient ?? null
    this.cleanupInterval = null

    // Start memory cleanup if using in-memory store
    if (!this.redisClient) {
      this.startMemoryCleanup()
    }
  }

  /**
   * Initialize Redis client if REDIS_URL is available
   */
  static async initRedis(): Promise<RedisClient | null> {
    const redisUrl = process.env.REDIS_URL

    if (!redisUrl) {
      logger.debug('REDIS_URL not set - using in-memory ephemeral store')
      return null
    }

    try {
      const Redis = await import('ioredis').then(m => m.default).catch(() => null)

      if (!Redis) {
        logger.warn('ioredis not installed - using in-memory ephemeral store. Run: npm install ioredis')
        return null
      }

      const client = new Redis(redisUrl)
      await client.ping()

      logger.info('Ephemeral store using Redis')

      return {
        get: async (key: string) => client.get(key),
        setex: async (key: string, seconds: number, value: string) => {
          await client.setex(key, seconds, value)
        },
        del: async (key: string) => {
          await client.del(key)
        },
        keys: async (pattern: string) => client.keys(pattern)
      }
    } catch (error) {
      logger.warn({ error }, 'Redis connection failed - using in-memory ephemeral store')
      return null
    }
  }

  /**
   * Create an ephemeral store with Redis if available
   */
  static async create<T extends Record<string, unknown>>(
    prefix: string,
    options: { ttlSeconds: number } = { ttlSeconds: 300 }
  ): Promise<EphemeralStore<T>> {
    const redisClient = await this.initRedis()
    return new EphemeralStore<T>(prefix, { ttlSeconds: options.ttlSeconds, redisClient })
  }

  /**
   * Start periodic cleanup of expired in-memory entries
   */
  private startMemoryCleanup(): void {
    // Clean up every minute
    this.cleanupInterval = setInterval(() => {
      const now = Date.now()
      for (const [key, entry] of this.memoryStore.entries()) {
        if (entry.expiresAt < now) {
          this.memoryStore.delete(key)
        }
      }
    }, 60000)

    // Cleanup on process exit
    process.on('beforeExit', () => {
      if (this.cleanupInterval) {
        clearInterval(this.cleanupInterval)
      }
    })
  }

  /**
   * Store a value with TTL
   */
  async set(key: string, value: T): Promise<void> {
    const fullKey = `${this.prefix}:${key}`

    if (this.redisClient) {
      try {
        const serialized = JSON.stringify(value)
        await this.redisClient.setex(fullKey, this.ttlSeconds, serialized)
      } catch (error) {
        logger.warn({ error, key }, 'Redis set failed, falling back to memory')
        this.setMemory(key, value)
      }
    } else {
      this.setMemory(key, value)
    }
  }

  /**
   * Set value in memory store
   */
  private setMemory(key: string, value: T): void {
    const fullKey = `${this.prefix}:${key}`
    this.memoryStore.set(fullKey, {
      value,
      expiresAt: Date.now() + this.ttlSeconds * 1000
    })
  }

  /**
   * Get a value by key (returns null if expired or not found)
   */
  async get(key: string): Promise<T | null> {
    const fullKey = `${this.prefix}:${key}`

    if (this.redisClient) {
      try {
        const serialized = await this.redisClient.get(fullKey)
        if (!serialized) return null
        return JSON.parse(serialized) as T
      } catch (error) {
        logger.warn({ error, key }, 'Redis get failed, falling back to memory')
        return this.getMemory(key)
      }
    } else {
      return this.getMemory(key)
    }
  }

  /**
   * Get value from memory store
   */
  private getMemory(key: string): T | null {
    const fullKey = `${this.prefix}:${key}`
    const entry = this.memoryStore.get(fullKey)

    if (!entry) return null

    // Check if expired
    if (entry.expiresAt < Date.now()) {
      this.memoryStore.delete(fullKey)
      return null
    }

    return entry.value
  }

  /**
   * Delete a key
   */
  async delete(key: string): Promise<void> {
    const fullKey = `${this.prefix}:${key}`

    if (this.redisClient) {
      try {
        await this.redisClient.del(fullKey)
      } catch {
        // Ignore Redis errors on delete
      }
    }

    this.memoryStore.delete(fullKey)
  }

  /**
   * Check if a key exists (and is not expired)
   */
  async has(key: string): Promise<boolean> {
    const value = await this.get(key)
    return value !== null
  }

  /**
   * Clear all entries (for testing)
   */
  async clear(): Promise<void> {
    if (this.redisClient) {
      try {
        const keys = await this.redisClient.keys(`${this.prefix}:*`)
        for (const key of keys) {
          await this.redisClient.del(key)
        }
      } catch {
        // Ignore Redis errors on clear
      }
    }

    this.memoryStore.clear()
  }

  /**
   * Get store statistics (for debugging)
   */
  async stats(): Promise<{ size: number; source: 'redis' | 'memory' }> {
    if (this.redisClient) {
      try {
        const keys = await this.redisClient.keys(`${this.prefix}:*`)
        return { size: keys.length, source: 'redis' }
      } catch {
        // Fall through to memory
      }
    }

    // Clean before counting
    const now = Date.now()
    for (const [key, entry] of this.memoryStore.entries()) {
      if (entry.expiresAt < now) {
        this.memoryStore.delete(key)
      }
    }

    return { size: this.memoryStore.size, source: 'memory' }
  }

  /**
   * Dispose of the store (cleanup resources)
   */
  dispose(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval)
      this.cleanupInterval = null
    }
  }
}

/**
 * Create an ephemeral store instance
 *
 * @param prefix - Unique prefix for store keys (e.g., '2fa-pending')
 * @param ttlSeconds - Time-to-live in seconds (default: 300 = 5 minutes)
 *
 * @example
 * const pendingSessions = useEphemeralStore<Pending2FASession>('2fa-pending', { ttlSeconds: 300 })
 * await pendingSessions.set(userId, { secret, qrCode })
 */
export function useEphemeralStore<T extends Record<string, unknown>>(
  prefix: string,
  options: { ttlSeconds?: number } = {}
): EphemeralStore<T> {
  const ttlSeconds = options.ttlSeconds ?? 300
  const redisClientPromise = EphemeralStore.initRedis()

  // Create store synchronously, Redis will be used if available
  const store = new EphemeralStore<T>(prefix, {
    ttlSeconds,
    redisClient: null // Will be set asynchronously
  })

  // Initialize Redis client asynchronously
  redisClientPromise.then(client => {
    if (client) {
      ;(store as any).redisClient = client
      if ((store as any).cleanupInterval) {
        clearInterval((store as any).cleanupInterval)
        ;(store as any).cleanupInterval = null
      }
    }
  })

  return store
}

/**
 * Type for 2FA pending session data
 */
export interface Pending2FASession {
  secret: string
  encryptedSecret: string
  backupCodes: { plain: string[]; hashed: string[] }
  createdAt: number
}

/**
 * Pre-configured ephemeral store for 2FA pending sessions
 * TTL: 5 minutes (300 seconds)
 */
export const pending2FASessions = useEphemeralStore<Pending2FASession>('2fa-pending', { ttlSeconds: 300 })

/**
 * Type for pending 2FA verification session (during login)
 */
export interface Pending2faVerification {
  userId: number
  email: string
  rememberMe: boolean
  expiresAt: number
  attempts: number
}

/**
 * Pre-configured ephemeral store for 2FA verifications during login
 * TTL: 10 minutes (600 seconds)
 */
export const pending2faVerifications = useEphemeralStore<Pending2faVerification>('2fa-verify', { ttlSeconds: 600 })
