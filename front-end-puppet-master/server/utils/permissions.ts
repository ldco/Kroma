/**
 * Page-Based Access Control Utilities
 *
 * Simple model: each admin page is a permission.
 * If role has permission for a page, user sees it in nav.
 *
 * Works with dynamic roles stored in the `roles` table.
 * Falls back to legacy role column for backward compatibility.
 *
 * Features:
 * - Process-level cache with TTL (60 seconds)
 * - Redis-backed cache when available (multi-node safe)
 * - Automatic invalidation on role updates
 */
import { eq } from 'drizzle-orm'
import {
  type AdminPageId,
  type RolePermissions,
  type UserRole,
  ADMIN_PAGE_IDS,
  roles
} from '../database/schema'
import { useDatabase } from '../database/client'
import { safeJsonParse } from './json'
import { logger } from './logger'

/**
 * Legacy role permissions (used when roleId is not set)
 * Each key is an admin page ID
 */
const LEGACY_ROLE_PERMISSIONS: Record<UserRole, RolePermissions> = {
  master: {
    // System pages
    users: true,
    roles: true,
    translations: true,
    settings: true,
    health: true,
    // Content pages
    sections: true,
    blog: true,
    portfolios: true,
    team: true,
    testimonials: true,
    faq: true,
    clients: true,
    pricing: true,
    features: true,
    contacts: true
  },
  admin: {
    // System pages (limited - users set to false for master-only)
    users: false,
    roles: false,
    translations: true,
    settings: true,
    health: false,
    // Content pages (all)
    sections: true,
    blog: true,
    portfolios: true,
    team: true,
    testimonials: true,
    faq: true,
    clients: true,
    pricing: true,
    features: true,
    contacts: true
  },
  editor: {
    // System pages (none)
    users: false,
    roles: false,
    translations: true,
    settings: false,
    health: false,
    // Content pages (editorial)
    sections: true,
    blog: true,
    portfolios: false,
    team: false,
    testimonials: true,
    faq: true,
    clients: false,
    pricing: false,
    features: true,
    contacts: false
  }
}

/**
 * User object with role information
 */
interface UserWithRole {
  id: number
  role: UserRole
  roleId?: number | null
}

/**
 * Cached role permissions with TTL to avoid repeated DB queries
 * GAP-011: Added TTL (60 seconds) and Redis support for multi-node deployments
 */
interface CachedRolePermissions {
  rawPermissions: Record<string, unknown>
  level: number
  expiresAt: number
}

const rolePermissionsCache = new Map<number, CachedRolePermissions>()
const ROLE_PERMISSIONS_TTL_MS = 60 * 1000 // 60 seconds

// Redis client for distributed cache (lazy-loaded)
let redisClient: {
  get: (key: string) => Promise<string | null>
  setex: (key: string, seconds: number, value: string) => Promise<void>
  del: (pattern: string) => Promise<void>
} | null = null

let redisInitialized = false

/**
 * Initialize Redis client for distributed cache
 */
async function initRedis(): Promise<boolean> {
  if (redisInitialized) return redisClient !== null

  redisInitialized = true

  try {
    const redisUrl = process.env.REDIS_URL

    if (!redisUrl) {
      logger.debug('REDIS_URL not set - using in-memory permissions cache with TTL')
      return false
    }

    const Redis = await import('ioredis').then(m => m.default).catch(() => null)

    if (!Redis) {
      logger.warn('ioredis not installed - using in-memory permissions cache. Run: npm install ioredis')
      return false
    }

    const client = new Redis(redisUrl)
    await client.ping()

    redisClient = {
      get: async (key: string) => client.get(key),
      setex: async (key: string, seconds: number, value: string) => {
        await client.setex(key, seconds, value)
      },
      del: async (pattern: string) => {
        const keys = await client.keys(pattern)
        if (keys.length > 0) {
          await client.del(...keys)
        }
      }
    }

    logger.info('Permissions cache using Redis for distributed invalidation')
    return true
  } catch (error) {
    logger.warn({ error }, 'Redis connection failed - using in-memory permissions cache')
    return false
  }
}

const LEGACY_ROLE_PERMISSION_KEYS = [
  'manageUsers',
  'manageRoles',
  'manageSettings',
  'viewHealth',
  'manageContent',
  'viewContent'
] as const

const CONTENT_PAGE_IDS: readonly AdminPageId[] = [
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

interface LegacyCapabilityPermissions {
  manageUsers?: boolean
  manageRoles?: boolean
  manageSettings?: boolean
  viewHealth?: boolean
  manageContent?: boolean
  viewContent?: boolean
}

function normalizeRolePermissions(
  rawPermissions: Record<string, unknown>,
  fallbackRole: UserRole
): RolePermissions {
  const normalized = getEmptyPermissions()

  let hasPageBasedPermission = false
  for (const pageId of ADMIN_PAGE_IDS) {
    if (typeof rawPermissions[pageId] === 'boolean') {
      normalized[pageId] = rawPermissions[pageId] as boolean
      hasPageBasedPermission = true
    }
  }

  if (hasPageBasedPermission) {
    return normalized
  }

  const hasLegacyCapabilityPermissions = LEGACY_ROLE_PERMISSION_KEYS.some(
    key => typeof rawPermissions[key] === 'boolean'
  )

  if (hasLegacyCapabilityPermissions) {
    const capabilities = rawPermissions as LegacyCapabilityPermissions
    const hasContentAccess = capabilities.manageContent === true || capabilities.viewContent === true

    normalized.users = capabilities.manageUsers === true
    normalized.roles = capabilities.manageRoles === true
    normalized.settings = capabilities.manageSettings === true
    normalized.health = capabilities.viewHealth === true
    normalized.translations = LEGACY_ROLE_PERMISSIONS[fallbackRole]?.translations === true

    for (const contentPageId of CONTENT_PAGE_IDS) {
      normalized[contentPageId] = hasContentAccess
    }

    return normalized
  }

  return LEGACY_ROLE_PERMISSIONS[fallbackRole] || LEGACY_ROLE_PERMISSIONS.editor
}

/**
 * Clear the role permissions cache (call when roles are updated)
 * GAP-011: Also invalidates Redis cache for multi-node consistency
 */
export async function clearRolePermissionsCache(): Promise<void> {
  rolePermissionsCache.clear()

  // Invalidate Redis cache as well
  if (redisClient) {
    try {
      await redisClient.del('role:*:permissions')
      logger.debug('Redis permissions cache invalidated')
    } catch (error) {
      logger.warn({ error }, 'Failed to invalidate Redis permissions cache')
    }
  }
}

/**
 * Get role data from database by ID
 * Uses TTL cache and Redis (if available) for multi-node consistency
 */
async function getRoleById(
  roleId: number
): Promise<{ rawPermissions: Record<string, unknown>; level: number } | null> {
  const now = Date.now()

  // Check in-memory cache first (with TTL validation)
  const cached = rolePermissionsCache.get(roleId)
  if (cached && cached.expiresAt > now) {
    return { rawPermissions: cached.rawPermissions, level: cached.level }
  }

  // Try Redis cache if available
  if (redisClient) {
    try {
      const redisKey = `role:${roleId}:permissions`
      const serialized = await redisClient.get(redisKey)

      if (serialized) {
        const parsed = JSON.parse(serialized) as { rawPermissions: Record<string, unknown>; level: number }

        // Update in-memory cache with TTL
        rolePermissionsCache.set(roleId, {
          ...parsed,
          expiresAt: now + ROLE_PERMISSIONS_TTL_MS
        })

        return parsed
      }
    } catch (error) {
      logger.warn({ error, roleId }, 'Redis cache get failed, falling back to database')
    }
  }

  // Initialize Redis if not done yet
  if (!redisInitialized) {
    await initRedis()
  }

  // Fetch from database
  const db = useDatabase()
  const role = await db.select().from(roles).where(eq(roles.id, roleId)).get()

  if (!role) return null

  const roleData = {
    rawPermissions: safeJsonParse<Record<string, unknown>>(role.permissions, {}),
    level: role.level
  }

  // Update in-memory cache with TTL
  rolePermissionsCache.set(roleId, {
    ...roleData,
    expiresAt: now + ROLE_PERMISSIONS_TTL_MS
  })

  // Update Redis cache if available
  if (redisClient) {
    try {
      const redisKey = `role:${roleId}:permissions`
      await redisClient.setex(redisKey, 60, JSON.stringify(roleData))
    } catch (error) {
      logger.warn({ error, roleId }, 'Redis cache set failed')
    }
  }

  return roleData
}

/**
 * Get default empty permissions (no access to any page)
 */
function getEmptyPermissions(): RolePermissions {
  const perms: RolePermissions = {}
  for (const pageId of ADMIN_PAGE_IDS) {
    perms[pageId] = false
  }
  return perms
}

/**
 * Get permissions for a user
 * Falls back to legacy role column if roleId is not set
 */
export async function getUserPermissions(user: UserWithRole | null): Promise<RolePermissions> {
  if (!user) {
    return getEmptyPermissions()
  }

  // Try to get permissions from roleId first
  if (user.roleId) {
    const roleData = await getRoleById(user.roleId)
    if (roleData) {
      return normalizeRolePermissions(roleData.rawPermissions, user.role)
    }
  }

  // Fall back to legacy role column
  return LEGACY_ROLE_PERMISSIONS[user.role] || LEGACY_ROLE_PERMISSIONS.editor
}

/**
 * Get user's role level (higher = more authority)
 */
export async function getUserRoleLevel(user: UserWithRole | null): Promise<number> {
  if (!user) return 0

  // Try to get level from roleId first
  if (user.roleId) {
    const roleData = await getRoleById(user.roleId)
    if (roleData) {
      return roleData.level
    }
  }

  // Fall back to legacy role levels
  const legacyLevels: Record<UserRole, number> = {
    master: 100,
    admin: 50,
    editor: 25
  }

  return legacyLevels[user.role] || 0
}

/**
 * Check if user has access to a specific admin page
 */
export async function hasPageAccess(
  user: UserWithRole | null,
  pageId: AdminPageId
): Promise<boolean> {
  const permissions = await getUserPermissions(user)
  return permissions[pageId] === true
}

/**
 * Check if user has access to all specified pages
 */
export async function hasAllPageAccess(
  user: UserWithRole | null,
  pageIds: AdminPageId[]
): Promise<boolean> {
  const permissions = await getUserPermissions(user)
  return pageIds.every(p => permissions[p] === true)
}

/**
 * Check if user has access to any of the specified pages
 */
export async function hasAnyPageAccess(
  user: UserWithRole | null,
  pageIds: AdminPageId[]
): Promise<boolean> {
  const permissions = await getUserPermissions(user)
  return pageIds.some(p => permissions[p] === true)
}

/**
 * Require access to a specific page - throws 403 if not met
 */
export async function requirePageAccess(
  user: UserWithRole | null,
  pageId: AdminPageId,
  message?: string
): Promise<void> {
  const hasAccess = await hasPageAccess(user, pageId)
  if (!hasAccess) {
    throw createError({
      statusCode: 403,
      message: message || `Access denied: ${pageId}`
    })
  }
}

// Legacy aliases for backward compatibility during transition
export const hasPermission = hasPageAccess
export const requirePermission = requirePageAccess

/**
 * Check if user can manage another user based on role levels
 * A user can only manage users with lower role levels
 */
export async function canManageUserByLevel(
  actor: UserWithRole | null,
  target: UserWithRole | null
): Promise<boolean> {
  if (!actor || !target) return false

  const actorLevel = await getUserRoleLevel(actor)
  const targetLevel = await getUserRoleLevel(target)

  // Can only manage users with strictly lower levels
  return actorLevel > targetLevel
}

/**
 * Get all admin page IDs
 */
export function getAllPageIds(): readonly AdminPageId[] {
  return ADMIN_PAGE_IDS
}

/**
 * Calculate role level based on permissions
 * Higher permissions = higher level (more authority)
 *
 * - roles access → level 90 (can manage roles, nearly master)
 * - users access → level 50 (can manage users, admin-like)
 * - neither → level 25 (editor/content level)
 */
export function calculateLevel(permissions: RolePermissions): number {
  if (permissions.roles === true) return 90
  if (permissions.users === true) return 50
  return 25
}

/**
 * Validate that a permissions object is valid
 * (all keys must be valid page IDs with boolean values)
 */
export function isValidPermissionsObject(obj: unknown): obj is RolePermissions {
  if (typeof obj !== 'object' || obj === null) return false

  const record = obj as Record<string, unknown>
  for (const key of Object.keys(record)) {
    if (!ADMIN_PAGE_IDS.includes(key as AdminPageId)) return false
    if (typeof record[key] !== 'boolean') return false
  }
  return true
}
