/**
 * GET /api/admin/settings
 *
 * Returns config-defined settings for the admin form.
 * Secret settings are masked for non-master roles.
 */
import { requireAdmin } from '../../utils/roles'
import { buildGroupedSettings, buildSettingsMap } from '../../utils/site-settings'

export default defineEventHandler(async event => {
  requireAdmin(event.context.user?.role)

  const settingsMap = buildSettingsMap()
  const grouped = buildGroupedSettings(settingsMap, {
    includeSecrets: true,
    viewerRole: event.context.user?.role
  })

  return {
    success: true,
    data: grouped
  }
})
