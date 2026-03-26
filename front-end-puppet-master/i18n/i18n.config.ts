/**
 * Vue I18n Configuration
 *
 * ARCHITECTURE:
 * 1. Messages are loaded from locale files (i18n/locales/*.ts)
 * 2. Locale files use defineI18nLocale for lazy loading
 * 3. Database translations can OVERRIDE these via API (future)
 * 4. Admin panel can edit translations in database
 *
 * This file only configures Vue I18n options, NOT messages.
 * Messages are in i18n/locales/*.ts files.
 */

export default defineI18nConfig(() => ({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  // Suppress warnings for missing translations
  missingWarn: false,
  fallbackWarn: false
  // Messages are loaded from locale files via lazy loading
}))
