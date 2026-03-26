<script setup lang="ts">
/**
 * Pricing Card Component
 *
 * Individual pricing tier card with features list and CTA.
 * Uses i18n for all static text.
 */
import IconCheck from '~icons/tabler/check'
import IconX from '~icons/tabler/x'

// Tier type from database/API
interface PricingTierData {
  id: number
  slug: string
  name: string
  description: string | null
  price: number | null
  currency: string | null
  period: 'month' | 'year' | 'one-time' | null
  featured: boolean | null
  ctaText: string | null
  ctaUrl: string | null
  features: Array<{
    id: number
    text: string
    included: boolean | null
  }>
}

const { t } = useI18n()

const props = defineProps<{
  tier: PricingTierData
  highlighted?: boolean
  billingPeriod?: 'month' | 'year'
  yearlyDiscount?: number
  currency?: string
}>()

// Calculate price based on billing period
const displayPrice = computed(() => {
  if (props.tier.price === null) return null

  const basePrice = props.tier.price
  if (props.billingPeriod === 'year' && props.yearlyDiscount) {
    const yearlyPrice = basePrice * 12
    const discountedPrice = yearlyPrice * (1 - props.yearlyDiscount / 100)
    return Math.round(discountedPrice / 12)
  }
  return basePrice
})

const isEnterprise = computed(() => props.tier.price === null)

// Format currency
const formattedPrice = computed(() => {
  if (displayPrice.value === null) return null
  const currency = props.currency || props.tier.currency || 'USD'

  // Simple currency formatting
  const symbols: Record<string, string> = {
    USD: '$',
    EUR: '€',
    GBP: '£',
    RUB: '₽',
    ILS: '₪'
  }
  const symbol = symbols[currency] || currency
  return `${symbol}${displayPrice.value}`
})

const periodLabel = computed(() => {
  if (props.tier.period === 'one-time') return t('pricing.oneTime')
  return props.billingPeriod === 'year' ? t('pricing.perMonth') : t('pricing.perMonth')
})

// CTA text with fallback
const ctaText = computed(() => {
  if (props.tier.ctaText) return props.tier.ctaText
  return isEnterprise.value ? t('pricing.contactSales') : t('pricing.getStarted')
})
</script>

<template>
  <div
    class="pricing-card"
    :class="{
      'pricing-card--featured': tier.featured || highlighted,
      'pricing-card--enterprise': isEnterprise
    }"
  >
    <!-- Featured badge -->
    <div v-if="tier.featured" class="pricing-card__badge">
      {{ t('pricing.mostPopular') }}
    </div>

    <!-- Header -->
    <div class="pricing-card__header">
      <h3 class="pricing-card__name">{{ tier.name }}</h3>
      <p v-if="tier.description" class="pricing-card__description">
        {{ tier.description }}
      </p>
    </div>

    <!-- Price -->
    <div class="pricing-card__price">
      <template v-if="isEnterprise">
        <span class="pricing-card__price-custom">{{ t('pricing.custom') }}</span>
      </template>
      <template v-else-if="displayPrice === 0">
        <span class="pricing-card__price-value">{{ t('pricing.free') }}</span>
      </template>
      <template v-else>
        <span class="pricing-card__price-value">{{ formattedPrice }}</span>
        <span class="pricing-card__price-period">{{ periodLabel }}</span>
      </template>
    </div>

    <!-- Features -->
    <ul class="pricing-card__features">
      <li
        v-for="feature in tier.features"
        :key="feature.id"
        class="pricing-card__feature"
        :class="{ 'pricing-card__feature--excluded': !feature.included }"
      >
        <span class="pricing-card__feature-icon">
          <IconCheck v-if="feature.included" />
          <IconX v-else />
        </span>
        <span class="pricing-card__feature-text">{{ feature.text }}</span>
      </li>
    </ul>

    <!-- CTA -->
    <div class="pricing-card__cta">
      <NuxtLink
        :to="tier.ctaUrl || '/contact'"
        class="btn"
        :class="tier.featured ? 'btn-primary' : 'btn-secondary'"
      >
        {{ ctaText }}
      </NuxtLink>
    </div>
  </div>
</template>
