/**
 * Sentry Client Plugin
 *
 * Initializes Sentry for client-side error tracking in Vue/Nuxt.
 * Captures Vue errors, unhandled exceptions, and user interactions.
 *
 * Configuration via environment variables:
 * - NUXT_PUBLIC_SENTRY_DSN: Sentry project DSN
 * - NUXT_PUBLIC_SENTRY_ENVIRONMENT: Environment name
 */
import * as Sentry from '@sentry/vue'

export default defineNuxtPlugin(nuxtApp => {
  const config = useRuntimeConfig()
  const router = useRouter()

  // Only initialize if DSN is provided
  const dsn = config.public.sentry?.dsn || ''
  if (!dsn) {
    console.info('[Sentry] No DSN provided, client error tracking disabled')
    return
  }

  const environment = config.public.sentry?.environment || 'development'
  const release = config.public.sentry?.release || ''

  // Initialize Sentry for Vue
  Sentry.init({
    app: nuxtApp.vueApp,
    dsn,
    environment,
    release,

    // Vue-specific integrations
    integrations: [
      // Browser tracing with Vue Router
      Sentry.browserTracingIntegration({
        router
      }),
      // Replay integration for session recording (optional)
      // Sentry.replayIntegration({
      //   maskAllText: true,
      //   blockAllMedia: true
      // })
    ],

    // Performance monitoring
    tracesSampleRate: environment === 'production' ? 0.1 : 1.0,

    // Session replay (if enabled)
    // replaysSessionSampleRate: 0.1,
    // replaysOnErrorSampleRate: 1.0,

    // Track components
    trackComponents: true,

    // Hooks for Vue lifecycle
    hooks: ['activate', 'create', 'destroy', 'mount', 'update'],

    // Log errors to console in development
    logErrors: environment !== 'production',

    // Filter sensitive data
    beforeSend(event) {
      // Remove sensitive user data
      if (event.user) {
        delete event.user.email
        delete event.user.ip_address
      }

      // Filter out certain URLs from breadcrumbs
      if (event.breadcrumbs) {
        event.breadcrumbs = event.breadcrumbs.filter(breadcrumb => {
          // Filter out requests to analytics, etc.
          if (breadcrumb.category === 'xhr' || breadcrumb.category === 'fetch') {
            const url = breadcrumb.data?.url || ''
            if (url.includes('analytics') || url.includes('tracking')) {
              return false
            }
          }
          return true
        })
      }

      return event
    },

    // Ignore certain errors
    ignoreErrors: [
      // Ignore ResizeObserver errors (common in browsers)
      'ResizeObserver loop limit exceeded',
      'ResizeObserver loop completed with undelivered notifications',
      // Ignore network errors (user's connection issues)
      'Network request failed',
      'Failed to fetch',
      'Load failed',
      // Ignore cancelled navigation
      'Navigation cancelled',
      // Ignore chunk load failures (version mismatch after deploy)
      /Loading chunk .* failed/
    ],

    // Deny URLs (don't track errors from these)
    denyUrls: [
      // Chrome extensions
      /extensions\//i,
      /^chrome:\/\//i,
      // Firefox extensions
      /^moz-extension:\/\//i,
      // Safari extensions
      /^safari-extension:\/\//i
    ]
  })

  // Set up user context when authenticated
  const auth = useAuth()
  watch(
    () => auth.user.value,
    user => {
      if (user) {
        Sentry.setUser({
          id: String(user.id),
          // Don't include email for privacy
          username: user.name || undefined
        })
      } else {
        Sentry.setUser(null)
      }
    },
    { immediate: true }
  )

  // Add custom tags
  Sentry.setTag('nuxt_version', '4')
  Sentry.setTag('vue_version', '3.5')

  console.info(`[Sentry] Client initialized for ${environment}`)
})
