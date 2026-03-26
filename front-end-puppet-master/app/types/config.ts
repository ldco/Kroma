/**
 * Puppet Master Configuration Types
 *
 * Modular entity architecture with flexible per-model RBAC.
 * See docs/PM-ARCHITECTURE.md for full documentation.
 */

import type { UserRole } from './auth'

// ═══════════════════════════════════════════════════════════════════════════════
// PM MODE - Setup State
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Project setup state
 *
 * - unconfigured: Fresh clone, needs wizard setup
 * - build: Configured for client project (website or app)
 * - develop: Configured for PM framework development (showcase mode)
 */
export type PmMode = 'unconfigured' | 'build' | 'develop'

/**
 * AI Agent Workflow selection
 *
 * - claude: Claude Code workflow with basic commands
 * - qwen: Qwen Code workflow with 42 expert personas (7 specialties × 6 countries)
 * - codex: Codex workflow (future/experimental)
 */
export type AiWorkflow = 'claude' | 'qwen' | 'codex'

/**
 * Project type when in BUILD mode
 */
export type ProjectType = 'website' | 'app'

// ═══════════════════════════════════════════════════════════════════════════════
// ROLE SYSTEM
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Role hierarchy - higher roles inherit lower role permissions
 */
export const ROLE_HIERARCHY: Record<UserRole, UserRole[]> = {
  master: ['admin', 'editor', 'user'],
  admin: ['editor', 'user'],
  editor: ['user'],
  user: []
}

/**
 * Role hierarchy as numeric levels - higher number = more permissions
 * Used for simple comparisons: hasRole(userRole, minRole)
 */
export const ROLE_LEVELS: Record<UserRole, number> = {
  user: 0,
  editor: 1,
  admin: 2,
  master: 3
}

/**
 * Check if a role has at least the minimum required level
 */
export function hasRoleLevel(userRole: UserRole | undefined, minRole: UserRole): boolean {
  if (!userRole) return false
  return ROLE_LEVELS[userRole] >= ROLE_LEVELS[minRole]
}

/**
 * Check if a role can access a resource requiring specific roles
 */
export function canAccess(userRole: UserRole, requiredRoles: UserRole[]): boolean {
  if (requiredRoles.length === 0) return true // Empty = all roles
  const effectiveRoles: UserRole[] = [userRole, ...ROLE_HIERARCHY[userRole]]
  return requiredRoles.some(r => effectiveRoles.includes(r))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENTITIES CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * System entities - what exists in the project
 */
export interface EntitiesConfig {
  /** Public marketing/landing pages (Website UX) */
  website: boolean
  /** User-facing application features (App UX) */
  app: boolean
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADMIN MODULE CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Base admin module config - enabled + roles
 */
export interface AdminModuleConfig {
  enabled: boolean
  roles: UserRole[]
}

/**
 * System modules - PM provides, universal for all projects
 */
export interface SystemModulesConfig {
  /** User management */
  users: AdminModuleConfig
  /** Role assignment (ALWAYS master-only, roles config ignored) */
  roles: AdminModuleConfig
  /** Translation management */
  translations: AdminModuleConfig
  /** Application settings */
  settings: AdminModuleConfig
  /** System health monitoring */
  health: AdminModuleConfig
  /** Activity/error logs */
  logs: AdminModuleConfig
}

/**
 * Website content modules - PM provides for website content management
 */
export interface WebsiteModulesConfig {
  /** Hero, About, Contact sections */
  sections: AdminModuleConfig
  /** Blog posts, categories, tags */
  blog: AdminModuleConfig
  /** Project showcase, galleries */
  portfolio: AdminModuleConfig
  /** Team member profiles */
  team: AdminModuleConfig
  /** Customer testimonials */
  testimonials: AdminModuleConfig
  /** FAQ accordion */
  faq: AdminModuleConfig
  /** Client/partner logos */
  clients: AdminModuleConfig
  /** Pricing tiers */
  pricing: AdminModuleConfig
  /** Feature cards */
  features: AdminModuleConfig
  /** Contact form submissions */
  contacts: AdminModuleConfig
}

/**
 * App data modules - Developer defines custom admin pages per project
 * Structure is flexible - developer adds modules as needed
 */
export interface AppModulesConfig {
  [key: string]: AdminModuleConfig
}

/**
 * Full admin configuration
 */
export interface AdminConfig {
  /** Enable admin panel */
  enabled: boolean
  /** System modules (PM provides, universal) */
  system: SystemModulesConfig
  /** Website content modules (PM provides) */
  websiteModules: WebsiteModulesConfig
  /** App data modules (developer defines) */
  appModules: AppModulesConfig
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEFAULT CONFIGURATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Default system modules configuration
 * 
 * Comment 1: users role set to ['master'] only to align with API enforcement
 */
export const defaultSystemModules: SystemModulesConfig = {
  users: { enabled: true, roles: ['master'] },
  roles: { enabled: true, roles: ['master'] }, // ALWAYS master-only
  translations: { enabled: true, roles: ['master', 'admin', 'editor'] },
  settings: { enabled: true, roles: ['master', 'admin'] },
  health: { enabled: true, roles: ['master'] },
  logs: { enabled: false, roles: ['master'] }
}

/**
 * Default website modules configuration
 */
export const defaultWebsiteModules: WebsiteModulesConfig = {
  sections: { enabled: true, roles: ['master', 'admin', 'editor'] },
  blog: { enabled: false, roles: ['master', 'admin', 'editor'] },
  portfolio: { enabled: false, roles: ['master', 'admin'] },
  team: { enabled: false, roles: ['master', 'admin'] },
  testimonials: { enabled: false, roles: ['master', 'admin', 'editor'] },
  faq: { enabled: false, roles: ['master', 'admin', 'editor'] },
  clients: { enabled: false, roles: ['master', 'admin'] },
  pricing: { enabled: false, roles: ['master', 'admin'] },
  features: { enabled: false, roles: ['master', 'admin', 'editor'] },
  contacts: { enabled: true, roles: ['master', 'admin'] }
}

/**
 * Default admin configuration
 */
export const defaultAdminConfig: AdminConfig = {
  enabled: true,
  system: defaultSystemModules,
  websiteModules: defaultWebsiteModules,
  appModules: {}
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADMIN SECTION TYPE (for navigation)
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Admin navigation section
 */
export interface AdminSection {
  id: string
  icon: string
  label: string
  badge?: boolean
  roles: UserRole[]
  group: 'system' | 'content' | 'app'
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSITE SECTION CONFIGURATION (navigation + routing contract)
// ═══════════════════════════════════════════════════════════════════════════════

export type BuiltInWebsiteSectionId =
  | 'home'
  | 'about'
  | 'portfolio'
  | 'features'
  | 'team'
  | 'testimonials'
  | 'blog'
  | 'faq'
  | 'clients'
  | 'pricing'
  | 'contact'

export interface WebsiteSectionPropsMap {
  about: {
    photoPlacement?: 'inside-panel' | 'outside-panel'
    showStats?: boolean
  }
  contact: {
    showInfo?: boolean
    showMap?: boolean
  }
  features: {
    bento?: boolean
  }
}

type BuiltInWithProps = keyof WebsiteSectionPropsMap
type BuiltInWithoutProps = Exclude<BuiltInWebsiteSectionId, BuiltInWithProps>

type BuiltInSectionWithProps = {
  [K in BuiltInWithProps]: {
    id: K
    inNav: boolean
    props?: WebsiteSectionPropsMap[K]
  }
}[BuiltInWithProps]

type BuiltInSectionWithoutProps = {
  [K in BuiltInWithoutProps]: {
    id: K
    inNav: boolean
    props?: Record<string, never>
  }
}[BuiltInWithoutProps]

type CustomWebsiteSection = {
  id: string
  inNav: boolean
  props?: Record<string, unknown>
}

export type WebsiteSection = BuiltInSectionWithProps | BuiltInSectionWithoutProps | CustomWebsiteSection

export interface WebsiteNavLink {
  id: string
  href: string
  isAnchor: boolean
}

const WEBSITE_SECTION_ID_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/

function assertValidWebsiteSectionId(sectionId: string): void {
  if (sectionId.trim() !== sectionId) {
    throw new Error(`Invalid website section id "${sectionId}": leading/trailing whitespace is not allowed`)
  }
  if (!WEBSITE_SECTION_ID_PATTERN.test(sectionId)) {
    throw new Error(
      `Invalid website section id "${sectionId}": expected lowercase-kebab-case (letters, numbers, dashes)`
    )
  }
}

export function defineWebsiteSections<const T extends readonly WebsiteSection[]>(sections: T): T {
  if (sections.length === 0) {
    throw new Error('Invalid website sections config: at least one section is required')
  }

  const seenIds = new Set<string>()
  for (const section of sections) {
    assertValidWebsiteSectionId(section.id)
    if (seenIds.has(section.id)) {
      throw new Error(`Invalid website sections config: duplicate id "${section.id}"`)
    }
    seenIds.add(section.id)
  }

  const homeIndex = sections.findIndex(section => section.id === 'home')
  if (homeIndex > 0) {
    throw new Error('Invalid website sections config: "home" must be the first section when present')
  }

  return sections
}

type SectionHrefOptions =
  | { onepager: true; localePath?: (path: string) => string }
  | { onepager: false; localePath: (path: string) => string }

export function getWebsiteSectionHref(sectionId: string, options: SectionHrefOptions): string {
  if (options.onepager) {
    return `#${sectionId}`
  }

  return options.localePath(sectionId === 'home' ? '/' : `/${sectionId}`)
}

export function getNavWebsiteSectionIds(sections: readonly WebsiteSection[]): string[] {
  return sections.filter(section => section.inNav).map(section => section.id)
}

export function getDynamicWebsiteSectionIds(sections: readonly WebsiteSection[]): string[] {
  return sections.filter(section => section.id !== 'home').map(section => section.id)
}

export function buildWebsiteNavLinks(
  sections: readonly WebsiteSection[],
  options: SectionHrefOptions
): WebsiteNavLink[] {
  return sections
    .filter(section => section.inNav)
    .map(section => ({
      id: section.id,
      href: getWebsiteSectionHref(section.id, options),
      isAnchor: options.onepager
    }))
}

/**
 * Generate admin sections from config
 */
export function getAdminSections(admin: AdminConfig): AdminSection[] {
  const sections: AdminSection[] = []

  // System modules
  if (admin.system.users.enabled) {
    sections.push({ id: 'users', icon: 'users', label: 'users', roles: admin.system.users.roles, group: 'system' })
  }
  if (admin.system.roles.enabled) {
    sections.push({ id: 'roles', icon: 'shield', label: 'roles', roles: ['master'], group: 'system' }) // Always master-only
  }
  if (admin.system.translations.enabled) {
    sections.push({ id: 'translations', icon: 'language', label: 'translations', roles: admin.system.translations.roles, group: 'system' })
  }
  if (admin.system.settings.enabled) {
    sections.push({ id: 'settings', icon: 'settings', label: 'settings', roles: admin.system.settings.roles, group: 'system' })
  }
  if (admin.system.health.enabled) {
    sections.push({ id: 'health', icon: 'heartbeat', label: 'health', roles: admin.system.health.roles, group: 'system' })
  }
  if (admin.system.logs.enabled) {
    sections.push({ id: 'logs', icon: 'list', label: 'logs', roles: admin.system.logs.roles, group: 'system' })
  }

  // Website content modules
  if (admin.websiteModules.sections.enabled) {
    sections.push({ id: 'sections', icon: 'layout', label: 'sections', roles: admin.websiteModules.sections.roles, group: 'content' })
  }
  if (admin.websiteModules.blog.enabled) {
    sections.push({ id: 'blog', icon: 'article', label: 'blog', roles: admin.websiteModules.blog.roles, group: 'content' })
  }
  if (admin.websiteModules.portfolio.enabled) {
    sections.push({ id: 'portfolios', icon: 'photo', label: 'portfolio', roles: admin.websiteModules.portfolio.roles, group: 'content' })
  }
  if (admin.websiteModules.team.enabled) {
    sections.push({ id: 'team', icon: 'users-group', label: 'team', roles: admin.websiteModules.team.roles, group: 'content' })
  }
  if (admin.websiteModules.testimonials.enabled) {
    sections.push({ id: 'testimonials', icon: 'quote', label: 'testimonials', roles: admin.websiteModules.testimonials.roles, group: 'content' })
  }
  if (admin.websiteModules.faq.enabled) {
    sections.push({ id: 'faq', icon: 'help-circle', label: 'faq', roles: admin.websiteModules.faq.roles, group: 'content' })
  }
  if (admin.websiteModules.clients.enabled) {
    sections.push({ id: 'clients', icon: 'building', label: 'clients', roles: admin.websiteModules.clients.roles, group: 'content' })
  }
  if (admin.websiteModules.pricing.enabled) {
    sections.push({ id: 'pricing', icon: 'credit-card', label: 'pricing', roles: admin.websiteModules.pricing.roles, group: 'content' })
  }
  if (admin.websiteModules.features.enabled) {
    sections.push({ id: 'features', icon: 'sparkles', label: 'features', roles: admin.websiteModules.features.roles, group: 'content' })
  }
  if (admin.websiteModules.contacts.enabled) {
    sections.push({ id: 'contacts', icon: 'mail', label: 'contacts', badge: true, roles: admin.websiteModules.contacts.roles, group: 'content' })
  }

  // App data modules (custom)
  for (const [id, moduleConfig] of Object.entries(admin.appModules)) {
    if (moduleConfig.enabled) {
      sections.push({ id, icon: 'database', label: id, roles: moduleConfig.roles, group: 'app' })
    }
  }

  return sections
}

/**
 * Filter admin sections by user role
 */
export function filterSectionsByRole(sections: AdminSection[], userRole: UserRole): AdminSection[] {
  return sections.filter(section => canAccess(userRole, section.roles))
}
