<script setup lang="ts">
/**
 * Pricing Comparison Component
 *
 * Feature comparison table across all tiers.
 */
import type { PricingTier } from '~/types/modules'
import IconCheck from '~icons/tabler/check'
import IconX from '~icons/tabler/x'

const props = defineProps<{
  /** Override tiers from config */
  tiers?: PricingTier[]
}>()

// Get config from module system
const { config: moduleConfig, isEnabled } = useModule('pricing')

// Merge props with module config
const effectiveTiers = computed(() => {
  return props.tiers || moduleConfig.value.tiers || []
})

// Extract unique features across all tiers
const allFeatures = computed(() => {
  const featureSet = new Map<string, boolean[]>()

  effectiveTiers.value.forEach((tier, tierIndex) => {
    tier.features.forEach((feature) => {
      if (!featureSet.has(feature.text)) {
        // Initialize with false for all tiers
        featureSet.set(feature.text, new Array(effectiveTiers.value.length).fill(false))
      }
      const arr = featureSet.get(feature.text)!
      arr[tierIndex] = feature.included
    })
  })

  return Array.from(featureSet.entries()).map(([text, included]) => ({
    text,
    included
  }))
})
</script>

<template>
  <div v-if="isEnabled && moduleConfig.showComparison" class="pricing-comparison" v-reveal>
    <h3 class="pricing-comparison__title">Feature Comparison</h3>

    <div class="pricing-comparison__table-wrapper">
      <table class="pricing-comparison__table">
        <thead>
          <tr>
            <th class="pricing-comparison__feature-header">Features</th>
            <th
              v-for="tier in effectiveTiers"
              :key="tier.id"
              class="pricing-comparison__tier-header"
              :class="{ 'pricing-comparison__tier-header--featured': tier.featured }"
            >
              {{ tier.name }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="feature in allFeatures" :key="feature.text">
            <td class="pricing-comparison__feature-name">{{ feature.text }}</td>
            <td
              v-for="(included, idx) in feature.included"
              :key="idx"
              class="pricing-comparison__cell"
              :class="{
                'pricing-comparison__cell--included': included,
                'pricing-comparison__cell--featured': effectiveTiers[idx]?.featured
              }"
            >
              <IconCheck v-if="included" class="pricing-comparison__icon pricing-comparison__icon--check" />
              <IconX v-else class="pricing-comparison__icon pricing-comparison__icon--x" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
