/**
 * GET /api/settings
 *
 * Returns all site settings grouped by category.
 * Public endpoint - returns only non-sensitive settings.
 */
import { buildGroupedSettings, buildSettingsMap } from '../../utils/site-settings'

export default defineEventHandler(async () => {
  const settingsMap = buildSettingsMap()
  const grouped = buildGroupedSettings(settingsMap, {
    includeSecrets: false
  })

  return {
    success: true,
    data: grouped
  }
})
