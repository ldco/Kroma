/**
 * Setup API Guard
 *
 * Prevents access to setup endpoints when project is already configured.
 * Setup APIs should only be accessible when pmMode is 'unconfigured'.
 *
 * Security: Fails CLOSED - denies access when config cannot be read.
 *
 * Used by: config.get.ts, config.post.ts, import-zip.post.ts, import-zip.delete.ts
 */
import { readFileSync, existsSync } from 'fs'
import { resolve } from 'path'
import type { H3Event } from 'h3'

type PmMode = 'unconfigured' | 'build' | 'develop'

// Result type for config read operation
type ConfigReadResult = {
  success: true
  mode: PmMode
} | {
  success: false
  error: 'not_found' | 'read_error' | 'parse_error'
  message: string
}

/**
 * Read pmMode from puppet-master.config.ts
 * Returns a result object indicating success/failure
 */
function readPmModeResult(): ConfigReadResult {
  const configPath = resolve(process.cwd(), 'app', 'puppet-master.config.ts')

  if (!existsSync(configPath)) {
    // Config file doesn't exist - this is expected for fresh installs
    // Allow setup in this case
    return { success: true, mode: 'unconfigured' }
  }

  try {
    const content = readFileSync(configPath, 'utf-8')
    // Match pmMode value regardless of type annotation style
    const match = content.match(/pmMode:\s*['"](\w+)['"]/)
    if (match && ['unconfigured', 'build', 'develop'].includes(match[1])) {
      return { success: true, mode: match[1] as PmMode }
    }
    // Config exists but pmMode not found or invalid - treat as parse error
    return {
      success: false,
      error: 'parse_error',
      message: 'Could not parse pmMode from config file'
    }
  } catch (e: any) {
    // File exists but couldn't be read - security risk, fail closed
    return {
      success: false,
      error: 'read_error',
      message: `Failed to read config file: ${e.message}`
    }
  }
}

/**
 * Options for setup guard
 */
export interface SetupGuardOptions {
  /** Allow access in specific modes (default: only 'unconfigured') */
  allowModes?: PmMode[]
  /** Allow GET requests regardless of mode (for reading current config) */
  allowReadOnly?: boolean
  /** Require production setup authorization secret */
  requireProductionAuth?: boolean
}

/**
 * Validate production setup authorization secret
 * Must match PM_SETUP_SECRET environment variable
 */
function validateProductionAuth(event: H3Event): boolean {
  const authHeader = getHeader(event, 'x-pm-setup-authorization')
  const setupSecret = process.env.PM_SETUP_SECRET

  // If no secret is configured, allow access (development mode)
  if (!setupSecret) {
    return true
  }

  // Require exact match with authorization header
  return authHeader === setupSecret
}

/**
 * Guard function for setup API endpoints
 *
 * Security: Fails CLOSED - denies access when config cannot be determined.
 * This prevents setup endpoints from being accessible in misconfigured deployments.
 *
 * Throws 403 error if:
 * - pmMode cannot be determined (read/parse errors)
 * - pmMode is not 'unconfigured' (for mutating operations)
 * - pmMode is not in allowModes (if specified)
 * - Production authorization secret is missing or invalid (when required)
 *
 * @param event - H3 event object
 * @param options - Guard options
 * @returns Current pmMode if access is allowed
 * @throws 403/500 error if access is denied or config cannot be read
 */
export function requireSetupAccess(event: H3Event, options: SetupGuardOptions = {}): PmMode {
  const { allowModes = ['unconfigured'], allowReadOnly = false, requireProductionAuth = false } = options

  const result = readPmModeResult()
  const method = event.method.toUpperCase()

  // Validate production authorization if required
  if (requireProductionAuth && !validateProductionAuth(event)) {
    throw createError({
      statusCode: 403,
      statusMessage: 'PM_SETUP_003: Production authorization failed',
      message: 'Invalid or missing setup authorization secret. Provide a valid X-PM-Setup-Authorization header matching PM_SETUP_SECRET environment variable.'
    })
  }

  // If config read failed, deny access (fail closed)
  if (!result.success) {
    // For read-only requests, we can be slightly more lenient with parse errors
    // since they're just reading state, not modifying it
    if (allowReadOnly && method === 'GET' && result.error === 'parse_error') {
      // Allow GET to show current (broken) state
      // Return unconfigured so the wizard can help fix the config
      return 'unconfigured'
    }

    throw createError({
      statusCode: 500,
      statusMessage: 'PM_SETUP_002: Config read failed',
      message: `Cannot determine setup state: ${result.message}. Setup endpoints are blocked for security. Please verify puppet-master.config.ts is readable and properly formatted.`
    })
  }

  const pmMode = result.mode

  // Allow read-only access if option is set
  if (allowReadOnly && method === 'GET') {
    return pmMode
  }

  // Check if current mode allows access
  if (!allowModes.includes(pmMode)) {
    throw createError({
      statusCode: 403,
      statusMessage: 'PM_SETUP_001: Setup access denied',
      message: `Setup API is only accessible when project is unconfigured. Current mode: '${pmMode}'. To reconfigure, run '/pm-init --reset' or manually set pmMode to 'unconfigured' in puppet-master.config.ts.`
    })
  }

  return pmMode
}

/**
 * Check if setup is allowed without throwing
 * Useful for conditional logic
 * Returns false if config cannot be read (fail closed)
 */
export function isSetupAllowed(options: SetupGuardOptions = {}): boolean {
  const { allowModes = ['unconfigured'] } = options
  const result = readPmModeResult()

  if (!result.success) {
    return false // Fail closed
  }

  return allowModes.includes(result.mode)
}

/**
 * Get current pmMode for informational purposes
 * Returns 'unconfigured' if config cannot be read
 */
export function getCurrentPmMode(): PmMode {
  const result = readPmModeResult()
  return result.success ? result.mode : 'unconfigured'
}

/**
 * Get detailed config read status for diagnostics
 */
export function getConfigStatus(): ConfigReadResult {
  return readPmModeResult()
}
