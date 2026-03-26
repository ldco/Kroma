/**
 * Admin Translations API - DELETE
 *
 * Delete a translation by ID.
 * - Content translations: deletable by admin+
 * - System translations: deletable by master only
 * Requires authentication.
 */
import { useDatabase, schema } from '../../../database/client'
import { eq } from 'drizzle-orm'
import { isSystemKey } from '../../../../i18n/system'
import { translationCache } from '../../../utils/translationCache'
import { syncSystemTranslationsToFile } from '../../../utils/systemTranslationSync'
import { hasRole } from '../../../utils/roles'
import type { UserRole } from '../../../database/schema'

export default defineEventHandler(async event => {
  // Auth is handled by middleware - check user context
  const user = event.context.user
  if (!user) {
    throw createError({ statusCode: 401, message: 'Unauthorized' })
  }

  const id = Number(getRouterParam(event, 'id'))

  if (!id || isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const db = useDatabase()

  // Check if translation exists
  const existing = db.select().from(schema.translations).where(eq(schema.translations.id, id)).get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Translation not found' })
  }

  // Check permissions for system translations
  if (isSystemKey(existing.key)) {
    if (!hasRole(user.role as UserRole, 'master')) {
      throw createError({
        statusCode: 403,
        message: 'Only master users can delete system translations'
      })
    }
  }

  const isSystem = isSystemKey(existing.key)

  db.delete(schema.translations).where(eq(schema.translations.id, id)).run()

  // Invalidate cache for this locale
  translationCache.invalidate(existing.locale)

  // Sync system translations to JSON file for new deployments
  if (isSystem) {
    syncSystemTranslationsToFile()
  }

  return {
    success: true,
    data: {}
  }
})
