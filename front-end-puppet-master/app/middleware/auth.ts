/**
 * Auth Middleware
 *
 * Protects /admin and /app routes by checking authentication BEFORE rendering.
 * This prevents the "flash of unauthenticated content" issue.
 *
 * Usage: Add `definePageMeta({ middleware: 'auth' })` to protected pages
 * Or use in layout for all protected pages
 */
export default defineNuxtRouteMiddleware(async to => {
  // Determine route type
  const isAdminRoute = to.path.startsWith('/admin') && to.path !== '/admin/login'
  const isAppRoute = to.path.startsWith('/app')
  
  // Only protect /admin and /app routes
  if (!isAdminRoute && !isAppRoute) {
    return
  }
  
  // For /admin routes, use Puppet Master auth
  if (isAdminRoute) {
    const { user, checkSession } = useAuth()

    // If we don't have user data, check the session
    if (!user.value) {
      await checkSession()
    }

    // Still no user? Redirect to admin login
    if (!user.value) {
      return navigateTo('/admin/login', { replace: true })
    }
  }
  
  // For /app routes, check Kroma Bearer token
  if (isAppRoute) {
    if (import.meta.client) {
      const kromaAuth = useKromaAuthStore()
      kromaAuth.initFromStorage()
      
      if (!kromaAuth.token) {
        // Attempt to bootstrap the first token (J00 onboarding)
        const token = await kromaAuth.bootstrapToken()
        
        if (!token) {
          // Bootstrap failed — redirect to a non-app error or onboarding page
          // Do NOT redirect to an /app/* route (infinite loop)
          return navigateTo('/', { replace: true })
        }
        // Token bootstrapped successfully — allow navigation to proceed
      }
    }
  }
})
