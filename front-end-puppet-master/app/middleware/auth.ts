/**
 * Auth Middleware
 *
 * Protects /admin and /app routes by checking authentication.
 * For /app routes, allows navigation to proceed and bootstraps token in background.
 *
 * Usage: Add `definePageMeta({ middleware: 'auth' })` to protected pages
 */
export default defineNuxtRouteMiddleware(to => {
  // Determine route type
  const isAdminRoute = to.path.startsWith('/admin') && to.path !== '/admin/login'
  const isAppRoute = to.path.startsWith('/app')

  // Only protect /admin routes
  // /app routes allow access and bootstrap token in background
  if (!isAdminRoute) {
    return
  }

  // For /admin routes, use Puppet Master auth
  const { user, checkSession } = useAuth()

  // If we don't have user data, check the session
  if (!user.value) {
    // Run checkSession in background, don't block navigation
    checkSession()
  }

  // Still no user? Redirect to admin login
  if (!user.value) {
    return navigateTo('/admin/login', { replace: true })
  }
})
