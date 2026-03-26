<script setup lang="ts">
/**
 * Blog Posts Grid Component
 *
 * Displays blog posts in a grid layout.
 */

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

const { t, locale } = useI18n()

const props = withDefaults(defineProps<{
  category?: string
  limit?: number
  showExcerpt?: boolean
  showMeta?: boolean
}>(), {
  limit: 6,
  showExcerpt: true,
  showMeta: true
})

const { isEnabled } = useModule('blog')

const { data: posts, pending } = await useFetch<BlogPostData[]>('/api/blog/posts', {
  key: `blog-posts-${locale.value}-${props.category || 'all'}-${props.limit}`,
  query: {
    locale: locale.value,
    category: props.category,
    limit: props.limit
  },
  watch: [locale]
})
</script>

<template>
  <div v-if="isEnabled" class="blog-grid" v-reveal>
    <!-- Loading -->
    <div v-if="pending" class="blog-grid__loading">
      {{ t('common.loading') }}
    </div>

    <!-- Empty -->
    <div v-else-if="!posts?.length" class="blog-grid__empty">
      {{ t('blog.noPosts') }}
    </div>

    <!-- Grid -->
    <div v-else class="blog-grid__items">
      <MoleculesBlogPostCard
        v-for="post in posts"
        :key="post.id"
        :post="post"
        :show-excerpt="showExcerpt"
        :show-meta="showMeta"
      />
    </div>
  </div>
</template>
