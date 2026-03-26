/**
 * Delete Import Files API
 *
 * DELETE /api/setup/import-zip
 * Returns: { success: true }
 *
 * Clears the ./import/ folder
 * Security: Only accessible when pmMode is 'unconfigured'
 */
import { existsSync, rmSync } from 'fs'
import { resolve } from 'path'
import { requireSetupAccess } from '../../utils/setup-guard'

export default defineEventHandler(async (event) => {
  // Security: Only allow deletion when project is unconfigured
  requireSetupAccess(event)
  const importDir = resolve(process.cwd(), 'import')

  if (existsSync(importDir)) {
    rmSync(importDir, { recursive: true, force: true })
  }

  return {
    success: true,
    data: {}
  }
})
