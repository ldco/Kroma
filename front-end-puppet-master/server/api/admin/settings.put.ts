/**
 * PUT /api/admin/settings
 *
 * Updates site settings. Requires admin authentication.
 * Accepts an object of key-value pairs to update.
 *
 * Body: { "site.name": "New Name", "contact.email": "new@email.com" }
 *
 * SECURITY: Only whitelisted setting keys are allowed to prevent arbitrary data injection.
 */
import { eq } from 'drizzle-orm'
import { z } from 'zod'
import { schema, transactionSync } from '../../database/client'
import { audit } from '../../utils/audit'
import { requireAdmin } from '../../utils/roles'
import config from '~~/app/puppet-master.config'
import {
  SECRET_MASK_VALUE,
  getSettingDefinition,
  isSecretSettingDefinition,
  normalizeSettingValueForStorage
} from '../../utils/site-settings'
import { decryptSecretValue, isEncryptedSecretValue } from '../../utils/secrets'

/**
 * Whitelist of allowed setting keys, sourced from config.
 * This keeps API validation aligned with the Admin Settings UI schema.
 */
const ALLOWED_SETTINGS_KEYS = new Set(config.settings.map(setting => setting.key))

// Validation schema for settings update
const updateSchema = z.record(z.string(), z.string().nullable())

export default defineEventHandler(async event => {
  requireAdmin(event.context.user?.role)

  // Parse and validate request body
  const body = await readBody(event)
  const result = updateSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid request body',
      data: result.error.flatten()
    })
  }

  const updates = result.data

  // SECURITY: Validate all keys against whitelist
  const invalidKeys = Object.keys(updates).filter(key => !ALLOWED_SETTINGS_KEYS.has(key))
  if (invalidKeys.length > 0) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid settings keys',
      data: { invalidKeys, message: `Unknown setting keys: ${invalidKeys.join(', ')}` }
    })
  }

  // Update each setting atomically in a transaction
  const changes = transactionSync(db => {
    const updated: string[] = []
    const created: string[] = []
    const skipped: string[] = []
    const secretPolicy: Record<string, { configured: boolean }> = {}

    for (const [key, value] of Object.entries(updates)) {
      const definition = getSettingDefinition(key)
      if (!definition) {
        continue
      }

      const isSecret = isSecretSettingDefinition(definition)
      const prepared = normalizeSettingValueForStorage(key, value)

      // Check if setting exists
      const existing = db.select().from(schema.settings).where(eq(schema.settings.key, key)).get()

      if (isSecret) {
        if (prepared.value === SECRET_MASK_VALUE || prepared.value === '') {
          skipped.push(key)
          continue
        }
      }

      if (existing && isSecret && prepared.value !== null) {
        const existingPlain = existing.value
          ? isEncryptedSecretValue(existing.value)
            ? decryptSecretValue(existing.value)
            : existing.value
          : null
        const incomingPlain = value?.trim() || null

        if (existingPlain === incomingPlain) {
          skipped.push(key)
          continue
        }
      }

      if (existing && !isSecret && existing.value === prepared.value) {
        skipped.push(key)
        continue
      }

      if (existing) {
        // Update existing setting
        db.update(schema.settings)
          .set({
            value: prepared.value,
            updatedAt: new Date()
          })
          .where(eq(schema.settings.key, key))
          .run()
        updated.push(key)
      } else {
        // Create new setting
        db.insert(schema.settings)
          .values({
            key,
            value: prepared.value,
            type: 'string',
            group: definition.group
          })
          .run()
        created.push(key)
      }

      if (isSecret) {
        const configured = (value?.trim().length || 0) > 0
        secretPolicy[key] = { configured }
      }
    }

    return { updated, created, skipped, secretPolicy }
  })

  if (event.context.user?.id) {
    await audit.settingsUpdate(event, event.context.user.id, {
      updatedKeys: changes.updated,
      createdKeys: changes.created,
      skippedKeys: changes.skipped,
      secretPolicy: changes.secretPolicy
    })
  }

  return {
    success: true,
    data: changes
  }
})
