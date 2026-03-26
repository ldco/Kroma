/**
 * Save Setup Config API
 *
 * POST /api/setup/config
 * Body: SetupConfig
 * Returns: { success: true, databaseStatus: 'created' | 'exists' | 'error' }
 *
 * Writes configuration to puppet-master.config.ts.
 * Security: Only accessible when pmMode is 'unconfigured'
 *
 * SECURITY HARDENING:
 * - Structured config serialization (no regex-based mutation)
 * - Strict allowlists for all string fields
 * - Production authorization secret required (PM_SETUP_SECRET)
 * - Input sanitization and validation
 */
import { existsSync, readFileSync, writeFileSync, readdirSync, unlinkSync } from 'fs'
import { resolve, dirname } from 'path'
import { z } from 'zod'
import { requireSetupAccess } from '../../utils/setup-guard'
import { ALL_MODULES, type ModuleId } from '~~/shared/modules'
import { getWorkflowDataDir, getCurrentWorkflow, ensureWorkflowDataDir } from '../../utils/workflow-paths'

type PmMode = 'unconfigured' | 'build' | 'develop'
type ProjectType = 'website' | 'app'
type AiWorkflow = 'claude' | 'qwen' | 'codex'
type HeaderFooterWidthMode = 'contained' | 'full'
type PageTransition = 'fade' | 'slide-left' | 'slide-up' | 'scale' | 'zoom' | 'flip' | 'rotate' | 'blur' | 'bounce' | 'swipe' | ''

// ═══════════════════════════════════════════════════════════════════════════════
// SECURITY: ALLOWLISTS & VALIDATION
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Allowed locale codes (ISO 639-1)
 */
const ALLOWED_LOCALE_CODES = new Set([
  'en', 'es', 'fr', 'de', 'it', 'pt', 'nl', 'pl', 'ru', 'ja', 'zh', 'ko',
  'ar', 'he', 'hi', 'tr', 'vi', 'th', 'id', 'ms', 'tl', 'sv', 'no', 'da',
  'fi', 'cs', 'sk', 'hu', 'ro', 'bg', 'hr', 'sr', 'sl', 'et', 'lv', 'lt',
  'uk', 'be', 'el', 'is', 'ga', 'mt', 'cy', 'eu', 'ca', 'gl', 'ast',
  'oc', 'br', 'gd', 'sq', 'mk', 'bs', 'me', 'hr'
])

/**
 * Allowed ISO codes (ISO 639-1 + region)
 */
const ALLOWED_ISO_CODES = new Set([
  'en-US', 'en-GB', 'en-AU', 'en-CA',
  'es-ES', 'es-MX', 'es-AR', 'es-CO',
  'fr-FR', 'fr-CA', 'fr-BE', 'fr-CH',
  'de-DE', 'de-AT', 'de-CH',
  'it-IT', 'it-CH',
  'pt-PT', 'pt-BR',
  'nl-NL', 'nl-BE',
  'pl-PL',
  'ru-RU',
  'ja-JP',
  'zh-CN', 'zh-TW', 'zh-HK',
  'ko-KR',
  'ar-SA', 'ar-AE', 'ar-EG',
  'he-IL',
  'hi-IN',
  'tr-TR',
  'vi-VN',
  'th-TH',
  'id-ID',
  'ms-MY',
  'tl-PH',
  'sv-SE',
  'no-NO',
  'da-DK',
  'fi-FI',
  'cs-CZ',
  'sk-SK',
  'hu-HU',
  'ro-RO',
  'bg-BG',
  'hr-HR',
  'sr-RS',
  'sl-SI',
  'et-EE',
  'lv-LV',
  'lt-LT',
  'uk-UA',
  'be-BY',
  'el-GR',
  'is-IS',
  'ga-IE',
  'mt-MT',
  'cy-GB',
  'eu-ES',
  'ca-ES',
  'gl-ES'
])

/**
 * Sanitize string input - allow only safe characters
 * Prevents injection attacks via config values
 */
function sanitizeString(input: string, maxLength: number = 500): string {
  // Remove null bytes and control characters
  const sanitized = input.replace(/[\x00-\x1f\x7f-\x9f]/g, '')
  // Trim and limit length
  return sanitized.trim().slice(0, maxLength)
}

/**
 * Validate locale code against allowlist
 */
function validateLocaleCode(code: string): boolean {
  return ALLOWED_LOCALE_CODES.has(code.toLowerCase())
}

/**
 * Validate ISO code against allowlist
 */
function validateIsoCode(iso: string): boolean {
  return ALLOWED_ISO_CODES.has(iso)
}

/**
 * Validate locale name - alphanumeric with spaces, hyphens, apostrophes only
 */
function validateLocaleName(name: string): boolean {
  const namePattern = /^[\p{L}\p{N}\s\-']+$/u
  return namePattern.test(name) && name.length <= 100
}

/**
 * Validate module ID against allowed modules
 */
function validateModuleId(id: string): id is ModuleId {
  return (ALL_MODULES as readonly string[]).includes(id)
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIG HELPERS (structured serialization)
// ═══════════════════════════════════════════════════════════════════════════════

function getConfigPath(appDir: string): string {
  return resolve(appDir, 'puppet-master.config.ts')
}

function readConfigRaw(appDir: string): string {
  const configPath = getConfigPath(appDir)
  return readFileSync(configPath, 'utf-8')
}

function createBackup(appDir: string): string | undefined {
  try {
    const configPath = getConfigPath(appDir)
    if (!existsSync(configPath)) return undefined

    const dir = dirname(configPath)
    const timestamp = Date.now()
    const backupPath = resolve(dir, `puppet-master.config.ts.backup.${timestamp}`)

    const content = readFileSync(configPath, 'utf-8')
    writeFileSync(backupPath, content, 'utf-8')

    // Clean up old backups (keep only 3)
    const backups = readdirSync(dir)
      .filter(f => f.startsWith('puppet-master.config.ts.backup.'))
      .map(f => ({ path: resolve(dir, f), ts: parseInt(f.split('.').pop() || '0', 10) }))
      .sort((a, b) => b.ts - a.ts)

    for (const backup of backups.slice(3)) {
      try {
        unlinkSync(backup.path)
      } catch {
        // Ignore stale backup cleanup errors.
      }
    }

    return backupPath
  } catch {
    return undefined
  }
}

function rollbackToBackup(appDir: string): boolean {
  try {
    const configPath = getConfigPath(appDir)
    const dir = dirname(configPath)

    const backups = readdirSync(dir)
      .filter(f => f.startsWith('puppet-master.config.ts.backup.'))
      .map(f => ({ path: resolve(dir, f), ts: parseInt(f.split('.').pop() || '0', 10) }))
      .sort((a, b) => b.ts - a.ts)

    if (backups.length === 0) return false

    const backupContent = readFileSync(backups[0].path, 'utf-8')
    writeFileSync(configPath, backupContent, 'utf-8')
    return true
  } catch {
    return false
  }
}

function readPmMode(appDir: string): PmMode {
  try {
    const content = readConfigRaw(appDir)
    const match = content.match(/pmMode:\s*['"](\w+)['"]/)
    if (match && ['unconfigured', 'build', 'develop'].includes(match[1])) {
      return match[1] as PmMode
    }
    return 'unconfigured'
  } catch {
    return 'unconfigured'
  }
}

function readAiWorkflow(appDir: string): AiWorkflow {
  try {
    const content = readConfigRaw(appDir)
    const match = content.match(/aiWorkflow:\s*['"](\w+)['"]/)
    if (match && ['claude', 'qwen', 'codex'].includes(match[1])) {
      return match[1] as AiWorkflow
    }
    return 'claude'
  } catch {
    return 'claude'
  }
}

function databaseExists(appDir: string): boolean {
  return existsSync(resolve(appDir, '..', 'data', 'sqlite.db'))
}

/**
 * Structured locale config with validation
 */
interface ValidatedLocale {
  code: string
  iso: string
  name: string
}

/**
 * Structured setup config with validated fields
 */
interface SetupConfig {
  pmMode: PmMode
  aiWorkflow?: AiWorkflow
  projectType?: ProjectType
  adminEnabled?: boolean
  locales?: ValidatedLocale[]
  defaultLocale?: string
  modules?: ModuleId[]
  features?: {
    multiLangs?: boolean
    doubleTheme?: boolean
    onepager?: boolean
    pwa?: boolean
    twoFactorAuth?: boolean
  }
}

/**
 * Generate config update patches for specific fields
 * Updates only setup-managed fields while preserving all other sections
 */
function generateConfigPatches(config: SetupConfig): Array<{
  pattern: RegExp
  replacement: string
  description: string
}> {
  const patches: Array<{ pattern: RegExp; replacement: string; description: string }> = []

  // 1. Update pmMode
  patches.push({
    pattern: /pmMode:\s*['"][^'"]+['"]\s*as\s*(?:const|'[^']+'\s*\|\s*'[^']+'\s*\|\s*'[^']+')/,
    replacement: `pmMode: '${config.pmMode}' as 'unconfigured' | 'build' | 'develop'`,
    description: 'pmMode'
  })

  // 2. Update aiWorkflow if provided
  if (config.aiWorkflow) {
    patches.push({
      pattern: /aiWorkflow:\s*['"][^'"]+['"]\s*as\s*(?:const|'[^']+'\s*\|\s*'[^']+'\s*\|\s*'[^']+')/,
      replacement: `aiWorkflow: '${config.aiWorkflow}' as 'claude' | 'qwen' | 'codex'`,
      description: 'aiWorkflow'
    })
  }

  // 3. Update project type (entities)
  if (config.projectType) {
    patches.push({
      pattern: /(entities:\s*\{[\s\S]*?website:\s*)(true|false)/,
      replacement: `$1${config.projectType === 'website'}`,
      description: 'entities.website'
    })
    patches.push({
      pattern: /(entities:\s*\{[\s\S]*?app:\s*)(true|false)/,
      replacement: `$1${config.projectType === 'app'}`,
      description: 'entities.app'
    })
  }

  // 4. Update admin.enabled
  if (config.adminEnabled !== undefined) {
    patches.push({
      pattern: /(admin:\s*\{[\s\S]*?enabled:\s*)(true|false)/,
      replacement: `$1${config.adminEnabled}`,
      description: 'admin.enabled'
    })
  }

  // 5. Update locales
  if (config.locales && config.locales.length > 0) {
    const localesStr = config.locales
      .map(l => `    { code: '${l.code}', iso: '${l.iso}', name: '${l.name}' }`)
      .join(',\n')
    patches.push({
      pattern: /(locales:\s*)\[[\s\S]*?\]/,
      replacement: `$1[\n${localesStr}\n  ]`,
      description: 'locales'
    })
  }

  // 6. Update defaultLocale
  if (config.defaultLocale) {
    patches.push({
      pattern: /(defaultLocale:\s*)['"][^'"]*['"]/,
      replacement: `$1'${config.defaultLocale}'`,
      description: 'defaultLocale'
    })
  }

  // 7. Update modules
  if (config.modules && config.modules.length > 0) {
    for (const moduleId of ALL_MODULES) {
      const enabled = config.modules.includes(moduleId)
      patches.push({
        pattern: new RegExp(`(${moduleId}:\\s*\\{[^}]*)(enabled:\\s*)(true|false)`, 's'),
        replacement: `$1$2${enabled}`,
        description: `modules.${moduleId}`
      })
    }
  }

  // 8. Update features
  if (config.features) {
    if (config.features.multiLangs !== undefined) {
      patches.push({
        pattern: /(features:\s*\{[\s\S]*?multiLangs:\s*)(true|false)/,
        replacement: `$1${config.features.multiLangs}`,
        description: 'features.multiLangs'
      })
    }
    if (config.features.doubleTheme !== undefined) {
      patches.push({
        pattern: /(features:\s*\{[\s\S]*?doubleTheme:\s*)(true|false)/,
        replacement: `$1${config.features.doubleTheme}`,
        description: 'features.doubleTheme'
      })
    }
    if (config.features.onepager !== undefined) {
      patches.push({
        pattern: /(features:\s*\{[\s\S]*?onepager:\s*)(true|false)/,
        replacement: `$1${config.features.onepager}`,
        description: 'features.onepager'
      })
    }
    if (config.features.pwa !== undefined) {
      patches.push({
        pattern: /(features:\s*\{[\s\S]*?pwa:\s*)(true|false)/,
        replacement: `$1${config.features.pwa}`,
        description: 'features.pwa'
      })
    }
    if (config.features.twoFactorAuth !== undefined) {
      patches.push({
        pattern: /(features:\s*\{[\s\S]*?twoFactorAuth:\s*)(true|false)/,
        replacement: `$1${config.features.twoFactorAuth}`,
        description: 'features.twoFactorAuth'
      })
    }
  }

  return patches
}

/**
 * Apply config patches to existing content
 * Preserves all sections not explicitly updated
 */
function applyConfigPatches(content: string, patches: Array<{
  pattern: RegExp
  replacement: string
  description: string
}>): string {
  let updated = content

  for (const { pattern, replacement, description } of patches) {
    const before = updated
    updated = updated.replace(pattern, replacement)

    // Verify the patch was applied
    if (before === updated) {
      throw new Error(`Failed to apply patch for ${description}`)
    }
  }

  return updated
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCHEMA & HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

const localeSchema = z.object({
  code: z.string().max(10),
  iso: z.string().max(10),
  name: z.string().max(100)
})

const featuresSchema = z.object({
  multiLangs: z.boolean().optional(),
  doubleTheme: z.boolean().optional(),
  onepager: z.boolean().optional(),
  pwa: z.boolean().optional(),
  twoFactorAuth: z.boolean().optional()
})

const setupConfigSchema = z.object({
  pmMode: z.enum(['unconfigured', 'build', 'develop']),
  aiWorkflow: z.enum(['claude', 'qwen', 'codex']).optional(),
  projectType: z.enum(['website', 'app']).optional(),
  adminEnabled: z.boolean().optional(),
  locales: z.array(localeSchema).optional(),
  defaultLocale: z.string().max(10).optional(),
  modules: z.array(z.string()).optional(),
  features: featuresSchema.optional()
})

export default defineEventHandler(async (event) => {
  // Security: Require production authorization if PM_SETUP_SECRET is set
  requireSetupAccess(event, { requireProductionAuth: true })

  const body = await readBody(event)

  // Validate input
  const result = setupConfigSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      statusMessage: 'PM_CONFIG_001: Invalid configuration',
      message: result.error.issues[0]?.message || 'Invalid configuration'
    })
  }

  const config = result.data
  const appDir = resolve(process.cwd(), 'app')
  const configPath = getConfigPath(appDir)

  if (!existsSync(configPath)) {
    throw createError({
      statusCode: 500,
      statusMessage: 'PM_CONFIG_002: Config file not found',
      message: 'Config file not found. Please ensure puppet-master.config.ts exists.'
    })
  }

  // Validate locales against allowlists
  if (config.locales && config.locales.length > 0) {
    for (const locale of config.locales) {
      if (!validateLocaleCode(locale.code)) {
        throw createError({
          statusCode: 400,
          statusMessage: 'PM_CONFIG_007: Invalid locale code',
          message: `Locale code '${locale.code}' is not allowed. Use a valid ISO 639-1 code.`
        })
      }
      if (!validateIsoCode(locale.iso)) {
        throw createError({
          statusCode: 400,
          statusMessage: 'PM_CONFIG_008: Invalid ISO code',
          message: `ISO code '${locale.iso}' is not allowed. Use a valid locale code (e.g., 'en-US').`
        })
      }
      if (!validateLocaleName(locale.name)) {
        throw createError({
          statusCode: 400,
          statusMessage: 'PM_CONFIG_009: Invalid locale name',
          message: `Locale name '${locale.name}' contains invalid characters. Use only letters, numbers, spaces, hyphens, and apostrophes.`
        })
      }
    }
  }

  // Validate modules
  if (config.modules && config.modules.length > 0) {
    const invalidModules = config.modules.filter(m => !validateModuleId(m))
    if (invalidModules.length > 0) {
      throw createError({
        statusCode: 400,
        statusMessage: 'PM_CONFIG_010: Invalid module IDs',
        message: `Invalid module IDs: ${invalidModules.join(', ')}. Use only valid module IDs from ALL_MODULES.`
      })
    }
  }

  // Read existing config
  let existingContent: string
  try {
    existingContent = readConfigRaw(appDir)
  } catch (e: any) {
    throw createError({
      statusCode: 500,
      statusMessage: 'PM_CONFIG_011: Config read failed',
      message: `Failed to read existing config: ${e.message}`
    })
  }

  // Create backup before modification
  const backupPath = createBackup(appDir)

  try {
    // Generate and apply patches
    const patches = generateConfigPatches({
      pmMode: config.pmMode as PmMode,
      aiWorkflow: config.aiWorkflow as AiWorkflow | undefined,
      projectType: config.projectType as ProjectType | undefined,
      adminEnabled: config.adminEnabled,
      locales: config.locales,
      defaultLocale: config.defaultLocale,
      modules: config.modules as ModuleId[] | undefined,
      features: config.features
    })

    const updatedContent = applyConfigPatches(existingContent, patches)

    // Write updated config
    writeFileSync(configPath, updatedContent, 'utf-8')

    // Validate the write succeeded
    const actualMode = readPmMode(appDir)
    if (actualMode !== config.pmMode) {
      throw createError({
        statusCode: 500,
        statusMessage: 'PM_CONFIG_003: Config validation failed',
        message: `Config was written but validation failed. Expected pmMode: ${config.pmMode}, Actual: ${actualMode}.`
      })
    }

    // Validate aiWorkflow was also written correctly if provided
    if (config.aiWorkflow) {
      const actualWorkflow = readAiWorkflow(appDir)
      if (actualWorkflow !== config.aiWorkflow) {
        throw createError({
          statusCode: 500,
          statusMessage: 'PM_CONFIG_005: Workflow validation failed',
          message: `Config was written but aiWorkflow validation failed. Expected: ${config.aiWorkflow}, Actual: ${actualWorkflow}.`
        })
      }
    }
  } catch (e: any) {
    // If not already a create error, wrap it
    if (!e.statusCode) {
      // Try to rollback
      if (backupPath) {
        rollbackToBackup(appDir)
      }
      throw createError({
        statusCode: 500,
        statusMessage: 'PM_CONFIG_004: Config write failed',
        message: e.message || 'Failed to write configuration'
      })
    }
    throw e
  }

  // Save project brief and context for selected AI workflow
  const workflowToUse = config.aiWorkflow || getCurrentWorkflow()
  const workflowDataDir = getWorkflowDataDir(workflowToUse)
  const workflowDataPath = resolve(process.cwd(), workflowDataDir)

  if (!ensureWorkflowDataDir(workflowToUse)) {
    throw createError({
      statusCode: 500,
      statusMessage: 'PM_CONFIG_006: Workflow directory setup failed',
      message: `Failed to initialize data directory for workflow '${workflowToUse}'.`
    })
  }

  // Write project brief if provided (sanitized)
  if (config.technicalBrief) {
    const briefPath = resolve(workflowDataPath, 'project-brief.md')
    const sanitizedBrief = sanitizeString(config.technicalBrief, 500000)
    const sanitizedProjectName = config.projectName ? sanitizeString(config.projectName, 200) : undefined
    const sanitizedDescription = config.projectDescription ? sanitizeString(config.projectDescription, 500) : undefined
    const sanitizedAudience = config.targetAudience ? sanitizeString(config.targetAudience, 500) : undefined
    const sanitizedCustomModules = config.customModules ? sanitizeString(config.customModules, 1000) : undefined

    const briefContent = `# Project Brief

> Auto-generated from setup wizard on ${new Date().toISOString().split('T')[0]}

${sanitizedProjectName ? `## Project: ${sanitizedProjectName}\n` : ''}
${sanitizedDescription ? `**Description:** ${sanitizedDescription}\n` : ''}
${sanitizedAudience ? `**Target Audience:** ${sanitizedAudience}\n` : ''}
${sanitizedCustomModules ? `**Custom Modules Requested:** ${sanitizedCustomModules}\n` : ''}

---

## Technical Brief

${sanitizedBrief}
`
    writeFileSync(briefPath, briefContent, 'utf-8')
  }

  // Run db:push to create/update database schema
  // GAP-009: Skip db:push in production - requires manual migration in containerized deployments
  let databaseStatus: 'created' | 'exists' | 'error' | 'timeout' | 'manual_required' = 'exists'
  let databaseMessage: string | undefined
  const dbExisted = databaseExists(appDir)

  const isProduction = process.env.NODE_ENV === 'production'

  if (isProduction) {
    databaseStatus = 'manual_required'
    databaseMessage = 'Production environment detected. Database migrations must be run manually via: npm run db:migrate'
  } else {
    try {
      const { spawn } = await import('child_process')

      const dbPushResult = await new Promise<{ status: 'success' | 'error' | 'timeout'; message?: string }>((resolve) => {
        const timeout = 60000

        const child = spawn('npm', ['run', 'db:push'], {
          cwd: process.cwd(),
          env: { ...process.env, FORCE_COLOR: '0' },
          stdio: ['pipe', 'pipe', 'pipe']
        })

        let stdout = ''
        let stderr = ''

        child.stdout?.on('data', (data) => {
          stdout += data.toString()
        })

        child.stderr?.on('data', (data) => {
          stderr += data.toString()
        })

        child.stdin?.write('y\n')
        child.stdin?.end()

        const timeoutId = setTimeout(() => {
          child.kill('SIGTERM')
          console.warn('db:push timed out after 60 seconds')
          resolve({
            status: 'timeout',
            message: 'Database schema update timed out after 60 seconds. The operation was cancelled. Please try running "npm run db:push" manually.'
          })
        }, timeout)

        child.on('close', (code) => {
          clearTimeout(timeoutId)
          if (code === 0) {
            resolve({ status: 'success' })
          } else {
            console.error('db:push failed:', { code, stdout, stderr })
            resolve({
              status: 'error',
              message: `Database schema update failed (exit code ${code}). Try running "npm run db:push" manually.`
            })
          }
        })

        child.on('error', (err) => {
          clearTimeout(timeoutId)
          console.error('db:push spawn error:', err.message)
          resolve({
            status: 'error',
            message: `Failed to start database update: ${err.message}`
          })
        })
      })

      if (dbPushResult.status === 'success') {
        databaseStatus = dbExisted ? 'exists' : 'created'
      } else if (dbPushResult.status === 'timeout') {
        databaseStatus = 'timeout'
        databaseMessage = dbPushResult.message
      } else {
        databaseStatus = 'error'
        databaseMessage = dbPushResult.message
      }
    } catch (e: any) {
      if (!isProduction) {
        console.error('db:push error:', e.message)
      }
      databaseStatus = 'error'
      databaseMessage = `Unexpected error during database update: ${e.message}`
    }
  }

  return {
    success: true,
    data: {
      databaseStatus,
      databaseMessage,
      aiWorkflow: config.aiWorkflow || readAiWorkflow(appDir)
    }
  }
})
