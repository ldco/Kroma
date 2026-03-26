/**
 * Auth Flow E2E Tests
 *
 * Tests complete authentication user journeys using browser mode.
 * Uses @nuxt/test-utils with Playwright under the hood.
 */
import { describe, it, expect } from 'vitest'
import { setup, createPage } from '@nuxt/test-utils/e2e'

describe('Auth Flow E2E', async () => {
  await setup({
    server: true,
    browser: true,
    browserOptions: {
      type: 'chromium'
    }
  })

  describe('Login Page', () => {
    it('displays login form correctly', async () => {
      const page = await createPage('/admin/login')

      // Check form elements exist
      expect(await page.locator('input[type="email"]').first().isVisible()).toBe(true)
      expect(await page.locator('input[type="password"]').first().isVisible()).toBe(true)
      expect(await page.locator('button[type="submit"]').first().isVisible()).toBe(true)

      // Check title is visible
      const title = await page.locator('h1').textContent()
      expect(title?.trim().length).toBeGreaterThan(0)
    })

    it('shows password visibility toggle', async () => {
      const page = await createPage('/admin/login')

      const passwordInput = page.locator('input#password')
      const toggleButton = page.locator('.input-action')

      // Initially password type
      expect(await passwordInput.getAttribute('type')).toBe('password')

      // Click toggle
      await toggleButton.click()

      // Now should be text type
      expect(await passwordInput.getAttribute('type')).toBe('text')

      // Click again to toggle back
      await toggleButton.click()
      expect(await passwordInput.getAttribute('type')).toBe('password')
    })

    it('shows error for invalid credentials', async () => {
      const page = await createPage('/admin/login')

      // Fill form with invalid credentials
      await page.fill('input[type="email"]', 'invalid@example.com')
      await page.fill('input[type="password"]', 'wrongpassword')

      // Submit
      await page.click('button[type="submit"]')

      // Wait for error message
      await page.waitForSelector('.form-error', { timeout: 10000 })

      const errorText = await page.locator('.form-error').textContent()
      expect(errorText?.trim().length).toBeGreaterThan(0)
    })

    it('redirects to admin dashboard on successful login', async () => {
      const page = await createPage('/admin/login')

      // Fill form with valid credentials (seeded user)
      await page.fill('input[type="email"]', 'master@example.com')
      await page.fill('input[type="password"]', 'master123')

      // Submit
      await page.click('button[type="submit"]')

      // Wait for navigation to admin
      await page.waitForURL(url => !url.pathname.includes('/login'), { timeout: 15000 })

      // Should be on admin dashboard, not login
      expect(page.url()).toContain('/admin')
      expect(page.url()).not.toContain('/login')
    })
  })

  describe('Protected Route Access', () => {
    it('redirects unauthenticated users from /admin to login', async () => {
      const page = await createPage('/admin')

      // Should redirect to login
      await page.waitForURL(url => url.pathname.includes('/login'), { timeout: 10000 })

      expect(page.url()).toContain('/login')
    })

    it('redirects unauthenticated users from /admin/settings to login', async () => {
      const page = await createPage('/admin/settings')

      // Should redirect to login
      await page.waitForURL(url => url.pathname.includes('/login'), { timeout: 10000 })

      expect(page.url()).toContain('/login')
    })
  })

  describe('Logout Flow', () => {
    it('logs out user and redirects to login', async () => {
      const page = await createPage('/admin/login')

      // Login first
      await page.fill('input[type="email"]', 'master@example.com')
      await page.fill('input[type="password"]', 'master123')
      await page.click('button[type="submit"]')

      // Wait for admin dashboard
      await page.waitForURL(url => !url.pathname.includes('/login'), { timeout: 15000 })

      // Click user avatar to open menu (supports both horizontal and vertical admin nav)
      const avatar = page.locator('.admin-header-user-avatar, .sidebar-user-avatar').first()
      await avatar.click()

      // Wait for menu and click logout
      await page.waitForSelector('.user-menu-logout', { timeout: 5000 })
      await page.click('.user-menu-logout')

      // Should redirect to login
      await page.waitForURL(url => url.pathname.includes('/login'), { timeout: 10000 })
      expect(page.url()).toContain('/login')
    })
  })
})
