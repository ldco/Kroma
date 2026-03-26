/**
 * Puppet Master Module & Locale Registry
 *
 * SINGLE SOURCE OF TRUTH for all PM modules and locales.
 * Used by: config-writer, init-cli, setup API, wizard UI
 *
 * This file is in /shared/ to be importable from both:
 * - Server (server/api/setup/*.ts)
 * - Frontend (app/pages/init.vue)
 * - Scripts (scripts/lib/*.ts)
 */

// ═══════════════════════════════════════════════════════════════════════════════
// MODULE DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

export interface ModuleMetadata {
  id: string
  name: string
  description: string
  category: 'content' | 'marketing' | 'social'
}

/**
 * All available modules in canonical order
 */
export const ALL_MODULES = [
  'blog',
  'portfolio',
  'team',
  'testimonials',
  'faq',
  'pricing',
  'clients',
  'features',
  'contact'
] as const

export type ModuleId = typeof ALL_MODULES[number]

/**
 * Module metadata for display in wizard and docs
 */
export const MODULE_METADATA: Record<ModuleId, ModuleMetadata> = {
  blog: {
    id: 'blog',
    name: 'Blog',
    description: 'Posts, categories, tags',
    category: 'content'
  },
  portfolio: {
    id: 'portfolio',
    name: 'Portfolio',
    description: 'Project showcase',
    category: 'content'
  },
  team: {
    id: 'team',
    name: 'Team',
    description: 'Team member profiles',
    category: 'social'
  },
  testimonials: {
    id: 'testimonials',
    name: 'Testimonials',
    description: 'Customer reviews',
    category: 'social'
  },
  faq: {
    id: 'faq',
    name: 'FAQ',
    description: 'Questions & answers',
    category: 'content'
  },
  pricing: {
    id: 'pricing',
    name: 'Pricing',
    description: 'Pricing tables',
    category: 'marketing'
  },
  clients: {
    id: 'clients',
    name: 'Clients',
    description: 'Client logos',
    category: 'social'
  },
  features: {
    id: 'features',
    name: 'Features',
    description: 'Feature cards',
    category: 'marketing'
  },
  contact: {
    id: 'contact',
    name: 'Contact',
    description: 'Contact form',
    category: 'content'
  }
}

/**
 * Get module list formatted for wizard display
 */
export function getModulesForWizard(): Array<{ id: string; name: string; desc: string }> {
  return ALL_MODULES.map(id => ({
    id,
    name: MODULE_METADATA[id].name,
    desc: MODULE_METADATA[id].description
  }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// VALIDATION
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Check if a string is a valid module ID
 */
export function isValidModuleId(id: string): id is ModuleId {
  return ALL_MODULES.includes(id as ModuleId)
}

/**
 * Filter array to only valid module IDs
 */
export function filterValidModules(ids: string[]): ModuleId[] {
  return ids.filter(isValidModuleId) as ModuleId[]
}

/**
 * Validate module IDs and return invalid ones
 */
export function validateModuleIds(ids: string[]): { valid: ModuleId[]; invalid: string[] } {
  const valid: ModuleId[] = []
  const invalid: string[] = []

  for (const id of ids) {
    if (isValidModuleId(id)) {
      valid.push(id)
    } else {
      invalid.push(id)
    }
  }

  return { valid, invalid }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LOCALE DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

export interface LocaleInfo {
  code: string
  iso: string
  name: string
}

/**
 * Supported locales with metadata
 */
export const LOCALE_MAP: Record<string, LocaleInfo> = {
  en: { code: 'en', iso: 'en-US', name: 'English' },
  ru: { code: 'ru', iso: 'ru-RU', name: 'Russian' },
  he: { code: 'he', iso: 'he-IL', name: 'Hebrew' },
  es: { code: 'es', iso: 'es-ES', name: 'Spanish' },
  fr: { code: 'fr', iso: 'fr-FR', name: 'French' },
  de: { code: 'de', iso: 'de-DE', name: 'German' },
  zh: { code: 'zh', iso: 'zh-CN', name: 'Chinese' },
  ja: { code: 'ja', iso: 'ja-JP', name: 'Japanese' }
}

export const SUPPORTED_LOCALE_CODES = Object.keys(LOCALE_MAP)

/**
 * Get all locales formatted for wizard display
 */
export function getLocalesForWizard(): LocaleInfo[] {
  return Object.values(LOCALE_MAP)
}

/**
 * Get locale info by code, or undefined if not supported
 */
export function getLocaleInfo(code: string): LocaleInfo | undefined {
  return LOCALE_MAP[code]
}

/**
 * Convert locale codes to LocaleInfo array
 */
export function parseLocales(codes: string[]): LocaleInfo[] {
  return codes
    .map(code => LOCALE_MAP[code.trim()])
    .filter((l): l is LocaleInfo => l !== undefined)
}
