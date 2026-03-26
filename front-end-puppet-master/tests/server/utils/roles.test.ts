import { describe, it, expect } from 'vitest'
import {
  ROLE_HIERARCHY,
  hasRole,
  isMaster,
  isAdmin,
  canManageUsers,
  canManageUser,
  getAssignableRoles
} from '../../../server/utils/roles'

describe('ROLE_HIERARCHY', () => {
  it('should have correct hierarchy values', () => {
    expect(ROLE_HIERARCHY.editor).toBe(0)
    expect(ROLE_HIERARCHY.admin).toBe(1)
    expect(ROLE_HIERARCHY.master).toBe(2)
  })

  it('should have master > admin > editor', () => {
    expect(ROLE_HIERARCHY.master).toBeGreaterThan(ROLE_HIERARCHY.admin)
    expect(ROLE_HIERARCHY.admin).toBeGreaterThan(ROLE_HIERARCHY.editor)
  })
})

describe('hasRole', () => {
  it('should return false for undefined role', () => {
    expect(hasRole(undefined, 'editor')).toBe(false)
    expect(hasRole(undefined, 'admin')).toBe(false)
    expect(hasRole(undefined, 'master')).toBe(false)
  })

  it('should return true when user has exact role', () => {
    expect(hasRole('editor', 'editor')).toBe(true)
    expect(hasRole('admin', 'admin')).toBe(true)
    expect(hasRole('master', 'master')).toBe(true)
  })

  it('should return true when user has higher role', () => {
    expect(hasRole('admin', 'editor')).toBe(true)
    expect(hasRole('master', 'editor')).toBe(true)
    expect(hasRole('master', 'admin')).toBe(true)
  })

  it('should return false when user has lower role', () => {
    expect(hasRole('editor', 'admin')).toBe(false)
    expect(hasRole('editor', 'master')).toBe(false)
    expect(hasRole('admin', 'master')).toBe(false)
  })
})

describe('isMaster', () => {
  it('should return true only for master role', () => {
    expect(isMaster('master')).toBe(true)
    expect(isMaster('admin')).toBe(false)
    expect(isMaster('editor')).toBe(false)
    expect(isMaster(undefined)).toBe(false)
  })
})

describe('isAdmin', () => {
  it('should return true for admin and master', () => {
    expect(isAdmin('master')).toBe(true)
    expect(isAdmin('admin')).toBe(true)
    expect(isAdmin('editor')).toBe(false)
    expect(isAdmin(undefined)).toBe(false)
  })
})

describe('canManageUsers', () => {
  it('should return true for admin and master only', () => {
    expect(canManageUsers('master')).toBe(true)
    expect(canManageUsers('admin')).toBe(true)
    expect(canManageUsers('editor')).toBe(false)
    expect(canManageUsers(undefined)).toBe(false)
  })
})

describe('canManageUser', () => {
  it('should return false for undefined actor', () => {
    expect(canManageUser(undefined, 'editor')).toBe(false)
    expect(canManageUser(undefined, 'admin')).toBe(false)
    expect(canManageUser(undefined, 'master')).toBe(false)
  })

  it('should allow master to manage anyone', () => {
    expect(canManageUser('master', 'editor')).toBe(true)
    expect(canManageUser('master', 'admin')).toBe(true)
    expect(canManageUser('master', 'master')).toBe(true)
  })

  it('should allow admin to manage admin and editor only', () => {
    expect(canManageUser('admin', 'editor')).toBe(true)
    expect(canManageUser('admin', 'admin')).toBe(true)
    expect(canManageUser('admin', 'master')).toBe(false)
  })

  it('should not allow editor to manage anyone', () => {
    expect(canManageUser('editor', 'editor')).toBe(false)
    expect(canManageUser('editor', 'admin')).toBe(false)
    expect(canManageUser('editor', 'master')).toBe(false)
  })
})

describe('getAssignableRoles', () => {
  it('should return empty array for undefined', () => {
    expect(getAssignableRoles(undefined)).toEqual([])
  })

  it('should return all roles for master', () => {
    const roles = getAssignableRoles('master')
    expect(roles).toContain('master')
    expect(roles).toContain('admin')
    expect(roles).toContain('editor')
    expect(roles).toHaveLength(3)
  })

  it('should return admin and editor for admin', () => {
    const roles = getAssignableRoles('admin')
    expect(roles).toContain('admin')
    expect(roles).toContain('editor')
    expect(roles).not.toContain('master')
    expect(roles).toHaveLength(2)
  })

  it('should return empty array for editor', () => {
    expect(getAssignableRoles('editor')).toEqual([])
  })
})
