import { describe, it, expect } from 'vitest'
import { generateCsrfToken } from '../../../server/utils/csrf'

describe('generateCsrfToken', () => {
  it('should return a 64-character hex string (32 bytes)', () => {
    const token = generateCsrfToken()
    expect(token).toHaveLength(64)
    expect(/^[a-f0-9]+$/i.test(token)).toBe(true)
  })

  it('should generate unique tokens', () => {
    const tokens = new Set<string>()
    for (let i = 0; i < 100; i++) {
      tokens.add(generateCsrfToken())
    }
    expect(tokens.size).toBe(100)
  })

  it('should not contain special characters', () => {
    const token = generateCsrfToken()
    expect(/^[a-f0-9]+$/i.test(token)).toBe(true)
  })

  it('should be cryptographically random (entropy check)', () => {
    // Generate multiple tokens and check they're sufficiently different
    const tokens: string[] = []
    for (let i = 0; i < 10; i++) {
      tokens.push(generateCsrfToken())
    }

    // Check that no two tokens share more than 25% of characters in same positions
    for (let i = 0; i < tokens.length; i++) {
      for (let j = i + 1; j < tokens.length; j++) {
        let samePositions = 0
        for (let k = 0; k < tokens[i].length; k++) {
          if (tokens[i][k] === tokens[j][k]) samePositions++
        }
        // Should have less than 25% matching positions (16 chars out of 64)
        expect(samePositions).toBeLessThan(16)
      }
    }
  })
})

describe('CSRF token format', () => {
  it('should be URL-safe (no special encoding needed)', () => {
    const token = generateCsrfToken()
    // URL-encode and compare - should be the same (no encoding needed)
    expect(encodeURIComponent(token)).toBe(token)
  })

  it('should be header-safe (no special characters)', () => {
    const token = generateCsrfToken()
    // Headers can contain alphanumeric and some special chars
    // Hex tokens are always safe
    expect(/^[a-zA-Z0-9]+$/.test(token)).toBe(true)
  })
})
