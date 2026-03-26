/**
 * Database Schema
 *
 * SQLite schema using Drizzle ORM.
 * Tables: users, sessions, settings, portfolios, portfolio_items, contact_submissions,
 *         pricing_tiers, pricing_features, translations, audit_logs,
 *         team_members, blog_posts, blog_categories, blog_tags, blog_media,
 *         clients, features, testimonials, faq_items
 */
import { sqliteTable, text, integer, unique, index } from 'drizzle-orm/sqlite-core'

// ═══════════════════════════════════════════════════════════════════════════
// USERS & AUTH
// ═══════════════════════════════════════════════════════════════════════════

/**
 * User roles hierarchy (legacy - for backward compatibility):
 * - master: Developer/agency who builds the site (full access)
 * - admin: Client who owns the site (can manage content + users except master)
 * - editor: Client's employees (can only edit content)
 */
export const USER_ROLES = ['master', 'admin', 'editor'] as const
export type UserRole = (typeof USER_ROLES)[number]

/**
 * Admin page IDs - each admin page is a permission
 * Simple model: if page is true, user sees it in nav
 */
export const ADMIN_PAGE_IDS = [
  // System pages
  'users',
  'roles',
  'translations',
  'settings',
  'health',
  // Content pages
  'sections',
  'blog',
  'portfolios',
  'team',
  'testimonials',
  'faq',
  'clients',
  'pricing',
  'features',
  'contacts'
] as const
export type AdminPageId = (typeof ADMIN_PAGE_IDS)[number]
export type RolePermissions = Partial<Record<AdminPageId, boolean>>

/**
 * Badge color options for roles
 */
export const ROLE_COLORS = ['primary', 'secondary', 'warning', 'success', 'danger'] as const
export type RoleColor = (typeof ROLE_COLORS)[number]

/**
 * Roles - Dynamic role management
 */
export const roles = sqliteTable('roles', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull().unique(),
  slug: text('slug').notNull().unique(),
  description: text('description'),
  permissions: text('permissions').notNull(), // JSON of RolePermissions
  level: integer('level').notNull().default(0), // Higher = more authority
  isBuiltIn: integer('is_built_in', { mode: 'boolean' }).default(false),
  color: text('color').default('secondary'),
  icon: text('icon').default('pencil'), // Icon name for custom roles
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

/**
 * Admin users
 */
export const users = sqliteTable('users', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  email: text('email').notNull().unique(),
  passwordHash: text('password_hash').notNull(),
  name: text('name'),
  // Legacy role column (for backward compatibility during migration)
  role: text('role', { enum: ['master', 'admin', 'editor'] })
    .default('editor')
    .notNull(),
  // New dynamic role reference
  roleId: integer('role_id').references(() => roles.id, { onDelete: 'set null' }),
  // Account lockout fields (CRIT-04)
  failedLoginAttempts: integer('failed_login_attempts').default(0),
  lockedUntil: integer('locked_until', { mode: 'timestamp' }),
  lastFailedLogin: integer('last_failed_login', { mode: 'timestamp' }),
  // Two-factor authentication
  twoFactorEnabled: integer('two_factor_enabled', { mode: 'boolean' }).default(false),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

/**
 * User sessions for authentication
 */
export const sessions = sqliteTable(
  'sessions',
  {
    id: text('id').primaryKey(), // UUID
    userId: integer('user_id')
      .notNull()
      .references(() => users.id, { onDelete: 'cascade' }),
    expiresAt: integer('expires_at', { mode: 'timestamp' }).notNull(),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    index('sessions_user_expires_idx').on(table.userId, table.expiresAt),
    index('sessions_expires_idx').on(table.expiresAt)
  ]
)

/**
 * Two-Factor Authentication secrets and backup codes
 * Stores encrypted TOTP secret and one-time backup codes
 */
export const user2fa = sqliteTable(
  'user_2fa',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    userId: integer('user_id')
      .notNull()
      .unique()
      .references(() => users.id, { onDelete: 'cascade' }),
    // Encrypted TOTP secret (base32 encoded)
    secret: text('secret').notNull(),
    // JSON array of hashed backup codes (10 codes, each usable once)
    backupCodes: text('backup_codes').notNull(),
    // Number of backup codes remaining
    backupCodesRemaining: integer('backup_codes_remaining').default(10),
    // Timestamps
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('user_2fa_user_idx').on(table.userId)]
)

// ═══════════════════════════════════════════════════════════════════════════
// SITE SETTINGS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Key-value settings store (editable via admin)
 */
export const settings = sqliteTable('settings', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  key: text('key').notNull().unique(),
  value: text('value'),
  type: text('type', { enum: ['string', 'number', 'boolean', 'json'] })
    .default('string')
    .notNull(),
  group: text('group').default('general'), // For organizing in admin UI
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

// ═══════════════════════════════════════════════════════════════════════════
// CONTENT
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Portfolio types:
 * - gallery: Collection of images, videos, and external links
 * - case_study: Collection of detailed project case studies
 */
export const PORTFOLIO_TYPES = ['gallery', 'case_study'] as const
export type PortfolioType = (typeof PORTFOLIO_TYPES)[number]

/**
 * Portfolio item types:
 * - image: Uploaded image file
 * - video: Uploaded video file
 * - link: External URL (e.g., Behance, Dribbble)
 * - case_study: Rich content project with full details
 */
export const PORTFOLIO_ITEM_TYPES = ['image', 'video', 'link', 'case_study'] as const
export type PortfolioItemType = (typeof PORTFOLIO_ITEM_TYPES)[number]

/**
 * Portfolios - Collections of portfolio items
 */
export const portfolios = sqliteTable(
  'portfolios',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    slug: text('slug').notNull().unique(),
    name: text('name').notNull(),
    description: text('description'),
    type: text('type', { enum: ['gallery', 'case_study'] }).notNull(),
    coverImageUrl: text('cover_image_url'),
    coverThumbnailUrl: text('cover_thumbnail_url'),
    order: integer('order').default(0),
    published: integer('published', { mode: 'boolean' }).default(false),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('portfolios_published_idx').on(table.published)]
)

/**
 * Portfolio items - Individual items within a portfolio
 * Uses discriminated union pattern: itemType determines which fields are used
 */
export const portfolioItems = sqliteTable(
  'portfolio_items',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    portfolioId: integer('portfolio_id')
      .notNull()
      .references(() => portfolios.id, { onDelete: 'cascade' }),
    itemType: text('item_type', { enum: ['image', 'video', 'link', 'case_study'] }).notNull(),

    // Common fields
    order: integer('order').default(0),
    published: integer('published', { mode: 'boolean' }).default(true),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),

    // Media fields (for gallery items: image, video, link)
    mediaUrl: text('media_url'), // Image/video URL or external link
    thumbnailUrl: text('thumbnail_url'), // Thumbnail for image/video
    caption: text('caption'), // Brief caption for gallery items

    // Case study fields (for case_study items)
    slug: text('slug'), // Unique within portfolio
    title: text('title'), // Case study title
    description: text('description'), // Short description
    content: text('content'), // Rich text / markdown
    tags: text('tags'), // JSON array stored as text
    category: text('category'), // Category label
    publishedAt: integer('published_at', { mode: 'timestamp' })
  },
  table => [index('portfolio_items_portfolio_idx').on(table.portfolioId)]
)

/**
 * Contact form submissions
 */
export const contactSubmissions = sqliteTable('contact_submissions', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
  email: text('email').notNull(),
  phone: text('phone'),
  subject: text('subject'),
  message: text('message').notNull(),
  read: integer('read', { mode: 'boolean' }).default(false),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

// ═══════════════════════════════════════════════════════════════════════════
// PRICING
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Pricing periods
 */
export const PRICING_PERIODS = ['month', 'year', 'one-time'] as const
export type PricingPeriod = (typeof PRICING_PERIODS)[number]

/**
 * Pricing tiers - The pricing plans (e.g., Starter, Pro, Enterprise)
 */
export const pricingTiers = sqliteTable('pricing_tiers', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  slug: text('slug').notNull().unique(), // 'starter', 'pro', 'enterprise'
  name: text('name').notNull(), // Display name (translatable via i18n key)
  description: text('description'), // Short description
  price: integer('price'), // Price in cents (null = custom/contact)
  currency: text('currency').default('USD'),
  period: text('period', { enum: ['month', 'year', 'one-time'] }).default('month'),
  featured: integer('featured', { mode: 'boolean' }).default(false), // Highlight this tier
  ctaText: text('cta_text'), // Button text (translatable via i18n key)
  ctaUrl: text('cta_url').default('/contact'), // Button link
  order: integer('order').default(0),
  published: integer('published', { mode: 'boolean' }).default(true),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

/**
 * Pricing features - Features for each tier
 */
export const pricingFeatures = sqliteTable(
  'pricing_features',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    tierId: integer('tier_id')
      .notNull()
      .references(() => pricingTiers.id, { onDelete: 'cascade' }),
    text: text('text').notNull(), // Feature description (default/fallback)
    included: integer('included', { mode: 'boolean' }).default(true), // ✓ or ✗
    order: integer('order').default(0),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('pricing_features_tier_idx').on(table.tierId)]
)

/**
 * Pricing tier translations - Localized text for tiers
 */
export const pricingTierTranslations = sqliteTable(
  'pricing_tier_translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    tierId: integer('tier_id')
      .notNull()
      .references(() => pricingTiers.id, { onDelete: 'cascade' }),
    locale: text('locale').notNull(), // 'en', 'ru', 'he'
    name: text('name'), // Translated name
    description: text('description'), // Translated description
    ctaText: text('cta_text'), // Translated CTA button text
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    unique().on(table.tierId, table.locale),
    index('pricing_tier_translations_tier_idx').on(table.tierId)
  ]
)

/**
 * Pricing feature translations - Localized text for features
 */
export const pricingFeatureTranslations = sqliteTable(
  'pricing_feature_translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    featureId: integer('feature_id')
      .notNull()
      .references(() => pricingFeatures.id, { onDelete: 'cascade' }),
    locale: text('locale').notNull(), // 'en', 'ru', 'he'
    text: text('text'), // Translated feature text
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    unique().on(table.featureId, table.locale),
    index('pricing_feature_translations_feature_idx').on(table.featureId)
  ]
)

// ═══════════════════════════════════════════════════════════════════════════
// TEAM
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Team members
 */
export const teamMembers = sqliteTable(
  'team_members',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    slug: text('slug').notNull().unique(),
    name: text('name').notNull(),
    position: text('position'),
    bio: text('bio'),
    photoUrl: text('photo_url'),
    hoverPhotoUrl: text('hover_photo_url'), // Photo shown on hover
    email: text('email'),
    phone: text('phone'),
    department: text('department'),
    socialLinks: text('social_links'), // JSON: {linkedin, twitter, github, etc.}
    published: integer('published', { mode: 'boolean' }).default(true),
    order: integer('order').default(0),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('team_members_published_idx').on(table.published)]
)

/**
 * Team member translations
 */
export const teamMemberTranslations = sqliteTable(
  'team_member_translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    memberId: integer('member_id')
      .notNull()
      .references(() => teamMembers.id, { onDelete: 'cascade' }),
    locale: text('locale').notNull(),
    position: text('position'),
    bio: text('bio'),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    unique().on(table.memberId, table.locale),
    index('team_member_translations_member_idx').on(table.memberId)
  ]
)

// ═══════════════════════════════════════════════════════════════════════════
// BLOG
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Blog categories
 */
export const blogCategories = sqliteTable('blog_categories', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  slug: text('slug').notNull().unique(),
  order: integer('order').default(0),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

/**
 * Blog category translations
 */
export const blogCategoryTranslations = sqliteTable(
  'blog_category_translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    categoryId: integer('category_id')
      .notNull()
      .references(() => blogCategories.id, { onDelete: 'cascade' }),
    locale: text('locale').notNull(),
    name: text('name').notNull(),
    description: text('description'),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    unique().on(table.categoryId, table.locale),
    index('blog_category_translations_category_idx').on(table.categoryId)
  ]
)

/**
 * Blog tags
 */
export const blogTags = sqliteTable('blog_tags', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  slug: text('slug').notNull().unique(),
  name: text('name').notNull(),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
})

/**
 * Blog posts
 */
export const blogPosts = sqliteTable(
  'blog_posts',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    slug: text('slug').notNull().unique(),
    authorId: integer('author_id').references(() => users.id, { onDelete: 'set null' }),
    categoryId: integer('category_id').references(() => blogCategories.id, { onDelete: 'set null' }),
    coverImageUrl: text('cover_image_url'),
    coverImageAlt: text('cover_image_alt'),
    published: integer('published', { mode: 'boolean' }).default(false),
    publishedAt: integer('published_at', { mode: 'timestamp' }),
    readingTimeMinutes: integer('reading_time_minutes'),
    viewCount: integer('view_count').default(0),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    index('blog_posts_published_idx').on(table.published),
    index('blog_posts_author_idx').on(table.authorId),
    index('blog_posts_category_idx').on(table.categoryId)
  ]
)

/**
 * Blog post translations
 */
export const blogPostTranslations = sqliteTable(
  'blog_post_translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    postId: integer('post_id')
      .notNull()
      .references(() => blogPosts.id, { onDelete: 'cascade' }),
    locale: text('locale').notNull(),
    title: text('title').notNull(),
    excerpt: text('excerpt'),
    content: text('content'), // Markdown
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    unique().on(table.postId, table.locale),
    index('blog_post_translations_post_idx').on(table.postId)
  ]
)

/**
 * Blog post tags (many-to-many)
 */
export const blogPostTags = sqliteTable(
  'blog_post_tags',
  {
    postId: integer('post_id')
      .notNull()
      .references(() => blogPosts.id, { onDelete: 'cascade' }),
    tagId: integer('tag_id')
      .notNull()
      .references(() => blogTags.id, { onDelete: 'cascade' }),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    index('blog_post_tags_post_idx').on(table.postId),
    index('blog_post_tags_tag_idx').on(table.tagId)
  ]
)

/**
 * Blog media (images & videos in content)
 */
export const blogMedia = sqliteTable(
  'blog_media',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    postId: integer('post_id')
      .notNull()
      .references(() => blogPosts.id, { onDelete: 'cascade' }),
    type: text('type', { enum: ['image', 'video'] }).notNull(),
    url: text('url').notNull(),
    thumbnailUrl: text('thumbnail_url'),
    alt: text('alt'),
    width: integer('width'),
    height: integer('height'),
    sizeBytes: integer('size_bytes'),
    order: integer('order').default(0),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('blog_media_post_idx').on(table.postId)]
)

// ═══════════════════════════════════════════════════════════════════════════
// CLIENTS / SPONSORS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Client categories
 */
export const CLIENT_CATEGORIES = ['client', 'sponsor', 'partner'] as const
export type ClientCategory = (typeof CLIENT_CATEGORIES)[number]

/**
 * Clients / Sponsors / Partners
 */
export const clients = sqliteTable(
  'clients',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    name: text('name').notNull(),
    logoUrl: text('logo_url').notNull(),
    websiteUrl: text('website_url'),
    category: text('category', { enum: ['client', 'sponsor', 'partner'] }).default('client'),
    featured: integer('featured', { mode: 'boolean' }).default(false),
    published: integer('published', { mode: 'boolean' }).default(true),
    order: integer('order').default(0),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('clients_published_idx').on(table.published)]
)

// ═══════════════════════════════════════════════════════════════════════════
// FEATURES
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Features (replaces services section)
 */
export const features = sqliteTable(
  'features',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    slug: text('slug').notNull().unique(),
    imageUrl: text('image_url'), // Feature image (required for display)
    hoverImageUrl: text('hover_image_url'), // Image shown on hover
    category: text('category'),
    published: integer('published', { mode: 'boolean' }).default(true),
    order: integer('order').default(0),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('features_published_idx').on(table.published)]
)

/**
 * Feature translations
 */
export const featureTranslations = sqliteTable(
  'feature_translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    featureId: integer('feature_id')
      .notNull()
      .references(() => features.id, { onDelete: 'cascade' }),
    locale: text('locale').notNull(),
    title: text('title').notNull(),
    description: text('description'),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    unique().on(table.featureId, table.locale),
    index('feature_translations_feature_idx').on(table.featureId)
  ]
)

// ═══════════════════════════════════════════════════════════════════════════
// TESTIMONIALS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Testimonials
 */
export const testimonials = sqliteTable(
  'testimonials',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    authorName: text('author_name').notNull(),
    authorTitle: text('author_title'),
    authorCompany: text('author_company'),
    authorPhotoUrl: text('author_photo_url'),
    rating: integer('rating'), // 1-5 stars, optional
    featured: integer('featured', { mode: 'boolean' }).default(false),
    published: integer('published', { mode: 'boolean' }).default(true),
    order: integer('order').default(0),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('testimonials_published_idx').on(table.published)]
)

/**
 * Testimonial translations
 */
export const testimonialTranslations = sqliteTable(
  'testimonial_translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    testimonialId: integer('testimonial_id')
      .notNull()
      .references(() => testimonials.id, { onDelete: 'cascade' }),
    locale: text('locale').notNull(),
    quote: text('quote').notNull(),
    authorTitle: text('author_title'),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    unique().on(table.testimonialId, table.locale),
    index('testimonial_translations_testimonial_idx').on(table.testimonialId)
  ]
)

// ═══════════════════════════════════════════════════════════════════════════
// FAQ
// ═══════════════════════════════════════════════════════════════════════════

/**
 * FAQ items
 */
export const faqItems = sqliteTable(
  'faq_items',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    slug: text('slug').notNull().unique(),
    category: text('category'),
    published: integer('published', { mode: 'boolean' }).default(true),
    order: integer('order').default(0),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [index('faq_items_published_idx').on(table.published)]
)

/**
 * FAQ item translations
 */
export const faqItemTranslations = sqliteTable(
  'faq_item_translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    faqItemId: integer('faq_item_id')
      .notNull()
      .references(() => faqItems.id, { onDelete: 'cascade' }),
    locale: text('locale').notNull(),
    question: text('question').notNull(),
    answer: text('answer').notNull(), // Markdown supported
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    unique().on(table.faqItemId, table.locale),
    index('faq_item_translations_faq_idx').on(table.faqItemId)
  ]
)

// ═══════════════════════════════════════════════════════════════════════════
// TRANSLATIONS (Database-driven i18n)
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Translation strings by locale
 */
export const translations = sqliteTable(
  'translations',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    locale: text('locale').notNull(), // 'en', 'ru', 'he'
    key: text('key').notNull(), // 'nav.home', 'common.submit'
    value: text('value').notNull(),
    updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [unique().on(table.locale, table.key)]
)

// ═══════════════════════════════════════════════════════════════════════════
// AUDIT LOGGING (HIGH-04)
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Audit actions for security-relevant events
 */
export const AUDIT_ACTIONS = [
  'login',
  'login_failed',
  'logout',
  'password_change',
  'role_change',
  'user_create',
  'user_update',
  'user_delete',
  'account_locked',
  'account_unlocked',
  'session_expired',
  // Role management
  'role_create',
  'role_update',
  'role_delete',
  'settings_update',
  // Two-Factor Authentication
  '2fa_enabled',
  '2fa_disabled',
  '2fa_verified',
  '2fa_failed'
] as const
export type AuditAction = (typeof AUDIT_ACTIONS)[number]

/**
 * Audit log for security-relevant events
 */
export const auditLogs = sqliteTable(
  'audit_logs',
  {
    id: integer('id').primaryKey({ autoIncrement: true }),
    action: text('action').notNull(), // AuditAction
    userId: integer('user_id').references(() => users.id, { onDelete: 'set null' }), // Actor (null for system)
    targetUserId: integer('target_user_id').references(() => users.id, { onDelete: 'set null' }), // Affected user
    ipAddress: text('ip_address'),
    userAgent: text('user_agent'),
    details: text('details'), // JSON with additional context
    success: integer('success', { mode: 'boolean' }).default(true),
    createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date())
  },
  table => [
    index('audit_logs_created_idx').on(table.createdAt),
    index('audit_logs_action_idx').on(table.action)
  ]
)

// ═══════════════════════════════════════════════════════════════════════════
// TYPE EXPORTS
// ═══════════════════════════════════════════════════════════════════════════

export type AuditLog = typeof auditLogs.$inferSelect
export type NewAuditLog = typeof auditLogs.$inferInsert
export type Role = typeof roles.$inferSelect
export type NewRole = typeof roles.$inferInsert
export type User = typeof users.$inferSelect
export type NewUser = typeof users.$inferInsert
export type Session = typeof sessions.$inferSelect
export type NewSession = typeof sessions.$inferInsert
export type User2fa = typeof user2fa.$inferSelect
export type NewUser2fa = typeof user2fa.$inferInsert
export type Setting = typeof settings.$inferSelect
export type NewSetting = typeof settings.$inferInsert
export type Portfolio = typeof portfolios.$inferSelect
export type NewPortfolio = typeof portfolios.$inferInsert
export type PortfolioItem = typeof portfolioItems.$inferSelect
export type NewPortfolioItem = typeof portfolioItems.$inferInsert
export type ContactSubmission = typeof contactSubmissions.$inferSelect
export type NewContactSubmission = typeof contactSubmissions.$inferInsert
export type Translation = typeof translations.$inferSelect
export type NewTranslation = typeof translations.$inferInsert
export type PricingTier = typeof pricingTiers.$inferSelect
export type NewPricingTier = typeof pricingTiers.$inferInsert
export type PricingFeature = typeof pricingFeatures.$inferSelect
export type NewPricingFeature = typeof pricingFeatures.$inferInsert
export type PricingTierTranslation = typeof pricingTierTranslations.$inferSelect
export type NewPricingTierTranslation = typeof pricingTierTranslations.$inferInsert
export type PricingFeatureTranslation = typeof pricingFeatureTranslations.$inferSelect
export type NewPricingFeatureTranslation = typeof pricingFeatureTranslations.$inferInsert

// Team
export type TeamMember = typeof teamMembers.$inferSelect
export type NewTeamMember = typeof teamMembers.$inferInsert
export type TeamMemberTranslation = typeof teamMemberTranslations.$inferSelect
export type NewTeamMemberTranslation = typeof teamMemberTranslations.$inferInsert

// Blog
export type BlogCategory = typeof blogCategories.$inferSelect
export type NewBlogCategory = typeof blogCategories.$inferInsert
export type BlogCategoryTranslation = typeof blogCategoryTranslations.$inferSelect
export type NewBlogCategoryTranslation = typeof blogCategoryTranslations.$inferInsert
export type BlogTag = typeof blogTags.$inferSelect
export type NewBlogTag = typeof blogTags.$inferInsert
export type BlogPost = typeof blogPosts.$inferSelect
export type NewBlogPost = typeof blogPosts.$inferInsert
export type BlogPostTranslation = typeof blogPostTranslations.$inferSelect
export type NewBlogPostTranslation = typeof blogPostTranslations.$inferInsert
export type BlogPostTag = typeof blogPostTags.$inferSelect
export type NewBlogPostTag = typeof blogPostTags.$inferInsert
export type BlogMediaItem = typeof blogMedia.$inferSelect
export type NewBlogMediaItem = typeof blogMedia.$inferInsert

// Clients
export type Client = typeof clients.$inferSelect
export type NewClient = typeof clients.$inferInsert

// Features
export type Feature = typeof features.$inferSelect
export type NewFeature = typeof features.$inferInsert
export type FeatureTranslation = typeof featureTranslations.$inferSelect
export type NewFeatureTranslation = typeof featureTranslations.$inferInsert

// Testimonials
export type Testimonial = typeof testimonials.$inferSelect
export type NewTestimonial = typeof testimonials.$inferInsert
export type TestimonialTranslation = typeof testimonialTranslations.$inferSelect
export type NewTestimonialTranslation = typeof testimonialTranslations.$inferInsert

// FAQ
export type FaqItem = typeof faqItems.$inferSelect
export type NewFaqItem = typeof faqItems.$inferInsert
export type FaqItemTranslation = typeof faqItemTranslations.$inferSelect
export type NewFaqItemTranslation = typeof faqItemTranslations.$inferInsert
