<script setup lang="ts">
/**
 * Blog Section
 *
 * Displays latest blog posts.
 *
 * CSS: ui/content/blog.css
 * Classes: .blog-grid, .blog-card, .blog-card__image, .blog-card__content, .blog-card__meta
 */

const { t, te } = useI18n()

defineProps<{
  title?: string
}>()

const sectionTitle = computed(() => {
  if (te('blog.title')) return t('blog.title')
  if (te('nav.blog')) return t('nav.blog')
  return ''
})
</script>

<template>
  <section id="blog" class="section">
    <div class="container">
      <h2 v-if="title || sectionTitle || $slots.title" v-reveal class="section-title section-title--center">
        <slot name="title">{{ title ?? sectionTitle }}</slot>
      </h2>
      <div class="section-body">
        <OrganismsBlogPostsGrid :limit="3" />
      </div>
    </div>
  </section>
</template>
