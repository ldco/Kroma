// https://nuxt.com/docs/api/configuration/nuxt-config
import { resolve } from 'path'
import config from './app/puppet-master.config'
import Icons from 'unplugin-icons/vite'
import svgLoader from 'vite-svg-loader'

export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  // DevTools only enabled in development, not production (Comment 2)
  devtools: { enabled: process.env.NODE_ENV !== 'production' },

  // Alias for cleaner imports across the app
  // Components can use: import config from '@config'
  alias: {
    '@config': resolve(__dirname, './app/puppet-master.config.ts')
  },

  // CSS entry point - our custom CSS system
  css: ['~/assets/css/main.css'],

  // App head configuration
  app: {
    head: {
      // Viewport meta - includes viewport-fit=cover for iOS safe-area-inset support
      // Required for PWA and proper bottom nav handling with iOS notch/home indicator
      meta: [
        { name: 'viewport', content: 'width=device-width, initial-scale=1, viewport-fit=cover' }
      ],
      // Favicons - generated via `npm run generate:favicons`
      link: [
        { rel: 'icon', href: '/favicon.ico', sizes: 'any' },
        { rel: 'icon', href: '/favicon.svg', type: 'image/svg+xml' },
        { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
        // Google Fonts - Montserrat (matches logo design)
        // Only load weights we actually use: 400, 500, 600, 700, 900
        // Preconnect for faster DNS/TLS handshake
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        // Comment 8: Preload Google Fonts CSS for SSR (reduces FOUT)
        {
          rel: 'preload',
          as: 'style',
          href: 'https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;600;700;900&display=swap'
        }
        // Font stylesheet is loaded via plugins/fonts.client.ts (CSP-compliant, non-render-blocking)
      ]
    },
    // Page transitions (SPA mode only - disabled in onepager mode)
    // Transition name maps to CSS classes: page-{name}-enter-active, etc.
    ...(config.features.pageTransitions && !config.features.onepager
      ? {
          pageTransition: {
            name: `page-${config.features.pageTransitions}`,
            mode: 'out-in'
          }
        }
      : {})
  },

  // Modules
  modules: [
    '@nuxtjs/color-mode',
    '@nuxtjs/i18n',
    '@pinia/nuxt',
    // Critical CSS inlining - only in production (causes memory issues in dev)
    ...(process.env.NODE_ENV === 'production' ? ['@nuxtjs/critters'] : []),
    // PWA module - conditionally loaded based on config.features.pwa
    ...(config.features.pwa ? ['@vite-pwa/nuxt'] : [])
  ],

  // PWA configuration (only applied when config.features.pwa is true)
  // Uses favicon assets generated via `npm run generate:favicons`
  ...(config.features.pwa
    ? {
        pwa: {
          registerType: 'autoUpdate',
          manifest: {
            name: 'Puppet Master',
            short_name: 'PM',
            description: 'Puppet Master Application',
            theme_color: config.colors.brand,
            background_color: config.colors.white,
            display: 'standalone',
            icons: [
              {
                src: '/favicon-192.png',
                sizes: '192x192',
                type: 'image/png'
              },
              {
                src: '/favicon-512.png',
                sizes: '512x512',
                type: 'image/png'
              },
              {
                src: '/favicon-512.png',
                sizes: '512x512',
                type: 'image/png',
                purpose: 'maskable'
              }
            ]
          },
          workbox: {
            // Cache static assets
            globPatterns: ['**/*.{js,css,html,png,jpg,jpeg,svg,ico,woff,woff2}'],
            // Runtime caching for API calls
            runtimeCaching: [
              {
                urlPattern: /^https:\/\/fonts\.googleapis\.com\/.*/i,
                handler: 'CacheFirst',
                options: {
                  cacheName: 'google-fonts-cache',
                  expiration: {
                    maxEntries: 10,
                    maxAgeSeconds: 60 * 60 * 24 * 365 // 1 year
                  },
                  cacheableResponse: {
                    statuses: [0, 200]
                  }
                }
              },
              {
                urlPattern: /^https:\/\/fonts\.gstatic\.com\/.*/i,
                handler: 'CacheFirst',
                options: {
                  cacheName: 'gstatic-fonts-cache',
                  expiration: {
                    maxEntries: 10,
                    maxAgeSeconds: 60 * 60 * 24 * 365 // 1 year
                  },
                  cacheableResponse: {
                    statuses: [0, 200]
                  }
                }
              }
            ]
          },
          client: {
            installPrompt: true,
            periodicSyncForUpdates: 3600 // Check for updates every hour
          },
          devOptions: {
            enabled: false, // Disable in dev by default (set true to test PWA locally)
            type: 'module'
          }
        }
      }
    : {}),

  // Color mode configuration
  // Uses defaultTheme from puppet-master.config.ts
  colorMode: {
    preference: config.defaultTheme,
    fallback: config.defaultTheme === 'system' ? 'light' : config.defaultTheme,
    classSuffix: '',
    storageKey: 'pm-color-mode'
  },

  // i18n configuration
  // NOTE: Translations come from DATABASE via API, not per-locale files!
  // A single loader.ts handles ALL locales - no manual file creation needed
  i18n: {
    locales: config.locales.map(l => ({
      ...l,
      file: '../../i18n/loader.ts',
      preload: l.code === config.defaultLocale
    })),
    lazy: true,
    langDir: 'app',
    defaultLocale: config.defaultLocale,
    strategy: 'prefix_except_default',
    detectBrowserLanguage: {
      useCookie: true,
      cookieKey: 'pm-i18n-redirected',
      fallbackLocale: config.defaultLocale,
      alwaysRedirect: false,
      redirectOn: 'root'
    },
    bundle: {
      fullInstall: false,
      dropMessageCompiler: true
    },
    compilation: {
      strictMessage: true
    },
    vueI18n: './i18n/i18n.config.ts'
  },

  // PostCSS configuration
  // postcss-preset-env includes autoprefixer + 50+ plugins for modern CSS fallbacks
  postcss: {
    plugins: {
      'postcss-preset-env': {
        // Stage 2 = reasonable stability (default)
        // Stage 3 = more stable, fewer features
        stage: 2,
        // Require 2 browser implementations for extra stability
        minimumVendorImplementations: 2,
        // Enable specific features we use
        features: {
          'custom-media-queries': true, // @custom-media support
          'nesting-rules': true, // CSS nesting
          'color-mix': true, // color-mix() fallbacks
          'custom-properties': true // CSS variables fallback for older browsers
        },
        // Autoprefixer options
        autoprefixer: {
          flexbox: 'no-2009' // Don't generate old flexbox syntax
        }
      },
      // PurgeCSS - remove unused CSS for smaller bundle size
      // Only in production to speed up dev builds
      // Fixed paths for Nuxt 4 app directory structure (Comment 11)
      ...(process.env.NODE_ENV === 'production'
        ? {
            '@fullhuman/postcss-purgecss': {
              content: [
                './app/**/*.vue',
                './app/**/*.ts'
              ],
              // Preserve dynamic classes and CSS variables
              safelist: {
                standard: [
                  /^dark/,
                  /^light/,
                  /^page-/,
                  /^router-/,
                  /^nuxt-/,
                  /^v-/,
                  /^hamburger/,
                  /-enter/,
                  /-leave/,
                  /-active/,
                  /data-/
                ],
                // Preserve :lang() selectors for multi-language CSS rules
                deep: [/:lang/],
                // Keep CSS custom properties
                variables: [/^--/]
              },
              // Extract classes from Vue SFC
              defaultExtractor: (content: string) => {
                const contentWithoutStyleBlocks = content.replace(/<style[^]+?<\/style>/gi, '')
                return contentWithoutStyleBlocks.match(/[\w-/:]+(?<!:)/g) || []
              }
            }
          }
        : {})
    }
  },

  // Experimental features
  experimental: {
    // Note: inlineStyles was removed in Nuxt 4 - CSS is optimized automatically
  },

  // Vite configuration
  vite: {
    // Dev build optimizations
    build: {
      // Skip minification in dev for faster builds
      minify: process.env.NODE_ENV === 'production',
      // Reduce sourcemap memory in dev
      sourcemap: process.env.NODE_ENV === 'production'
    },
    // Optimize deps loading
    optimizeDeps: {
      include: ['vue', 'vue-router', 'pinia']
    },
    plugins: [
      Icons({
        compiler: 'vue3',
        autoInstall: true
      }),
      svgLoader()
    ]
  },

  // Nitro configuration
  nitro: {
    // Reduce workers in dev to save memory
    ...(process.env.NODE_ENV !== 'production' ? { workers: 1 } : {}),
    // Scheduled tasks (MED-02: Session cleanup)
    scheduledTasks: {
      // Run session cleanup every hour at minute 0
      '0 * * * *': ['cleanup-sessions']
    },
    // Experimental features required for scheduled tasks
    experimental: {
      tasks: true
    },
    // Route rules for caching static assets and security limits
    routeRules: {
      // Static assets - long cache (1 year, immutable since they have hashed names)
      '/_nuxt/**': {
        headers: {
          'Cache-Control': 'public, max-age=31536000, immutable'
        }
      },
      // Favicons and static files
      '/favicon.*': {
        headers: {
          'Cache-Control': 'public, max-age=86400'
        }
      },
      '/apple-touch-icon.png': {
        headers: {
          'Cache-Control': 'public, max-age=86400'
        }
      }
      // Note: Body size limits for /api/setup/import-zip are enforced at the handler level
      // via Content-Length header check (first line of defense before body parsing)
    }
  },

  // Runtime config
  runtimeConfig: {
    // Database
    databaseUrl: process.env.DATABASE_URL || './data/sqlite.db',

    // Email (SMTP)
    smtpHost: process.env.SMTP_HOST || '',
    smtpPort: process.env.SMTP_PORT || '465',
    smtpUser: process.env.SMTP_USER || '',
    smtpPass: process.env.SMTP_PASS || '',
    smtpFrom: process.env.SMTP_FROM || '',

    // Telegram Bot
    telegramBotToken: process.env.TELEGRAM_BOT_TOKEN || '',
    telegramChatId: process.env.TELEGRAM_CHAT_ID || '',

    // External API (for dataSource.provider = 'api' or 'hybrid')
    apiBaseUrl: process.env.API_BASE_URL || '',
    apiClientId: process.env.API_CLIENT_ID || '',
    apiClientSecret: process.env.API_CLIENT_SECRET || '',
    apiTokenUrl: process.env.API_TOKEN_URL || '',
    apiJwtToken: process.env.API_JWT_TOKEN || '',
    apiKey: process.env.API_KEY || '',
    apiTokenRefreshBuffer: process.env.API_TOKEN_REFRESH_BUFFER || '300',

    // Redis (for distributed cache in multi-instance deployments)
    redisUrl: process.env.REDIS_URL || '',
    redisPrefix: process.env.REDIS_PREFIX || 'pm-cache:',

    // Site domain (used for constructing URLs)
    siteDomain: process.env.SITE_DOMAIN || '',

    public: {
      features: config.features,
      // PM Mode - env var overrides config file (allows framework devs to set PM_MODE=develop)
      pmMode: process.env.PM_MODE || config.pmMode || 'unconfigured',
      // Kroma Backend API Base URL - environment-driven API routing
      kromaApiBaseUrl: process.env.KROMA_API_BASE_URL || 'http://127.0.0.1:8788',
      // External monitoring dashboard URL (constructed from subdomain + domain)
      uptimeKumaUrl:
        process.env.UPTIME_KUMA_SUBDOMAIN && process.env.SITE_DOMAIN
          ? `https://${process.env.UPTIME_KUMA_SUBDOMAIN}.${process.env.SITE_DOMAIN}`
          : ''
    }
  }
})
