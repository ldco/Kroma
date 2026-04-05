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
    const response = await $fetch(`/api/i18n/${locale}`)
    // API returns { success: true, data: {...} }
    if (response && typeof response === 'object' && 'data' in response) {
      return (response as any).data
    }
    return response as Record<string, unknown>
  } catch (err) {
    console.warn(`[i18n] Failed to load translations for: ${locale}`, err)
    return {}
  }
})
