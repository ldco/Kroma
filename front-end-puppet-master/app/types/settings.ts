/**
 * Site Settings Types
 *
 * Runtime settings fetched from the API.
 * Groups: contact, social, legal, footer, seo, analytics, verification
 */

export interface SiteSettings {
  contact: {
    email: string | null
    phone: string | null
    location: string | null // empty=hidden, address text, or "lat,lng" for map
  }
  social: {
    // Messaging
    telegram: string | null
    whatsapp: string | null
    viber: string | null
    discord: string | null
    max: string | null
    // Social networks
    instagram: string | null
    facebook: string | null
    twitter: string | null
    threads: string | null
    tiktok: string | null
    pinterest: string | null
    vk: string | null
    // Video
    youtube: string | null
    twitch: string | null
    // Professional
    linkedin: string | null
    medium: string | null
    // Dev/Design
    github: string | null
    gitlab: string | null
    dribbble: string | null
    behance: string | null
  }
  legal: {
    companyName: string | null // For copyright: © 2024 {companyName}
    inn: string | null // Tax ID
    ogrn: string | null // Registration Number
    address: string | null // Legal address
    email: string | null // Legal email
  }
  footer: {
    ctaUrl: string | null // CTA button URL (text is in translations: cta.footerButton)
    privacyUrl: string | null // Privacy policy URL
    termsUrl: string | null // Terms of service URL
  }
  seo: {
    title: string | null // Page title (also used as site name fallback)
    description: string | null // SEO description
    keywords: string | null // Meta keywords
  }
  analytics: {
    googleId: string | null // Google Analytics ID (G-XXXXXXXX)
    yandexId: string | null // Yandex Metrica ID
    facebookPixel: string | null // Facebook Pixel ID
  }
  verification: {
    google: string | null // Google Search Console
    yandex: string | null // Yandex Webmaster
  }
}
