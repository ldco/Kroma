/**
 * Auth Middleware
 *
 * Protects admin routes by checking authentication BEFORE rendering.
 * This prevents the "flash of admin content" issue.
 *
 * Usage: Add `definePageMeta({ middleware: 'auth' })` to protected pages
 * Or use in layout for all admin pages
 */
export default defineNuxtRouteMiddleware(async to => {
  // Only protect /admin routes (except login)
  if (!to.path.startsWith('/admin') || to.path === '/admin/login') {
    return
  }

  const { user, checkSession } = useAuth()

  // If we don't have user data, check the session
  if (!user.value) {
    await checkSession()
  }

  // Still no user? Redirect to login
  if (!user.value) {
    return navigateTo('/admin/login', { replace: true })
  }
})
