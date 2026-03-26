/**
 * Cache Layer
 * In-memory cache with optional Redis support for multi-instance deployments
 */
import type { CacheConfig, CacheEntry } from './types'
import { logger } from '../logger'

export function useCache(config: CacheConfig) {
  // In-memory cache (Map for single-instance deployments)
  const memoryCache = new Map<string, CacheEntry<unknown>>()

  // Cleanup interval (remove expired entries every 5 minutes)
  const cleanupInterval = 5 * 60 * 1000
  let cleanupTimer: NodeJS.Timeout | null = null

  /**
   * Start automatic cleanup of expired entries
   */
  function startCleanup() {
    if (cleanupTimer) return

    cleanupTimer = setInterval(() => {
      const now = Date.now()
      let expiredCount = 0

      for (const [key, entry] of memoryCache.entries()) {
        if (entry.expiresAt <= now) {
          memoryCache.delete(key)
          expiredCount++
        }
      }

      if (expiredCount > 0) {
        logger.debug({ expiredCount }, 'Cache cleanup: removed expired entries')
      }
    }, cleanupInterval)
  }

  /**
   * Stop automatic cleanup
   */
  function stopCleanup() {
    if (cleanupTimer) {
      clearInterval(cleanupTimer)
      cleanupTimer = null
    }
  }

  /**
   * Get value from cache
   */
  async function get<T>(key: string): Promise<T | null> {
    if (!config.enabled) return null

    // Try memory cache
    const entry = memoryCache.get(key)
    if (entry) {
      // Check expiration
      if (entry.expiresAt > Date.now()) {
        return entry.data as T
      }
      // Expired - remove
      memoryCache.delete(key)
    }

    // TODO: Add Redis support for multi-instance deployments
    // const runtimeConfig = useRuntimeConfig()
    // if (runtimeConfig.redisUrl) {
    //   const redis = await connectRedis(runtimeConfig.redisUrl)
    //   const value = await redis.get(runtimeConfig.redisPrefix + key)
    //   if (value) return JSON.parse(value)
    // }

    return null
  }

  /**
   * Set value in cache
   */
  async function set<T>(key: string, data: T, ttl?: number): Promise<void> {
    if (!config.enabled) return

    const expiresAt = Date.now() + (ttl || 300) * 1000

    // Store in memory cache
    memoryCache.set(key, { data, expiresAt })

    // TODO: Add Redis support
    // const runtimeConfig = useRuntimeConfig()
    // if (runtimeConfig.redisUrl) {
    //   const redis = await connectRedis(runtimeConfig.redisUrl)
    //   await redis.setex(
    //     runtimeConfig.redisPrefix + key,
    //     ttl || 300,
    //     JSON.stringify(data)
    //   )
    // }
  }

  /**
   * Delete value from cache
   */
  async function del(key: string): Promise<void> {
    memoryCache.delete(key)

    // TODO: Add Redis support
    // const runtimeConfig = useRuntimeConfig()
    // if (runtimeConfig.redisUrl) {
    //   const redis = await connectRedis(runtimeConfig.redisUrl)
    //   await redis.del(runtimeConfig.redisPrefix + key)
    // }
  }

  /**
   * Clear all cache entries
   */
  async function clear(): Promise<void> {
    const count = memoryCache.size
    memoryCache.clear()
    logger.info({ count }, 'Cache cleared')

    // TODO: Add Redis support
    // const runtimeConfig = useRuntimeConfig()
    // if (runtimeConfig.redisUrl) {
    //   const redis = await connectRedis(runtimeConfig.redisUrl)
    //   await redis.flushdb()
    // }
  }

  /**
   * Clear cache entries by prefix
   */
  async function clearPrefix(prefix: string): Promise<void> {
    let count = 0

    // Memory cache
    for (const key of memoryCache.keys()) {
      if (key.startsWith(prefix)) {
        memoryCache.delete(key)
        count++
      }
    }

    if (count > 0) {
      logger.debug({ count, prefix }, 'Cache prefix cleared')
    }

    // TODO: Add Redis support with SCAN pattern
    // const runtimeConfig = useRuntimeConfig()
    // if (runtimeConfig.redisUrl) {
    //   const redis = await connectRedis(runtimeConfig.redisUrl)
    //   const pattern = runtimeConfig.redisPrefix + prefix + '*'
    //   // Use SCAN to find matching keys, then DEL them
    // }
  }

  /**
   * Get cache statistics
   */
  function getStats() {
    const now = Date.now()
    let total = memoryCache.size
    let expired = 0

    for (const entry of memoryCache.values()) {
      if (entry.expiresAt <= now) {
        expired++
      }
    }

    return {
      total,
      active: total - expired,
      expired
    }
  }

  // Start cleanup on creation
  if (config.enabled) {
    startCleanup()
  }

  // Cleanup on server shutdown (prevent memory leaks)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const hot = (import.meta as any).hot
  if (hot) {
    // HMR cleanup (development)
    hot.dispose(() => {
      stopCleanup()
    })
  }

  // Node.js process cleanup (production)
  if (typeof process !== 'undefined') {
    const cleanup = () => stopCleanup()
    process.on('SIGTERM', cleanup)
    process.on('SIGINT', cleanup)
    process.on('beforeExit', cleanup)
  }

  return {
    get,
    set,
    del,
    clear,
    clearPrefix,
    getStats,
    startCleanup,
    stopCleanup
  }
}
