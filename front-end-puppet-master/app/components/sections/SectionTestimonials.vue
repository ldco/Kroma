<script setup lang="ts">
/**
 * Testimonials Section
 *
 * Displays customer testimonials.
 *
 * CSS: ui/content/testimonials.css
 * Classes: .testimonials-grid, .testimonial-card, .testimonial-quote, .testimonial-author
 */
import IconStarFilled from '~icons/tabler/star-filled'
import IconQuote from '~icons/tabler/quote'

interface TestimonialData {
  id: number
  authorName: string
  authorTitle: string | null
  authorCompany: string | null
  authorPhotoUrl: string | null
  rating: number | null
  quote: string
}

const { t, te, locale } = useI18n()

defineProps<{
  title?: string
}>()

const sectionTitle = computed(() => {
  if (te('testimonials.title')) return t('testimonials.title')
  if (te('nav.testimonials')) return t('nav.testimonials')
  return ''
})

const { isEnabled } = useModule('testimonials')

// Comment 6: Use lazy: true for below-fold section to not block SSR
const { data: testimonials, pending } = await useFetch<TestimonialData[]>('/api/testimonials', {
  key: `testimonials-${locale.value}`,
  query: { locale: locale.value },
  watch: [locale],
  lazy: true
})
</script>

<template>
  <section v-if="isEnabled" id="testimonials" class="section section--bg-alt">
    <div class="container">
      <h2 v-if="title || sectionTitle || $slots.title" v-reveal class="section-title section-title--center">
        <slot name="title">{{ title ?? sectionTitle }}</slot>
      </h2>

      <div class="section-body">
        <div v-if="pending" class="testimonials-loading">{{ t('common.loading') }}</div>

        <p v-else-if="!testimonials?.length" class="text-center text-secondary">{{ t('testimonials.noItems') }}</p>

        <div v-else class="testimonials-grid" v-reveal>
        <article v-for="testimonial in testimonials" :key="testimonial.id" class="testimonial-card">
          <IconQuote class="testimonial-card__quote-icon" />

          <blockquote class="testimonial-card__quote">
            {{ testimonial.quote }}
          </blockquote>

          <!-- Rating -->
          <div v-if="testimonial.rating" class="testimonial-card__rating">
            <IconStarFilled v-for="i in testimonial.rating" :key="i" class="testimonial-card__star" />
          </div>

          <!-- Author -->
          <footer class="testimonial-card__author">
            <div v-if="testimonial.authorPhotoUrl" class="testimonial-card__photo">
              <!-- Comment 5: Add width/height to prevent CLS -->
              <img :src="testimonial.authorPhotoUrl" :alt="testimonial.authorName" loading="lazy" width="50" height="50" />
            </div>
            <div class="testimonial-card__info">
              <cite class="testimonial-card__name">{{ testimonial.authorName }}</cite>
              <span v-if="testimonial.authorTitle || testimonial.authorCompany" class="testimonial-card__title">
                {{ testimonial.authorTitle }}
                <template v-if="testimonial.authorTitle && testimonial.authorCompany">, </template>
                {{ testimonial.authorCompany }}
              </span>
            </div>
          </footer>
        </article>
        </div>
      </div>
    </div>
  </section>
</template>
