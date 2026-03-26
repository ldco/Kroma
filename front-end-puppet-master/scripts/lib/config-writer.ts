/**
 * Puppet Master Config Writer
 *
 * Modifies specific values in the puppet-master.config.ts file.
 * Used by: setup wizard API, CLI prompts, headless scripts
 *
 * Strategy: Regex-based replacement to preserve formatting and comments.
 * Falls back to full rewrite only when adding new fields.
 *
 * Features:
 * - Backup before writes
 * - Validation after writes
 * - Rollback on failure
 */

import { readFileSync, writeFileSync, existsSync, unlinkSync, readdirSync, statSync } from 'fs'
import { resolve, dirname, basename } from 'path'
import type { PmMode, ProjectType } from './config-reader'
import { getConfigPath, readConfigRaw, readPmMode } from './config-reader'
import { ALL_MODULES, type ModuleId } from './modules'

// ═══════════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════════

export interface SetupConfig {
  pmMode: PmMode
  projectType?: ProjectType
  adminEnabled?: boolean
  locales?: Array<{ code: string; iso: string; name: string }>
  defaultLocale?: string
  modules?: string[]
  features?: {
    multiLangs?: boolean
    doubleTheme?: boolean
    onepager?: boolean
    pwa?: boolean
  }
}

export interface WriteResult {
  success: boolean
  backupPath?: string
  error?: string
}

// ═══════════════════════════════════════════════════════════════════════════════
// BACKUP & ROLLBACK
// ═══════════════════════════════════════════════════════════════════════════════

const MAX_BACKUPS = 3

/**
 * Create a backup of the config file before modification
 * Returns the backup path or throws on failure
 */
export function createConfigBackup(appDir?: string): string {
  const configPath = getConfigPath(appDir)
  if (!existsSync(configPath)) {
    throw new Error(`Config file not found: ${configPath}`)
  }

  const dir = dirname(configPath)
  const timestamp = Date.now()
  const backupPath = resolve(dir, `puppet-master.config.ts.backup.${timestamp}`)

  const content = readFileSync(configPath, 'utf-8')
  writeFileSync(backupPath, content, 'utf-8')

  // Clean up old backups (keep only MAX_BACKUPS)
  cleanupOldBackups(dir)

  return backupPath
}

/**
 * Rollback to a specific backup file
 */
export function rollbackConfig(backupPath: string, appDir?: string): void {
  if (!existsSync(backupPath)) {
    throw new Error(`Backup file not found: ${backupPath}`)
  }

  const configPath = getConfigPath(appDir)
  const backupContent = readFileSync(backupPath, 'utf-8')
  writeFileSync(configPath, backupContent, 'utf-8')
}

/**
 * Rollback to the most recent backup
 */
export function rollbackToLatestBackup(appDir?: string): boolean {
  const configPath = getConfigPath(appDir)
  const dir = dirname(configPath)
  const backups = getBackupFiles(dir)

  if (backups.length === 0) {
    return false
  }

  // Backups are sorted newest first
  rollbackConfig(backups[0], appDir)
  return true
}

/**
 * Get list of backup files sorted by timestamp (newest first)
 */
function getBackupFiles(dir: string): string[] {
  if (!existsSync(dir)) return []

  const files = readdirSync(dir)
    .filter(f => f.startsWith('puppet-master.config.ts.backup.'))
    .map(f => ({
      path: resolve(dir, f),
      timestamp: parseInt(f.split('.').pop() || '0', 10)
    }))
    .sort((a, b) => b.timestamp - a.timestamp)
    .map(f => f.path)

  return files
}

/**
 * Remove old backups, keeping only the most recent MAX_BACKUPS
 */
function cleanupOldBackups(dir: string): void {
  const backups = getBackupFiles(dir)

  // Remove backups beyond MAX_BACKUPS
  for (const backup of backups.slice(MAX_BACKUPS)) {
    try {
      unlinkSync(backup)
    } catch {
      // Ignore cleanup errors
    }
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VALIDATION
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Validate that the config file is valid after modification
 * Checks that pmMode can be read back correctly
 */
export function validateConfigAfterWrite(appDir?: string, expectedMode?: PmMode): boolean {
  try {
    const actualMode = readPmMode(appDir)

    // If we expected a specific mode, verify it
    if (expectedMode && actualMode !== expectedMode) {
      return false
    }

    // Basic check - mode should be valid
    if (!['unconfigured', 'build', 'develop'].includes(actualMode)) {
      return false
    }

    return true
  } catch {
    return false
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REPLACEMENT PATTERNS (exported for reuse)
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Replace a simple value in the config
 * Pattern: key: value or key: 'value'
 */
export function replaceValue(content: string, pattern: RegExp, newValue: string): string {
  if (pattern.test(content)) {
    return content.replace(pattern, newValue)
  }
  return content
}

/**
 * Replace boolean value
 * Matches: key: true or key: false
 */
export function replaceBoolean(content: string, key: string, value: boolean): string {
  const pattern = new RegExp(`(${key}:\\s*)(true|false)`, 'g')
  return content.replace(pattern, `$1${value}`)
}

/**
 * Replace string value
 * Matches: key: 'value' or key: "value"
 * Note: Handles values with hyphens (e.g., 'en-US')
 */
export function replaceString(content: string, key: string, value: string): string {
  // Updated pattern to handle values with hyphens and special chars
  const pattern = new RegExp(`(${key}:\\s*)['"][^'"]*['"]`, 'g')
  return content.replace(pattern, `$1'${value}'`)
}

// ═══════════════════════════════════════════════════════════════════════════════
// FIELD WRITERS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Add pmMode field to config if missing
 */
function ensurePmModeField(content: string): string {
  // Check if pmMode already exists
  if (/pmMode:/.test(content)) {
    return content
  }

  // Add pmMode after the opening of const config = {
  const configStart = content.indexOf('const config = {')
  if (configStart === -1) {
    throw new Error('Could not find "const config = {" in config file')
  }

  const insertPoint = content.indexOf('{', configStart) + 1
  const before = content.slice(0, insertPoint)
  const after = content.slice(insertPoint)

  return `${before}
  // ═══════════════════════════════════════════════════════════════════════════
  // PM MODE - Setup state (unconfigured, build, develop)
  // ═══════════════════════════════════════════════════════════════════════════
  pmMode: 'unconfigured' as const,

${after}`
}

/**
 * Set pmMode value
 * Handles both type annotation forms:
 *   - pmMode: 'value' as const
 *   - pmMode: 'value' as 'unconfigured' | 'build' | 'develop'
 */
export function writePmMode(appDir: string | undefined, mode: PmMode): void {
  let content = readConfigRaw(appDir)

  // Ensure field exists
  content = ensurePmModeField(content)

  // Replace value - handle both annotation styles:
  // 1. pmMode: 'value' as const
  // 2. pmMode: 'value' as 'unconfigured' | 'build' | 'develop'
  const pmModePatterns = [
    // Match: pmMode: 'value' as const
    /pmMode:\s*['"][^'"]+['"]\s*as\s+const/,
    // Match: pmMode: 'value' as 'unconfigured' | 'build' | 'develop' (with any whitespace)
    /pmMode:\s*['"][^'"]+['"]\s*as\s+['"][^'"]+['"]\s*\|\s*['"][^'"]+['"]\s*\|\s*['"][^'"]+['"]/
  ]

  let replaced = false
  for (const pattern of pmModePatterns) {
    if (pattern.test(content)) {
      // Use the union type form as the canonical output
      content = content.replace(
        pattern,
        `pmMode: '${mode}' as 'unconfigured' | 'build' | 'develop'`
      )
      replaced = true
      break
    }
  }

  // Fallback: if no pattern matched but pmMode exists, try simple replacement
  if (!replaced && /pmMode:\s*['"][^'"]+['"]/.test(content)) {
    content = content.replace(
      /pmMode:\s*['"][^'"]+['"]/,
      `pmMode: '${mode}' as 'unconfigured' | 'build' | 'develop'`
    )
  }

  const configPath = getConfigPath(appDir)
  writeFileSync(configPath, content, 'utf-8')
}

/**
 * Set project type (entities.website/app)
 */
export function writeProjectType(appDir: string | undefined, type: ProjectType): void {
  let content = readConfigRaw(appDir)

  if (type === 'website') {
    content = replaceBoolean(content, 'website', true)
    content = replaceBoolean(content, 'app', false)
  } else {
    content = replaceBoolean(content, 'website', false)
    content = replaceBoolean(content, 'app', true)
  }

  const configPath = getConfigPath(appDir)
  writeFileSync(configPath, content, 'utf-8')
}

/**
 * Set admin enabled
 */
export function writeAdminEnabled(appDir: string | undefined, enabled: boolean): void {
  let content = readConfigRaw(appDir)

  // Find admin section and replace enabled
  // Match: admin: { ... enabled: true/false
  const adminPattern = /(admin:\s*\{[\s\S]*?)(enabled:\s*)(true|false)/
  content = content.replace(adminPattern, `$1$2${enabled}`)

  const configPath = getConfigPath(appDir)
  writeFileSync(configPath, content, 'utf-8')
}

/**
 * Set locales array
 */
export function writeLocales(
  appDir: string | undefined,
  locales: Array<{ code: string; iso: string; name: string }>,
  defaultLocale: string
): void {
  let content = readConfigRaw(appDir)

  // Generate new locales array
  const localesStr = locales
    .map(l => `    { code: '${l.code}', iso: '${l.iso}', name: '${l.name}' }`)
    .join(',\n')

  // Replace locales array
  const localesPattern = /locales:\s*\[[\s\S]*?\]/
  content = content.replace(localesPattern, `locales: [\n${localesStr}\n  ]`)

  // Replace defaultLocale
  content = replaceString(content, 'defaultLocale', defaultLocale)

  const configPath = getConfigPath(appDir)
  writeFileSync(configPath, content, 'utf-8')
}

/**
 * Set module enabled state
 */
export function writeModuleEnabled(appDir: string | undefined, moduleId: string, enabled: boolean): void {
  let content = readConfigRaw(appDir)

  // Find the module section and set enabled
  // Pattern: moduleId: { ... enabled: true/false
  const modulePattern = new RegExp(`(${moduleId}:\\s*\\{[^}]*)(enabled:\\s*)(true|false)`, 's')
  content = content.replace(modulePattern, `$1$2${enabled}`)

  const configPath = getConfigPath(appDir)
  writeFileSync(configPath, content, 'utf-8')
}

/**
 * Set multiple modules at once
 * Uses centralized ALL_MODULES from modules.ts
 */
export function writeModules(appDir: string | undefined, enabledModules: string[]): void {
  for (const moduleId of ALL_MODULES) {
    writeModuleEnabled(appDir, moduleId, enabledModules.includes(moduleId))
  }
}

/**
 * Set feature flags
 */
export function writeFeatures(
  appDir: string | undefined,
  features: Partial<{
    multiLangs: boolean
    doubleTheme: boolean
    onepager: boolean
    pwa: boolean
  }>
): void {
  let content = readConfigRaw(appDir)

  if (features.multiLangs !== undefined) {
    content = replaceBoolean(content, 'multiLangs', features.multiLangs)
  }
  if (features.doubleTheme !== undefined) {
    content = replaceBoolean(content, 'doubleTheme', features.doubleTheme)
  }
  if (features.onepager !== undefined) {
    content = replaceBoolean(content, 'onepager', features.onepager)
  }
  if (features.pwa !== undefined) {
    content = replaceBoolean(content, 'pwa', features.pwa)
  }

  const configPath = getConfigPath(appDir)
  writeFileSync(configPath, content, 'utf-8')
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN WRITER
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Apply full setup configuration
 * Used by wizard to save all selections at once
 */
export function applySetupConfig(appDir: string | undefined, config: SetupConfig): void {
  // Order matters - some writes depend on others
  writePmMode(appDir, config.pmMode)

  if (config.projectType) {
    writeProjectType(appDir, config.projectType)
  }

  if (config.adminEnabled !== undefined) {
    writeAdminEnabled(appDir, config.adminEnabled)
  }

  if (config.locales && config.defaultLocale) {
    writeLocales(appDir, config.locales, config.defaultLocale)
  }

  if (config.modules) {
    writeModules(appDir, config.modules)
  }

  if (config.features) {
    writeFeatures(appDir, config.features)
  }
}

/**
 * Reset config to unconfigured state
 * Preserves existing values but sets pmMode to 'unconfigured'
 */
export function resetConfig(appDir?: string): void {
  writePmMode(appDir, 'unconfigured')
}

/**
 * Quick setup for DEVELOP mode
 * Enables showcase features
 */
export function setupDevelopMode(appDir?: string): void {
  applySetupConfig(appDir, {
    pmMode: 'develop',
    projectType: 'website',
    adminEnabled: true,
    modules: ['portfolio', 'blog', 'team', 'testimonials', 'features', 'clients', 'faq', 'pricing', 'contact'],
    features: {
      multiLangs: true,
      doubleTheme: true,
      onepager: false,
      pwa: false
    }
  })
}

/**
 * Quick setup for BUILD mode with minimal config
 */
export function setupBuildMode(appDir: string | undefined, type: ProjectType): void {
  applySetupConfig(appDir, {
    pmMode: 'build',
    projectType: type,
    adminEnabled: type === 'app',
    modules: type === 'website' ? ['contact'] : [],
    features: {
      multiLangs: false,
      doubleTheme: true,
      onepager: false,
      pwa: false
    }
  })
}
