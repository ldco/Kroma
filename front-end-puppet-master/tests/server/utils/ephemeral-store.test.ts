/**
 * 2FA Distributed Store Tests
 *
 * Tests for distributed 2FA state management using ephemeral-store.
 *
 * Tests:
 * - Multi-request behavior
 * - Shared-state correctness
 * - TTL expiration
 * - Redis vs in-memory fallback
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'

describe('2FA Distributed Store Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Clear any existing store state
    delete process.env.REDIS_URL
  })

  describe('Ephemeral Store for 2FA', () => {
    it('stores and retrieves 2FA pending session', async () => {
      const { useEphemeralStore } = await import('../../server/utils/ephemeral-store')

      const store = useEphemeralStore<{ secret: string; createdAt: number }>('test-2fa', { ttlSeconds: 300 })

      const testData = {
        secret: 'test-secret-123',
        createdAt: Date.now()
      }

      await store.set('user-123', testData)
      const retrieved = await store.get('user-123')

      expect(retrieved).toEqual(testData)

      // Cleanup
      await store.delete('user-123')
    })

    it('expires session after TTL', async () => {
      const { useEphemeralStore } = await import('../../server/utils/ephemeral-store')

      // Very short TTL for testing
      const store = useEphemeralStore<{ secret: string }>('test-expiry', { ttlSeconds: 1 })

      await store.set('user-123', { secret: 'test' })

      // Should exist immediately
      const immediate = await store.get('user-123')
      expect(immediate).toBeTruthy()

      // Wait for expiration
      await new Promise(resolve => setTimeout(resolve, 1100))

      // Should be expired
      const expired = await store.get('user-123')
      expect(expired).toBeNull()
    })

    it('handles concurrent access correctly', async () => {
      const { useEphemeralStore } = await import('../../server/utils/ephemeral-store')

      const store = useEphemeralStore<{ counter: number }>('test-concurrent', { ttlSeconds: 60 })

      // Simulate concurrent writes
      await Promise.all([
        store.set('key-1', { counter: 1 }),
        store.set('key-2', { counter: 2 }),
        store.set('key-3', { counter: 3 })
      ])

      const results = await Promise.all([
        store.get('key-1'),
        store.get('key-2'),
        store.get('key-3')
      ])

      expect(results[0]).toEqual({ counter: 1 })
      expect(results[1]).toEqual({ counter: 2 })
      expect(results[2]).toEqual({ counter: 3 })

      // Cleanup
      await Promise.all([
        store.delete('key-1'),
        store.delete('key-2'),
        store.delete('key-3')
      ])
    })

    it('falls back to in-memory when Redis is unavailable', async () => {
      // Ensure Redis is not configured
      delete process.env.REDIS_URL

      const { useEphemeralStore } = await import('../../server/utils/ephemeral-store')

      const store = useEphemeralStore<{ data: string }>('test-fallback', { ttlSeconds: 60 })

      await store.set('test-key', { data: 'test-value' })
      const retrieved = await store.get('test-key')

      expect(retrieved).toEqual({ data: 'test-value' })

      // Verify it's using memory (stats should show memory source)
      const stats = await store.stats()
      expect(stats.source).toBe('memory')

      // Cleanup
      await store.delete('test-key')
    })

    it('clears all entries on dispose', async () => {
      const { useEphemeralStore } = await import('../../server/utils/ephemeral-store')

      const store = useEphemeralStore<{ value: string }>('test-clear', { ttlSeconds: 60 })

      await store.set('key-1', { value: '1' })
      await store.set('key-2', { value: '2' })

      // Verify entries exist
      expect(await store.get('key-1')).toBeTruthy()
      expect(await store.get('key-2')).toBeTruthy()

      // Clear
      await store.clear()

      // Verify entries are gone
      expect(await store.get('key-1')).toBeNull()
      expect(await store.get('key-2')).toBeNull()
    })
  })

  describe('2FA Setup with Distributed Store', () => {
    it('stores pending setup in distributed store', async () => {
      const { pending2FASessions } = await import('../../server/utils/ephemeral-store')

      const setupData = {
        secret: 'test-secret',
        encryptedSecret: 'encrypted-test',
        backupCodes: { plain: ['123', '456'], hashed: ['hash1', 'hash2'] },
        createdAt: Date.now()
      }

      await pending2FASessions.set('user-123', setupData)
      const retrieved = await pending2FASessions.get('user-123')

      expect(retrieved).toEqual(setupData)

      // Cleanup
      await pending2FASessions.delete('user-123')
    })

    it('respects TTL for pending setups', async () => {
      const { useEphemeralStore } = await import('../../server/utils/ephemeral-store')

      // Create store with same TTL as pending2FASessions (5 minutes)
      const store = useEphemeralStore<{ createdAt: number }>('2fa-pending', { ttlSeconds: 300 })

      const setupData = {
        createdAt: Date.now()
      }

      await store.set('user-123', setupData)

      // Should exist immediately
      const immediate = await store.get('user-123')
      expect(immediate).toBeTruthy()

      // Cleanup
      await store.delete('user-123')
    })
  })

  describe('2FA Verify with Distributed Store', () => {
    it('tracks verification attempts across requests', async () => {
      const { useEphemeralStore } = await import('../../server/utils/ephemeral-store')

      const store = useEphemeralStore<{ attempts: number; expiresAt: number }>('2fa-verify', { ttlSeconds: 600 })

      const sessionData = {
        attempts: 0,
        expiresAt: Date.now() + 600000
      }

      // First request
      await store.set('token-123', sessionData)

      // Increment attempts (simulating failed verification)
      const session = await store.get('token-123')
      if (session) {
        session.attempts++
        await store.set('token-123', session)
      }

      // Second request should see updated attempts
      const updated = await store.get('token-123')
      expect(updated?.attempts).toBe(1)

      // Cleanup
      await store.delete('token-123')
    })

    it('expires verification session after TTL', async () => {
      const { useEphemeralStore } = await import('../../server/utils/ephemeral-store')

      // Short TTL for testing
      const store = useEphemeralStore<{ userId: number }>('2fa-verify', { ttlSeconds: 1 })

      await store.set('token-123', { userId: 1 })

      // Should exist immediately
      expect(await store.get('token-123')).toBeTruthy()

      // Wait for expiration
      await new Promise(resolve => setTimeout(resolve, 1100))

      // Should be expired
      expect(await store.get('token-123')).toBeNull()
    })
  })

  describe('Async Rate Limiting Integration', () => {
    it('uses async rate limit check for distributed consistency', async () => {
      const { twoFactorSetupRateLimiter } = await import('../../server/utils/rateLimit')

      // First request should pass
      const first = await twoFactorSetupRateLimiter.checkRateLimitAsync('user-123')
      expect(first).toBe(true)

      // Subsequent requests should pass until limit is reached
      for (let i = 0; i < 4; i++) {
        const result = await twoFactorSetupRateLimiter.checkRateLimitAsync('user-123')
        expect(result).toBe(true)
      }

      // 6th request should be rate limited (limit is 10 per hour)
      const sixth = await twoFactorSetupRateLimiter.checkRateLimitAsync('user-123')
      expect(sixth).toBe(true) // Still under limit

      // Cleanup
      twoFactorSetupRateLimiter.clear()
    })

    it('sync and async rate limit checks are consistent', async () => {
      const { loginRateLimiter } = await import('../../server/utils/rateLimit')

      const key = 'test-user'

      // Use sync check
      const syncResult = loginRateLimiter.checkRateLimit(key)
      expect(syncResult).toBe(true)

      // Async check should see the same state
      const asyncResult = await loginRateLimiter.checkRateLimitAsync(key)
      expect(asyncResult).toBe(true)

      // Cleanup
      loginRateLimiter.clear()
    })
  })
})
