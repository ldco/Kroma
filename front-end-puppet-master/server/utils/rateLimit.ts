/**
 * Rate Limiting Utility
 *
 * Provides rate limiting for API endpoints with two storage backends:
 * - In-memory (default): Simple Map-based storage for single-instance deployments
 * - Redis (optional): For multi-instance deployments behind a load balancer
 *
 * To enable Redis rate limiting:
 * 1. Install ioredis: npm install ioredis
 * 2. Set REDIS_URL environment variable
 *
 * Usage:
 *   import { loginRateLimiter, getClientIp } from '../utils/rateLimit'
 *
 *   const ip = getClientIp(event)
 *   if (!loginRateLimiter.checkRateLimit(ip)) {
 *     throw createError({ statusCode: 429, message: 'Too many requests' })
 *   }
 */
import { logger } from './logger'

interface RateLimitEntry {
  count: number
  resetAt: number
}

// Redis client (lazy-loaded if REDIS_URL is set)
let redisClient: {
  get: (key: string) => Promise<string | null>
  incr: (key: string) => Promise<number>
  expire: (key: string, seconds: number) => Promise<boolean>
  setex: (key: string, seconds: number, value: string) => Promise<void>
  del: (key: string) => Promise<void>
} | null = null

let redisInitialized = false

/**
 * Initialize Redis client if REDIS_URL is available
 * Returns true if Redis is available, false if using in-memory fallback
 */
async function initRedis(): Promise<boolean> {
  if (redisInitialized) return redisClient !== null

  redisInitialized = true

  try {
    const redisUrl = process.env.REDIS_URL

    if (!redisUrl) {
      logger.debug('REDIS_URL not set - using in-memory rate limiting')
      return false
    }

    // Dynamic import ioredis (may not be installed)
    const Redis = await import('ioredis').then(m => m.default).catch(() => null)

    if (!Redis) {
      logger.warn('ioredis not installed - using in-memory rate limiting. Run: npm install ioredis')
      return false
    }

    const client = new Redis(redisUrl)

    // Test connection
    await client.ping()

    redisClient = {
      get: async (key: string) => client.get(key),
      incr: async (key: string) => client.incr(key),
      expire: async (key: string, seconds: number) => (await client.expire(key, seconds)) === 1,
      setex: async (key: string, seconds: number, value: string) => {
        await client.setex(key, seconds, value)
      },
      del: async (key: string) => {
        await client.del(key)
      }
    }

    logger.info('Rate limiting using Redis')
    return true
  } catch (error) {
    logger.warn({ error }, 'Redis connection failed - using in-memory rate limiting')
    return false
  }
}

/**
 * Create a rate limiter with configurable limits
 * Supports both in-memory and Redis storage
 */
export function createRateLimiter(limit: number, windowMs: number, prefix: string = 'rl') {
  const rateLimitMap = new Map<string, RateLimitEntry>()
  const windowSec = Math.ceil(windowMs / 1000)

  /**
   * Check if request is within rate limit
   * @param key - Unique identifier (e.g., IP address)
   * @returns true if request is allowed, false if rate limited
   */
  async function checkRateLimitAsync(key: string): Promise<boolean> {
    // Try Redis first
    if (redisClient) {
      return checkRateLimitRedis(key)
    }

    // Initialize Redis if not done yet
    await initRedis()

    if (redisClient) {
      return checkRateLimitRedis(key)
    }

    // Fallback to in-memory
    return checkRateLimitMemory(key)
  }

  /**
   * Synchronous rate limit check (in-memory only)
   * Use this for performance-critical paths where Redis isn't needed
   */
  function checkRateLimit(key: string): boolean {
    return checkRateLimitMemory(key)
  }

  /**
   * In-memory rate limit check
   */
  function checkRateLimitMemory(key: string): boolean {
    const now = Date.now()
    const entry = rateLimitMap.get(key)

    // No entry or window expired - allow and reset
    if (!entry || entry.resetAt < now) {
      rateLimitMap.set(key, { count: 1, resetAt: now + windowMs })
      return true
    }

    // Within window but at limit - deny
    if (entry.count >= limit) {
      return false
    }

    // Within window and under limit - allow and increment
    entry.count++
    return true
  }

  /**
   * Redis rate limit check using atomic INCR
   * Uses atomic increment to prevent race conditions in concurrent requests
   */
  async function checkRateLimitRedis(key: string): Promise<boolean> {
    if (!redisClient) return checkRateLimitMemory(key)

    const redisKey = `${prefix}:${key}`

    try {
      // Atomic increment - returns the new count after incrementing
      const count = await redisClient.incr(redisKey)

      if (count === 1) {
        // First request in window - set expiration
        await redisClient.expire(redisKey, windowSec)
        return true
      }

      if (count > limit) {
        // Over limit - deny (don't increment expiration, let it expire naturally)
        return false
      }

      // Within limit - allow
      return true
    } catch (error) {
      logger.warn({ error, key }, 'Redis rate limit check failed, falling back to memory')
      return checkRateLimitMemory(key)
    }
  }

  /**
   * Get remaining requests for a key
   */
  function getRemaining(key: string): number {
    const now = Date.now()
    const entry = rateLimitMap.get(key)

    if (!entry || entry.resetAt < now) {
      return limit
    }

    return Math.max(0, limit - entry.count)
  }

  /**
   * Reset rate limit for a key (e.g., after successful captcha)
   */
  async function reset(key: string): Promise<void> {
    rateLimitMap.delete(key)

    if (redisClient) {
      try {
        await redisClient.del(`${prefix}:${key}`)
      } catch {
        // Ignore Redis errors on reset
      }
    }
  }

  /**
   * Clear all rate limit entries (for testing)
   */
  function clear(): void {
    rateLimitMap.clear()
  }

  return {
    checkRateLimit,
    checkRateLimitAsync,
    getRemaining,
    reset,
    clear
  }
}

// Default contact form rate limiter (5 per hour)
export const contactRateLimiter = createRateLimiter(5, 60 * 60 * 1000, 'contact')

// Login rate limiter (5 attempts per 15 minutes per IP)
// Stricter than contact form to prevent brute force
export const loginRateLimiter = createRateLimiter(5, 15 * 60 * 1000, 'login')

// ═══════════════════════════════════════════════════════════════════════════
// Two-Factor Authentication Rate Limiters
// ═══════════════════════════════════════════════════════════════════════════

// 2FA Setup rate limiter (10 attempts per hour per user)
// Prevents abuse of QR code generation
export const twoFactorSetupRateLimiter = createRateLimiter(10, 60 * 60 * 1000, '2fa-setup')

// 2FA Enable rate limiter (5 attempts per 15 minutes per user)
// Prevents brute-forcing the verification code
export const twoFactorEnableRateLimiter = createRateLimiter(5, 15 * 60 * 1000, '2fa-enable')

// 2FA Verify rate limiter (5 attempts per 15 minutes per IP)
// Strict limit during login to prevent code brute-forcing
export const twoFactorVerifyRateLimiter = createRateLimiter(5, 15 * 60 * 1000, '2fa-verify')

// 2FA Disable rate limiter (3 attempts per 15 minutes per user)
// Stricter since this is a sensitive operation
export const twoFactorDisableRateLimiter = createRateLimiter(3, 15 * 60 * 1000, '2fa-disable')

/**
 * Get client IP from H3 event
 * Handles proxies (X-Forwarded-For, X-Real-IP)
 */
export function getClientIp(event: { node?: { req?: { headers?: Record<string, string | string[] | undefined>; socket?: { remoteAddress?: string } } } }): string {
  // Try X-Forwarded-For first (for proxies/load balancers)
  const forwardedFor = event.node?.req?.headers?.['x-forwarded-for']
  if (forwardedFor) {
    // X-Forwarded-For can be comma-separated, first is original client
    const ip =
      typeof forwardedFor === 'string'
        ? forwardedFor.split(',')[0]?.trim()
        : forwardedFor[0]?.split(',')[0]?.trim()
    if (ip) return ip
  }

  // Try X-Real-IP (used by some proxies)
  const realIp = event.node?.req?.headers?.['x-real-ip']
  if (realIp) {
    return typeof realIp === 'string' ? realIp : (realIp[0] ?? 'unknown')
  }

  // Fall back to socket address
  const socketAddress = event.node?.req?.socket?.remoteAddress
  if (socketAddress) return socketAddress

  return 'unknown'
}
