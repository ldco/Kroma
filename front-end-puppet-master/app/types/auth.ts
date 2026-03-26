/**
 * Authentication Types
 *
 * User roles hierarchy (each inherits from roles below):
 * - master: Developer/agency who builds the site (full access, can assign roles)
 * - admin: Client who owns the site (can manage content + users except master)
 * - editor: Client's employees (can only edit content)
 * - user: End users of the app (app features only, no admin access)
 */

export type UserRole = 'master' | 'admin' | 'editor' | 'user'

export interface User {
  id: number
  email: string
  name: string | null
  role: UserRole
  createdAt?: Date
}

export interface LoginCredentials {
  email: string
  password: string
  rememberMe?: boolean
}

/**
 * Admin page IDs - each admin page is a permission
 * Simple model: if page is true, user sees it in nav
 */
export const ADMIN_PAGE_IDS = [
  // System pages
  'users',
  'roles',
  'translations',
  'settings',
  'health',
  // Content pages
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
] as const

export type AdminPageId = (typeof ADMIN_PAGE_IDS)[number]

/**
 * Role permissions - map of page ID to access boolean
 * Simple: true = page visible in nav, false/missing = hidden
 */
export type RolePermissions = Partial<Record<AdminPageId, boolean>>

/**
 * Role definition for dynamic roles
 */
export interface Role {
  id: number
  name: string
  slug: string
  description: string | null
  permissions: RolePermissions
  level: number
  isBuiltIn: boolean
  color: string
  userCount?: number
  createdAt?: Date
  updatedAt?: Date
}
