/**
 * Workflow Path Resolution Utility
 * 
 * Provides functions to resolve paths for different AI workflows (Claude, Qwen, Codex)
 * allowing the system to dynamically switch between different AI agent configurations.
 */

import type { AiWorkflow } from '~/types/config'
import fs from 'fs'
import path from 'path'
import config from '~~/app/puppet-master.config'

function isValidWorkflow(value: unknown): value is AiWorkflow {
  return value === 'claude' || value === 'qwen' || value === 'codex'
}

/**
 * Get the data directory path for the specified workflow
 * @param workflow - The AI workflow ('claude', 'qwen', or 'codex')
 * @returns The path to the workflow's data directory (e.g., '.claude-data', '.qwen-data')
 */
export function getWorkflowDataDir(workflow: AiWorkflow): string {
  switch (workflow) {
    case 'claude':
      return '.claude-data'
    case 'qwen':
      return '.qwen-data'
    case 'codex':
      return '.codex-data'
    default:
      console.warn(`Invalid workflow: ${workflow}. Falling back to 'claude'.`)
      return '.claude-data'
  }
}

/**
 * Get the config directory path for the specified workflow
 * @param workflow - The AI workflow ('claude', 'qwen', or 'codex')
 * @returns The path to the workflow's config directory (e.g., '.claude', '.qwen')
 */
export function getWorkflowConfigDir(workflow: AiWorkflow): string {
  switch (workflow) {
    case 'claude':
      return '.claude'
    case 'qwen':
      return '.qwen'
    case 'codex':
      return '.codex'
    default:
      console.warn(`Invalid workflow: ${workflow}. Falling back to 'claude'.`)
      return '.claude'
  }
}

/**
 * Get the current workflow from the puppet-master configuration
 * @returns The currently selected AI workflow
 */
export function getCurrentWorkflow(): AiWorkflow {
  const workflow = config.aiWorkflow

  if (isValidWorkflow(workflow)) {
    return workflow
  }

  console.warn(`Invalid workflow in config: ${String(workflow)}. Falling back to 'claude'.`)
  return 'claude'
}

/**
 * Validate that the selected workflow directory exists and is properly configured
 * @param workflow - The AI workflow to validate
 * @returns True if the workflow is valid and properly configured, false otherwise
 */
export function validateWorkflow(workflow: AiWorkflow): boolean {
  try {
    const configDir = getWorkflowConfigDir(workflow)
    const configPath = path.join(process.cwd(), configDir, 'config.json')
    
    // Check if config directory exists
    if (!fs.existsSync(path.join(process.cwd(), configDir))) {
      console.warn(`Workflow config directory does not exist: ${configDir}`)
      return false
    }
    
    // Check if config.json exists in the workflow directory
    if (!fs.existsSync(configPath)) {
      console.warn(`Workflow config file does not exist: ${configPath}`)
      return false
    }
    
    // Check if commands directory exists
    const commandsDir = path.join(process.cwd(), configDir, 'commands')
    if (!fs.existsSync(commandsDir)) {
      console.warn(`Workflow commands directory does not exist: ${commandsDir}`)
      return false
    }
    
    return true
  } catch (error) {
    console.error(`Error validating workflow ${workflow}:`, error)
    return false
  }
}

/**
 * Get the current workflow from the puppet-master configuration with validation
 * @returns The currently selected AI workflow
 * @throws Error if config file is missing or workflow is invalid
 */
export function getCurrentWorkflowWithErrorHandling(): AiWorkflow {
  const workflow = getCurrentWorkflow()

  if (!validateWorkflow(workflow)) {
    console.warn(`Workflow directory for '${workflow}' is not properly configured. Using 'claude' as fallback.`)
    return 'claude'
  }

  return workflow
}

/**
 * Validate the puppet-master.config.ts file for required workflow fields
 * @returns True if config is valid, false otherwise
 */
export function validateConfigForWorkflows(): boolean {
  try {
    const configPath = path.join(process.cwd(), 'app', 'puppet-master.config.ts')
    if (!fs.existsSync(configPath)) {
      console.error('puppet-master.config.ts not found at expected location')
      return false
    }
    
    const configContent = fs.readFileSync(configPath, 'utf-8')
    
    // Check if aiWorkflow field exists
    if (!/aiWorkflow:\s*['"]\w+['"]/.test(configContent)) {
      console.warn('aiWorkflow field not found in puppet-master.config.ts. This may cause issues with workflow selection.')
      return false
    }
    
    return true
  } catch (error) {
    console.error('Error validating config for workflows:', error)
    return false
  }
}

/**
 * Create the workflow data directory if it doesn't exist
 * @param workflow - The AI workflow for which to create the data directory
 * @returns True if the directory exists or was successfully created, false otherwise
 */
export function ensureWorkflowDataDir(workflow: AiWorkflow): boolean {
  try {
    const dataDir = getWorkflowDataDir(workflow)
    const fullPath = path.join(process.cwd(), dataDir)
    
    if (!fs.existsSync(fullPath)) {
      fs.mkdirSync(fullPath, { recursive: true })
      console.log(`Created workflow data directory: ${fullPath}`)
    }
    
    return true
  } catch (error) {
    console.error(`Error creating workflow data directory for ${workflow}:`, error)
    return false
  }
}

/**
 * Get the path to the project brief file for the current workflow
 * @returns The path to the project brief file
 */
export function getProjectBriefPath(): string {
  const workflow = getCurrentWorkflow()
  const dataDir = getWorkflowDataDir(workflow)
  return path.join(dataDir, 'project-brief.md')
}

/**
 * Get the path to the context file for the current workflow
 * @returns The path to the context file
 */
export function getContextPath(): string {
  const workflow = getCurrentWorkflow()
  const dataDir = getWorkflowDataDir(workflow)
  return path.join(dataDir, 'context.md')
}
