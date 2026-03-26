import { describe, expect, it } from 'vitest'
import {
  decryptSecretValue,
  encryptSecretValue,
  isEncryptedSecretValue
} from '../../../server/utils/secrets'

describe('settings secrets', () => {
  it('encrypts with enc:v1 envelope prefix', () => {
    const encrypted = encryptSecretValue('telegram-token')
    expect(encrypted.startsWith('enc:v1:')).toBe(true)
    expect(isEncryptedSecretValue(encrypted)).toBe(true)
  })

  it('decrypts encrypted values back to original plaintext', () => {
    const encrypted = encryptSecretValue('sensitive-value')
    expect(decryptSecretValue(encrypted)).toBe('sensitive-value')
  })

  it('keeps legacy plaintext values readable', () => {
    expect(decryptSecretValue('legacy-plaintext')).toBe('legacy-plaintext')
    expect(isEncryptedSecretValue('legacy-plaintext')).toBe(false)
  })
})
