#!/usr/bin/env node

/**
 * Check Documentation Entrypoints
 *
 * Validates that all npm scripts documented in CONTRIBUTING.md and docs/
 * actually exist in package.json.
 *
 * This is a CI gate to prevent documentation drift.
 *
 * Usage:
 *   node scripts/check-doc-entrypoints.mjs
 *   node scripts/check-doc-entrypoints.mjs --enforce  # Exit with error if mismatch
 *
 * Exit codes:
 *   0 - All documented scripts exist
 *   1 - Mismatch found (only in --enforce mode)
 */

import { readFileSync, existsSync } from 'fs'
import { fileURLToPath } from 'url'
import { dirname, resolve } from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const rootDir = resolve(__dirname, '..')

/**
 * Extract npm commands from a text content
 * Looks for patterns like:
 *   npm run <script>
 *   `npm run <script>`
 */
function extractNpmCommands(content) {
  const commands = new Set()

  // Match: npm run <script-name>
  const npmRunPattern = /npm\s+run\s+([a-zA-Z0-9:_-]+)/g
  let match

  while ((match = npmRunPattern.exec(content)) !== null) {
    commands.add(match[1])
  }

  // Match: `npm run <script-name>` (backtick wrapped)
  const backtickPattern = /`npm\s+run\s+([a-zA-Z0-9:_-]+)`/g
  while ((match = backtickPattern.exec(content)) !== null) {
    commands.add(match[1])
  }

  return commands
}

/**
 * Get all scripts from package.json
 */
function getPackageScripts() {
  const packagePath = resolve(rootDir, 'package.json')

  if (!existsSync(packagePath)) {
    console.error('❌ package.json not found')
    process.exit(1)
  }

  const packageJson = JSON.parse(readFileSync(packagePath, 'utf-8'))
  return new Set(Object.keys(packageJson.scripts || {}))
}

/**
 * Find all documented scripts in CONTRIBUTING.md and docs/
 */
async function findDocumentedScripts() {
  const documentedScripts = new Set()
  const filesToCheck = []

  // Check CONTRIBUTING.md
  const contributingPath = resolve(rootDir, 'CONTRIBUTING.md')
  if (existsSync(contributingPath)) {
    filesToCheck.push(contributingPath)
  }

  // Check docs/ directory recursively
  const docsDir = resolve(rootDir, 'docs')
  if (existsSync(docsDir)) {
    const { readdirSync, statSync } = await import('fs')
    const { join } = await import('path')

    function scanDir(dir) {
      const entries = readdirSync(dir)
      for (const entry of entries) {
        const fullPath = join(dir, entry)
        const stat = statSync(fullPath)

        if (stat.isDirectory() && !entry.startsWith('.')) {
          scanDir(fullPath)
        } else if (stat.isFile() && entry.endsWith('.md')) {
          filesToCheck.push(fullPath)
        }
      }
    }

    scanDir(docsDir)
  }

  // Extract commands from all files
  for (const file of filesToCheck) {
    try {
      const content = readFileSync(file, 'utf-8')
      const commands = extractNpmCommands(content)

      for (const cmd of commands) {
        documentedScripts.add(cmd)
      }
    } catch (error) {
      console.warn(`⚠️  Could not read ${file}: ${error.message}`)
    }
  }

  return documentedScripts
}

/**
 * Main validation
 */
async function checkDocEntrypoints(enforce = false) {
  console.log('🔍 Checking documentation entrypoints...\n')

  const packageScripts = getPackageScripts()
  const documentedScripts = await findDocumentedScripts()

  const missing = []
  const orphaned = []

  // Find documented scripts that don't exist in package.json
  for (const script of documentedScripts) {
    if (!packageScripts.has(script)) {
      missing.push(script)
    }
  }

  // Find package.json scripts that might not be documented (informational only)
  // Exclude internal scripts (starting with _)
  for (const script of packageScripts) {
    if (!script.startsWith('_') && !documentedScripts.has(script)) {
      orphaned.push(script)
    }
  }

  // Report results
  if (missing.length > 0) {
    console.error('❌ Documented scripts missing from package.json:\n')
    for (const script of missing) {
      console.error(`   - npm run ${script}`)
    }
    console.error()
  } else {
    console.log('✅ All documented scripts exist in package.json')
  }

  if (orphaned.length > 0) {
    console.log('\n⚠️  Scripts in package.json that may not be documented:\n')
    for (const script of orphaned) {
      console.log(`   - npm run ${script}`)
    }
    console.log()
  }

  // Summary
  console.log('─────────────────────────────────────')
  console.log(`Package scripts: ${packageScripts.size}`)
  console.log(`Documented scripts: ${documentedScripts.size}`)
  console.log(`Missing: ${missing.length}`)
  console.log(`Undocumented: ${orphaned.length}`)
  console.log('─────────────────────────────────────\n')

  // Exit with error if enforce mode and missing scripts found
  if (enforce && missing.length > 0) {
    console.error('❌ CI Gate Failed: Documented scripts must exist in package.json')
    process.exit(1)
  }

  if (missing.length > 0) {
    process.exit(1)
  }

  console.log('✅ Check passed!\n')
  process.exit(0)
}

// Parse command line arguments
const args = process.argv.slice(2)
const enforce = args.includes('--enforce')
const help = args.includes('--help') || args.includes('-h')

if (help) {
  console.log(`
Check Documentation Entrypoints

Validates that all npm scripts documented in CONTRIBUTING.md and docs/
actually exist in package.json.

Usage:
  node scripts/check-doc-entrypoints.mjs [options]

Options:
  --enforce    Exit with error code if mismatch found (CI mode)
  --help, -h   Show this help message

Exit Codes:
  0  All documented scripts exist
  1  Mismatch found or error occurred
`)
  process.exit(0)
}

checkDocEntrypoints(enforce)
