import { describe, expect, it } from 'vitest'
import { setup, $fetch } from '@nuxt/test-utils/e2e'
import { eq } from 'drizzle-orm'
import { schema, useDatabase } from '../../server/database/client'
import { SECRET_MASK_VALUE } from '../../server/utils/site-settings'
import { loginAsAdmin, loginAsMaster } from '../utils/helpers'

describe('Admin Settings Secret Contract', async () => {
  await setup({
    server: true,
    browser: false
  })

  it('persists telegram bot token encrypted at rest', async () => {
    const auth = await loginAsMaster()
    const token = `token-${Date.now()}`

    await $fetch('/api/admin/settings', {
      method: 'PUT',
      headers: auth.headers,
      body: {
        'contact.telegramBotToken': token
      }
    })

    const db = useDatabase()
    const row = db
      .select()
      .from(schema.settings)
      .where(eq(schema.settings.key, 'contact.telegramBotToken'))
      .get()

    expect(row).toBeDefined()
    expect(row?.value).toMatch(/^enc:v1:/)
    expect(row?.value).not.toBe(token)
  })

  it('never exposes secret settings on the public settings endpoint', async () => {
    const response = await $fetch('/api/settings')

    expect(response.success).toBe(true)
    expect(response.data.contact?.telegramBotToken).toBeUndefined()
  })

  it('shows decrypted secret value for master and masked value for admin', async () => {
    const masterAuth = await loginAsMaster()
    const token = `token-${Date.now()}`

    await $fetch('/api/admin/settings', {
      method: 'PUT',
      headers: masterAuth.headers,
      body: {
        'contact.telegramBotToken': token
      }
    })

    const masterView = await $fetch('/api/admin/settings', {
      headers: masterAuth.headers
    })
    expect(masterView.success).toBe(true)
    expect(masterView.data.contact?.telegramBotToken).toBe(token)

    const adminAuth = await loginAsAdmin()
    const adminView = await $fetch('/api/admin/settings', {
      headers: adminAuth.headers
    })
    expect(adminView.success).toBe(true)
    expect(adminView.data.contact?.telegramBotToken).toBe(SECRET_MASK_VALUE)
  })
})
