/**
 * Auth Composable
 *
 * Client-side authentication state management.
 * Provides login, logout, session checking, and page-based access control.
 * Integrates with CSRF protection via useCsrf composable.
 * Supports two-factor authentication (2FA) flow.
 */
import type { UserRole, User, LoginCredentials, RolePermissions, AdminPageId } from '~/types'
import { ADMIN_PAGE_IDS, ROLE_LEVELS } from '~/types'

// Re-export for backward compatibility
export type { UserRole } from '~/types'

/**
 * Login result with 2FA support
 */
export interface LoginResult {
  success: boolean
  error?: string
  requires2fa?: boolean
}

/**
 * Get default permissions (no access to any page)
 */
function getDefaultPermissions(): RolePermissions {
  const perms: RolePermissions = {}
  for (const pageId of ADMIN_PAGE_IDS) {
    perms[pageId] = false
  }
  return perms
}

export function useAuth() {
  const user = useState<User | null>('auth-user', () => null)
  const permissions = useState<RolePermissions>('auth-permissions', getDefaultPermissions)
  const isLoading = useState<boolean>('auth-loading', () => false)
  // 2FA state
  const requires2fa = useState<boolean>('auth-requires-2fa', () => false)

  const isAuthenticated = computed(() => !!user.value)

  // Role-based computed properties (legacy - for backward compatibility)
  const isMaster = computed(() => user.value?.role === 'master')
  const isAdmin = computed(() => hasRole('admin'))
  const isEditor = computed(() => hasRole('editor'))

  // Page-based access computed properties
  // Convenience shortcuts for commonly checked pages
  const canManageUsers = computed(() => permissions.value.users === true)
  const canManageRoles = computed(() => permissions.value.roles === true)
  const canManageSettings = computed(() => permissions.value.settings === true)
  const canViewHealth = computed(() => permissions.value.health === true)

  /**
   * Check if current user can access a specific admin page
   */
  function canAccessPage(pageId: AdminPageId): boolean {
    return permissions.value[pageId] === true
  }

  /**
   * Get list of page IDs the current user can access
   */
  function getAccessiblePages(): AdminPageId[] {
    return ADMIN_PAGE_IDS.filter(pageId => permissions.value[pageId] === true)
  }

  /**
   * Check if current user has at least the minimum required role
   */
  function hasRole(minRole: UserRole): boolean {
    if (!user.value?.role) return false
    const userLevel = ROLE_LEVELS[user.value.role] ?? 0
    const minLevel = ROLE_LEVELS[minRole] ?? 0
    return userLevel >= minLevel
  }

  /**
   * Get roles that current user can assign to others
   */
  function getAssignableRoles(): UserRole[] {
    if (!user.value?.role) return []
    if (user.value.role === 'master') return ['master', 'admin', 'editor']
    if (user.value.role === 'admin') return ['admin', 'editor']
    return []
  }

  /**
   * Check current session on app load
   * Also restores CSRF token and permissions from server
   */
  async function checkSession(): Promise<User | null> {
    const { setToken, clearToken } = useCsrf()
    isLoading.value = true
    try {
      const response = await apiFetch<{
        user: User | null
        permissions?: RolePermissions
        csrfToken: string | null
      }>('/api/auth/session')

      user.value = response.user ?? null
      permissions.value = response.permissions ?? getDefaultPermissions()

      // Restore CSRF token
      if (response.csrfToken) {
        setToken(response.csrfToken)
      } else {
        clearToken()
      }

      return user.value
    } catch {
      user.value = null
      permissions.value = getDefaultPermissions()
      clearToken()
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Login with email and password
   * Sets CSRF token from response for subsequent requests
   * Returns requires2fa: true if 2FA verification is needed
   */
  async function login(credentials: LoginCredentials): Promise<LoginResult> {
    const { setToken } = useCsrf()
    isLoading.value = true
    requires2fa.value = false

    try {
      const response = await apiFetch<{
        user?: User
        csrfToken?: string
        requires2fa?: boolean
      }>('/api/auth/login', {
        method: 'POST',
        body: credentials
      })

      // Check if 2FA is required
      if (response.requires2fa) {
        requires2fa.value = true
        // Store CSRF token for subsequent 2FA verification request
        if (response.csrfToken) {
          setToken(response.csrfToken)
        }
        return { success: true, requires2fa: true }
      }

      // Normal login success
      if (response.user) {
        user.value = response.user
      }

      // Store CSRF token for subsequent requests
      if (response.csrfToken) {
        setToken(response.csrfToken)
      }

      return { success: true }
    } catch (error: unknown) {
      const err = error as { data?: { message?: string }; message?: string }
      const message = err?.data?.message || err?.message || 'Login failed'
      return { success: false, error: message }
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Verify 2FA code (TOTP or backup code)
   * Called after login returns requires2fa: true
   */
  async function verify2fa(code: string): Promise<LoginResult> {
    const { setToken } = useCsrf()
    isLoading.value = true

    try {
      const response = await apiFetch<{
        user: User
        csrfToken: string
        backupCodesRemaining?: number
      }>('/api/user/2fa/verify', {
        method: 'POST',
        body: { code }
      })

      user.value = response.user
      requires2fa.value = false

      // Store CSRF token for subsequent requests
      if (response.csrfToken) {
        setToken(response.csrfToken)
      }

      return { success: true }
    } catch (error: unknown) {
      const err = error as { data?: { message?: string; attemptsRemaining?: number }; message?: string }
      const message = err?.data?.message || err?.message || '2FA verification failed'
      return { success: false, error: message }
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Cancel pending 2FA verification and return to login
   */
  function cancel2fa(): void {
    requires2fa.value = false
  }

  /**
   * Logout current user
   * Clears CSRF token and permissions
   */
  async function logout(): Promise<void> {
    const { clearToken } = useCsrf()
    isLoading.value = true
    try {
      await apiFetch<null>('/api/auth/logout', { method: 'POST' })
    } finally {
      user.value = null
      permissions.value = getDefaultPermissions()
      clearToken()
      isLoading.value = false
      // Redirect to login
      navigateTo('/admin/login')
    }
  }

  /**
   * Check if current user has access to a specific page
   * (alias for canAccessPage for backward compatibility)
   */
  function hasPermission(pageId: AdminPageId): boolean {
    return permissions.value[pageId] === true
  }

  /**
   * Require authentication - redirect to login if not authenticated
   */
  async function requireAuth(): Promise<boolean> {
    if (user.value) return true

    await checkSession()

    if (!user.value) {
      navigateTo('/admin/login')
      return false
    }

    return true
  }

  return {
    // State
    user: readonly(user),
    permissions: readonly(permissions),
    isLoading: readonly(isLoading),
    isAuthenticated,

    // 2FA state
    requires2fa: readonly(requires2fa),

    // Role-based state (legacy - for backward compatibility)
    isMaster,
    isAdmin,
    isEditor,

    // Page access computed properties (convenience shortcuts)
    canManageUsers,
    canManageRoles,
    canManageSettings,
    canViewHealth,

    // Actions
    checkSession,
    login,
    logout,
    requireAuth,
    hasRole,
    hasPermission,
    canAccessPage,
    getAccessiblePages,
    getAssignableRoles,

    // 2FA actions
    verify2fa,
    cancel2fa
  }
}
