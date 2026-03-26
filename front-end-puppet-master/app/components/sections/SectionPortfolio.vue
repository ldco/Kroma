<script setup lang="ts">
/**
 * Portfolio Section
 *
 * Shows ALL portfolios with their items.
 * Self-contained: fetches title from i18n if not provided.
 */
import type { PortfolioWithItems, PortfolioItem } from '~/types'
import IconPhoto from '~icons/tabler/photo'

const { t, te } = useI18n()

const props = defineProps<{
  /** Section title */
  title?: string
}>()

// Self-contained: fetch title from i18n if not provided
const sectionTitle = computed(() => {
  if (te('portfolio.title')) return t('portfolio.title')
  if (te('nav.portfolio')) return t('nav.portfolio')
  return 'Our Work'
})

// Fetch ALL portfolios with their items
// Comment 6: Use lazy: true for below-fold section to not block SSR
const { data: portfoliosData, pending } = await useFetch<PortfolioWithItems[]>('/api/portfolios', {
  key: 'portfolio-section-all',
  query: { includeItems: 'true' },
  lazy: true
})

const portfolios = computed(() => portfoliosData.value || [])

// Flatten all items for lightbox navigation
const allItems = computed(() => portfolios.value.flatMap(p => p.items || []))

// Lightbox state
const lightboxOpen = ref(false)
const lightboxIndex = ref(0)
const currentItem = computed(() => allItems.value[lightboxIndex.value])

function openLightbox(item: PortfolioItem) {
  const index = allItems.value.findIndex(i => i.id === item.id)
  if (index >= 0) {
    lightboxIndex.value = index
    lightboxOpen.value = true
  }
}

function closeLightbox() {
  lightboxOpen.value = false
}

function navigateLightbox(index: number) {
  lightboxIndex.value = index
}
</script>

<template>
  <section id="portfolio" class="section">
    <div class="container">
      <h2 v-reveal class="section-title section-title--center">
        <slot name="title">{{ title ?? sectionTitle }}</slot>
      </h2>

      <div class="section-body">
        <!-- Show message if no portfolios -->
        <p v-if="!portfolios.length" v-reveal class="text-center text-secondary">{{ t('portfolio.noItems') }}</p>

        <!-- Loop through each portfolio -->
        <template v-for="portfolio in portfolios" :key="portfolio.id">
        <!-- Portfolio header (only show if multiple portfolios) -->
        <h3 v-if="portfolios.length > 1" v-reveal class="portfolio-section-title">
          {{ portfolio.name }}
        </h3>

        <!-- Portfolio items grid -->
        <div v-if="portfolio.items?.length" class="section-grid-auto section-grid-auto--lg portfolio-items-grid" data-reveal-stagger="fast">
          <article
            v-for="item in portfolio.items"
            :key="item.id"
            v-reveal="'scale'"
            class="portfolio-card portfolio-card--clickable"
            :class="{ 'portfolio-card--info-visible': portfolio.type === 'case_study' }"
            @click="openLightbox(item)"
          >
            <div class="portfolio-card-image">
              <!-- Comment 4: Use thumbnailUrl only for grid, show placeholder if absent -->
              <img
                v-if="item.thumbnailUrl"
                :src="item.thumbnailUrl"
                :alt="item.title || item.caption || ''"
                loading="lazy"
                width="400"
                height="300"
              />
              <div v-else class="content-card-placeholder">
                <IconPhoto />
              </div>
            </div>
            <div class="portfolio-card-info">
              <span v-if="item.category" class="portfolio-card-category">
                {{ item.category }}
              </span>
              <h3 v-if="item.title" class="portfolio-card-title">{{ item.title }}</h3>
              <h3 v-else-if="item.caption" class="portfolio-card-title">{{ item.caption }}</h3>
              <p v-if="item.description" class="portfolio-card-description">
                {{ item.description }}
              </p>
            </div>
          </article>
        </div>
      </template>
      </div>
    </div>
  </section>

  <!-- Lightbox -->
  <Teleport to="body">
    <div v-if="lightboxOpen && currentItem" class="lightbox" @click.self="closeLightbox">
      <button class="lightbox-close" @click="closeLightbox">&times;</button>

      <button
        v-if="lightboxIndex > 0"
        class="lightbox-prev"
        @click="navigateLightbox(lightboxIndex - 1)"
      >
        &#8249;
      </button>

      <div class="lightbox-content">
        <img
          v-if="currentItem.itemType !== 'video'"
          :src="currentItem.mediaUrl || currentItem.thumbnailUrl || ''"
          :alt="currentItem.title || currentItem.caption || ''"
        />
        <video
          v-else
          :src="currentItem.mediaUrl || ''"
          controls
          autoplay
        />
      </div>

      <button
        v-if="lightboxIndex < allItems.length - 1"
        class="lightbox-next"
        @click="navigateLightbox(lightboxIndex + 1)"
      >
        &#8250;
      </button>

      <div v-if="currentItem.title || currentItem.caption" class="lightbox-caption">
        {{ currentItem.title || currentItem.caption }}
      </div>
    </div>
  </Teleport>
</template>
<!-- Styles in app/assets/css/layout/sections.css -->
