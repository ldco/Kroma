/**
 * CSP Nonce Plugin
 *
 * Generates a unique nonce per request and injects it into:
 * 1. All inline script tags in the HTML
 * 2. The event context for the security headers middleware
 *
 * This allows removing 'unsafe-inline' from script-src CSP directive.
 */
import { randomBytes } from 'crypto'

export default defineNitroPlugin((nitroApp) => {
  // Hook into rendering to inject nonces into script tags
  nitroApp.hooks.hook('render:html', (html, { event }) => {
    // Generate a cryptographically secure nonce (16 bytes = 128 bits)
    const nonce = randomBytes(16).toString('base64')

    // Store nonce in event context for the security headers middleware
    event.context.cspNonce = nonce

    // Inject nonce into all inline script tags
    // Matches: <script>, <script type="...">, <script id="...">
    // But NOT: <script src="..."> (external scripts don't need nonces)
    html.head = html.head.map(segment =>
      segment.replace(/<script(?![^>]*\bsrc\b)([^>]*)>/gi, `<script nonce="${nonce}"$1>`)
    )

    html.bodyAppend = html.bodyAppend.map(segment =>
      segment.replace(/<script(?![^>]*\bsrc\b)([^>]*)>/gi, `<script nonce="${nonce}"$1>`)
    )

    html.bodyPrepend = html.bodyPrepend.map(segment =>
      segment.replace(/<script(?![^>]*\bsrc\b)([^>]*)>/gi, `<script nonce="${nonce}"$1>`)
    )
  })

  // Hook into response to set CSP header with nonce
  // This runs after render:html so we have the nonce
  nitroApp.hooks.hook('render:response', (response, { event }) => {
    const nonce = event.context.cspNonce
    if (!nonce) return

    // Build CSP directives with nonce
    const isProduction = process.env.NODE_ENV === 'production'

    const cspDirectives = [
      "default-src 'self'",
      // Scripts: self + nonce (no unsafe-inline!)
      isProduction
        ? `script-src 'self' 'nonce-${nonce}' 'strict-dynamic'`
        : `script-src 'self' 'nonce-${nonce}' 'unsafe-eval'`, // eval needed for Vue devtools
      // Styles: self + unsafe-inline (required for Vue's dynamic styles)
      "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
      "img-src 'self' data: blob: https:",
      "font-src 'self' https://fonts.gstatic.com",
      "connect-src 'self' ws: wss:", // Add WebSocket for HMR in dev
      "media-src 'self' https:",
      "object-src 'none'",
      "frame-ancestors 'none'",
      "base-uri 'self'",
      "form-action 'self'",
      ...(isProduction ? ['upgrade-insecure-requests'] : [])
    ]

    // Set CSP header (overwrites any previous CSP from middleware)
    if (response.headers) {
      response.headers['Content-Security-Policy'] = cspDirectives.join('; ')
    }
  })
})
