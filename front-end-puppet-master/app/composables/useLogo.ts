/**
 * ═══════════════════════════════════════════════════════════════════════════
 * LOGO COMPOSABLE
 * ═══════════════════════════════════════════════════════════════════════════
 *
 * Automatically picks the correct logo variant based on:
 *   1. Current theme (light/dark mode)
 *   2. Current language
 *   3. Fallback chain if variant doesn't exist
 *
 * Usage:
 *   const { headerLogo, shortLogo, getLogoSrc } = useLogo()
 *
 * The logo path is reactive - changes automatically when theme or lang changes!
 *
 * Logo types:
 *   - headerLogo: Full horizontal logo for desktop header
 *   - shortLogo: Compact circle/icon logo for sidebar, footer, etc.
 *
 * ═══════════════════════════════════════════════════════════════════════════
 */

import config from '~/puppet-master.config'

export type LogoShape = 'horizontal' | 'circle'

export interface UseLogoReturn {
  /** Current logo path for header (horizontal shape) */
  headerLogo: ComputedRef<string>
  /** Current logo path for compact uses - sidebar, footer (circle shape) */
  shortLogo: ComputedRef<string>
  /** Get logo path for specific shape */
  getLogoSrc: (shape: LogoShape) => string
  /** Check if a specific logo variant exists */
  hasVariant: (shape: LogoShape, theme: 'light' | 'dark', lang: string) => boolean
}

export function useLogo(): UseLogoReturn {
  const colorMode = useColorMode()
  const { locale } = useI18n()

  /**
   * Get the effective language for logo selection
   * Uses fallback chain if current language doesn't have a logo
   */
  function getEffectiveLang(lang: string): string {
    const fallback = config.logo.langFallback as Record<string, string>

    // Check if this language has a direct logo
    const hasDirectLogo = config.logo.available.some(name => name.endsWith(`_${lang}`))

    if (hasDirectLogo) {
      return lang
    }

    // Use fallback chain
    if (fallback[lang]) {
      return fallback[lang]
    }

    // Ultimate fallback: default locale
    return config.defaultLocale
  }

  /**
   * Get the theme suffix for logo file
   * Note: 'dark' theme uses 'light' logo (white text on dark bg)
   *       'light' theme uses 'dark' logo (dark text on light bg)
   */
  function getThemeSuffix(): 'light' | 'dark' {
    // Invert: dark mode needs light logo, light mode needs dark logo
    return colorMode.value === 'dark' ? 'light' : 'dark'
  }

  /**
   * Check if a specific logo variant exists
   */
  function hasVariant(shape: LogoShape, theme: 'light' | 'dark', lang: string): boolean {
    const fileName = `${shape}_${theme}_${lang}`
    return config.logo.available.includes(fileName)
  }

  /**
   * Get logo source path for a specific shape
   */
  function getLogoSrc(shape: LogoShape): string {
    const theme = getThemeSuffix()
    const lang = getEffectiveLang(locale.value)
    const fileName = `${shape}_${theme}_${lang}`

    // Check if this variant exists, otherwise use default
    if (config.logo.available.includes(fileName)) {
      return `${config.logo.basePath}/${fileName}.svg`
    }

    // Fallback to English
    const fallbackFileName = `${shape}_${theme}_en`
    if (config.logo.available.includes(fallbackFileName)) {
      return `${config.logo.basePath}/${fallbackFileName}.svg`
    }

    // Ultimate fallback
    return `${config.logo.basePath}/horizontal_dark_en.svg`
  }

  // Reactive computed logos
  const headerLogo = computed(() => getLogoSrc('horizontal'))
  const shortLogo = computed(() => getLogoSrc('circle'))

  return {
    headerLogo,
    shortLogo,
    getLogoSrc,
    hasVariant
  }
}
