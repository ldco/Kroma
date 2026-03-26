/**
 * Puppet Master Config Reader
 *
 * Reads and parses the puppet-master.config.ts file.
 * Used by: setup wizard, CLI prompts, headless scripts, /pm-status command
 */

import { readFileSync, existsSync } from 'fs'
import { resolve } from 'path'

// ═══════════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════════

export type PmMode = 'unconfigured' | 'build' | 'develop'
export type ProjectType = 'website' | 'app'

export interface PmConfigSummary {
  /** Setup state */
  pmMode: PmMode
  /** Project type (only relevant in build mode) */
  projectType: ProjectType | null
  /** Admin panel enabled */
  adminEnabled: boolean
  /** Active locales */
  locales: string[]
  /** Default locale */
  defaultLocale: string
  /** Enabled modules */
  enabledModules: string[]
  /** Feature flags */
  features: {
    multiLangs: boolean
    doubleTheme: boolean
    onepager: boolean
    pwa: boolean
  }
  /** Has import folder with content */
  hasBrownfieldContent: boolean
  /** Database exists */
  databaseExists: boolean
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIG PATH
// ═══════════════════════════════════════════════════════════════════════════════

const CONFIG_FILENAME = 'puppet-master.config.ts'

/**
 * Get the path to the config file
 * @param appDir - The app directory containing puppet-master.config.ts
 *                 If not provided, defaults to process.cwd() (for use from project root)
 */
export function getConfigPath(appDir?: string): string {
  const dir = appDir || process.cwd()
  // If appDir is provided, it should be the actual app directory
  // If not provided (cwd), we're likely running from project root, so add 'app'
  if (appDir) {
    return resolve(dir, CONFIG_FILENAME)
  }
  return resolve(dir, 'app', CONFIG_FILENAME)
}

/**
 * Check if config file exists
 */
export function configExists(appDir?: string): boolean {
  return existsSync(getConfigPath(appDir))
}

// ═══════════════════════════════════════════════════════════════════════════════
// REGEX PATTERNS FOR PARSING
// ═══════════════════════════════════════════════════════════════════════════════

const PATTERNS = {
  // pmMode: 'unconfigured' | 'build' | 'develop'
  pmMode: /pmMode:\s*['"](\w+)['"]/,

  // entities.website: true/false
  entityWebsite: /website:\s*(true|false)/,
  entityApp: /app:\s*(true|false)/,

  // admin.enabled: true/false
  adminEnabled: /admin:\s*\{[\s\S]*?enabled:\s*(true|false)/,

  // locales array
  locales: /locales:\s*\[([\s\S]*?)\]/,
  localeCode: /code:\s*['"](\w+)['"]/g,

  // defaultLocale
  defaultLocale: /defaultLocale:\s*['"](\w+)['"]/,

  // modules - each module with enabled: true/false
  moduleEnabled: /(\w+):\s*\{[^}]*enabled:\s*(true|false)/g,

  // features
  featureMultiLangs: /multiLangs:\s*(true|false)/,
  featureDoubleTheme: /doubleTheme:\s*(true|false)/,
  featureOnepager: /onepager:\s*(true|false)/,
  featurePwa: /pwa:\s*(true|false)/
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Extract a complete brace-balanced block starting at a given position
 * Returns the full content between the opening { and closing }
 */
function extractBraceBlock(content: string, startKeyword: string): string | null {
  const keywordIndex = content.indexOf(startKeyword)
  if (keywordIndex === -1) return null

  // Find the opening brace after the keyword
  let openBraceIndex = content.indexOf('{', keywordIndex)
  if (openBraceIndex === -1) return null

  // Use brace balancing to find the matching closing brace
  let depth = 1
  let i = openBraceIndex + 1

  while (i < content.length && depth > 0) {
    const char = content[i]
    if (char === '{') {
      depth++
    } else if (char === '}') {
      depth--
    }
    i++
  }

  if (depth !== 0) {
    // Unbalanced braces, return what we have
    return null
  }

  // Return the full block including the keyword
  return content.slice(keywordIndex, i)
}

// ═══════════════════════════════════════════════════════════════════════════════
// READER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Read raw config file content
 */
export function readConfigRaw(appDir?: string): string {
  const configPath = getConfigPath(appDir)
  if (!existsSync(configPath)) {
    throw new Error(`Config file not found: ${configPath}`)
  }
  return readFileSync(configPath, 'utf-8')
}

/**
 * Extract pmMode from config
 */
export function readPmMode(appDir?: string): PmMode {
  const content = readConfigRaw(appDir)
  const match = content.match(PATTERNS.pmMode)
  if (match && ['unconfigured', 'build', 'develop'].includes(match[1])) {
    return match[1] as PmMode
  }
  // If pmMode not found, check if this is an old config (no pmMode field)
  // Old configs without pmMode are considered 'unconfigured'
  return 'unconfigured'
}

/**
 * Extract project type from entities config
 */
export function readProjectType(appDir?: string): ProjectType | null {
  const content = readConfigRaw(appDir)
  const websiteMatch = content.match(PATTERNS.entityWebsite)
  const appMatch = content.match(PATTERNS.entityApp)

  const hasWebsite = websiteMatch?.[1] === 'true'
  const hasApp = appMatch?.[1] === 'true'

  if (hasWebsite && !hasApp) return 'website'
  if (hasApp && !hasWebsite) return 'app'
  if (hasWebsite && hasApp) return 'website' // Website takes precedence
  return null
}

/**
 * Extract admin enabled status
 */
export function readAdminEnabled(appDir?: string): boolean {
  const content = readConfigRaw(appDir)
  const match = content.match(PATTERNS.adminEnabled)
  return match?.[1] === 'true'
}

/**
 * Extract locale codes
 */
export function readLocales(appDir?: string): string[] {
  const content = readConfigRaw(appDir)
  const localesMatch = content.match(PATTERNS.locales)
  if (!localesMatch) return ['en']

  const localesBlock = localesMatch[1]
  const codes: string[] = []
  let match
  while ((match = PATTERNS.localeCode.exec(localesBlock)) !== null) {
    codes.push(match[1])
  }
  return codes.length > 0 ? codes : ['en']
}

/**
 * Extract default locale
 */
export function readDefaultLocale(appDir?: string): string {
  const content = readConfigRaw(appDir)
  const match = content.match(PATTERNS.defaultLocale)
  return match?.[1] || 'en'
}

/**
 * Extract enabled modules using brace-balanced parsing
 * This ensures we capture all modules regardless of config size
 */
export function readEnabledModules(appDir?: string): string[] {
  const content = readConfigRaw(appDir)

  // Extract the complete modules block using brace balancing
  const modulesBlock = extractBraceBlock(content, 'modules:')
  if (!modulesBlock) return []

  const enabled: string[] = []

  // List of known non-module keys to skip
  const skipKeys = ['config', 'system', 'websiteModules', 'appModules']

  // Parse each module entry
  // Pattern: moduleName: { ... enabled: true ... }
  // We need to handle nested braces within module configs
  const moduleEntryPattern = /(\w+):\s*\{/g
  let match

  while ((match = moduleEntryPattern.exec(modulesBlock)) !== null) {
    const moduleName = match[1]

    // Skip non-module keys
    if (skipKeys.includes(moduleName)) continue

    // Extract this module's config block
    const startIndex = match.index + match[0].length - 1 // Position of opening {
    let depth = 1
    let endIndex = startIndex + 1

    while (endIndex < modulesBlock.length && depth > 0) {
      const char = modulesBlock[endIndex]
      if (char === '{') depth++
      else if (char === '}') depth--
      endIndex++
    }

    // Extract the module's config content
    const moduleConfig = modulesBlock.slice(startIndex, endIndex)

    // Check if enabled: true exists in this module's config
    // Only match at the module level, not in nested config objects
    if (/enabled:\s*true/.test(moduleConfig)) {
      enabled.push(moduleName)
    }
  }

  return enabled
}

/**
 * Extract feature flags
 */
export function readFeatures(appDir?: string): PmConfigSummary['features'] {
  const content = readConfigRaw(appDir)
  return {
    multiLangs: content.match(PATTERNS.featureMultiLangs)?.[1] === 'true',
    doubleTheme: content.match(PATTERNS.featureDoubleTheme)?.[1] === 'true',
    onepager: content.match(PATTERNS.featureOnepager)?.[1] === 'true',
    pwa: content.match(PATTERNS.featurePwa)?.[1] === 'true'
  }
}

/**
 * Check if import folder has content (brownfield detection)
 * Detects: PROJECT.md with content, package.json, common project structures
 * Import folder is at project-root/import/ (sibling to app directory)
 */
export function hasBrownfieldContent(appDir?: string): boolean {
  const dir = appDir || process.cwd()
  // If appDir is provided, go up one level to project root
  const importPath = appDir ? resolve(dir, '..', 'import') : resolve(dir, 'import')
  if (!existsSync(importPath)) return false

  // Check for PROJECT.md with filled content
  const projectMd = resolve(importPath, 'PROJECT.md')
  if (existsSync(projectMd)) {
    const content = readFileSync(projectMd, 'utf-8')
    if (content.includes('## Project Overview') && !content.includes('[Your project name]')) {
      return true
    }
  }

  // Check for common project indicators
  const projectIndicators = [
    'package.json',
    'composer.json',
    'requirements.txt',
    'Gemfile',
    'pom.xml',
    'build.gradle',
    'Cargo.toml',
    'go.mod'
  ]

  for (const indicator of projectIndicators) {
    if (existsSync(resolve(importPath, indicator))) {
      return true
    }
  }

  // Check for common source directories
  const sourceDirs = ['src', 'app', 'lib', 'components', 'pages']
  for (const sourceDir of sourceDirs) {
    if (existsSync(resolve(importPath, sourceDir))) {
      return true
    }
  }

  return false
}

/**
 * Detect project framework from import folder
 */
export interface DetectedProject {
  hasContent: boolean
  framework?: 'react' | 'vue' | 'angular' | 'nuxt' | 'next' | 'svelte' | 'express' | 'unknown'
  packageManager?: 'npm' | 'yarn' | 'pnpm'
  typescript: boolean
  fileCount: number
}

export function detectProjectInfo(appDir?: string): DetectedProject {
  const dir = appDir || process.cwd()
  // If appDir is provided, go up one level to project root
  const importPath = appDir ? resolve(dir, '..', 'import') : resolve(dir, 'import')

  const result: DetectedProject = {
    hasContent: false,
    typescript: false,
    fileCount: 0
  }

  if (!existsSync(importPath)) return result

  // Check for package.json
  const packageJsonPath = resolve(importPath, 'package.json')
  if (existsSync(packageJsonPath)) {
    result.hasContent = true
    try {
      const pkg = JSON.parse(readFileSync(packageJsonPath, 'utf-8'))
      const deps = { ...pkg.dependencies, ...pkg.devDependencies }

      // Detect framework
      if (deps.nuxt || deps['nuxt3']) result.framework = 'nuxt'
      else if (deps.next) result.framework = 'next'
      else if (deps.vue) result.framework = 'vue'
      else if (deps.react) result.framework = 'react'
      else if (deps['@angular/core']) result.framework = 'angular'
      else if (deps.svelte) result.framework = 'svelte'
      else if (deps.express) result.framework = 'express'
      else result.framework = 'unknown'

      // Detect TypeScript
      result.typescript = !!deps.typescript

      // Detect package manager from lock files
      if (existsSync(resolve(importPath, 'pnpm-lock.yaml'))) result.packageManager = 'pnpm'
      else if (existsSync(resolve(importPath, 'yarn.lock'))) result.packageManager = 'yarn'
      else if (existsSync(resolve(importPath, 'package-lock.json'))) result.packageManager = 'npm'
    } catch {
      result.framework = 'unknown'
    }
  }

  // Count files (quick scan)
  try {
    result.fileCount = countFilesShallow(importPath)
    if (result.fileCount > 0) result.hasContent = true
  } catch {
    // Ignore errors
  }

  return result
}

/**
 * Count files in directory (shallow, skips node_modules)
 */
function countFilesShallow(dir: string, depth = 0): number {
  if (depth > 3) return 0 // Limit depth for performance

  const { readdirSync, statSync } = require('fs')
  let count = 0

  try {
    const entries = readdirSync(dir, { withFileTypes: true })
    for (const entry of entries) {
      if (entry.name.startsWith('.') || entry.name === 'node_modules') continue
      if (entry.isFile()) count++
      else if (entry.isDirectory()) {
        count += countFilesShallow(resolve(dir, entry.name), depth + 1)
      }
    }
  } catch {
    // Ignore errors
  }

  return count
}

/**
 * Check if database exists
 * Database is at project-root/data/sqlite.db (sibling to app directory)
 */
export function databaseExists(appDir?: string): boolean {
  const dir = appDir || process.cwd()
  // If appDir is provided, go up one level to project root
  // If not provided (cwd from project root), use directly
  if (appDir) {
    return existsSync(resolve(dir, '..', 'data', 'sqlite.db'))
  }
  return existsSync(resolve(dir, 'data', 'sqlite.db'))
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN READER
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Read full config summary
 */
export function readConfigSummary(appDir?: string): PmConfigSummary {
  return {
    pmMode: readPmMode(appDir),
    projectType: readProjectType(appDir),
    adminEnabled: readAdminEnabled(appDir),
    locales: readLocales(appDir),
    defaultLocale: readDefaultLocale(appDir),
    enabledModules: readEnabledModules(appDir),
    features: readFeatures(appDir),
    hasBrownfieldContent: hasBrownfieldContent(appDir),
    databaseExists: databaseExists(appDir)
  }
}

/**
 * Get human-readable status for display
 */
export function getStatusDisplay(appDir?: string): string {
  const summary = readConfigSummary(appDir)

  const lines: string[] = []

  // Mode
  const modeLabels: Record<PmMode, string> = {
    unconfigured: 'Unconfigured (needs setup)',
    build: 'BUILD mode (client project)',
    develop: 'DEVELOP mode (framework development)'
  }
  lines.push(`Mode: ${modeLabels[summary.pmMode]}`)

  // Project type (if build mode)
  if (summary.pmMode === 'build' && summary.projectType) {
    lines.push(`Project Type: ${summary.projectType === 'website' ? 'Website' : 'App'}`)
  }

  // Admin
  lines.push(`Admin Panel: ${summary.adminEnabled ? 'Enabled' : 'Disabled'}`)

  // Locales
  lines.push(`Locales: ${summary.locales.join(', ')} (default: ${summary.defaultLocale})`)

  // Modules
  if (summary.enabledModules.length > 0) {
    lines.push(`Modules: ${summary.enabledModules.join(', ')}`)
  } else {
    lines.push(`Modules: None enabled`)
  }

  // Features
  const activeFeatures = Object.entries(summary.features)
    .filter(([, v]) => v)
    .map(([k]) => k)
  if (activeFeatures.length > 0) {
    lines.push(`Features: ${activeFeatures.join(', ')}`)
  }

  // Database
  lines.push(`Database: ${summary.databaseExists ? 'Exists' : 'Not created'}`)

  // Brownfield
  if (summary.hasBrownfieldContent) {
    lines.push(`Import: Brownfield content detected`)
  }

  return lines.join('\n')
}
