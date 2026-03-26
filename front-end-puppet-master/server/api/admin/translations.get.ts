/**
 * Admin Translations API - GET
 *
 * Returns translations grouped by locale for admin management.
 * - Content translations: visible to all admins
 * - System translations: visible only to master users
 * Requires authentication.
 */
import { useDatabase, schema } from '../../database/client'
import { asc } from 'drizzle-orm'
import { isSystemKey } from '../../../i18n/system'
import { hasRole } from '../../utils/roles'
import type { UserRole } from '../../database/schema'
import config from '~/puppet-master.config'

export default defineEventHandler(async event => {
  // Auth is handled by middleware for /api/admin/* routes
  const user = event.context.user
  const isMaster = user && hasRole(user.role as UserRole, 'master')

  const db = useDatabase()

  // Fetch all translations ordered by locale and key
  const rows = db
    .select()
    .from(schema.translations)
    .orderBy(asc(schema.translations.locale), asc(schema.translations.key))
    .all()

  // Group by locale and type (system vs content)
  const systemTranslations: Record<string, Array<{ id: number; key: string; value: string }>> = {}
  const contentTranslations: Record<string, Array<{ id: number; key: string; value: string }>> = {}

  for (const row of rows) {
    const isSystem = isSystemKey(row.key)
    const target = isSystem ? systemTranslations : contentTranslations

    const localeGroup = (target[row.locale] ??= [])
    localeGroup.push({
      id: row.id,
      key: row.key,
      value: row.value ?? ''
    })
  }

  // Get locale codes from config
  const locales = config.locales.map(l => l.code)

  return {
    success: true,
    data: {
      locales,
      localeNames: Object.fromEntries(config.locales.map(l => [l.code, l.name])),
      content: contentTranslations,
      // Only include system translations for master users
      system: isMaster ? systemTranslations : null,
      canEditSystem: isMaster
    }
  }
})
