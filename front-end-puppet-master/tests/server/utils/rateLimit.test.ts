import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createRateLimiter } from '../../../server/utils/rateLimit'

describe('createRateLimiter', () => {
  // Use fake timers for testing time-based logic
  beforeEach(() => {
    vi.useFakeTimers()
  })

  describe('checkRateLimit', () => {
    it('should allow first request', () => {
      const limiter = createRateLimiter(5, 60000)
      expect(limiter.checkRateLimit('ip1')).toBe(true)
    })

    it('should allow requests up to the limit', () => {
      const limiter = createRateLimiter(3, 60000)

      expect(limiter.checkRateLimit('ip1')).toBe(true)
      expect(limiter.checkRateLimit('ip1')).toBe(true)
      expect(limiter.checkRateLimit('ip1')).toBe(true)
    })

    it('should deny requests over the limit', () => {
      const limiter = createRateLimiter(3, 60000)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')

      expect(limiter.checkRateLimit('ip1')).toBe(false)
      expect(limiter.checkRateLimit('ip1')).toBe(false)
    })

    it('should track different keys separately', () => {
      const limiter = createRateLimiter(2, 60000)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')

      // ip1 is at limit
      expect(limiter.checkRateLimit('ip1')).toBe(false)

      // ip2 should still be allowed
      expect(limiter.checkRateLimit('ip2')).toBe(true)
      expect(limiter.checkRateLimit('ip2')).toBe(true)
      expect(limiter.checkRateLimit('ip2')).toBe(false)
    })

    it('should reset after window expires', () => {
      const limiter = createRateLimiter(2, 60000) // 1 minute window

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')
      expect(limiter.checkRateLimit('ip1')).toBe(false)

      // Advance time past the window
      vi.advanceTimersByTime(60001)

      // Should be allowed again
      expect(limiter.checkRateLimit('ip1')).toBe(true)
    })

    it('should not reset before window expires', () => {
      const limiter = createRateLimiter(2, 60000)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')

      // Advance time but not past window
      vi.advanceTimersByTime(30000)

      expect(limiter.checkRateLimit('ip1')).toBe(false)
    })
  })

  describe('getRemaining', () => {
    it('should return full limit for new key', () => {
      const limiter = createRateLimiter(5, 60000)
      expect(limiter.getRemaining('ip1')).toBe(5)
    })

    it('should return remaining count after requests', () => {
      const limiter = createRateLimiter(5, 60000)

      limiter.checkRateLimit('ip1')
      expect(limiter.getRemaining('ip1')).toBe(4)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')
      expect(limiter.getRemaining('ip1')).toBe(2)
    })

    it('should return 0 when at limit', () => {
      const limiter = createRateLimiter(2, 60000)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')

      expect(limiter.getRemaining('ip1')).toBe(0)
    })

    it('should return full limit after window expires', () => {
      const limiter = createRateLimiter(5, 60000)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')

      vi.advanceTimersByTime(60001)

      expect(limiter.getRemaining('ip1')).toBe(5)
    })
  })

  describe('reset', () => {
    it('should reset rate limit for a key', () => {
      const limiter = createRateLimiter(2, 60000)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')
      expect(limiter.checkRateLimit('ip1')).toBe(false)

      limiter.reset('ip1')

      expect(limiter.checkRateLimit('ip1')).toBe(true)
      expect(limiter.getRemaining('ip1')).toBe(1)
    })

    it('should only reset specified key', () => {
      const limiter = createRateLimiter(2, 60000)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip2')
      limiter.checkRateLimit('ip2')

      limiter.reset('ip1')

      expect(limiter.checkRateLimit('ip1')).toBe(true)
      expect(limiter.checkRateLimit('ip2')).toBe(false)
    })
  })

  describe('clear', () => {
    it('should clear all rate limits', () => {
      const limiter = createRateLimiter(2, 60000)

      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip1')
      limiter.checkRateLimit('ip2')
      limiter.checkRateLimit('ip2')

      limiter.clear()

      expect(limiter.checkRateLimit('ip1')).toBe(true)
      expect(limiter.checkRateLimit('ip2')).toBe(true)
    })
  })
})
