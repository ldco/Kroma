import { eq } from 'drizzle-orm'
import config from '~~/app/puppet-master.config'
import { schema, useDatabase } from '../database/client'
import { decryptSecretValue, encryptSecretValue, isEncryptedSecretValue } from './secrets'
import type { UserRole } from '../database/schema'

export const SECRET_MASK_VALUE = '••••••••'

type ConfigSetting = (typeof config.settings)[number]

const SETTING_DEFINITIONS = [...config.settings] as readonly ConfigSetting[]
const SETTING_DEFINITION_BY_KEY = new Map<string, ConfigSetting>(
  SETTING_DEFINITIONS.map(setting => [setting.key, setting])
)

export function getSettingDefinition(key: string): ConfigSetting | undefined {
  return SETTING_DEFINITION_BY_KEY.get(key)
}

export function isSecretSettingDefinition(setting: Pick<ConfigSetting, 'type'> & { public?: boolean }): boolean {
  return setting.type === 'password' || setting.public === false
}

export function isSecretSettingKey(key: string): boolean {
  const setting = getSettingDefinition(key)
  return !!setting && isSecretSettingDefinition(setting)
}

export function isPublicSettingKey(key: string): boolean {
  const setting = getSettingDefinition(key)
  return !!setting && !isSecretSettingDefinition(setting)
}

function splitSettingKey(key: string): { group: string; shortKey: string } {
  const [group, ...rest] = key.split('.')
  return {
    group: group || 'general',
    shortKey: rest.length > 0 ? rest.join('.') : key
  }
}

function parseStoredSettingValue(key: string, storedValue: string | null): string | null {
  if (storedValue === null) {
    return null
  }
  if (!isSecretSettingKey(key)) {
    return storedValue
  }
  try {
    return decryptSecretValue(storedValue)
  } catch {
    return null
  }
}

export function normalizeSettingValueForStorage(
  key: string,
  value: string | null
): { value: string | null; isSecret: boolean } {
  if (!isSecretSettingKey(key)) {
    return { value, isSecret: false }
  }

  if (value === null) {
    return { value: null, isSecret: true }
  }

  if (value === SECRET_MASK_VALUE) {
    return { value: SECRET_MASK_VALUE, isSecret: true }
  }

  const normalized = value.trim()
  if (!normalized) {
    return { value: '', isSecret: true }
  }

  return {
    value: encryptSecretValue(normalized),
    isSecret: true
  }
}

interface BuildGroupedOptions {
  includeSecrets: boolean
  viewerRole?: UserRole | undefined
}

export function buildGroupedSettings(
  byKey: Map<string, { key: string; value: string | null }>,
  options: BuildGroupedOptions
): Record<string, Record<string, string | null>> {
  const grouped: Record<string, Record<string, string | null>> = {}

  for (const definition of SETTING_DEFINITIONS) {
    const isSecret = isSecretSettingDefinition(definition)
    if (!options.includeSecrets && isSecret) {
      continue
    }

    const settingValue = byKey.get(definition.key)?.value ?? null
    const { group, shortKey } = splitSettingKey(definition.key)
    grouped[group] ||= {}

    if (!isSecret) {
      grouped[group][shortKey] = settingValue
      continue
    }

    const parsed = parseStoredSettingValue(definition.key, settingValue)
    if (options.viewerRole === 'master') {
      grouped[group][shortKey] = parsed
      continue
    }

    grouped[group][shortKey] = parsed ? SECRET_MASK_VALUE : null
  }

  return grouped
}

export function getSecretSettingConfiguredFlag(
  byKey: Map<string, { key: string; value: string | null }>,
  key: string
): boolean {
  const current = byKey.get(key)?.value
  if (!current) return false
  if (isEncryptedSecretValue(current)) {
    try {
      return decryptSecretValue(current).trim().length > 0
    } catch {
      return false
    }
  }
  return current.trim().length > 0
}

export function listSettingDefinitions(): readonly ConfigSetting[] {
  return SETTING_DEFINITIONS
}

export function buildSettingsMap(): Map<string, { key: string; value: string | null }> {
  const db = useDatabase()
  const rows = db.select().from(schema.settings).all()
  return new Map(rows.map(row => [row.key, { key: row.key, value: row.value }]))
}

export function getSettingValue(key: string): string | null {
  const db = useDatabase()
  const row = db.select().from(schema.settings).where(eq(schema.settings.key, key)).get()
  if (!row) {
    return null
  }
  return parseStoredSettingValue(key, row.value)
}
