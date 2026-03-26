/**
 * Admin RBAC helpers.
 *
 * Centralizes section-level authorization for admin and privileged non-admin API paths.
 */
import { ADMIN_PAGE_IDS, type AdminPageId, type UserRole } from '../database/schema'
import { hasAnyPageAccess, hasPageAccess } from './permissions'

export interface RbacUser {
  id: number
  role: UserRole
  roleId?: number | null
}

const ADMIN_PAGE_ID_SET = new Set<string>(ADMIN_PAGE_IDS as readonly string[])

// Route aliases used by shared streams that belong to a broader admin section contract.
const ADMIN_SECTION_ALIASES: Record<string, string> = {
  stats: 'health',
  logs: 'health',
  'audit-logs': 'health',
  gradebook: 'progress'
}

const CONTENT_ADMIN_SECTIONS: readonly AdminPageId[] = [
  'sections',
  'blog',
  'portfolios',
  'team',
  'testimonials',
  'faq',
  'clients',
  'pricing',
  'features',
  'contacts'
]

export function resolveAdminSectionFromPath(path: string): AdminPageId | null {
  if (!path.startsWith('/api/admin/')) {
    return null
  }

  const sectionCandidate = path.slice('/api/admin/'.length).split('/')[0]
  if (!sectionCandidate) {
    return null
  }

  const mappedSection = ADMIN_SECTION_ALIASES[sectionCandidate] ?? sectionCandidate
  if (!ADMIN_PAGE_ID_SET.has(mappedSection)) {
    return null
  }

  return mappedSection as AdminPageId
}

export async function hasAdminSectionAccess(
  user: RbacUser | null | undefined,
  section: AdminPageId
): Promise<boolean> {
  if (!user) {
    return false
  }

  return hasPageAccess(user, section)
}

export async function requireAdminSectionAccess(
  user: RbacUser | null | undefined,
  section: AdminPageId,
  message = `Access denied: ${section}`
): Promise<void> {
  if (!user) {
    throw createError({
      statusCode: 401,
      message: 'Authentication required'
    })
  }

  if (!(await hasAdminSectionAccess(user, section))) {
    throw createError({
      statusCode: 403,
      message
    })
  }
}

export async function requireAnyContentAdminSectionAccess(
  user: RbacUser | null | undefined,
  message = 'Content admin access required'
): Promise<void> {
  if (!user) {
    throw createError({
      statusCode: 401,
      message: 'Authentication required'
    })
  }

  if (!(await hasAnyPageAccess(user, [...CONTENT_ADMIN_SECTIONS]))) {
    throw createError({
      statusCode: 403,
      message
    })
  }
}

export async function requireAdminRouteAccessByPath(
  user: RbacUser | null | undefined,
  path: string
): Promise<AdminPageId> {
  if (!user) {
    throw createError({
      statusCode: 401,
      message: 'Authentication required'
    })
  }

  const section = resolveAdminSectionFromPath(path)
  if (!section) {
    throw createError({
      statusCode: 403,
      message: 'Access denied: unknown admin route section'
    })
  }

  await requireAdminSectionAccess(user, section)
  return section
}
