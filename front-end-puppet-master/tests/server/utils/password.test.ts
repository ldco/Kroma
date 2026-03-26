import { describe, it, expect } from 'vitest'
import { hashPassword, verifyPassword, generateSessionId } from '../../../server/utils/password'

describe('hashPassword', () => {
  it('should return a string with salt:hash format', () => {
    const hash = hashPassword('testpassword')
    expect(hash).toContain(':')
    const [salt, hashPart] = hash.split(':')
    expect(salt).toBeDefined()
    expect(hashPart).toBeDefined()
  })

  it('should return 32-char hex salt (16 bytes)', () => {
    const hash = hashPassword('testpassword')
    const [salt] = hash.split(':')
    expect(salt).toHaveLength(32)
    expect(/^[a-f0-9]+$/i.test(salt)).toBe(true)
  })

  it('should return 128-char hex hash (64 bytes)', () => {
    const hash = hashPassword('testpassword')
    const [, hashPart] = hash.split(':')
    expect(hashPart).toHaveLength(128)
    expect(/^[a-f0-9]+$/i.test(hashPart)).toBe(true)
  })

  it('should generate different hashes for same password (unique salt)', () => {
    const hash1 = hashPassword('testpassword')
    const hash2 = hashPassword('testpassword')
    expect(hash1).not.toBe(hash2)
  })

  it('should generate different hashes for different passwords', () => {
    const hash1 = hashPassword('password1')
    const hash2 = hashPassword('password2')
    expect(hash1).not.toBe(hash2)
  })
})

describe('verifyPassword', () => {
  it('should return true for correct password', () => {
    const password = 'correctPassword123'
    const hash = hashPassword(password)
    expect(verifyPassword(password, hash)).toBe(true)
  })

  it('should return false for incorrect password', () => {
    const hash = hashPassword('correctPassword')
    expect(verifyPassword('wrongPassword', hash)).toBe(false)
  })

  it('should return false for empty password', () => {
    const hash = hashPassword('somePassword')
    expect(verifyPassword('', hash)).toBe(false)
  })

  it('should return false for invalid hash format (no colon)', () => {
    expect(verifyPassword('password', 'invalidhash')).toBe(false)
  })

  it('should return false for empty hash', () => {
    expect(verifyPassword('password', '')).toBe(false)
  })

  it('should return false for hash with only salt', () => {
    expect(verifyPassword('password', 'abc123:')).toBe(false)
  })

  it('should return false for malformed hash', () => {
    expect(verifyPassword('password', ':')).toBe(false)
    expect(verifyPassword('password', 'a:')).toBe(false)
    expect(verifyPassword('password', ':b')).toBe(false)
  })

  it('should handle special characters in password', () => {
    const password = 'p@$$w0rd!#%^&*()_+-=[]{}|;:",.<>?/'
    const hash = hashPassword(password)
    expect(verifyPassword(password, hash)).toBe(true)
  })

  it('should handle unicode characters in password', () => {
    const password = 'пароль密码🔐'
    const hash = hashPassword(password)
    expect(verifyPassword(password, hash)).toBe(true)
  })
})

describe('generateSessionId', () => {
  it('should return a 64-character hex string (32 bytes)', () => {
    const sessionId = generateSessionId()
    expect(sessionId).toHaveLength(64)
    expect(/^[a-f0-9]+$/i.test(sessionId)).toBe(true)
  })

  it('should generate unique session IDs', () => {
    const ids = new Set<string>()
    for (let i = 0; i < 100; i++) {
      ids.add(generateSessionId())
    }
    expect(ids.size).toBe(100)
  })

  it('should not contain special characters', () => {
    const sessionId = generateSessionId()
    expect(/^[a-f0-9]+$/i.test(sessionId)).toBe(true)
  })
})
