/**
 * Translation Loader
 *
 * Loads all translations from database via API.
 * Both system and content translations are stored in the database.
 *
 * System translations (common.*, nav.*, admin.*, etc.):
 *   - Editable only by master role
 *
 * Content translations (hero.*, about.*, cta.*, etc.):
 *   - Editable by admin+ roles
 */

export default defineI18nLocale(async locale => {
  try {
    return await apiFetch<Record<string, unknown>>(`/api/i18n/${locale}`)
  } catch {
    console.warn(`[i18n] Failed to load translations for: ${locale}`)
    return {}
  }
})
