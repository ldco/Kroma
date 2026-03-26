#!/usr/bin/env node

/**
 * Workflow Switching Utility
 * 
 * Allows users to switch between AI workflows (Claude, Qwen, Codex)
 * Updates the aiWorkflow field in puppet-master.config.ts and ensures proper directory structure
 */

import fs from 'fs'
import path from 'path'

// Define valid workflows
type AiWorkflow = 'claude' | 'qwen' | 'codex'

// Get workflow from command line argument
const args = process.argv.slice(2)
const newWorkflow: AiWorkflow | undefined = args[0] as AiWorkflow

if (!newWorkflow || !isValidWorkflow(newWorkflow)) {
  console.error('Usage: node scripts/switch-workflow.js <claude|qwen|codex>')
  console.error('Current workflows: claude, qwen, codex')
  process.exit(1)
}

// Validate workflow
function isValidWorkflow(workflow: string): workflow is AiWorkflow {
  return ['claude', 'qwen', 'codex'].includes(workflow)
}

// Get project root (repository root)
const projectRoot = path.resolve(process.cwd())
const configPath = path.join(projectRoot, 'app', 'puppet-master.config.ts')

console.log(`Switching to workflow: ${newWorkflow}`)
console.log(`Project root: ${projectRoot}`)

try {
  // Read current config file
  if (!fs.existsSync(configPath)) {
    throw new Error(`Config file not found: ${configPath}`)
  }

  let configContent = fs.readFileSync(configPath, 'utf-8')

  // Update aiWorkflow field
  // Look for the aiWorkflow line and update it
  const aiWorkflowRegex = /(aiWorkflow:\s*['"])(\w+)(['"]\s*as\s*['"]\w+\s*['"]\s*\|\s*['"]\w+\s*['"]\s*\|\s*['"]\w+\s*['"])/
  const match = configContent.match(aiWorkflowRegex)

  if (match) {
    // Replace with new workflow
    const newLine = `${match[1]}${newWorkflow}${match[3]}`
    configContent = configContent.replace(aiWorkflowRegex, newLine)
    console.log(`Updated aiWorkflow to: ${newWorkflow}`)
  } else {
    // If aiWorkflow doesn't exist, add it after pmMode
    const pmModeRegex = /(pmMode:\s*['"]\w+['"]\s*as\s*['"][^'"]+['"])/
    if (pmModeRegex.test(configContent)) {
      const replacement = `$1,\n  aiWorkflow: '${newWorkflow}' as 'claude' | 'qwen' | 'codex'`
      configContent = configContent.replace(pmModeRegex, replacement)
      console.log(`Added aiWorkflow: ${newWorkflow}`)
    } else {
      throw new Error('Could not find pmMode field to insert aiWorkflow after')
    }
  }

  // Write updated config back
  fs.writeFileSync(configPath, configContent, 'utf-8')
  console.log(`Configuration updated successfully`)

  // Create the workflow data directory if it doesn't exist
  const workflowDataDirs: Record<AiWorkflow, string> = {
    claude: '.claude-data',
    qwen: '.qwen-data',
    codex: '.codex-data'
  }
  
  const dataDir = path.join(projectRoot, workflowDataDirs[newWorkflow])
  if (!fs.existsSync(dataDir)) {
    fs.mkdirSync(dataDir, { recursive: true })
    console.log(`Created workflow data directory: ${dataDir}`)
  }

  // Create the workflow config directory if it doesn't exist
  const workflowConfigDirs: Record<AiWorkflow, string> = {
    claude: '.claude',
    qwen: '.qwen',
    codex: '.codex'
  }
  
  const configDir = path.join(projectRoot, workflowConfigDirs[newWorkflow])
  if (!fs.existsSync(configDir)) {
    fs.mkdirSync(configDir, { recursive: true })
    console.log(`Created workflow config directory: ${configDir}`)
    
    // Create a basic config.json for the workflow
    const configJsonPath = path.join(configDir, 'config.json')
    if (!fs.existsSync(configJsonPath)) {
      const configJson = {
        project: "Puppet Master",
        version: "1.0.0",
        stack: {
          frontend: "Nuxt 4 (Vue 3.5, TypeScript)",
          backend: "Nitro (H3)",
          database: "SQLite (Drizzle ORM)",
          mobile: null
        },
        roles: ["fullstack-nuxt", "frontend", "backend", "security", "ux-ui", "devops"],
        defaultRole: "fullstack-nuxt",
        team: ["fullstack-nuxt", "frontend", "backend", "security", "ux-ui", "devops"],
        rag: {
          enabled: false,
          indexed_at: null,
          files_count: null,
          exclude: [
            "node_modules",
            ".nuxt",
            ".output",
            "dist",
            ".git",
            "*.min.js",
            "*.min.css",
            "*.map",
            "package-lock.json",
            "pnpm-lock.yaml",
            "data/sqlite.db"
          ]
        },
        globalLibrary: `~/${workflowConfigDirs[newWorkflow]}`
      }
      
      fs.writeFileSync(configJsonPath, JSON.stringify(configJson, null, 2), 'utf-8')
      console.log(`Created basic config.json for ${newWorkflow} workflow`)
    }
  }

  // Create commands directory if it doesn't exist
  const commandsDir = path.join(configDir, 'commands')
  if (!fs.existsSync(commandsDir)) {
    fs.mkdirSync(commandsDir, { recursive: true })
    console.log(`Created commands directory: ${commandsDir}`)
  }

  // Create roles directory if it doesn't exist
  const rolesDir = path.join(configDir, 'roles', 'pm')
  if (!fs.existsSync(rolesDir)) {
    fs.mkdirSync(rolesDir, { recursive: true })
    console.log(`Created roles directory: ${rolesDir}`)
  }

  console.log('\n✅ Workflow switched successfully!')
  console.log(`Current workflow: ${newWorkflow}`)
  console.log('\n💡 Next steps:')
  console.log('- Restart your development server to apply changes')
  console.log('- The new workflow will be used for AI assistance')
  console.log('- Your project configuration remains unchanged')

} catch (error) {
  console.error('❌ Error switching workflow:', error instanceof Error ? error.message : String(error))
  process.exit(1)
}
