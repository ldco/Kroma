#!/usr/bin/env node

/**
 * Workflow Information Utility
 * 
 * Shows the current AI workflow and related information
 */

import fs from 'fs'
import path from 'path'

// Define valid workflows
type AiWorkflow = 'claude' | 'qwen' | 'codex'

// Get project root (repository root)
const projectRoot = path.resolve(process.cwd())
const configPath = path.join(projectRoot, 'app', 'puppet-master.config.ts')

console.log('🔍 Puppet Master Workflow Information\n')

try {
  // Read config file
  if (!fs.existsSync(configPath)) {
    throw new Error(`Config file not found: ${configPath}`)
  }

  const configContent = fs.readFileSync(configPath, 'utf-8')

  // Extract aiWorkflow value
  const aiWorkflowRegex = /aiWorkflow:\s*['"](\w+)['"]/
  const aiWorkflowMatch = configContent.match(aiWorkflowRegex)
  const currentWorkflow: AiWorkflow | undefined = aiWorkflowMatch ? aiWorkflowMatch[1] as AiWorkflow : undefined

  // Extract pmMode value
  const pmModeRegex = /pmMode:\s*['"](\w+)['"]/
  const pmModeMatch = configContent.match(pmModeRegex)
  const pmMode = pmModeMatch ? pmModeMatch[1] : 'unknown'

  if (!currentWorkflow) {
    console.log('⚠️  No aiWorkflow found in config. Defaulting to "claude".')
    console.log('   Run "npm run workflow:switch <workflow>" to set a workflow.\n')
  } else {
    console.log(`Current Workflow: ${currentWorkflow.toUpperCase()}`)
    console.log(`Project Mode: ${pmMode}\n`)
  }

  // Show workflow details
  const workflowDetails: Record<AiWorkflow, string> = {
    claude: 'Claude Code - Basic PM commands and workflows',
    qwen: 'Qwen Code - Advanced with 42 expert personas (7 specialties × 6 countries)',
    codex: 'Codex - Experimental workflow (coming soon)'
  }

  if (currentWorkflow) {
    console.log(`Workflow Details: ${workflowDetails[currentWorkflow]}\n`)
  }

  // Show available commands for each workflow
  console.log('Available Commands by Workflow:')
  console.log('├─ CLAUDE: Basic commands (pm-init, pm-dev, pm-status, pm-migrate)')
  console.log('├─ QWEN: Full command set (42 expert personas, country/specialty commands)')
  console.log('└─ CODEX: Experimental (will be expanded in future)\n')

  // Show directory structure
  console.log('Workflow Directories:')
  const workflows: AiWorkflow[] = ['claude', 'qwen', 'codex']
  for (const wf of workflows) {
    const configDir = `.${wf}`
    const dataDir = `.${wf}-data`
    const configDirPath = path.join(projectRoot, configDir)
    const dataDirPath = path.join(projectRoot, dataDir)
    
    const configExists = fs.existsSync(configDirPath)
    const dataExists = fs.existsSync(dataDirPath)
    
    console.log(`├─ ${wf.toUpperCase()}:`)
    console.log(`│  ├─ Config: ./${configDir} ${configExists ? '✅' : '❌'}`)
    console.log(`│  └─ Data: ./${dataDir} ${dataExists ? '✅' : '❌'}`)
  }

  console.log('\n💡 Tip: Use "npm run workflow:switch <workflow>" to change workflows.')
  console.log('💡 Tip: Workflow changes affect which AI commands and expert personas are available.')

} catch (error) {
  console.error('❌ Error getting workflow information:', error instanceof Error ? error.message : String(error))
  process.exit(1)
}
