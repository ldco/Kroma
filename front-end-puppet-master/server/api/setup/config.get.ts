/**
 * Get Setup Config API
 *
 * GET /api/setup/config
 * Returns: Current configuration summary for wizard
 *
 * Security: Only accessible when pmMode is 'unconfigured'
 */
import { readFileSync, existsSync, readdirSync, statSync } from 'fs'
import { resolve, join } from 'path'
import { requireSetupAccess } from '../../utils/setup-guard'
import { getWorkflowDataDir, getCurrentWorkflow } from '../../utils/workflow-paths'

interface ConfigSummary {
  pmMode: 'unconfigured' | 'build' | 'develop'
  aiWorkflow: 'claude' | 'qwen' | 'codex'  // NEW FIELD: Current AI workflow
  projectType: 'website' | 'app' | null
  adminEnabled: boolean
  locales: Array<{ code: string; iso: string; name: string }>
  defaultLocale: string
  enabledModules: string[]
  features: {
    multiLangs: boolean
    doubleTheme: boolean
    onepager: boolean
    pwa: boolean
  }
  hasBrownfieldContent: boolean
  importFolderFiles: string[]
  databaseExists: boolean
  // Project info (for Claude context)
  projectName?: string
  projectDescription?: string
  targetAudience?: string
  technicalBrief?: string
  customModules?: string
}

export default defineEventHandler(async (event): Promise<ConfigSummary> => {
  // Security: Allow read-only access to check current state
  // This lets the wizard display current config even if already configured
  requireSetupAccess(event, { allowReadOnly: true })

  // Path to config file (relative to project root)
  const configPath = resolve(process.cwd(), 'app', 'puppet-master.config.ts')

  if (!existsSync(configPath)) {
    throw createError({
      statusCode: 500,
      message: 'Config file not found'
    })
  }

  const content = readFileSync(configPath, 'utf-8')

  // Parse pmMode
  const pmModeMatch = content.match(/pmMode:\s*['"](\w+)['"]/)
  const pmMode = (pmModeMatch?.[1] as ConfigSummary['pmMode']) || 'unconfigured'

  // Parse entities
  const websiteMatch = content.match(/website:\s*(true|false)/)
  const appMatch = content.match(/app:\s*(true|false)/)
  const hasWebsite = websiteMatch?.[1] === 'true'
  const hasApp = appMatch?.[1] === 'true'

  let projectType: ConfigSummary['projectType'] = null
  if (hasWebsite && !hasApp) projectType = 'website'
  else if (hasApp && !hasWebsite) projectType = 'app'
  else if (hasWebsite) projectType = 'website'

  // Parse admin.enabled
  const adminMatch = content.match(/admin:\s*\{[\s\S]*?enabled:\s*(true|false)/)
  const adminEnabled = adminMatch?.[1] === 'true'

  // Parse locales
  const localesMatch = content.match(/locales:\s*\[([\s\S]*?)\]/)
  const locales: ConfigSummary['locales'] = []
  if (localesMatch) {
    const localePattern = /\{\s*code:\s*['"](\w+)['"]\s*,\s*iso:\s*['"]([^'"]+)['"]\s*,\s*name:\s*['"]([^'"]+)['"]\s*\}/g
    let match
    while ((match = localePattern.exec(localesMatch[1])) !== null) {
      locales.push({ code: match[1], iso: match[2], name: match[3] })
    }
  }
  if (locales.length === 0) {
    locales.push({ code: 'en', iso: 'en-US', name: 'English' })
  }

  // Parse defaultLocale
  const defaultLocaleMatch = content.match(/defaultLocale:\s*['"](\w+)['"]/)
  const defaultLocale = defaultLocaleMatch?.[1] || 'en'

  // Parse enabled modules using brace-balanced extraction
  const enabledModules: string[] = []
  const modulesStart = content.indexOf('modules:')
  if (modulesStart !== -1) {
    // Extract complete modules block using brace balancing
    const openBrace = content.indexOf('{', modulesStart)
    if (openBrace !== -1) {
      let depth = 1
      let i = openBrace + 1
      while (i < content.length && depth > 0) {
        if (content[i] === '{') depth++
        else if (content[i] === '}') depth--
        i++
      }
      const modulesBlock = content.slice(modulesStart, i)

      // List of known non-module keys to skip
      const skipKeys = ['config', 'system', 'websiteModules', 'appModules']

      // Parse each module entry with brace balancing
      const moduleEntryPattern = /(\w+):\s*\{/g
      let match
      while ((match = moduleEntryPattern.exec(modulesBlock)) !== null) {
        const moduleName = match[1]
        if (skipKeys.includes(moduleName)) continue

        // Extract this module's config block
        const startIdx = match.index + match[0].length - 1
        let d = 1
        let endIdx = startIdx + 1
        while (endIdx < modulesBlock.length && d > 0) {
          if (modulesBlock[endIdx] === '{') d++
          else if (modulesBlock[endIdx] === '}') d--
          endIdx++
        }
        const moduleConfig = modulesBlock.slice(startIdx, endIdx)

        // Check if enabled: true exists in this module's config
        if (/enabled:\s*true/.test(moduleConfig)) {
          enabledModules.push(moduleName)
        }
      }
    }
  }

  // Parse features
  const features = {
    multiLangs: /multiLangs:\s*true/.test(content),
    doubleTheme: /doubleTheme:\s*true/.test(content),
    onepager: /onepager:\s*true/.test(content),
    pwa: /pwa:\s*true/.test(content)
  }

  // Check for brownfield content
  const importPath = resolve(process.cwd(), 'import')
  let hasBrownfieldContent = false
  const importFolderFiles: string[] = []

  if (existsSync(importPath)) {
    // Scan import folder for files (non-recursive top-level)
    try {
      const entries = readdirSync(importPath)
      for (const entry of entries) {
        const entryPath = join(importPath, entry)
        const stat = statSync(entryPath)
        if (stat.isDirectory()) {
          importFolderFiles.push(`${entry}/`)
        } else if (!entry.startsWith('.')) {
          importFolderFiles.push(entry)
        }
      }
    } catch {
      // Ignore read errors
    }

    // Check for PROJECT.md with filled content
    const projectMd = resolve(importPath, 'PROJECT.md')
    if (existsSync(projectMd)) {
      const projectContent = readFileSync(projectMd, 'utf-8')
      hasBrownfieldContent = projectContent.includes('## Project Overview') && !projectContent.includes('[Your project name]')
    }

    // Also consider folder with any non-template files as brownfield
    if (!hasBrownfieldContent && importFolderFiles.length > 0) {
      // If there are folders or non-template files besides PROJECT.md
      const hasRealContent = importFolderFiles.some(f =>
        f !== 'PROJECT.md' && !f.startsWith('.')
      )
      if (hasRealContent) {
        hasBrownfieldContent = true
      }
    }
  }

  // Check if database exists
  const dbPath = resolve(process.cwd(), 'data', 'sqlite.db')
  const databaseExists = existsSync(dbPath)

  // Read project brief and info from workflow-specific data directory
  let projectName: string | undefined
  let projectDescription: string | undefined
  let targetAudience: string | undefined
  let technicalBrief: string | undefined
  let customModules: string | undefined

  // Get the current workflow and its data directory
  const currentWorkflow = getCurrentWorkflow()
  const workflowDataDir = getWorkflowDataDir(currentWorkflow)
  const briefPath = resolve(process.cwd(), workflowDataDir, 'project-brief.md')
  
  if (existsSync(briefPath)) {
    try {
      const briefContent = readFileSync(briefPath, 'utf-8')

      // Parse project name from "## Project: Name"
      const nameMatch = briefContent.match(/## Project:\s*(.+)/)
      if (nameMatch) projectName = nameMatch[1].trim()

      // Parse description from "**Description:** text"
      const descMatch = briefContent.match(/\*\*Description:\*\*\s*(.+)/)
      if (descMatch) projectDescription = descMatch[1].trim()

      // Parse target audience from "**Target Audience:** text"
      const audienceMatch = briefContent.match(/\*\*Target Audience:\*\*\s*(.+)/)
      if (audienceMatch) targetAudience = audienceMatch[1].trim()

      // Parse custom modules from "**Custom Modules Requested:** text"
      const modulesMatch = briefContent.match(/\*\*Custom Modules Requested:\*\*\s*(.+)/)
      if (modulesMatch) customModules = modulesMatch[1].trim()

      // Extract technical brief (everything after "## Technical Brief")
      const briefSection = briefContent.split('## Technical Brief')
      if (briefSection.length > 1) {
        technicalBrief = briefSection[1].trim()
      }
    } catch {
      // Ignore read errors
    }
  }

  return {
    success: true,
    data: {
      pmMode,
      aiWorkflow: currentWorkflow, // NEW FIELD: Include current AI workflow
      projectType,
      adminEnabled,
      locales,
      defaultLocale,
      enabledModules,
      features,
      hasBrownfieldContent,
      importFolderFiles,
      databaseExists,
      // Project info
      projectName,
      projectDescription,
      targetAudience,
      technicalBrief,
      customModules
    }
  }
})
