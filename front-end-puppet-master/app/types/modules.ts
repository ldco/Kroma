/**
 * PuppetMaster Module System
 *
 * Pre-built, config-driven features that can be enabled per-project.
 * Each module provides: database tables, API endpoints, admin UI, and frontend pages.
 */

// ═══════════════════════════════════════════════════════════════════════════════
// MODULE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Available module IDs
 */
export type ModuleId =
  | 'portfolio'
  | 'pricing'
  | 'contact'
  | 'blog'
  | 'team'
  | 'testimonials'
  | 'features'
  | 'clients'
  | 'faq'

/**
 * Module definition - describes what a module provides
 * Using 'unknown' for defaultConfig to allow specific types in registry
 */
export interface ModuleDefinition {
  id: ModuleId
  name: string
  description: string
  version: string

  /** Database tables this module requires */
  tables: string[]

  /** Admin sections this module adds */
  adminSections: Array<{
    id: string
    icon: string
    label: string
    roles: string[]
  }>

  /** Frontend routes this module adds */
  routes: string[]

  /** Default configuration */
  defaultConfig: unknown

  /** Whether this module is currently implemented */
  implemented: boolean
}

/**
 * Module configuration in puppet-master.config.ts
 */
export interface ModuleConfig<TConfig> {
  enabled: boolean
  config: Partial<TConfig>
}

// ═══════════════════════════════════════════════════════════════════════════════
// PORTFOLIO MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface PortfolioModuleConfig {
  /** Display layout */
  layout: 'grid' | 'masonry' | 'list' | 'carousel'
  /** Card visual style */
  cardStyle: 'minimal' | 'detailed' | 'overlay' | 'split'
  /** Show filter buttons by category/technology */
  showFilters: boolean
  /** Enable case study pages */
  showCaseStudies: boolean
  /** Enable image galleries */
  showGallery: boolean
  /** Items per page */
  itemsPerPage: number
  /** Default sort order */
  sortDefault: 'date' | 'title' | 'featured' | 'order'
  /** Show categories */
  showCategories: boolean
  /** Show technologies/tags */
  showTechnologies: boolean
}

export const portfolioModuleDefaults: PortfolioModuleConfig = {
  layout: 'grid',
  cardStyle: 'overlay',
  showFilters: true,
  showCaseStudies: true,
  showGallery: true,
  itemsPerPage: 12,
  sortDefault: 'order',
  showCategories: true,
  showTechnologies: true
}

// ═══════════════════════════════════════════════════════════════════════════════
// PRICING MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface PricingTier {
  id: string
  name: string
  description?: string
  price: number | null // null = "Contact us"
  period?: 'month' | 'year' | 'one-time'
  currency?: string
  featured?: boolean
  features: Array<{
    text: string
    included: boolean
    tooltip?: string
  }>
  cta?: {
    text: string
    url: string
  }
}

export interface PricingModuleConfig {
  /** Pricing tiers */
  tiers: PricingTier[]
  /** Show feature comparison table */
  showComparison: boolean
  /** Show monthly/yearly toggle */
  showToggle: boolean
  /** Yearly discount percentage */
  yearlyDiscount: number
  /** Default currency */
  currency: string
  /** Display style */
  style: 'cards' | 'table' | 'mixed'
  /** CTA button style */
  ctaStyle: 'button' | 'link'
  /** Highlight featured tier */
  highlightFeatured: boolean
  /** Show FAQ section */
  showFAQ: boolean
}

export const pricingModuleDefaults: PricingModuleConfig = {
  tiers: [],
  showComparison: true,
  showToggle: true,
  yearlyDiscount: 20,
  currency: 'USD',
  style: 'cards',
  ctaStyle: 'button',
  highlightFeatured: true,
  showFAQ: false
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTACT MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface ContactModuleConfig {
  /** Show map */
  showMap: boolean
  /** Map provider */
  mapProvider: 'yandex' | 'google' | 'osm'
  /** Show contact form */
  showForm: boolean
  /** Show contact info (address, phone, email) */
  showInfo: boolean
  /** Custom form fields */
  formFields: Array<{
    name: string
    type: 'text' | 'email' | 'tel' | 'textarea' | 'select'
    label: string
    required: boolean
    options?: string[] // for select type
  }>
  /** Notification methods */
  notifications: 'email' | 'telegram' | 'both' | 'none'
  /** Show social links */
  showSocial: boolean
}

export const contactModuleDefaults: ContactModuleConfig = {
  showMap: false,
  mapProvider: 'yandex',
  showForm: true,
  showInfo: true,
  formFields: [
    { name: 'name', type: 'text', label: 'Name', required: true },
    { name: 'email', type: 'email', label: 'Email', required: true },
    { name: 'message', type: 'textarea', label: 'Message', required: true }
  ],
  notifications: 'email',
  showSocial: true
}

// ═══════════════════════════════════════════════════════════════════════════════
// BLOG MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface BlogModuleConfig {
  /** Posts per page */
  postsPerPage: number
  /** Show categories */
  showCategories: boolean
  /** Show tags */
  showTags: boolean
  /** Show author info */
  showAuthor: boolean
  /** Show reading time */
  showReadingTime: boolean
  /** Show view count */
  showViewCount: boolean
  /** Excerpt length in characters */
  excerptLength: number
  /** Display layout */
  layout: 'grid' | 'list'
  /** Number of latest posts to show in section preview */
  latestPostsCount: number
}

export const blogModuleDefaults: BlogModuleConfig = {
  postsPerPage: 10,
  showCategories: true,
  showTags: true,
  showAuthor: true,
  showReadingTime: true,
  showViewCount: false,
  excerptLength: 160,
  layout: 'grid',
  latestPostsCount: 3
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEAM MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface TeamModuleConfig {
  /** Display layout */
  layout: 'grid' | 'list' | 'carousel'
  /** Card visual style */
  cardStyle: 'minimal' | 'detailed' | 'photo-focus'
  /** Show social links */
  showSocial: boolean
  /** Show bio */
  showBio: boolean
  /** Show email */
  showEmail: boolean
  /** Show phone */
  showPhone: boolean
  /** Group by department */
  groupByDepartment: boolean
  /** Grid columns on desktop */
  columnsDesktop: number
  /** Grid columns on mobile */
  columnsMobile: number
}

export const teamModuleDefaults: TeamModuleConfig = {
  layout: 'grid',
  cardStyle: 'detailed',
  showSocial: true,
  showBio: true,
  showEmail: false,
  showPhone: false,
  groupByDepartment: false,
  columnsDesktop: 4,
  columnsMobile: 2
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTIMONIALS MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface TestimonialsModuleConfig {
  /** Display layout */
  layout: 'carousel' | 'grid' | 'masonry'
  /** Show star rating */
  showRating: boolean
  /** Show avatar/photo */
  showPhoto: boolean
  /** Show company/org */
  showCompany: boolean
  /** Auto-play carousel */
  autoPlay: boolean
  /** Auto-play interval in ms */
  autoPlayInterval: number
}

export const testimonialsModuleDefaults: TestimonialsModuleConfig = {
  layout: 'carousel',
  showRating: true,
  showPhoto: true,
  showCompany: true,
  autoPlay: false,
  autoPlayInterval: 5000
}

// ═══════════════════════════════════════════════════════════════════════════════
// FEATURES MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface FeaturesModuleConfig {
  /** Display layout */
  layout: 'grid' | 'list'
  /** Card style */
  cardStyle: 'icon-top' | 'icon-left'
  /** Show category labels */
  showCategory: boolean
  /** Grid columns on desktop */
  columnsDesktop: number
  /** Grid columns on mobile */
  columnsMobile: number
}

export const featuresModuleDefaults: FeaturesModuleConfig = {
  layout: 'grid',
  cardStyle: 'icon-top',
  showCategory: false,
  columnsDesktop: 3,
  columnsMobile: 1
}

// ═══════════════════════════════════════════════════════════════════════════════
// CLIENTS MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface ClientsModuleConfig {
  /** Display layout */
  layout: 'strip' | 'grid' | 'carousel'
  /** Show client names */
  showNames: boolean
  /** Grayscale logos by default */
  grayscale: boolean
  /** Color on hover */
  hoverColor: boolean
  /** Filter by categories */
  categories: Array<'client' | 'sponsor' | 'partner'>
}

export const clientsModuleDefaults: ClientsModuleConfig = {
  layout: 'strip',
  showNames: false,
  grayscale: true,
  hoverColor: true,
  categories: ['client', 'sponsor', 'partner']
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAQ MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

export interface FaqModuleConfig {
  /** Display layout */
  layout: 'accordion' | 'list'
  /** Show category grouping */
  showCategories: boolean
  /** Expand first item by default */
  expandFirst: boolean
  /** Allow multiple items open */
  allowMultipleOpen: boolean
}

export const faqModuleDefaults: FaqModuleConfig = {
  layout: 'accordion',
  showCategories: false,
  expandFirst: true,
  allowMultipleOpen: false
}

// ═══════════════════════════════════════════════════════════════════════════════
// MODULE REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Registry of all available modules with their definitions
 */
export const MODULE_REGISTRY: Record<ModuleId, ModuleDefinition> = {
  portfolio: {
    id: 'portfolio',
    name: 'Portfolio',
    description: 'Project showcase with galleries and case studies',
    version: '1.0.0',
    tables: ['portfolios', 'portfolio_items'],
    adminSections: [{ id: 'portfolios', icon: 'photo', label: 'portfolio', roles: [] }],
    routes: ['/portfolio', '/portfolio/[slug]'],
    defaultConfig: portfolioModuleDefaults,
    implemented: true
  },

  pricing: {
    id: 'pricing',
    name: 'Pricing',
    description: 'Pricing tiers with comparison table',
    version: '1.0.0',
    tables: ['pricing_tiers', 'pricing_features'],
    adminSections: [{ id: 'pricing', icon: 'credit-card', label: 'pricing', roles: [] }],
    routes: ['/pricing'],
    defaultConfig: pricingModuleDefaults,
    implemented: true
  },

  contact: {
    id: 'contact',
    name: 'Contact',
    description: 'Contact form, map, and info display',
    version: '1.0.0',
    tables: ['contact_submissions'],
    adminSections: [{ id: 'contacts', icon: 'mail', label: 'contacts', roles: [] }],
    routes: ['/contact'],
    defaultConfig: contactModuleDefaults,
    implemented: true
  },

  blog: {
    id: 'blog',
    name: 'Blog',
    description: 'Blog posts with categories, tags, and media',
    version: '1.0.0',
    tables: ['blog_posts', 'blog_categories', 'blog_tags', 'blog_post_tags', 'blog_media'],
    adminSections: [{ id: 'blog', icon: 'article', label: 'blog', roles: [] }],
    routes: ['/blog', '/blog/[slug]'],
    defaultConfig: blogModuleDefaults,
    implemented: true
  },

  team: {
    id: 'team',
    name: 'Team',
    description: 'Team member profiles with photos and social links',
    version: '1.0.0',
    tables: ['team_members', 'team_member_translations'],
    adminSections: [{ id: 'team', icon: 'users-group', label: 'team', roles: [] }],
    routes: [],
    defaultConfig: teamModuleDefaults,
    implemented: true
  },

  testimonials: {
    id: 'testimonials',
    name: 'Testimonials',
    description: 'Customer testimonials and reviews',
    version: '1.0.0',
    tables: ['testimonials', 'testimonial_translations'],
    adminSections: [{ id: 'testimonials', icon: 'quote', label: 'testimonials', roles: [] }],
    routes: [],
    defaultConfig: testimonialsModuleDefaults,
    implemented: true
  },

  features: {
    id: 'features',
    name: 'Features',
    description: 'Feature cards with icons and descriptions',
    version: '1.0.0',
    tables: ['features', 'feature_translations'],
    adminSections: [{ id: 'features', icon: 'sparkles', label: 'features', roles: [] }],
    routes: [],
    defaultConfig: featuresModuleDefaults,
    implemented: true
  },

  clients: {
    id: 'clients',
    name: 'Clients',
    description: 'Client, sponsor, and partner logos',
    version: '1.0.0',
    tables: ['clients'],
    adminSections: [{ id: 'clients', icon: 'building', label: 'clients', roles: [] }],
    routes: [],
    defaultConfig: clientsModuleDefaults,
    implemented: true
  },

  faq: {
    id: 'faq',
    name: 'FAQ',
    description: 'Frequently asked questions with accordion',
    version: '1.0.0',
    tables: ['faq_items', 'faq_item_translations'],
    adminSections: [{ id: 'faq', icon: 'help-circle', label: 'faq', roles: [] }],
    routes: [],
    defaultConfig: faqModuleDefaults,
    implemented: true
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Map of module IDs to their config types
 */
export interface ModuleConfigMap {
  portfolio: PortfolioModuleConfig
  pricing: PricingModuleConfig
  contact: ContactModuleConfig
  blog: BlogModuleConfig
  team: TeamModuleConfig
  testimonials: TestimonialsModuleConfig
  features: FeaturesModuleConfig
  clients: ClientsModuleConfig
  faq: FaqModuleConfig
}

/**
 * Full modules configuration object for puppet-master.config.ts
 */
export type ModulesConfig = {
  [K in ModuleId]?: ModuleConfig<ModuleConfigMap[K]>
}

/**
 * Get default config for a module (type-safe)
 */
export function getModuleDefaults<T extends ModuleId>(
  moduleId: T
): ModuleConfigMap[T] {
  // Cast is safe because MODULE_REGISTRY is defined with correct types
  return MODULE_REGISTRY[moduleId].defaultConfig as ModuleConfigMap[T]
}

/**
 * Get module definition
 */
export function getModuleDefinition(moduleId: ModuleId): ModuleDefinition {
  return MODULE_REGISTRY[moduleId]
}

/**
 * Check if a module is implemented
 */
export function isModuleImplemented(moduleId: ModuleId): boolean {
  return MODULE_REGISTRY[moduleId].implemented
}

/**
 * Get all implemented modules
 */
export function getImplementedModules(): ModuleDefinition[] {
  return Object.values(MODULE_REGISTRY).filter((m) => m.implemented)
}
