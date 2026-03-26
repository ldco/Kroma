/**
 * useSafeConfig - SSR-safe configuration access
 *
 * Provides reactive, SSR-safe access to puppet-master.config.ts
 * with defensive defaults to prevent hydration errors during HMR.
 *
 * Use this instead of direct config imports in components to avoid:
 * - "Cannot read properties of undefined" errors during HMR
 * - SSR hydration mismatches
 * - Undefined config access during module reloading
 *
 * @example
 * ```vue
 * <script setup>
 * const { isOnepager, logoPath, defaultLocale } = useSafeConfig()
 * </script>
 *
 * <template>
 *   <a v-if="isOnepager" :href="logoPath">...</a>
 * </template>
 * ```
 */
import config from '~/puppet-master.config'

export function useSafeConfig() {
  // SSR-safe computed values with defensive defaults
  const isOnepager = computed(() => config?.features?.onepager ?? false)
  const isMultiLang = computed(() => config?.isMultiLang ?? false)
  const hasThemeToggle = computed(() => config?.hasThemeToggle ?? false)
  const hasWebsite = computed(() => config?.hasWebsite ?? false)
  const hasApp = computed(() => config?.hasApp ?? false)
  const hasAdmin = computed(() => config?.hasAdmin ?? false)

  // Logo configuration with defaults
  const logoPath = computed(() => config?.logo?.basePath || '/logos')
  const logoHorizontal = computed((theme: 'light' | 'dark' = 'dark', locale?: string) => {
    const basePath = config?.logo?.basePath || '/logos'
    const defaultLocale = config?.defaultLocale || 'en'
    return `${basePath}/horizontal_${theme}_${locale || defaultLocale}.svg`
  })

  // Locale configuration with defaults
  const defaultLocale = computed(() => config?.defaultLocale || 'en')
  const locales = computed(() => config?.locales ?? [])

  // Theme configuration with defaults
  const defaultTheme = computed(() => config?.defaultTheme || 'light')

  // Full config with optional chaining (for advanced use cases)
  const getConfig = <T>(path: string, defaultValue: T): T => {
    const keys = path.split('.')
    let value: any = config

    for (const key of keys) {
      if (value === undefined || value === null) {
        return defaultValue
      }
      value = value[key]
    }

    return value ?? defaultValue
  }

  return {
    // Booleans
    isOnepager,
    isMultiLang,
    hasThemeToggle,
    hasWebsite,
    hasApp,
    hasAdmin,

    // Logo
    logoPath,
    logoHorizontal,

    // Locale
    defaultLocale,
    locales,

    // Theme
    defaultTheme,

    // Advanced
    getConfig,

    // Raw config (use with caution - not SSR-safe)
    config
  }
}
