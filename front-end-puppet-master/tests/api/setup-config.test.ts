/**
 * Setup Config API Tests
 *
 * Tests for POST /api/setup/config endpoint
 *
 * Security tests:
 * - Malicious payload rejection
 * - Production token enforcement
 * - Input validation and sanitization
 * - Allowlist enforcement
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'

// Mock dependencies
vi.mock('fs', () => ({
  existsSync: vi.fn(() => true),
  readFileSync: vi.fn(() => `
const config = {
  pmMode: 'unconfigured' as 'unconfigured' | 'build' | 'develop',
  aiWorkflow: 'claude' as 'claude' | 'qwen' | 'codex'
}
export default config
`),
  writeFileSync: vi.fn(),
  readdirSync: vi.fn(() => []),
  unlinkSync: vi.fn()
}))

vi.mock('path', () => ({
  resolve: vi.fn((...args) => args.join('/')),
  dirname: vi.fn((p) => p)
}))

vi.mock('../../server/utils/setup-guard', () => ({
  requireSetupAccess: vi.fn()
}))

vi.mock('../../server/utils/workflow-paths', () => ({
  getWorkflowDataDir: vi.fn(() => '.qwen-data/claude'),
  getCurrentWorkflow: vi.fn(() => 'claude'),
  ensureWorkflowDataDir: vi.fn(() => true)
}))

vi.mock('../../server/database/client', () => ({
  useDatabase: vi.fn(() => ({
    exec: vi.fn()
  }))
}))

describe('Setup Config API Security', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Default: no production secret (development mode)
    delete process.env.PM_SETUP_SECRET
  })

  describe('Production Authorization', () => {
    it('allows access when PM_SETUP_SECRET is not set (development)', async () => {
      const { requireSetupAccess } = await import('../../server/utils/setup-guard')
      const mockEvent = { method: 'POST' } as any

      // Should not throw when no secret is configured
      requireSetupAccess(mockEvent, { requireProductionAuth: true })

      expect(requireSetupAccess).toHaveBeenCalledWith(mockEvent, { requireProductionAuth: true })
    })

    it('rejects access when PM_SETUP_SECRET is set but header is missing', async () => {
      process.env.PM_SETUP_SECRET = 'test-secret-123'
      const { requireSetupAccess } = await import('../../server/utils/setup-guard')

      const mockEvent = {
        method: 'POST',
        node: {
          req: {
            headers: {}
          }
        }
      } as any

      expect(() => {
        requireSetupAccess(mockEvent, { requireProductionAuth: true })
      }).toThrow()
    })

    it('rejects access with invalid authorization header', async () => {
      process.env.PM_SETUP_SECRET = 'test-secret-123'
      const { requireSetupAccess } = await import('../../server/utils/setup-guard')

      const mockEvent = {
        method: 'POST',
        node: {
          req: {
            headers: {
              'x-pm-setup-authorization': 'wrong-secret'
            }
          }
        }
      } as any

      expect(() => {
        requireSetupAccess(mockEvent, { requireProductionAuth: true })
      }).toThrow()
    })

    it('allows access with valid authorization header', async () => {
      process.env.PM_SETUP_SECRET = 'test-secret-123'
      const { requireSetupAccess } = await import('../../server/utils/setup-guard')

      const mockEvent = {
        method: 'POST',
        node: {
          req: {
            headers: {
              'x-pm-setup-authorization': 'test-secret-123'
            }
          }
        }
      } as any

      // Should not throw with valid secret
      expect(() => {
        requireSetupAccess(mockEvent, { requireProductionAuth: true })
      }).not.toThrow()
    })
  })

  describe('Malicious Payload Rejection', () => {
    it('rejects locale codes not in allowlist', async () => {
      const { default: setupHandler } = await import('../../server/api/setup/config.post.ts')

      const maliciousBody = {
        pmMode: 'build',
        locales: [{
          code: 'xx', // Invalid locale code
          iso: 'xx-XX',
          name: 'Invalid'
        }]
      }

      const mockEvent = {
        method: 'POST',
        node: { req: { headers: {} } }
      } as any

      // Mock readBody to return malicious payload
      vi.mocked(await import('h3')).readBody = vi.fn().mockResolvedValue(maliciousBody)

      await expect(setupHandler(mockEvent)).rejects.toThrow()
    })

    it('rejects ISO codes not in allowlist', async () => {
      const { default: setupHandler } = await import('../../server/api/setup/config.post.ts')

      const maliciousBody = {
        pmMode: 'build',
        locales: [{
          code: 'en',
          iso: 'xx-INVALID', // Invalid ISO code
          name: 'English'
        }]
      }

      const mockEvent = {
        method: 'POST',
        node: { req: { headers: {} } }
      } as any

      vi.mocked(await import('h3')).readBody = vi.fn().mockResolvedValue(maliciousBody)

      await expect(setupHandler(mockEvent)).rejects.toThrow()
    })

    it('rejects locale names with injection attempts', async () => {
      const { default: setupHandler } = await import('../../server/api/setup/config.post.ts')

      const maliciousBody = {
        pmMode: 'build',
        locales: [{
          code: 'en',
          iso: 'en-US',
          name: "English'; DROP TABLE users; --" // SQL injection attempt
        }]
      }

      const mockEvent = {
        method: 'POST',
        node: { req: { headers: {} } }
      } as any

      vi.mocked(await import('h3')).readBody = vi.fn().mockResolvedValue(maliciousBody)

      await expect(setupHandler(mockEvent)).rejects.toThrow()
    })

    it('rejects module IDs not in ALL_MODULES', async () => {
      const { default: setupHandler } = await import('../../server/api/setup/config.post.ts')

      const maliciousBody = {
        pmMode: 'build',
        modules: ['malicious-module', 'another-bad-module']
      }

      const mockEvent = {
        method: 'POST',
        node: { req: { headers: {} } }
      } as any

      vi.mocked(await import('h3')).readBody = vi.fn().mockResolvedValue(maliciousBody)

      await expect(setupHandler(mockEvent)).rejects.toThrow()
    })

    it('sanitizes strings with control characters', async () => {
      const { sanitizeString } = await import('../../server/api/setup/config.post.ts')

      const maliciousInput = 'test\x00\x1f\x7fvalue'
      const sanitized = sanitizeString(maliciousInput)

      expect(sanitized).not.toContain('\x00')
      expect(sanitized).not.toContain('\x1f')
      expect(sanitized).not.toContain('\x7f')
      expect(sanitized).toBe('testvalue')
    })

    it('rejects overly long strings', async () => {
      const { sanitizeString } = await import('../../server/api/setup/config.post.ts')

      const longInput = 'a'.repeat(1000)
      const sanitized = sanitizeString(longInput, 500)

      expect(sanitized.length).toBeLessThanOrEqual(500)
    })
  })

  describe('Structured Config Serialization', () => {
    it('generates valid config content without regex mutation', async () => {
      const { generateConfigContent } = await import('../../server/api/setup/config.post.ts')

      const config = {
        pmMode: 'build' as const,
        aiWorkflow: 'qwen' as const,
        projectType: 'website' as const,
        adminEnabled: true,
        locales: [{ code: 'en', iso: 'en-US', name: 'English' }],
        defaultLocale: 'en',
        modules: [],
        features: {
          multiLangs: true,
          doubleTheme: true,
          onepager: false
        }
      }

      const content = generateConfigContent(config)

      // Should contain the config values
      expect(content).toContain("pmMode: 'build'")
      expect(content).toContain("aiWorkflow: 'qwen'")
      expect(content).toContain('website: true')
      expect(content).toContain('multiLangs: true')

      // Should be valid TypeScript
      expect(content).toContain('export default config')
      expect(content).toContain('const config = {')
    })

    it('filters invalid locales from config', async () => {
      const { generateConfigContent } = await import('../../server/api/setup/config.post.ts')

      const config = {
        pmMode: 'build' as const,
        locales: [
          { code: 'en', iso: 'en-US', name: 'English' },
          { code: 'xx', iso: 'xx-XX', name: 'Invalid' } // Should be filtered
        ]
      }

      const content = generateConfigContent(config)

      // Should only contain valid locale
      expect(content).toContain('en-US')
      expect(content).not.toContain('xx-XX')
    })

    it('ensures at least one valid locale', async () => {
      const { generateConfigContent } = await import('../../server/api/setup/config.post.ts')

      const config = {
        pmMode: 'build' as const,
        locales: [] // Empty - should default to English
      }

      const content = generateConfigContent(config)

      expect(content).toContain('en-US')
    })
  })

  describe('Input Validation', () => {
    it('rejects invalid pmMode values', async () => {
      const { setupConfigSchema } = await import('../../server/api/setup/config.post.ts')

      const invalidBody = {
        pmMode: 'invalid-mode'
      }

      const result = setupConfigSchema.safeParse(invalidBody)

      expect(result.success).toBe(false)
    })

    it('rejects invalid aiWorkflow values', async () => {
      const { setupConfigSchema } = await import('../../server/api/setup/config.post.ts')

      const invalidBody = {
        pmMode: 'build',
        aiWorkflow: 'invalid-workflow'
      }

      const result = setupConfigSchema.safeParse(invalidBody)

      expect(result.success).toBe(false)
    })

    it('accepts valid configuration', async () => {
      const { setupConfigSchema } = await import('../../server/api/setup/config.post.ts')

      const validBody = {
        pmMode: 'build' as const,
        aiWorkflow: 'claude' as const,
        projectType: 'website' as const,
        adminEnabled: true,
        locales: [{ code: 'en', iso: 'en-US', name: 'English' }],
        defaultLocale: 'en',
        features: {
          multiLangs: true,
          doubleTheme: true
        }
      }

      const result = setupConfigSchema.safeParse(validBody)

      expect(result.success).toBe(true)
    })
  })
})
