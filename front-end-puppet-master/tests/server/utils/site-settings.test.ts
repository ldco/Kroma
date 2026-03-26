import { describe, expect, it } from 'vitest'
import {
  SECRET_MASK_VALUE,
  buildGroupedSettings,
  normalizeSettingValueForStorage
} from '../../../server/utils/site-settings'
import { decryptSecretValue, encryptSecretValue, isEncryptedSecretValue } from '../../../server/utils/secrets'

describe('site settings visibility', () => {
  it('excludes secret keys from public settings payload', () => {
    const settingsMap = new Map([
      ['contact.email', { key: 'contact.email', value: 'owner@example.com' }],
      ['contact.telegramBotToken', { key: 'contact.telegramBotToken', value: encryptSecretValue('bot-token') }]
    ])

    const grouped = buildGroupedSettings(settingsMap, {
      includeSecrets: false
    })

    expect(grouped.contact?.email).toBe('owner@example.com')
    expect(grouped.contact?.telegramBotToken).toBeUndefined()
  })

  it('masks secret values for non-master admin viewers', () => {
    const settingsMap = new Map([
      ['contact.telegramBotToken', { key: 'contact.telegramBotToken', value: encryptSecretValue('bot-token') }]
    ])

    const grouped = buildGroupedSettings(settingsMap, {
      includeSecrets: true,
      viewerRole: 'admin'
    })

    expect(grouped.contact?.telegramBotToken).toBe(SECRET_MASK_VALUE)
  })

  it('returns decrypted secret values for master viewers', () => {
    const settingsMap = new Map([
      ['contact.telegramBotToken', { key: 'contact.telegramBotToken', value: encryptSecretValue('bot-token') }]
    ])

    const grouped = buildGroupedSettings(settingsMap, {
      includeSecrets: true,
      viewerRole: 'master'
    })

    expect(grouped.contact?.telegramBotToken).toBe('bot-token')
  })
})

describe('site settings secret storage normalization', () => {
  it('encrypts password settings before persistence', () => {
    const normalized = normalizeSettingValueForStorage('contact.telegramBotToken', 'bot-token')
    expect(normalized.isSecret).toBe(true)
    expect(isEncryptedSecretValue(normalized.value)).toBe(true)
    expect(decryptSecretValue(normalized.value!)).toBe('bot-token')
  })

  it('preserves mask sentinel for unchanged secret submissions', () => {
    const normalized = normalizeSettingValueForStorage('contact.telegramBotToken', SECRET_MASK_VALUE)
    expect(normalized.value).toBe(SECRET_MASK_VALUE)
  })
})
