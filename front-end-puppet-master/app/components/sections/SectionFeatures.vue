<script setup lang="ts">
/**
 * Features Section
 *
 * Displays feature cards with images (and hover images).
 * Cards show an image that swaps on hover for interactive effect.
 *
 * CSS: ui/content/features.css
 * Classes: .features-grid, .feature-card, .feature-card__image, .feature-card__content
 */
import IconPhoto from '~icons/tabler/photo'

interface FeatureData {
  id: number
  slug: string
  imageUrl: string | null
  hoverImageUrl: string | null
  category: string | null
  title: string
  description: string | null
}

const { t, te, locale } = useI18n()

defineProps<{
  title?: string
  /** Use bento grid layout */
  bento?: boolean
}>()

const sectionTitle = computed(() => {
  if (te('features.title')) return t('features.title')
  if (te('nav.features')) return t('nav.features')
  return ''
})

const { isEnabled } = useModule('features')

// Comment 6: Use lazy: true for below-fold section to not block SSR
const { data: features, pending } = await useFetch<FeatureData[]>('/api/features', {
  key: `features-${locale.value}`,
  query: { locale: locale.value },
  watch: [locale],
  lazy: true
})

// Track failed images
const failedImages = ref(new Set<number>())

function onImageError(featureId: number) {
  failedImages.value.add(featureId)
}

function hasValidImage(feature: FeatureData): boolean {
  return !!(feature.imageUrl && !failedImages.value.has(feature.id))
}
</script>

<template>
  <section v-if="isEnabled" id="features" class="section">
    <div class="container">
      <h2 v-if="title || sectionTitle || $slots.title" v-reveal class="section-title section-title--center">
        <slot name="title">{{ title ?? sectionTitle }}</slot>
      </h2>

      <div class="section-body">
        <div v-if="pending" class="features-loading">{{ t('common.loading') }}</div>

        <p v-else-if="!features?.length" class="text-center text-secondary">{{ t('features.noItems') }}</p>

        <div
          v-else
          class="features-grid"
          :class="{ 'features-grid--bento': bento && features.length === 6 }"
          v-reveal
        >
          <div
            v-for="feature in features"
            :key="feature.id"
            class="feature-card"
          >
            <!-- Image -->
            <div class="feature-card__image">
              <template v-if="hasValidImage(feature)">
                <img
                  :src="feature.imageUrl!"
                  :alt="feature.title"
                  class="feature-card__img feature-card__img--default"
                  @error="onImageError(feature.id)"
                />
                <img
                  v-if="feature.hoverImageUrl"
                  :src="feature.hoverImageUrl"
                  :alt="feature.title"
                  class="feature-card__img feature-card__img--hover"
                />
              </template>
              <div v-else class="content-card-placeholder">
                <IconPhoto />
              </div>
            </div>
            <!-- Content overlay -->
            <div class="feature-card__content">
              <h3 class="feature-card__title">{{ feature.title }}</h3>
              <p v-if="feature.description" class="feature-card__description">
                {{ feature.description }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
