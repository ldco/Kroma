/**
 * API Versioning Middleware
 *
 * Handles API version negotiation and backward compatibility.
 *
 * Version detection (in order of precedence):
 * 1. URL path: /api/v1/users
 * 2. Accept header: Accept: application/vnd.pm.v1+json
 * 3. Default: current version (v1)
 *
 * Usage in handlers:
 *   const version = event.context.apiVersion // 1, 2, etc.
 *
 * Deprecation warnings are added to responses for old versions.
 */

// Current and supported API versions
const CURRENT_VERSION = 1
const SUPPORTED_VERSIONS = [1]
const DEPRECATED_VERSIONS: number[] = []

// Version sunset dates (version -> date string)
const VERSION_SUNSET_DATES: Record<number, string> = {
  // v1: '2027-01-01' // Example sunset date
}

/**
 * Parse version from URL path
 * /api/v1/users -> 1
 * /api/v2/users -> 2
 */
function parseVersionFromPath(path: string): number | null {
  const match = path.match(/^\/api\/v(\d+)\//)
  if (match) {
    return parseInt(match[1], 10)
  }
  return null
}

/**
 * Parse version from Accept header
 * Accept: application/vnd.pm.v1+json -> 1
 */
function parseVersionFromAcceptHeader(accept: string | null): number | null {
  if (!accept) return null

  // Match: application/vnd.pm.v1+json
  const match = accept.match(/application\/vnd\.pm\.v(\d+)\+json/)
  if (match) {
    return parseInt(match[1], 10)
  }
  return null
}

/**
 * Check if version is supported
 */
function isVersionSupported(version: number): boolean {
  return SUPPORTED_VERSIONS.includes(version)
}

/**
 * Check if version is deprecated
 */
function isVersionDeprecated(version: number): boolean {
  return DEPRECATED_VERSIONS.includes(version)
}

export default defineEventHandler(event => {
  const url = getRequestURL(event)
  const path = url.pathname

  // Only handle API routes
  if (!path.startsWith('/api')) {
    return
  }

  // Parse version from URL path first
  let version = parseVersionFromPath(path)
  let rewrittenPath: string | null = null

  // If version is in the URL path, rewrite to unversioned path
  // This allows /api/v1/users to be handled by /api/users
  if (version !== null) {
    rewrittenPath = path.replace(/^\/api\/v\d+\//, '/api/')
  }

  // If no version in path, try Accept header
  if (version === null) {
    const accept = getHeader(event, 'accept')
    version = parseVersionFromAcceptHeader(accept)
  }

  // Default to current version
  if (version === null) {
    version = CURRENT_VERSION
  }

  // Check if version is supported
  if (!isVersionSupported(version)) {
    throw createError({
      statusCode: 400,
      message: `API version ${version} is not supported. Supported versions: ${SUPPORTED_VERSIONS.join(', ')}`,
      data: {
        requestedVersion: version,
        supportedVersions: SUPPORTED_VERSIONS,
        currentVersion: CURRENT_VERSION
      }
    })
  }

  // Set version in event context for use in handlers
  event.context.apiVersion = version

  // Rewrite the URL path if versioned path was used
  // This allows existing handlers at /api/* to handle /api/v1/* requests
  if (rewrittenPath) {
    // Store original path for logging/debugging
    event.context.originalApiPath = path

    // Rewrite the URL for routing
    // Update the node request URL for Nitro's routing
    if (event.node?.req) {
      event.node.req.url = rewrittenPath + url.search
    }

    // Also update the path in the event (for H3)
    Object.defineProperty(event, 'path', {
      value: rewrittenPath,
      writable: true,
      configurable: true
    })
  }

  // Add deprecation warning header if version is deprecated
  if (isVersionDeprecated(version)) {
    const sunsetDate = VERSION_SUNSET_DATES[version]
    appendHeader(event, 'Deprecation', 'true')
    appendHeader(event, 'X-API-Deprecated', 'true')
    appendHeader(
      event,
      'X-API-Deprecation-Info',
      `API v${version} is deprecated. Please migrate to v${CURRENT_VERSION}.`
    )
    if (sunsetDate) {
      appendHeader(event, 'Sunset', sunsetDate)
    }
  }

  // Add version info header to all responses
  appendHeader(event, 'X-API-Version', String(version))
})

/**
 * Export version utilities for use in handlers
 */
export const apiVersion = {
  current: CURRENT_VERSION,
  supported: SUPPORTED_VERSIONS,
  deprecated: DEPRECATED_VERSIONS,

  /**
   * Get API version from event context
   */
  get(event: { context: { apiVersion?: number } }): number {
    return event.context.apiVersion || CURRENT_VERSION
  },

  /**
   * Check if a specific version or higher
   */
  isAtLeast(event: { context: { apiVersion?: number } }, minVersion: number): boolean {
    const version = event.context.apiVersion || CURRENT_VERSION
    return version >= minVersion
  },

  /**
   * Check if exact version
   */
  isVersion(event: { context: { apiVersion?: number } }, targetVersion: number): boolean {
    const version = event.context.apiVersion || CURRENT_VERSION
    return version === targetVersion
  }
}
