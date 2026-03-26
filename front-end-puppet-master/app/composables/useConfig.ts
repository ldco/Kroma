/**
 * useConfig - Access build-time configuration
 *
 * Provides reactive access to puppet-master.config.ts settings.
 * Use this composable in components to access configuration.
 *
 * @example
 * ```vue
 * <script setup>
 * const { hasWebsite, hasAdmin, hasApp } = useConfig()
 * </script>
 *
 * <template>
 *   <LoginButton v-if="hasLoginButton" />
 * </template>
 * ```
 */
import config from '~/puppet-master.config'

export function useConfig() {
  return {
    // ═══════════════════════════════════════════════════════════════════════
    // ENTITIES - What exists in the project
    // ═══════════════════════════════════════════════════════════════════════
    entities: config.entities,

    // Entity-derived booleans (for conditional rendering)
    hasWebsite: config.hasWebsite, // Has public website
    hasApp: config.hasApp, // Has user application
    hasAdmin: config.hasAdmin, // Has admin panel
    hasLoginButton: config.hasLoginButton, // Show login button in header
    isAppOnly: config.isAppOnly, // No website, only app
    isWebsiteOnly: config.isWebsiteOnly, // Website only, no app/admin

    // ═══════════════════════════════════════════════════════════════════════
    // ADMIN - Module configuration with RBAC
    // ═══════════════════════════════════════════════════════════════════════
    admin: config.admin,
    getAdminSections: config.getAdminSections.bind(config),
    getAdminSectionsForRole: config.getAdminSectionsForRole.bind(config),

    // ═══════════════════════════════════════════════════════════════════════
    // FEATURES - Fine-tuned behavior toggles
    // ═══════════════════════════════════════════════════════════════════════
    features: config.features,

    // Website features
    isMultiLang: config.isMultiLang,
    hasThemeToggle: config.hasThemeToggle,
    isOnepager: config.useOnepager, // Combined: hasWebsite && onepager
    hasInteractiveHeader: config.useInteractiveHeader, // Combined: hasWebsite && interactiveHeader
    hideHeaderOnScroll: config.features.hideHeaderOnScroll,
    headerFooterWidthMode: config.headerFooterWidthMode,
    // Deprecated alias kept for one minor release for downstream compatibility.
    websiteChromeWidth: config.headerFooterWidthMode,

    // App/Admin features
    appVerticalNav: config.features.appVerticalNav,


    // ═══════════════════════════════════════════════════════════════════════
    // MODULES - Pre-built features (content modules)
    // ═══════════════════════════════════════════════════════════════════════
    modules: config.modules,

    // ═══════════════════════════════════════════════════════════════════════
    // DATA - Locales, colors, sections, logo
    // ═══════════════════════════════════════════════════════════════════════
    locales: config.locales,
    defaultLocale: config.defaultLocale,
    sections: config.sections,
    colors: config.colors,
    logo: config.logo,

    // Full config access (for advanced use cases)
    config
  }
}
