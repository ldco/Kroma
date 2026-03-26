<script setup lang="ts">
/**
 * Pricing Tiers Component
 *
 * Displays all pricing tiers with optional billing toggle.
 * Fetches tiers from database API, falls back to config if empty.
 * All UI text uses i18n for translations.
 */

// Define tier type for component (matches API response)
interface PricingTierData {
  id: number
  slug: string
  name: string
  description: string | null
  price: number | null // Already in dollars from API
  currency: string | null
  period: 'month' | 'year' | 'one-time' | null
  featured: boolean | null
  ctaText: string | null
  ctaUrl: string | null
  order: number | null
  features: Array<{
    id: number
    text: string
    included: boolean | null
  }>
}

const { t, locale } = useI18n()

const props = withDefaults(defineProps<{
  /** Show monthly/yearly toggle */
  showToggle?: boolean
  /** Yearly discount percentage */
  yearlyDiscount?: number
  /** Default currency */
  currency?: string
  /** Highlight featured tier */
  highlightFeatured?: boolean
}>(), {
  showToggle: true,
  yearlyDiscount: 20,
  currency: 'USD',
  highlightFeatured: true
})

// Get config from module system
const { config: moduleConfig, isEnabled } = useModule('pricing')

// Fetch tiers from database with current locale
const { data: dbTiers } = await useFetch<PricingTierData[]>('/api/pricing', {
  key: `pricing-tiers-${locale.value}`,
  query: { locale: locale.value },
  watch: [locale]
})

// Use database tiers if available, otherwise fall back to config
const effectiveTiers = computed(() => {
  if (dbTiers.value && dbTiers.value.length > 0) {
    return dbTiers.value
  }
  // Fall back to config tiers (convert to same shape)
  const configTiers = moduleConfig.value.tiers || []
  return configTiers.map((tier: any, index: number) => ({
    id: index,
    slug: tier.id || `tier-${index}`,
    name: tier.name,
    description: tier.description || null,
    price: tier.price,
    currency: tier.currency || 'USD',
    period: tier.period || 'month',
    featured: tier.featured || false,
    ctaText: tier.cta?.text || null,
    ctaUrl: tier.cta?.url || '/contact',
    order: index,
    features: (tier.features || []).map((f: any, i: number) => ({
      id: i,
      text: f.text,
      included: f.included ?? true
    }))
  }))
})

const effectiveShowToggle = computed(() => {
  return props.showToggle ?? moduleConfig.value.showToggle
})

const effectiveYearlyDiscount = computed(() => {
  return props.yearlyDiscount ?? moduleConfig.value.yearlyDiscount
})

// Billing period state
const billingPeriod = ref<'month' | 'year'>('month')

// Check if any tier has yearly option
const hasYearlyOption = computed(() => {
  return effectiveTiers.value.some(tier => tier.period === 'month' || tier.period === 'year')
})
</script>

<template>
  <div v-if="isEnabled" class="pricing-tiers" v-reveal>
    <!-- Billing Toggle -->
    <div v-if="effectiveShowToggle && hasYearlyOption" class="pricing-toggle">
      <button
        class="pricing-toggle__btn"
        :class="{ 'pricing-toggle__btn--active': billingPeriod === 'month' }"
        @click="billingPeriod = 'month'"
      >
        {{ t('pricing.monthly') }}
      </button>
      <button
        class="pricing-toggle__btn"
        :class="{ 'pricing-toggle__btn--active': billingPeriod === 'year' }"
        @click="billingPeriod = 'year'"
      >
        {{ t('pricing.yearly') }}
        <span v-if="effectiveYearlyDiscount" class="pricing-toggle__discount">
          {{ t('pricing.savePercent', { percent: effectiveYearlyDiscount }) }}
        </span>
      </button>
    </div>

    <!-- Tiers Grid -->
    <div class="pricing-tiers__grid">
      <MoleculesPricingCard
        v-for="tier in effectiveTiers"
        :key="tier.id"
        :tier="tier"
        :billing-period="billingPeriod"
        :yearly-discount="effectiveYearlyDiscount"
        :currency="props.currency"
        :highlighted="highlightFeatured && tier.featured"
      />
    </div>
  </div>
</template>
