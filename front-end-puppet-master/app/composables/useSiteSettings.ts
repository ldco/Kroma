/**
 * useSiteSettings Composable
 *
 * Fetches runtime site settings from the API.
 * Settings are cached and can be refreshed on demand.
 *
 * Groups: site, contact, social, seo
 */
import type { SiteSettings } from '~/types'

// Re-export for backward compatibility
export type { SiteSettings } from '~/types'

const defaultSettings: SiteSettings = {
  contact: {
    email: null,
    phone: null,
    location: null
  },
  social: {
    // Messaging
    telegram: null,
    whatsapp: null,
    viber: null,
    discord: null,
    max: null,
    // Social networks
    instagram: null,
    facebook: null,
    twitter: null,
    threads: null,
    tiktok: null,
    pinterest: null,
    vk: null,
    // Video
    youtube: null,
    twitch: null,
    // Professional
    linkedin: null,
    medium: null,
    // Dev/Design
    github: null,
    gitlab: null,
    dribbble: null,
    behance: null
  },
  legal: {
    companyName: null,
    inn: null,
    ogrn: null,
    address: null,
    email: null
  },
  footer: {
    ctaUrl: null, // Text is in translations: cta.footerButton
    privacyUrl: null,
    termsUrl: null
  },
  seo: {
    title: null,
    description: null,
    keywords: null
  },
  analytics: {
    googleId: null,
    yandexId: null,
    facebookPixel: null
  },
  verification: {
    google: null,
    yandex: null
  }
}

export function useSiteSettings() {
  // Use Nuxt's data fetching with caching
  const { data, pending, error, refresh } = useFetch('/api/settings', {
    key: 'site-settings',
    default: () => defaultSettings as SiteSettings,
    transform: (raw): SiteSettings => {
      // Merge API response with defaults
      const envelope = raw as
        | { success?: boolean; data?: Record<string, Record<string, string | null>> }
        | Record<string, Record<string, string | null>>
      const rawData =
        'data' in envelope && envelope.data
          ? envelope.data
          : (envelope as Record<string, Record<string, string | null>>)
      return {
        contact: {
          ...defaultSettings.contact,
          ...rawData.contact
        },
        social: {
          ...defaultSettings.social,
          ...rawData.social
        },
        legal: {
          ...defaultSettings.legal,
          ...rawData.legal
        },
        footer: {
          ...defaultSettings.footer,
          ...rawData.footer
        },
        seo: {
          ...defaultSettings.seo,
          ...rawData.seo
        },
        analytics: {
          ...defaultSettings.analytics,
          ...rawData.analytics
        },
        verification: {
          ...defaultSettings.verification,
          ...rawData.verification
        }
      } as SiteSettings
    }
  })

  // Computed helpers for common use cases
  // Site name: legal.companyName → seo.title → 'Puppet Master'
  const siteName = computed(
    () => data.value?.legal.companyName || data.value?.seo.title || 'Puppet Master'
  )
  const contactEmail = computed(() => data.value?.contact.email)
  const contactPhone = computed(() => data.value?.contact.phone)

  // Social links as array for iteration
  const socialLinks = computed(() => {
    const social = data.value?.social || {}
    return Object.entries(social)
      .filter(([_, url]) => url && url.trim() !== '')
      .map(([platform, url]) => ({ platform, url: url as string }))
  })

  // SEO meta
  const seoTitle = computed(() => data.value?.seo.title || siteName.value)
  const seoDescription = computed(() => data.value?.seo.description)

  return {
    // Raw data
    settings: data,
    isLoading: pending,
    error,
    refresh,

    // Helpers
    siteName,
    contactEmail,
    contactPhone,
    socialLinks,
    seoTitle,
    seoDescription
  }
}
