/**
 * Visual Regression Tests
 *
 * Captures screenshots of key pages and components for visual comparison.
 * Detects unintended visual changes across deployments.
 *
 * Run: npm run test:e2e:playwright
 * Update snapshots: npx playwright test --update-snapshots
 */
import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

test.describe('Visual Regression Tests', () => {
  test.describe('Homepage', () => {
    test('homepage renders correctly', async ({ page }) => {
      await page.goto('/')
      await page.waitForLoadState('networkidle')

      // Wait for any animations to complete
      await page.waitForTimeout(500)

      await expect(page).toHaveScreenshot('homepage.png', {
        fullPage: true
      })
    })

    test('homepage mobile view', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 })
      await page.goto('/')
      await page.waitForLoadState('networkidle')

      await expect(page).toHaveScreenshot('homepage-mobile.png', {
        fullPage: true
      })
    })

    test('homepage dark mode', async ({ page }) => {
      await page.goto('/')
      await page.waitForLoadState('networkidle')

      // Toggle dark mode
      const themeToggle = page.locator('.theme-toggle')
      if (await themeToggle.isVisible()) {
        await themeToggle.click()
        await page.waitForTimeout(500)

        await expect(page).toHaveScreenshot('homepage-dark.png', {
          fullPage: true
        })
      }
    })
  })

  test.describe('Admin Login', () => {
    test('login page renders correctly', async ({ page }) => {
      await page.goto('/admin/login')
      await page.waitForLoadState('networkidle')

      await expect(page).toHaveScreenshot('admin-login.png')
    })

    test('login form validation states', async ({ page }) => {
      await page.goto('/admin/login')
      await page.waitForLoadState('networkidle')

      // Submit empty form to trigger validation
      await page.click('button[type="submit"]')
      await page.waitForTimeout(300)

      await expect(page).toHaveScreenshot('admin-login-validation.png')
    })

    test('login page mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 })
      await page.goto('/admin/login')
      await page.waitForLoadState('networkidle')

      await expect(page).toHaveScreenshot('admin-login-mobile.png')
    })
  })

  test.describe('Components', () => {
    test('navigation renders correctly', async ({ page }) => {
      await page.goto('/')
      await page.waitForLoadState('networkidle')

      const header = page.locator('header').first()
      if (await header.isVisible()) {
        await expect(header).toHaveScreenshot('navigation.png')
      }
    })

    test('footer renders correctly', async ({ page }) => {
      await page.goto('/')
      await page.waitForLoadState('networkidle')

      const footer = page.locator('footer').first()
      if (await footer.isVisible()) {
        await footer.scrollIntoViewIfNeeded()
        await expect(footer).toHaveScreenshot('footer.png')
      }
    })
  })
})

test.describe('Accessibility Tests', () => {
  test('homepage should have no accessibility violations', async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze()

    expect(accessibilityScanResults.violations).toEqual([])
  })

  test('admin login should have no accessibility violations', async ({ page }) => {
    await page.goto('/admin/login')
    await page.waitForLoadState('networkidle')

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze()

    expect(accessibilityScanResults.violations).toEqual([])
  })
})

test.describe('Responsive Design', () => {
  const viewports = [
    { name: 'mobile-portrait', width: 375, height: 667 },
    { name: 'mobile-landscape', width: 667, height: 375 },
    { name: 'tablet-portrait', width: 768, height: 1024 },
    { name: 'tablet-landscape', width: 1024, height: 768 },
    { name: 'desktop', width: 1280, height: 800 },
    { name: 'wide-desktop', width: 1920, height: 1080 }
  ]

  for (const viewport of viewports) {
    test(`homepage at ${viewport.name} (${viewport.width}x${viewport.height})`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height })
      await page.goto('/')
      await page.waitForLoadState('networkidle')
      await page.waitForTimeout(300)

      await expect(page).toHaveScreenshot(`homepage-${viewport.name}.png`, {
        fullPage: true
      })
    })
  }
})

test.describe('Interactive Elements', () => {
  test('button hover states', async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')

    // Find a primary button
    const button = page.locator('.btn-primary').first()
    if (await button.isVisible()) {
      await button.hover()
      await page.waitForTimeout(200)

      await expect(button).toHaveScreenshot('button-hover.png')
    }
  })

  test('input focus states', async ({ page }) => {
    await page.goto('/admin/login')
    await page.waitForLoadState('networkidle')

    // Focus the email input
    const emailInput = page.locator('input[type="email"]')
    await emailInput.focus()
    await page.waitForTimeout(200)

    await expect(emailInput).toHaveScreenshot('input-focus.png')
  })

  test('mobile menu', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 })
    await page.goto('/')
    await page.waitForLoadState('networkidle')

    // Open mobile menu if hamburger exists
    const hamburger = page.locator('.hamburger, [aria-label="Menu"]').first()
    if (await hamburger.isVisible()) {
      await hamburger.click()
      await page.waitForTimeout(500)

      await expect(page).toHaveScreenshot('mobile-menu-open.png')
    }
  })
})

test.describe('Error States', () => {
  test('404 page renders correctly', async ({ page }) => {
    await page.goto('/non-existent-page-12345')
    await page.waitForLoadState('networkidle')

    await expect(page).toHaveScreenshot('404-page.png')
  })
})
