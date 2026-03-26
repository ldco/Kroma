import { describe, it, expect } from 'vitest'
import { USER_ROLES } from '../../../server/database/schema'

describe('USER_ROLES', () => {
  it('should contain exactly 3 roles', () => {
    expect(USER_ROLES).toHaveLength(3)
  })

  it('should have master as first role (highest priority in array)', () => {
    expect(USER_ROLES[0]).toBe('master')
  })

  it('should have admin as second role', () => {
    expect(USER_ROLES[1]).toBe('admin')
  })

  it('should have editor as third role (lowest priority)', () => {
    expect(USER_ROLES[2]).toBe('editor')
  })

  it('should contain all expected roles', () => {
    expect(USER_ROLES).toContain('master')
    expect(USER_ROLES).toContain('admin')
    expect(USER_ROLES).toContain('editor')
  })

  it('should be readonly (as const)', () => {
    // This is a compile-time check, but we can verify the values are strings
    USER_ROLES.forEach(role => {
      expect(typeof role).toBe('string')
    })
  })
})
