import { describe, expect, it } from 'vitest'
import {
  hasAdminSectionAccess,
  requireAdminRouteAccessByPath,
  requireAnyContentAdminSectionAccess,
  requireAdminSectionAccess,
  resolveAdminSectionFromPath,
  type RbacUser
} from '../../../server/utils/admin-rbac'

const masterUser: RbacUser = { id: 1, role: 'master' }
const adminUser: RbacUser = { id: 2, role: 'admin' }
const editorUser: RbacUser = { id: 3, role: 'editor' }

describe('admin-rbac helper contracts', () => {
  it('resolves admin section from path with aliases', () => {
    expect(resolveAdminSectionFromPath('/api/admin/blog/posts')).toBe('blog')
    expect(resolveAdminSectionFromPath('/api/admin/stats')).toBe('health')
    expect(resolveAdminSectionFromPath('/api/admin/audit-logs')).toBe('health')
    expect(resolveAdminSectionFromPath('/api/admin/roles/1')).toBe('roles')
    expect(resolveAdminSectionFromPath('/api/admin/unknown')).toBeNull()
  })

  it('checks section access via role permissions', async () => {
    await expect(hasAdminSectionAccess(editorUser, 'portfolios')).resolves.toBe(false)
    await expect(hasAdminSectionAccess(editorUser, 'blog')).resolves.toBe(true)
    await expect(hasAdminSectionAccess(adminUser, 'users')).resolves.toBe(false) // users is master-only
    await expect(hasAdminSectionAccess(adminUser, 'settings')).resolves.toBe(true)
    await expect(hasAdminSectionAccess(masterUser, 'users')).resolves.toBe(true)
  })

  it('requires authentication for section access checks', async () => {
    await expect(requireAdminSectionAccess(null, 'blog')).rejects.toMatchObject({
      statusCode: 401
    })
  })

  it('rejects access to unauthorized section routes', async () => {
    await expect(requireAdminRouteAccessByPath(editorUser, '/api/admin/users')).rejects.toMatchObject({
      statusCode: 403
    })
  })

  it('allows access to authorized section routes', async () => {
    await expect(requireAdminRouteAccessByPath(editorUser, '/api/admin/blog/posts')).resolves.toBe('blog')
    await expect(requireAdminRouteAccessByPath(masterUser, '/api/admin/roles')).resolves.toBe('roles')
  })

  it('requires any content-admin section for upload-like capabilities', async () => {
    await expect(requireAnyContentAdminSectionAccess(null)).rejects.toMatchObject({
      statusCode: 401
    })
    await expect(requireAnyContentAdminSectionAccess(editorUser)).resolves.toBeUndefined()
    await expect(requireAnyContentAdminSectionAccess(adminUser)).resolves.toBeUndefined()
  })
})
