<script setup lang="ts">
/**
 * Blog Post Card Component
 *
 * Displays a blog post preview card.
 */
import IconClock from '~icons/tabler/clock'
import IconEye from '~icons/tabler/eye'
import IconPhoto from '~icons/tabler/photo'

interface BlogPostData {
  id: number
  slug: string
  title: string
  excerpt: string | null
  coverImageUrl: string | null
  coverImageAlt: string | null
  publishedAt: Date | string | null
  readingTimeMinutes: number | null
  viewCount: number | null
  category: { slug: string; name: string } | null
  author: { name: string } | null
}

const { t } = useI18n()

defineProps<{
  post: BlogPostData
  showExcerpt?: boolean
  showMeta?: boolean
}>()

// Format date
function formatDate(date: Date | string | null): string {
  if (!date) return ''
  const d = typeof date === 'string' ? new Date(date) : date
  return d.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  })
}
</script>

<template>
  <article class="blog-card">
    <!-- Cover Image -->
    <NuxtLink :to="`/blog/${post.slug}`" class="blog-card__image">
      <img
        v-if="post.coverImageUrl"
        :src="post.coverImageUrl"
        :alt="post.coverImageAlt || post.title"
        loading="lazy"
      />
      <div v-else class="content-card-placeholder">
        <IconPhoto />
      </div>
    </NuxtLink>

    <!-- Content -->
    <div class="blog-card__content">
      <!-- Category -->
      <span v-if="post.category" class="blog-card__category">
        {{ post.category.name }}
      </span>

      <!-- Title -->
      <h3 class="blog-card__title">
        <NuxtLink :to="`/blog/${post.slug}`">
          {{ post.title }}
        </NuxtLink>
      </h3>

      <!-- Excerpt -->
      <p v-if="showExcerpt && post.excerpt" class="blog-card__excerpt">
        {{ post.excerpt }}
      </p>

      <!-- Meta -->
      <div v-if="showMeta" class="blog-card__meta">
        <span v-if="post.publishedAt" class="blog-card__date">
          {{ formatDate(post.publishedAt) }}
        </span>
        <span v-if="post.readingTimeMinutes" class="blog-card__reading-time">
          <IconClock />
          {{ post.readingTimeMinutes }} {{ t('blog.minRead') }}
        </span>
        <span v-if="post.viewCount" class="blog-card__views">
          <IconEye />
          {{ post.viewCount }}
        </span>
      </div>

      <!-- Author -->
      <div v-if="post.author" class="blog-card__author">
        {{ post.author.name }}
      </div>
    </div>
  </article>
</template>
