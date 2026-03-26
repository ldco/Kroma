/**
 * Sentry Server Plugin
 *
 * Initializes Sentry for server-side error tracking.
 * Captures unhandled errors, rejections, and provides breadcrumbs.
 *
 * Configuration via environment variables:
 * - SENTRY_DSN: Sentry project DSN
 * - SENTRY_ENVIRONMENT: Environment name (production, staging, development)
 * - SENTRY_RELEASE: Release version (defaults to git commit SHA)
 */
import * as Sentry from '@sentry/node'

export default defineNitroPlugin(nitroApp => {
  const config = useRuntimeConfig()

  // Only initialize if DSN is provided
  const dsn = config.sentry?.dsn || process.env.SENTRY_DSN
  if (!dsn) {
    console.info('[Sentry] No DSN provided, error tracking disabled')
    return
  }

  const environment = config.sentry?.environment || process.env.SENTRY_ENVIRONMENT || 'development'
  const release = config.sentry?.release || process.env.SENTRY_RELEASE || process.env.GIT_COMMIT_SHA

  // Initialize Sentry
  Sentry.init({
    dsn,
    environment,
    release,

    // Performance monitoring
    tracesSampleRate: environment === 'production' ? 0.1 : 1.0,

    // Error sampling
    sampleRate: 1.0,

    // Attach stack traces to all messages
    attachStacktrace: true,

    // Breadcrumbs configuration
    maxBreadcrumbs: 50,

    // Integrations
    integrations: [
      // Automatically instrument HTTP requests
      Sentry.httpIntegration(),
      // Capture console messages as breadcrumbs
      Sentry.consoleIntegration(),
      // Capture unhandled promise rejections
      Sentry.onUnhandledRejectionIntegration()
    ],

    // Filter sensitive data
    beforeSend(event) {
      // Remove sensitive headers
      if (event.request?.headers) {
        delete event.request.headers['authorization']
        delete event.request.headers['cookie']
        delete event.request.headers['x-csrf-token']
      }

      // Remove sensitive data from request body
      if (event.request?.data) {
        const sensitiveFields = ['password', 'passwordHash', 'secret', 'token', 'apiKey']
        for (const field of sensitiveFields) {
          if (typeof event.request.data === 'object' && field in event.request.data) {
            event.request.data[field] = '[REDACTED]'
          }
        }
      }

      return event
    },

    // Ignore certain errors
    ignoreErrors: [
      // Ignore client disconnection errors
      'ECONNRESET',
      'EPIPE',
      // Ignore 404s (handled elsewhere)
      /404/,
      // Ignore timeout errors
      'ETIMEDOUT'
    ]
  })

  // Add request context to errors
  nitroApp.hooks.hook('request', async event => {
    Sentry.setContext('request', {
      url: event.path,
      method: event.method,
      headers: {
        'user-agent': getHeader(event, 'user-agent'),
        'content-type': getHeader(event, 'content-type')
      }
    })

    // Set user context if authenticated
    const session = event.context.session
    if (session?.userId) {
      Sentry.setUser({
        id: String(session.userId)
      })
    } else {
      Sentry.setUser(null)
    }
  })

  // Capture errors
  nitroApp.hooks.hook('error', async (error, { event }) => {
    // Skip 4xx errors (client errors)
    const statusCode = (error as { statusCode?: number }).statusCode
    if (statusCode && statusCode >= 400 && statusCode < 500) {
      return
    }

    Sentry.captureException(error, {
      extra: {
        path: event?.path,
        method: event?.method
      }
    })
  })

  console.info(`[Sentry] Initialized for ${environment}`)
})
