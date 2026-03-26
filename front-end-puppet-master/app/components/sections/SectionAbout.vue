<script setup lang="ts">
/**
 * About Section
 *
 * Clean about section with optional image.
 *
 * CSS: ui/content/about.css
 * Classes: .about-section, .about-content, .about-visual, .about-text, .about-stats
 */

defineProps<{
  /** Section title (overrides i18n) */
  title?: string
  /** Image URL */
  image?: string
  /** Show stats */
  showStats?: boolean
}>()

const { t, te } = useI18n()

// Get content from i18n
const sectionTitle = computed(() => {
  if (te('about.title')) return t('about.title')
  if (te('nav.about')) return t('nav.about')
  return ''
})

const paragraph1 = computed(() => te('about.paragraph1') ? t('about.paragraph1') : '')
const paragraph2 = computed(() => te('about.paragraph2') ? t('about.paragraph2') : '')

// Stats from i18n
const stats = computed(() => {
  const items = []
  for (let i = 1; i <= 4; i++) {
    const numKey = `about.stat${i}.number`
    const labelKey = `about.stat${i}.label`
    if (te(numKey) && te(labelKey)) {
      items.push({
        number: t(numKey),
        label: t(labelKey)
      })
    }
  }
  return items
})
</script>

<template>
  <section id="about" class="section about-section">
    <div class="container">
      <h2 v-if="title || sectionTitle" v-reveal class="section-title section-title--center">
        {{ title || sectionTitle }}
      </h2>

      <div class="section-body">
        <div class="about-grid" :class="{ 'about-grid--no-image': !image }">
          <!-- Content side -->
          <div class="about-content" :class="{ 'about-content--centered': !image }" v-reveal>
            <p v-if="paragraph1" class="about-content__lead">{{ paragraph1 }}</p>
            <p v-if="paragraph2" class="about-content__body">{{ paragraph2 }}</p>

            <!-- Stats row -->
            <div v-if="showStats && stats.length" class="about-stats">
              <div v-for="stat in stats" :key="stat.label" class="about-stat">
                <div class="about-stat__number">{{ stat.number }}</div>
                <div class="about-stat__label">{{ stat.label }}</div>
              </div>
            </div>
          </div>

          <!-- Visual side - only show if image is provided -->
          <div v-if="image" class="about-visual" v-reveal="{ animation: 'fade-up', delay: 150 }">
            <div class="about-image">
              <img :src="image" alt="" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
