/**
 * Admin Translations API - POST
 *
 * Create or update a translation.
 * - Content translations: editable by admin+
 * - System translations: editable by master only
 * Requires authentication.
 */
import { useDatabase, schema } from '../../database/client'
import { eq, and } from 'drizzle-orm'
import { isSystemKey } from '../../../i18n/system'
import { translationCache } from '../../utils/translationCache'
import { syncSystemTranslationsToFile } from '../../utils/systemTranslationSync'
import { hasRole } from '../../utils/roles'
import type { UserRole } from '../../database/schema'

export default defineEventHandler(async event => {
  // Auth is handled by middleware - check user context
  const user = event.context.user
  if (!user) {
    throw createError({ statusCode: 401, message: 'Unauthorized' })
  }

  const body = await readBody(event)
  const { locale, key, value } = body

  if (!locale || !key || value === undefined) {
    throw createError({ statusCode: 400, message: 'locale, key, and value are required' })
  }

  // Check permissions for system translations
  if (isSystemKey(key)) {
    if (!hasRole(user.role as UserRole, 'master')) {
      throw createError({
        statusCode: 403,
        message: 'Only master users can edit system translations'
      })
    }
  }

  const db = useDatabase()

  // Check if translation exists
  const existing = db
    .select()
    .from(schema.translations)
    .where(and(eq(schema.translations.locale, locale), eq(schema.translations.key, key)))
    .limit(1)
    .all()

  const existingRow = existing[0]
  const isSystem = isSystemKey(key)

  if (existingRow) {
    // Update existing
    db.update(schema.translations)
      .set({ value, updatedAt: new Date() })
      .where(eq(schema.translations.id, existingRow.id))
      .run()

    // Invalidate cache for this locale
    translationCache.invalidate(locale)

    // Sync system translations to JSON file for new deployments
    if (isSystem) {
      syncSystemTranslationsToFile()
    }

    return {
      success: true,
      data: {
        action: 'updated',
        id: existingRow.id
      }
    }
  } else {
    // Create new
    const result = db
      .insert(schema.translations)
      .values({ locale, key, value })
      .returning({ id: schema.translations.id })
      .all()

    // Invalidate cache for this locale
    translationCache.invalidate(locale)

    // Sync system translations to JSON file for new deployments
    if (isSystem) {
      syncSystemTranslationsToFile()
    }

    const newRow = result[0]
    return {
      success: true,
      data: {
        action: 'created',
        id: newRow?.id
      }
    }
  }
})
