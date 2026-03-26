/**
 * Security Headers Middleware
 *
 * Adds security headers to all responses to protect against common web vulnerabilities.
 * Headers follow OWASP recommendations and modern security best practices.
 */

export default defineEventHandler(event => {
  const headers = event.node.res

  // Prevent clickjacking attacks
  // DENY = page cannot be displayed in a frame
  headers.setHeader('X-Frame-Options', 'DENY')

  // Prevent MIME type sniffing
  // Stops browsers from interpreting files as different MIME types
  headers.setHeader('X-Content-Type-Options', 'nosniff')

  // Control referrer information sent with requests
  // strict-origin-when-cross-origin: Send full URL for same-origin, only origin for cross-origin
  headers.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin')

  // HTTP Strict Transport Security (HSTS)
  // Only in production to avoid issues with local development
  if (process.env.NODE_ENV === 'production') {
    // max-age: 1 year, includeSubDomains, preload for HSTS preload list
    headers.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains; preload')
  }

  // NOTE: Content Security Policy (CSP) is handled by server/plugins/csp-nonce.ts
  // which injects nonces into inline scripts for improved security.
  // The CSP header is set during render:response hook with the generated nonce.

  // Permissions Policy (formerly Feature-Policy)
  // Restricts access to browser features
  const permissionsPolicies = [
    'camera=()', // Disable camera
    'microphone=()', // Disable microphone
    'geolocation=()', // Disable geolocation
    'payment=()', // Disable payment API
    'usb=()', // Disable USB
    'magnetometer=()', // Disable magnetometer
    'gyroscope=()', // Disable gyroscope
    'accelerometer=()' // Disable accelerometer
  ]

  headers.setHeader('Permissions-Policy', permissionsPolicies.join(', '))

  // Cross-Origin policies for additional isolation
  // Cross-Origin-Opener-Policy: Isolate browsing context
  headers.setHeader('Cross-Origin-Opener-Policy', 'same-origin')

  // Cross-Origin-Resource-Policy: Only allow same-origin access
  headers.setHeader('Cross-Origin-Resource-Policy', 'same-origin')
})
