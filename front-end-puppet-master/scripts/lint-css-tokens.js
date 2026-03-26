#!/usr/bin/env node

/**
 * CSS Token Linter
 *
 * Validates CSS files for:
 * - Undefined CSS custom properties (variables)
 * - Deprecated token usage (with warnings)
 * - Hardcoded colors and spacing (errors)
 *
 * Usage:
 *   node scripts/lint-css-tokens.js
 *   node scripts/lint-css-tokens.js --fix (future: auto-fix deprecated tokens)
 *
 * Exit codes:
 *   0 - Success (no errors)
 *   1 - Errors found (undefined variables, hardcoded values)
 *   2 - Warning only (deprecated tokens)
 */

import { readFileSync, readdirSync, existsSync } from 'fs'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const ROOT_DIR = resolve(__dirname, '..')
const CSS_DIR = resolve(ROOT_DIR, 'app', 'assets', 'css')

// ═══════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Defined CSS custom properties (organized by file)
 * These are the canonical tokens that should be used
 */
const DEFINED_TOKENS = new Set([
  // Primitives (colors)
  '--p-brand', '--p-accent', '--p-black', '--p-white',
  
  // Layout colors
  '--l-bg', '--l-bg-elevated', '--l-bg-sunken',
  '--l-border', '--l-border-strong',
  
  // Text colors
  '--t-primary', '--t-secondary', '--t-muted',
  '--t-on-brand', '--t-on-accent',
  
  // Interactive colors
  '--i-brand', '--i-brand-hover', '--i-brand-active',
  '--i-brand-subtle', '--i-brand-text',
  '--i-accent', '--i-accent-hover', '--i-accent-active',
  '--i-focus-ring',
  '--i-error', '--i-error-hover', '--i-error-active',
  '--t-on-error',
  
  // Semantic/data colors
  '--d-success', '--d-success-bg',
  '--d-warning', '--d-warning-bg',
  '--d-error', '--d-error-bg',
  '--d-info', '--d-info-bg',
  
  // Legacy aliases (deprecated but still valid)
  '--c-brand', '--c-brand-hover', '--c-brand-active',
  '--c-brand-alpha', '--c-brand-text',
  '--l-text', '--l-text-secondary', '--l-text-muted',
  '--border', '--border-strong',
  
  // Spacing
  '--space-1', '--space-2', '--space-3', '--space-4',
  '--space-5', '--space-6', '--space-8', '--space-10',
  '--space-12', '--space-16', '--space-20', '--space-24',
  '--gap-xs', '--gap-sm', '--gap-md', '--gap-lg', '--gap-xl',
  
  // Typography
  '--text-xs', '--text-sm', '--text-base', '--text-lg',
  '--text-xl', '--text-2xl', '--text-3xl', '--text-4xl',
  '--text-5xl', '--text-6xl',
  '--font-light', '--font-normal', '--font-medium',
  '--font-semibold', '--font-bold', '--font-black',
  '--leading-none', '--leading-tight', '--leading-snug',
  '--leading-normal', '--leading-relaxed', '--leading-loose',
  
  // Border radius
  '--radius-sm', '--radius-md', '--radius-lg',
  '--radius-xl', '--radius-2xl', '--radius-full',
  
  // Shadows
  '--shadow-sm', '--shadow-md', '--shadow-lg',
  '--shadow-xl', '--shadow-2xl',
  
  // Transitions
  '--transition-fast', '--transition-normal',
  '--transition-slow', '--transition-slower',
  
  // Custom media queries (used in @media)
  '--phone', '--tablet', '--tablet-only', '--desktop',
  
  // Scrollytelling runtime variables
  '--pm-scroll-progress', '--pm-scene-progress',
  '--pm-parallax-x', '--pm-parallax-y',
  
  // Other runtime variables
  '--header-height', '--content-default',
  
  // Fixed colors (gray scale, syntax highlighting)
  '--c-gray-50', '--c-gray-100', '--c-gray-200',
  '--c-gray-300', '--c-gray-400', '--c-gray-500',
  '--c-gray-600', '--c-gray-700', '--c-gray-800',
  '--c-gray-900', '--c-gray-950',
  '--c-purple-400', '--c-green-400', '--c-blue-400',
  '--c-yellow-400', '--c-red-400'
])

/**
 * Deprecated tokens (should be migrated)
 */
const DEPRECATED_TOKENS = new Map([
  ['--l-text', '--t-primary'],
  ['--l-text-secondary', '--t-secondary'],
  ['--l-text-muted', '--t-muted'],
  ['--c-brand', '--i-brand'],
  ['--c-brand-hover', '--i-brand-hover'],
  ['--c-brand-active', '--i-brand-active'],
  ['--c-brand-alpha', '--i-brand-subtle']
])

/**
 * Patterns that indicate hardcoded values (errors)
 */
const HARDCODED_PATTERNS = [
  {
    pattern: /#[0-9a-fA-F]{3,8}(?![0-9a-fA-F])/g,
    message: 'Hardcoded hex color',
    severity: 'error'
  },
  {
    pattern: /rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)/g,
    message: 'Hardcoded RGB color',
    severity: 'error'
  },
  {
    pattern: /rgba\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*[\d.]+\s*\)/g,
    message: 'Hardcoded RGBA color',
    severity: 'error'
  },
  {
    pattern: /hsl\(\s*\d+\s*,\s*[\d.]+%\s*,\s*[\d.]+%\s*\)/g,
    message: 'Hardcoded HSL color',
    severity: 'error'
  },
  {
    pattern: /:\s*(\d+\.?\d*)px(?!\s*\/)/g,
    message: 'Hardcoded pixel value (use spacing tokens)',
    severity: 'warning',
    allowlist: ['border-radius', 'border-width', 'outline-width', 'box-shadow', 'translate', 'min-height', 'max-height', 'width', 'height', 'font-size', 'line-height']
  }
]

// ═══════════════════════════════════════════════════════════════════════════
// UTILITIES
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Recursively get all CSS files in directory
 */
function getCssFiles(dir, files = []) {
  if (!existsSync(dir)) {
    return files
  }

  const entries = readdirSync(dir, { withFileTypes: true })

  for (const entry of entries) {
    const fullPath = resolve(dir, entry.name)

    if (entry.isDirectory()) {
      getCssFiles(fullPath, files)
    } else if (entry.isFile() && entry.name.endsWith('.css')) {
      files.push(fullPath)
    }
  }

  return files
}

/**
 * Extract all CSS variable usages from content
 */
function extractVariableUsages(content) {
  const usages = []
  const varPattern = /var\(\s*(--[\w-]+)\s*(?:,.*)?\)/g
  let match

  while ((match = varPattern.exec(content)) !== null) {
    usages.push({
      variable: match[1],
      position: match.index,
      line: content.substring(0, match.index).split('\n').length
    })
  }

  return usages
}

/**
 * Extract all CSS variable definitions from content
 */
function extractVariableDefinitions(content) {
  const definitions = new Set()
  const defPattern = /(--[\w-]+)\s*:/g
  let match

  while ((match = defPattern.exec(content)) !== null) {
    definitions.add(match[1])
  }

  return definitions
}

/**
 * Check for hardcoded patterns
 */
function checkHardcodedPatterns(content, filePath) {
  const issues = []

  for (const { pattern, message, severity, allowlist } of HARDCODED_PATTERNS) {
    let match
    while ((match = pattern.exec(content)) !== null) {
      const line = content.substring(0, match.index).split('\n').length
      const lineContent = content.split('\n')[line - 1] || ''

      // Check if property is in allowlist
      if (allowlist) {
        const propMatch = lineContent.match(/([\w-]+)\s*:/)
        if (propMatch && allowlist.includes(propMatch[1])) {
          continue
        }
      }

      issues.push({
        file: filePath,
        line,
        message: `${message}: ${match[0]}`,
        severity,
        content: lineContent.trim()
      })
    }
  }

  return issues
}

/**
 * Check for self-referential CSS variable assignments
 * Example: --l-bg: var(--l-bg); creates an invalid cycle
 */
function checkSelfReferential(content, filePath) {
  const issues = []
  
  // Match CSS custom property definitions that reference themselves
  const selfRefPattern = /(--[\w-]+)\s*:\s*var\(\s*\1\s*\)/g
  let match
  
  while ((match = selfRefPattern.exec(content)) !== null) {
    const line = content.substring(0, match.index).split('\n').length
    const lineContent = content.split('\n')[line - 1]?.trim() || ''
    
    issues.push({
      file: filePath,
      line,
      message: `Self-referential CSS variable: ${match[1]} references itself`,
      severity: 'error',
      content: lineContent
    })
  }
  
  return issues
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN LINTER
// ═══════════════════════════════════════════════════════════════════════════

function lintCssFiles() {
  const cssFiles = getCssFiles(CSS_DIR)
  const errors = []
  const warnings = []
  const definedInFiles = new Set()

  console.log('🔍 Linting CSS files...\n')

  // Hard failure: No CSS files found (invalid path)
  if (cssFiles.length === 0) {
    console.error(`❌ ERROR: No CSS files found in ${CSS_DIR}`)
    console.error('')
    console.error('The CSS_DIR path is incorrect or the directory is empty.')
    console.error('Please verify the CSS directory exists and contains .css files.')
    console.error('')
    console.error(`Expected path: ${CSS_DIR}`)
    console.error(`Root directory: ${ROOT_DIR}`)
    console.error('')
    return 1
  }

  // First pass: collect all definitions
  for (const file of cssFiles) {
    const content = readFileSync(file, 'utf-8')
    const definitions = extractVariableDefinitions(content)

    for (const def of definitions) {
      definedInFiles.add(def)
    }
  }

  // Combine with predefined tokens
  const allDefinedTokens = new Set([...DEFINED_TOKENS, ...definedInFiles])

  // Second pass: check usages and patterns
  for (const file of cssFiles) {
    const relativePath = file.replace(ROOT_DIR + '/', '')
    const content = readFileSync(file, 'utf-8')

    // Check variable usages
    const usages = extractVariableUsages(content)
    for (const { variable, line } of usages) {
      if (!allDefinedTokens.has(variable)) {
        errors.push({
          file: relativePath,
          line,
          message: `Undefined CSS variable: ${variable}`,
          severity: 'error',
          content: content.split('\n')[line - 1]?.trim() || ''
        })
      } else if (DEPRECATED_TOKENS.has(variable)) {
        warnings.push({
          file: relativePath,
          line,
          message: `Deprecated token: ${variable} (use ${DEPRECATED_TOKENS.get(variable)} instead)`,
          severity: 'warning',
          content: content.split('\n')[line - 1]?.trim() || ''
        })
      }
    }

    // Check for self-referential assignments (all files)
    const selfRefIssues = checkSelfReferential(content, relativePath)
    for (const issue of selfRefIssues) {
      errors.push(issue)
    }

    // Skip hardcoded color checks for token definition files
    // These files intentionally define hex colors as primitives
    const isTokenDefinitionFile =
      relativePath.includes('colors/primitives.css') ||
      relativePath.includes('colors/auto.css') ||
      relativePath.includes('colors/index.css')

    if (!isTokenDefinitionFile) {
      // Check hardcoded patterns (only in component/layout CSS)
      const hardcodedIssues = checkHardcodedPatterns(content, relativePath)
      for (const issue of hardcodedIssues) {
        if (issue.severity === 'error') {
          errors.push(issue)
        } else {
          warnings.push(issue)
        }
      }
    }
  }

  // Report results
  console.log(`📁 Scanned ${cssFiles.length} CSS files\n`)

  if (errors.length > 0) {
    console.log('❌ ERRORS:\n')
    for (const error of errors) {
      console.log(`  ${error.file}:${error.line}`)
      console.log(`    ${error.message}`)
      console.log(`    ${error.content}`)
      console.log()
    }
  }

  if (warnings.length > 0) {
    console.log('⚠️  WARNINGS:\n')
    for (const warning of warnings) {
      console.log(`  ${warning.file}:${warning.line}`)
      console.log(`    ${warning.message}`)
      console.log(`    ${warning.content}`)
      console.log()
    }
  }

  // Summary
  console.log('═══════════════════════════════════════════════════════════════')
  if (errors.length === 0 && warnings.length === 0) {
    console.log('✅ All CSS tokens are valid!')
    return 0
  }

  if (errors.length > 0) {
    console.log(`❌ ${errors.length} error(s) found`)
  }

  if (warnings.length > 0) {
    console.log(`⚠️  ${warnings.length} warning(s) found`)
  }

  console.log('═══════════════════════════════════════════════════════════════\n')

  // Exit with error code if errors found
  if (errors.length > 0) {
    console.log('💡 Tip: Define new tokens in app/assets/css/colors/auto.css')
    console.log('   or migrate to existing tokens from the token system.\n')
    return 1
  }

  // Warnings only
  return 2
}

// Run linter
const exitCode = lintCssFiles()
process.exit(exitCode)
