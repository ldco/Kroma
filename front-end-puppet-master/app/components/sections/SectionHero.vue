<script setup lang="ts">
/**
 * Hero Section
 *
 * Main landing section with big logo, subtext, and CTAs.
 *
 * Logo Strategy: Render BOTH theme variants, CSS shows correct one.
 * This avoids SSR hydration mismatch AND flash of wrong logo.
 *
 * Parallax Support: Optionally includes HeroParallaxScene for multi-layer
 * parallax backgrounds. Enable with the `parallax` prop.
 */
import config from '~/puppet-master.config'

const { locale } = useI18n()

// SSR-safe config access with defensive defaults
const logoConfig = computed(() => config?.logo ?? { basePath: '/logos', available: [], langFallback: {} })
const defaultLocale = computed(() => config?.defaultLocale || 'en')

// Get effective language for logo (fallback for languages without logos)
const langSuffix = computed(() => {
  const lang = locale.value
  // Check if logo exists for this language
  const hasLogo = logoConfig.value.available.some((name: string) => name.endsWith(`_${lang}`))
  if (hasLogo) return lang
  // Use fallback chain
  const fallback = logoConfig.value.langFallback as Record<string, string>
  return fallback[lang] || defaultLocale.value
})

// Get logo paths for both themes
const lightThemeLogo = computed(() => `${logoConfig.value.basePath}/circle_dark_${langSuffix.value}.svg`)
const darkThemeLogo = computed(() => `${logoConfig.value.basePath}/circle_light_${langSuffix.value}.svg`)

interface Props {
  /** Supporting text */
  subtitle?: string
  /** Primary CTA text */
  primaryCta?: string
  /** Primary CTA link */
  primaryLink?: string
  /** Secondary CTA text */
  secondaryCta?: string
  /** Secondary CTA link */
  secondaryLink?: string
  /** Enable parallax background layers */
  parallax?: boolean
}

defineProps<Props>()
</script>

<template>
  <!--
    Uses global classes from:
    - layout/sections.css (.section, .section-hero, .section-hero--center, .hero-actions)
    - typography/base.css (.hero-subtitle)
    - ui/content/hero-parallax.css (parallax layers)
  -->
  <section id="home" class="section section-hero section-hero--center">
    <!-- Parallax background layers (optional) -->
    <ClientOnly v-if="parallax">
      <MoleculesHeroParallaxScene />
    </ClientOnly>

    <!-- Floating decorations for candy effect -->
    <div class="hero-decorations" aria-hidden="true">
      <!-- Animated blobs -->
      <div class="hero-blob hero-blob--1" />
      <div class="hero-blob hero-blob--2" />
      <div class="hero-blob hero-blob--3" />
      <!-- Geometric shapes -->
      <div class="hero-shape hero-shape--circle" />
      <div class="hero-shape hero-shape--square" />
      <div class="hero-shape hero-shape--triangle" />
      <!-- Floating dots -->
      <div class="hero-dots hero-dots--1" />
      <div class="hero-dots hero-dots--2" />
      <div class="hero-dots hero-dots--3" />
      <div class="hero-dots hero-dots--4" />
    </div>

    <div class="container">
      <div v-reveal="'scale'" class="hero-logo">
        <slot name="logo">
          <!-- Render BOTH logos, CSS shows correct one based on theme class -->
          <!-- Comment 5: Add fetchpriority and loading for LCP optimization -->
          <img
            :src="lightThemeLogo"
            alt="Logo"
            class="hero-logo-img hero-logo-img--light"
            width="200"
            height="200"
            fetchpriority="high"
            loading="eager"
            decoding="async"
          />
          <img
            :src="darkThemeLogo"
            alt="Logo"
            class="hero-logo-img hero-logo-img--dark"
            width="200"
            height="200"
            fetchpriority="high"
            loading="eager"
            decoding="async"
          />
        </slot>
      </div>
      <p v-if="subtitle" v-reveal="{ animation: 'fade-up', delay: 100 }" class="hero-subtitle">
        <slot name="subtitle">{{ subtitle }}</slot>
      </p>
      <div v-if="primaryCta || secondaryCta" v-reveal="{ animation: 'fade-up', delay: 200 }" class="hero-actions">
        <slot name="actions">
          <AtomsCtaButton
            v-if="primaryCta && primaryLink"
            :to="primaryLink"
            variant="primary"
            size="lg"
          >
            {{ primaryCta }}
          </AtomsCtaButton>
          <AtomsCtaButton
            v-if="secondaryCta && secondaryLink"
            :to="secondaryLink"
            variant="outline"
            size="lg"
          >
            {{ secondaryCta }}
          </AtomsCtaButton>
        </slot>
      </div>
    </div>
  </section>
</template>

<!-- No scoped styles needed - all styles come from global CSS -->
