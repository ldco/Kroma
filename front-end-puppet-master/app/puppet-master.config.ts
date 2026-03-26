/**
 * Puppet Master Configuration
 * Build-time configuration - changes require rebuild
 *
 * Architecture: See docs/PM-ARCHITECTURE.md for full documentation
 */

import type { ModulesConfig } from './types/modules'
import type { EntitiesConfig, AdminConfig, AdminSection } from './types/config'
import { getAdminSections, filterSectionsByRole } from './types/config'
import { defineWebsiteSections } from './types/config'
import type { UserRole } from './types/auth'

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * PROJECT TYPE - PM builds ONE thing at a time
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * Choose ONE:
 *   - Website: Marketing site, landing pages, company info (Website UX)
 *   - App: Product, dashboard, user features (App UX)
 *
 * These are MUTUALLY EXCLUSIVE. If you need both a marketing site AND an app,
 * deploy them as separate PM instances:
 *   - marketing.example.com → Website
 *   - app.example.com → App
 *
 * ┌──────────────┬──────────────────┬─────────────────────────────────────────┐
 * │ Type         │ UX Paradigm      │ Purpose                                 │
 * ├──────────────┼──────────────────┼─────────────────────────────────────────┤
 * │ Website      │ Website UX       │ Public marketing, landing, info pages   │
 * │ App          │ App UX           │ User-facing product (/ → /login)        │
 * └──────────────┴──────────────────┴─────────────────────────────────────────┘
 *
 * Admin panel (admin.enabled) adds management interface to either type.
 * Admin uses App UX - it's the same paradigm as App.
 * ═══════════════════════════════════════════════════════════════════════════════
 */

const config = {
  // ═══════════════════════════════════════════════════════════════════════════
  // PM MODE - Setup state (set by wizard or /pm-init command)
  // ═══════════════════════════════════════════════════════════════════════════
  // - 'unconfigured': Fresh clone, needs wizard setup
  // - 'build': Configured for client project (website or app)
  // - 'develop': PM framework development mode (shows showcase)
  // ═══════════════════════════════════════════════════════════════════════════
  // KROMA: Build mode - configured for Kroma Tauri app
  pmMode: 'build' as 'unconfigured' | 'build' | 'develop',

  // ═══════════════════════════════════════════════════════════════════════════
  // AI WORKFLOW - Select AI agent workflow
  // ═══════════════════════════════════════════════════════════════════════════
  // - 'claude': Claude Code workflow with basic commands
  // - 'qwen': Qwen Code workflow with 42 expert personas (7 specialties × 6 countries)
  // - 'codex': Codex workflow (future/experimental)
  // ═══════════════════════════════════════════════════════════════════════════
  aiWorkflow: 'claude' as 'claude' | 'qwen' | 'codex',

  // ═══════════════════════════════════════════════════════════════════════════
  // PROJECT TYPE - Choose ONE: Website OR App (mutually exclusive)
  // ═══════════════════════════════════════════════════════════════════════════
  // KROMA: Building an App (comic/graphic-novel production tool)
  entities: {
    website: false, // FALSE = Not building a website
    app: true // TRUE = Building an App (product, dashboard) - / → /login
  } as EntitiesConfig,

  // ═══════════════════════════════════════════════════════════════════════════
  // FEATURE TOGGLES - Fine-tune behavior for website and app entities
  // ═══════════════════════════════════════════════════════════════════════════
  features: {
    // Website features (only apply when entities.website: true)
    multiLangs: true, // Multiple languages support
    doubleTheme: true, // Light/dark mode toggle
    onepager: false, // Website: Onepager (scroll nav) vs SPA (route nav). Only for website entity.
    interactiveHeader: true, // Header style changes on scroll
    hideHeaderOnScroll: false, // Hide header when scrolling down
    verticalNav: false, // true = icon sidebar, false = horizontal header
    // Header/footer INNER chrome width mode.
    // contained = constrained + centered inner content
    // full = edge-to-edge inner content (no max-width)
    // NOTE: This does NOT control main content/scene width.
    headerFooterWidthMode: 'contained' as 'contained' | 'full',

    // Page transitions (SPA mode only - ignored in onepager mode)
    // Animation options:
    //   Basic: 'fade' | 'slide-left' | 'slide-up' | 'scale'
    //   Fancy: 'zoom' | 'flip' | 'rotate' | 'blur' | 'bounce' | 'swipe'
    //   Disabled: '' (empty string)
    pageTransitions: 'zoom' as
      | 'fade'
      | 'slide-left'
      | 'slide-up'
      | 'scale'
      | 'zoom'
      | 'flip'
      | 'rotate'
      | 'blur'
      | 'bounce'
      | 'swipe'
      | '',

    // App/Admin features (only apply when entities.app: true or admin.enabled: true)
    appVerticalNav: false, // App UX: true = vertical sidebar, false = horizontal nav
    twoFactorAuth: false, // Enable two-factor authentication (TOTP) for admin/app login

    // PWA (Progressive Web App) support
    // Enables: installable app, offline support, service worker caching
    // Uses @vite-pwa/nuxt module - requires rebuild after toggle
    pwa: false,

    // Contact form notifications
    // ⚠️ Email confirmations require SMTP_* in .env.
    // ⚠️ Telegram notifications require TELEGRAM_CHAT_ID in .env and contact.telegramBotToken in Admin Settings.
    contactEmailConfirmation: false, // Send confirmation email to user (requires SMTP_* in .env)
    contactTelegramNotify: false, // Send Telegram notification to admin

    // Footer features
    footerNav: true, // Show footer navigation links (from sections)
    footerCta: true, // Show CTA button in footer
    footerLegalLinks: true, // Show legal links (Privacy, Terms)
    footerMadeWith: true, // Show "Made with Puppet Master" branding
    backToTop: true, // Show back-to-top button on scroll

  },

  // ═══════════════════════════════════════════════════════════════════════════
  // MODULES - Pre-built, config-driven features
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // Each module provides: database tables, API endpoints, admin UI, frontend pages.
  // Enable modules per-project and customize their behavior via config.
  //
  // Available modules:
  //   - portfolio:    Project showcase with galleries and case studies
  //   - pricing:      Pricing tiers with comparison table
  //   - contact:      Contact form, map, and info display
  //   - blog:         Blog posts with categories, tags, and media
  //   - team:         Team member profiles with photos and social links
  //   - testimonials: Customer testimonials and reviews
  //   - features:     Feature cards with icons (replaces services)
  //   - clients:      Client/sponsor/partner logos
  //   - faq:          Frequently asked questions accordion
  //
  // ═══════════════════════════════════════════════════════════════════════════
  modules: {
    portfolio: {
      enabled: true,
      config: {
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
    },
    pricing: {
      enabled: true,
      config: {
        tiers: [
          {
            id: 'starter',
            name: 'Starter',
            description: 'Perfect for small projects',
            price: 0,
            period: 'month',
            currency: 'USD',
            featured: false,
            features: [
              { text: 'Up to 3 pages', included: true },
              { text: 'Basic blocks', included: true },
              { text: 'Community support', included: true },
              { text: 'Visual editor', included: false },
              { text: 'Custom modules', included: false }
            ],
            cta: { text: 'Get Started', url: '/contact' }
          },
          {
            id: 'pro',
            name: 'Pro',
            description: 'For growing businesses',
            price: 29,
            period: 'month',
            currency: 'USD',
            featured: true,
            features: [
              { text: 'Unlimited pages', included: true },
              { text: 'All blocks', included: true },
              { text: 'Priority support', included: true },
              { text: 'Visual editor', included: true },
              { text: 'Custom modules', included: false }
            ],
            cta: { text: 'Start Free Trial', url: '/contact' }
          },
          {
            id: 'enterprise',
            name: 'Enterprise',
            description: 'Custom solutions',
            price: null,
            period: 'month',
            currency: 'USD',
            featured: false,
            features: [
              { text: 'Unlimited pages', included: true },
              { text: 'All blocks', included: true },
              { text: 'Dedicated support', included: true },
              { text: 'Visual editor', included: true },
              { text: 'Custom modules', included: true }
            ],
            cta: { text: 'Contact Sales', url: '/contact' }
          }
        ],
        showComparison: true,
        showToggle: true,
        yearlyDiscount: 20,
        currency: 'USD',
        style: 'cards',
        ctaStyle: 'button',
        highlightFeatured: true,
        showFAQ: false
      }
    },
    contact: {
      enabled: true,
      config: {
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
    },
    blog: {
      enabled: true,
      config: {
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
    },
    team: {
      enabled: true,
      config: {
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
    },
    testimonials: {
      enabled: true,
      config: {
        layout: 'carousel',
        showRating: true,
        showPhoto: true,
        showCompany: true,
        autoPlay: false,
        autoPlayInterval: 5000
      }
    },
    features: {
      enabled: true,
      config: {
        layout: 'grid',
        cardStyle: 'icon-top',
        showCategory: false,
        columnsDesktop: 3,
        columnsMobile: 1
      }
    },
    clients: {
      enabled: true,
      config: {
        layout: 'strip',
        showNames: false,
        grayscale: true,
        hoverColor: true,
        categories: ['client', 'sponsor', 'partner']
      }
    },
    faq: {
      enabled: true,
      config: {
        layout: 'accordion',
        showCategories: false,
        expandFirst: true,
        allowMultipleOpen: false
      }
    }
  } as ModulesConfig,

  // ═══════════════════════════════════════════════════════════════════════════
  // HEADER CONTACT - Quick contact buttons in header (phone + messenger)
  // ═══════════════════════════════════════════════════════════════════════════
  // Best practice: Max 2 buttons - phone + primary messenger for instant contact.
  // Uses setting keys from the settings array below.
  // Only shown if the setting has a value in database.
  // ═══════════════════════════════════════════════════════════════════════════
  headerContact: {
    enabled: true,
    items: ['contact.phone', 'social.telegram'] as const
  },

  // ═══════════════════════════════════════════════════════════════════════════
  // STORAGE - File uploads configuration (images & videos)
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // Provider options:
  //   - 'local': Files stored in public/uploads/ (good for small/medium sites)
  //   - 's3': Files stored in S3-compatible bucket (AWS S3, Cloudflare R2, MinIO)
  //
  // For S3, set these environment variables in .env:
  //   S3_ENDPOINT, S3_BUCKET, S3_ACCESS_KEY, S3_SECRET_KEY, S3_REGION,
  //   S3_VISIBILITY (public|private), S3_PUBLIC_URL (public mode), S3_PROXY_SIGNING_KEY (optional)
  //
  // Video processing uses FFmpeg to compress and optimize for web playback.
  // ═══════════════════════════════════════════════════════════════════════════
  storage: {
    provider: 'local' as 'local' | 's3',

    // Image settings
    image: {
      maxSizeMB: 10, // Max upload size
      maxWidth: 1920, // Resize to max width
      maxHeight: 1080, // Resize to max height
      quality: 85, // WebP quality (1-100)
      thumbnailWidth: 400, // Thumbnail width
      thumbnailHeight: 300, // Thumbnail height
      thumbnailQuality: 75 // Thumbnail WebP quality
    },

    // Video settings
    video: {
      enabled: true, // Allow video uploads
      maxSizeMB: 100, // Max upload size
      maxDurationSeconds: 300, // Max 5 minutes
      allowedTypes: ['mp4', 'webm', 'mov', 'avi'],
      outputFormat: 'mp4' as 'mp4' | 'webm',
      // FFmpeg compression settings
      compression: {
        videoBitrate: '2M', // Target video bitrate
        audioBitrate: '128k', // Target audio bitrate
        maxWidth: 1920, // Scale down if larger
        fps: 30 // Target frame rate
      }
    }
  },

  // ═══════════════════════════════════════════════════════════════════════════
  // DATA SOURCE - Configure where application data comes from
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // Provider options:
  //   - 'database': All data from local SQLite (default, zero config)
  //   - 'api': All data from external REST API (requires .env credentials)
  //   - 'hybrid': Per-resource configuration (see resources below)
  //
  // For API provider, set these environment variables in .env:
  //   API_BASE_URL, API_CLIENT_ID, API_CLIENT_SECRET, API_TOKEN_URL
  //
  // Hybrid mode allows mixing sources (e.g., auth in DB, content from API)
  // ═══════════════════════════════════════════════════════════════════════════
  dataSource: {
    // Global provider
    provider: 'database' as 'database' | 'api' | 'hybrid',

    // Per-resource configuration (only used when provider = 'hybrid')
    resources: {
      users: 'database' as 'database' | 'api',
      sessions: 'database' as 'database' | 'api',
      settings: 'database' as 'database' | 'api',
      portfolio: 'database' as 'database' | 'api',
      contacts: 'database' as 'database' | 'api',
      translations: 'database' as 'database' | 'api'
    },

    // API client configuration
    api: {
      timeout: 30000, // Request timeout (ms)

      // Retry with exponential backoff
      retry: {
        maxAttempts: 3,
        initialDelay: 1000,
        maxDelay: 10000,
        backoffMultiplier: 2
      },

      // Circuit breaker (prevent cascading failures)
      circuitBreaker: {
        enabled: true,
        failureThreshold: 5, // Open after 5 consecutive failures
        resetTimeout: 60000 // Try again after 60s
      },

      // Response caching (per-resource TTL in seconds)
      cache: {
        enabled: true,
        ttl: {
          users: 300, // 5 min (rarely changes)
          sessions: 60, // 1 min (needs freshness for auth)
          settings: 600, // 10 min (mostly static)
          portfolio: 180, // 3 min (content updates)
          contacts: 0, // No cache (always fresh)
          translations: 3600 // 1 hour (rarely changes)
        }
      }
    }
  },

  // ═══════════════════════════════════════════════════════════════════════════
  // SECTIONS - Website navigation and routing (DRY Architecture)
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // This array controls:
  //   1. Main navigation links (items with inNav: true)
  //   2. Routes handled by [section].vue dynamic page
  //   3. Onepager mode anchor targets (#about, #portfolio, etc.)
  //
  // ┌─────────────────────────────────────────────────────────────────────────┐
  // │ HOW TO ADD A NEW SECTION (only 3 steps!)                               │
  // ├─────────────────────────────────────────────────────────────────────────┤
  // │ 1. Add entry here:  { id: 'history', inNav: true }                     │
  // │ 2. Create component: app/components/sections/SectionHistory.vue        │
  // │ 3. Add translations: nav.history, history.title, etc.                  │
  // │                                                                         │
  // │ That's it! SectionRenderer handles both onepager and SPA modes.        │
  // │ Component naming: Section{PascalCase(id)} → SectionHistory             │
  // └─────────────────────────────────────────────────────────────────────────┘
  //
  // ┌─────────────────────────────────────────────────────────────────────────┐
  // │ SELF-CONTAINED SECTIONS                                                │
  // ├─────────────────────────────────────────────────────────────────────────┤
  // │ Each section component should be self-contained:                       │
  // │   - Fetch its own title from i18n (or use title prop as override)      │
  // │   - Fetch its own content from i18n or API                             │
  // │   - Have sensible defaults for all content                             │
  // │                                                                         │
  // │ Example pattern:                                                        │
  // │   const { t, te } = useI18n()                                          │
  // │   const sectionTitle = computed(() =>                                  │
  // │     te('history.title') ? t('history.title') : 'Our History'           │
  // │   )                                                                    │
  // └─────────────────────────────────────────────────────────────────────────┘
  //
  // ┌─────────────────────────────────────────────────────────────────────────┐
  // │ CUSTOM PAGE (not a section)                                            │
  // ├─────────────────────────────────────────────────────────────────────────┤
  // │ Just create the file: app/pages/terms.vue → accessible at /terms       │
  // │ No config changes needed. Link it from footer or elsewhere.            │
  // │ Add to sections array only if you want it in navigation.               │
  // └─────────────────────────────────────────────────────────────────────────┘
  //
  // ┌─────────────────────────────────────────────────────────────────────────┐
  // │ ONEPAGER vs SPA MODE                                                   │
  // ├─────────────────────────────────────────────────────────────────────────┤
  // │ Onepager (features.onepager: true):                                    │
  // │   - All sections rendered on index.vue via SectionRenderer             │
  // │   - Nav links are anchors: #about, #portfolio                          │
  // │   - Scrollspy highlights active section                                │
  // │                                                                         │
  // │ SPA mode (features.onepager: false):                                   │
  // │   - Each section is a separate page: /about, /portfolio                │
  // │   - Nav links are routes                                               │
  // │   - [section].vue uses SectionRenderer for dynamic routing             │
  // └─────────────────────────────────────────────────────────────────────────┘
  //
  sections: defineWebsiteSections([
    { id: 'home', inNav: true },
    { id: 'about', inNav: true },
    { id: 'portfolio', inNav: true },
    { id: 'features', inNav: true },
    { id: 'team', inNav: true },
    { id: 'testimonials', inNav: true },
    { id: 'blog', inNav: true },
    { id: 'faq', inNav: true },
    { id: 'clients', inNav: true },
    { id: 'pricing', inNav: true },
    { id: 'contact', inNav: true }
  ] as const),

  // ═══════════════════════════════════════════════════════════════════════════
  // ADMIN - Content management configuration with per-module RBAC
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // Three categories of admin modules:
  //   1. SYSTEM — PM provides, universal (users, roles, translations, settings, health, logs)
  //   2. WEBSITE CONTENT — PM provides (sections, blog, portfolio, team, etc.)
  //   3. APP DATA — Developer builds custom admin pages per project
  //
  // Each module has:
  //   enabled — Show/hide in admin panel
  //   roles   — Which roles can access (empty = all authenticated)
  //
  // Role hierarchy: master → admin → editor → user
  // Role assignment is ALWAYS master-only (hardcoded for security)
  //
  // Usage in admin components:
  //   const sections = getAdminSections(config.admin)
  //   const userSections = filterSectionsByRole(sections, user.role)
  // ═══════════════════════════════════════════════════════════════════════════
  admin: {
    enabled: true,

    // System modules (PM provides, universal)
    // NOTE: Users module is MASTER-ONLY per CONTRIBUTING.md #47 (2026-02-26)
    // Even though config shows ['master'], the canManageUser() helper enforces:
    // - Master can manage all users
    // - Admin can manage admin/editor (not master) - legacy behavior, being phased out
    // For strict master-only: keep roles as ['master'] and enforce in API handlers
    system: {
      users: { enabled: true, roles: ['master'] }, // MASTER-ONLY per framework policy
      roles: { enabled: true, roles: ['master'] }, // ALWAYS master-only
      translations: { enabled: true, roles: ['master', 'admin', 'editor'] },
      settings: { enabled: true, roles: ['master', 'admin'] },
      health: { enabled: true, roles: ['master'] },
      logs: { enabled: false, roles: ['master'] }
    },

    // Website content modules (PM provides)
    websiteModules: {
      sections: { enabled: false, roles: ['master', 'admin', 'editor'] },
      blog: { enabled: true, roles: ['master', 'admin', 'editor'] },
      portfolio: { enabled: true, roles: ['master', 'admin'] },
      team: { enabled: true, roles: ['master', 'admin'] },
      testimonials: { enabled: true, roles: ['master', 'admin', 'editor'] },
      faq: { enabled: true, roles: ['master', 'admin', 'editor'] },
      clients: { enabled: true, roles: ['master', 'admin'] },
      pricing: { enabled: true, roles: ['master', 'admin'] },
      features: { enabled: true, roles: ['master', 'admin', 'editor'] },
      contacts: { enabled: true, roles: ['master', 'admin'] }
    },

    // App data modules (developer builds custom)
    // Add your custom admin modules here
    appModules: {
      // Example:
      // products: { enabled: true, roles: ['master', 'admin'] },
      // orders: { enabled: true, roles: ['master', 'admin', 'editor'] },
    }
  } as AdminConfig,

  // ═══════════════════════════════════════════════════════════════════════════
  // LANGUAGE & THEME DEFAULTS
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // Language detection priority:
  //   1. User's saved preference (cookie from previous visit)
  //   2. Browser language (if matches one of available locales)
  //   3. defaultLocale (fallback when browser lang not supported)
  //
  // Theme detection priority:
  //   1. User's saved preference (cookie from previous visit)
  //   2. defaultTheme setting below
  //   3. System preference (only when defaultTheme is 'system')
  //
  // RTL is auto-detected from locale code (he, ar, fa, ur)
  // Translations come from DATABASE - run `npm run db:seed` for initial data
  // ═══════════════════════════════════════════════════════════════════════════
  locales: [
    { code: 'en', iso: 'en-US', name: 'English' },
    { code: 'ru', iso: 'ru-RU', name: 'Русский' },
    { code: 'he', iso: 'he-IL', name: 'עברית' }
  ],
  defaultLocale: 'en',

  // Default theme: 'system' | 'light' | 'dark'
  // - 'system': Respect user's OS preference (recommended for most sites)
  // - 'light': Force light mode by default (e.g., clean corporate site)
  // - 'dark': Force dark mode by default (e.g., gaming/photography portfolio)
  defaultTheme: 'system' as 'system' | 'light' | 'dark',

  // ═══════════════════════════════════════════════════════════════════════════
  // LOGO CONFIGURATION
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // Naming convention: /logos/{shape}_{theme}_{lang}.svg
  //   Shape: 'horizontal' (header) or 'circle' (short/compact)
  //   Theme: 'light' (for dark backgrounds) or 'dark' (for light backgrounds)
  //   Lang:  'en', 'ru', 'he', etc.
  //
  // The system automatically picks the right logo based on:
  //   1. Current theme (light/dark mode)
  //   2. Current language
  //   3. Fallback chain if variant doesn't exist
  //
  // Usage in components:
  //   const { headerLogo, shortLogo } = useLogo()
  // ═══════════════════════════════════════════════════════════════════════════
  logo: {
    // Base path to logos (in public folder)
    basePath: '/logos',

    // Available shapes and where they're used
    shapes: {
      horizontal: 'header', // Full logo for desktop header
      circle: 'short' // Compact logo for sidebar, footer, icons
    },

    // Fallback chain for languages without their own logo
    // If Hebrew logo doesn't exist, use English (not Russian)
    langFallback: {
      he: 'en' // Hebrew → English
      // Add more fallbacks as needed
    },

    // Available logo files (for validation/preloading)
    // The system generates paths like: /logos/horizontal_dark_en.svg
    available: [
      'horizontal_dark_en',
      'horizontal_dark_ru',
      'horizontal_light_en',
      'horizontal_light_ru',
      'circle_dark_en',
      'circle_dark_ru',
      'circle_light_en',
      'circle_light_ru'
    ]
  },

  // Brand colors - the 4 primitives
  // Derived from Puppet Master logo design
  // Everything else is auto-calculated via CSS color-mix() and light-dark()
  colors: {
    black: '#2f2f2f', // Charcoal gray (from logo dark text)
    white: '#f0f0f0', // Off-white (from logo light text)
    brand: '#aa0000', // Dark red/maroon (from logo accent)
    accent: '#0f172a' // Deep slate (for contrast)
  },

  // ═══════════════════════════════════════════════════════════════════════════
  // SETTINGS SCHEMA - Config-Driven Settings Definition
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // MOTTO: "The config file is the developer's best friend!"
  //
  // This defines WHICH settings exist. VALUES are entered via Admin Panel.
  //
  // Schema:
  //   key      - Unique identifier (group.name format)
  //   type     - Input type: 'string', 'url', 'email', 'tel', 'text', 'boolean'
  //   group    - Admin panel grouping
  //   label    - Human-readable label for admin UI
  //   icon     - Tabler icon name (optional, for display in components)
  //
  // Components:
  //   <SocialNav />    - Renders all social.* with values (icons auto: brand-{platform})
  //   <ContactInfo />  - Renders all contact.* with values (icons from config)
  //
  // ═══════════════════════════════════════════════════════════════════════════
  settings: [
    // Contact Information (displayed via <ContactInfo /> component)
    // Icons are Tabler icon names: https://tabler.io/icons
    { key: 'contact.email', type: 'email', group: 'contact', label: 'Email', icon: 'mail' },
    { key: 'contact.phone', type: 'tel', group: 'contact', label: 'Phone', icon: 'phone' },
    // Location: empty=hidden, address text=show text, coordinates (lat,lng)=show Yandex map
    {
      key: 'contact.location',
      type: 'string',
      group: 'contact',
      label: 'Location',
      icon: 'map-pin'
    },
    {
      key: 'contact.telegramBotToken',
      type: 'password',
      group: 'contact',
      label: 'Telegram Bot Token',
      icon: 'lock',
      public: false
    },

    // Social Links (displayed via <SocialNav /> component)
    // Icons: Tabler brand icons (https://tabler.io/icons) or custom-* for custom icons
    // ─── Messaging ───
    {
      key: 'social.telegram',
      type: 'url',
      group: 'social',
      label: 'Telegram',
      icon: 'brand-telegram'
    },
    {
      key: 'social.whatsapp',
      type: 'url',
      group: 'social',
      label: 'WhatsApp',
      icon: 'brand-whatsapp'
    },
    { key: 'social.viber', type: 'url', group: 'social', label: 'Viber', icon: 'brand-viber' },
    {
      key: 'social.discord',
      type: 'url',
      group: 'social',
      label: 'Discord',
      icon: 'brand-discord'
    },
    { key: 'social.max', type: 'url', group: 'social', label: 'MAX Messenger', icon: 'custom-max' },
    // ─── Social Networks ───
    {
      key: 'social.instagram',
      type: 'url',
      group: 'social',
      label: 'Instagram',
      icon: 'brand-instagram'
    },
    {
      key: 'social.facebook',
      type: 'url',
      group: 'social',
      label: 'Facebook',
      icon: 'brand-facebook'
    },
    { key: 'social.twitter', type: 'url', group: 'social', label: 'Twitter/X', icon: 'brand-x' },
    {
      key: 'social.threads',
      type: 'url',
      group: 'social',
      label: 'Threads',
      icon: 'brand-threads'
    },
    { key: 'social.tiktok', type: 'url', group: 'social', label: 'TikTok', icon: 'brand-tiktok' },
    {
      key: 'social.pinterest',
      type: 'url',
      group: 'social',
      label: 'Pinterest',
      icon: 'brand-pinterest'
    },
    { key: 'social.vk', type: 'url', group: 'social', label: 'VK (ВКонтакте)', icon: 'brand-vk' },
    // ─── Video ───
    {
      key: 'social.youtube',
      type: 'url',
      group: 'social',
      label: 'YouTube',
      icon: 'brand-youtube'
    },
    { key: 'social.twitch', type: 'url', group: 'social', label: 'Twitch', icon: 'brand-twitch' },
    // ─── Professional ───
    {
      key: 'social.linkedin',
      type: 'url',
      group: 'social',
      label: 'LinkedIn',
      icon: 'brand-linkedin'
    },
    { key: 'social.medium', type: 'url', group: 'social', label: 'Medium', icon: 'brand-medium' },
    // ─── Dev/Design ───
    { key: 'social.github', type: 'url', group: 'social', label: 'GitHub', icon: 'brand-github' },
    { key: 'social.gitlab', type: 'url', group: 'social', label: 'GitLab', icon: 'brand-gitlab' },
    {
      key: 'social.dribbble',
      type: 'url',
      group: 'social',
      label: 'Dribbble',
      icon: 'brand-dribbble'
    },
    {
      key: 'social.behance',
      type: 'url',
      group: 'social',
      label: 'Behance',
      icon: 'brand-behance'
    },

    // Legal/Juridical Info (displayed in footer small print)
    // Russian: ИНН, ОГРН, Юридический адрес
    {
      key: 'legal.companyName',
      type: 'string',
      group: 'legal',
      label: 'Company Name (for copyright)'
    },
    { key: 'legal.inn', type: 'string', group: 'legal', label: 'ИНН (Tax ID)' },
    { key: 'legal.ogrn', type: 'string', group: 'legal', label: 'ОГРН (Registration Number)' },
    { key: 'legal.address', type: 'string', group: 'legal', label: 'Legal Address' },
    { key: 'legal.email', type: 'email', group: 'legal', label: 'Legal Email' },

    // Footer Settings (CTA text is in translations: cta.footerButton)
    { key: 'footer.ctaUrl', type: 'url', group: 'footer', label: 'CTA Button URL' },
    { key: 'footer.privacyUrl', type: 'url', group: 'footer', label: 'Privacy Policy URL' },
    { key: 'footer.termsUrl', type: 'url', group: 'footer', label: 'Terms of Service URL' },

    // SEO & Meta
    { key: 'seo.title', type: 'string', group: 'seo', label: 'Default Page Title' },
    { key: 'seo.description', type: 'text', group: 'seo', label: 'Meta Description' },
    { key: 'seo.keywords', type: 'string', group: 'seo', label: 'Meta Keywords (comma-separated)' },

    // Analytics (IDs only - scripts injected by plugins)
    {
      key: 'analytics.googleId',
      type: 'string',
      group: 'analytics',
      label: 'Google Analytics ID (G-XXXXXXXX)'
    },
    { key: 'analytics.yandexId', type: 'string', group: 'analytics', label: 'Yandex Metrica ID' },
    {
      key: 'analytics.facebookPixel',
      type: 'string',
      group: 'analytics',
      label: 'Facebook Pixel ID'
    },

    // Verification codes (for webmaster tools)
    {
      key: 'verification.google',
      type: 'string',
      group: 'verification',
      label: 'Google Search Console'
    },
    { key: 'verification.yandex', type: 'string', group: 'verification', label: 'Yandex Webmaster' }
  ] as const,

  // Setting groups (for admin UI organization)
  // Labels are i18n keys - use t(group.label) in templates
  settingGroups: [
    { key: 'contact', label: 'admin.settingsContact', icon: 'address-book' },
    { key: 'social', label: 'admin.settingsSocial', icon: 'share' },
    { key: 'legal', label: 'admin.settingsLegal', icon: 'file-certificate' },
    { key: 'footer', label: 'admin.settingsFooter', icon: 'layout-bottombar' },
    { key: 'seo', label: 'admin.settingsSeo', icon: 'search' },
    { key: 'analytics', label: 'admin.settingsAnalytics', icon: 'chart-bar' },
    { key: 'verification', label: 'admin.settingsVerification', icon: 'certificate' }
  ] as const,

  // ═══════════════════════════════════════════════════════════════════════════
  // COMPUTED HELPERS - Convenient access to derived values
  // ═══════════════════════════════════════════════════════════════════════════

  // Entity-based helpers (replaces old mode system)
  get hasWebsite(): boolean {
    return this.entities.website
  },

  get hasApp(): boolean {
    return this.entities.app
  },

  get hasAdmin(): boolean {
    return this.admin.enabled
  },

  get hasLoginButton(): boolean {
    // Show login button when both website and app exist
    return this.entities.website && this.entities.app
  },

  get isAppOnly(): boolean {
    return !this.entities.website && this.entities.app
  },

  get isWebsiteOnly(): boolean {
    return this.entities.website && !this.entities.app && !this.admin.enabled
  },

  // Feature helpers
  get isMultiLang(): boolean {
    return this.features.multiLangs && this.locales.length > 1
  },

  get hasThemeToggle(): boolean {
    return this.features.doubleTheme
  },

  get has2FA(): boolean {
    return this.features.twoFactorAuth
  },

  // Combined helpers (entities + features)
  get useOnepager(): boolean {
    // Onepager applies to the website portion (when it exists)
    return this.hasWebsite && this.features.onepager
  },

  get useInteractiveHeader(): boolean {
    return this.hasWebsite && this.features.interactiveHeader
  },

  /**
   * Canonical header/footer inner width mode with legacy alias fallback.
   *
   * Legacy alias supported through at least one minor release:
   *   features.websiteChromeWidth
   *
   * Planned removal target for legacy alias: 1.4.0.
   */
  get headerFooterWidthMode(): 'contained' | 'full' {
    const next = (this.features as { headerFooterWidthMode?: 'contained' | 'full' }).headerFooterWidthMode
    if (next === 'contained' || next === 'full') {
      return next
    }

    const legacy = (this.features as { websiteChromeWidth?: 'contained' | 'full' }).websiteChromeWidth
    if (legacy === 'contained' || legacy === 'full') {
      return legacy
    }

    return 'contained'
  },

  // Admin section helpers
  getAdminSections(): AdminSection[] {
    return getAdminSections(this.admin)
  },

  getAdminSectionsForRole(role: UserRole): AdminSection[] {
    return filterSectionsByRole(this.getAdminSections(), role)
  }
}

export default config
