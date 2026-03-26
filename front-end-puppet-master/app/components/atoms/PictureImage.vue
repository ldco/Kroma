<script setup lang="ts">
/**
 * PictureImage Atom Component
 *
 * Responsive image component with automatic AVIF → WebP → PNG fallback chain.
 * Uses <picture> element for optimal format selection and lazy loading.
 *
 * @example
 * ```vue
 * <!-- Basic usage -->
 * <PictureImage path="hero/banner" alt="Hero banner" />
 *
 * <!-- With custom sizes -->
 * <PictureImage
 *   path="hero/banner"
 *   alt="Hero banner"
 *   :srcset-widths="[400, 800, 1600]"
 *   sizes="(max-width: 768px) 400px, (max-width: 1024px) 800px, 1600px"
 *   fetchpriority="high"
 *   loading="eager"
 * />
 *
 * <!-- For background-style images -->
 * <PictureImage
 *   path="backgrounds/site-bg"
 *   alt=""
 *   role="presentation"
 *   object-fit="cover"
 * />
 * ```
 */

import { getResponsiveImagePaths, getSrcSet, getSizes, getLoadingAttrs, getDefaultSizes } from '~/utils/image-assets'

// ═══════════════════════════════════════════════════════════════════════════
// PROPS
// ═══════════════════════════════════════════════════════════════════════════

interface Props {
  /** Image path without extension (e.g., 'hero/banner') */
  path: string
  /** Alt text for accessibility (empty string for decorative images) */
  alt: string
  /** Widths for srcset generation */
  srcsetWidths?: number[]
  /** Sizes attribute for responsive images */
  sizes?: string
  /** Layout preset for default sizes */
  layout?: 'hero' | 'thumbnail' | 'full' | 'auto'
  /** Loading priority: 'eager' for above-fold, 'lazy' for below-fold */
  loading?: 'eager' | 'lazy'
  /** Fetch priority hint */
  fetchpriority?: 'high' | 'low' | 'auto'
  /** Decoding mode */
  decoding?: 'sync' | 'async' | 'auto'
  /** Object fit for the image */
  objectFit?: 'contain' | 'cover' | 'fill' | 'none' | 'scale-down'
  /** Additional CSS classes */
  class?: string
  /** Image width attribute */
  width?: number | string
  /** Image height attribute */
  height?: number | string
  /** Role attribute (use 'presentation' for decorative images) */
  role?: string
}

const props = withDefaults(defineProps<Props>(), {
  srcsetWidths: () => [400, 800, 1200, 1600],
  layout: 'auto',
  loading: 'lazy',
  fetchpriority: 'auto',
  decoding: 'async',
  objectFit: 'cover',
  class: '',
  width: undefined,
  height: undefined,
  role: undefined
})

// ═══════════════════════════════════════════════════════════════════════════
// COMPUTED
// ═══════════════════════════════════════════════════════════════════════════

const imagePaths = computed(() => getResponsiveImagePaths(props.path))

const avifSrcSet = computed(() => getSrcSet(props.path, props.srcsetWidths, 'avif'))
const webpSrcSet = computed(() => getSrcSet(props.path, props.srcsetWidths, 'webp'))
const pngSrcSet = computed(() => getSrcSet(props.path, props.srcsetWidths, 'png'))

const sizesValue = computed(() => {
  if (props.sizes) return props.sizes
  if (props.layout !== 'auto') return getDefaultSizes(props.layout)
  return undefined
})

const loadingAttrs = computed(() => {
  const position = props.loading === 'eager' ? 'above-fold' : 'below-fold'
  const defaults = getLoadingAttrs(position)
  return {
    loading: props.loading,
    fetchpriority: props.fetchpriority !== 'auto' ? props.fetchpriority : defaults.fetchpriority,
    decoding: props.decoding !== 'auto' ? props.decoding : defaults.decoding
  }
})

// ═══════════════════════════════════════════════════════════════════════════
// STATE
// ═══════════════════════════════════════════════════════════════════════════

const isLoaded = ref(false)
const hasError = ref(false)

function onLoad() {
  isLoaded.value = true
}

function onError() {
  hasError.value = true
}
</script>

<template>
  <span
    class="picture-image"
    :class="[
      { 'picture-image--loaded': isLoaded, 'picture-image--error': hasError },
      props.class
    ]"
  >
    <picture>
      <!-- AVIF source (primary format) -->
      <source
        :srcset="avifSrcSet"
        :sizes="sizesValue"
        type="image/avif"
      />

      <!-- WebP source (fallback) -->
      <source
        :srcset="webpSrcSet"
        :sizes="sizesValue"
        type="image/webp"
      />

      <!-- PNG fallback (universal support) -->
      <img
        :src="imagePaths.png"
        :alt="alt"
        :width="width"
        :height="height"
        :loading="loadingAttrs.loading"
        :fetchpriority="loadingAttrs.fetchpriority"
        :decoding="loadingAttrs.decoding"
        :class="[
          'picture-image__img',
          `picture-image__img--${objectFit}`,
          { 'picture-image__img--loaded': isLoaded, 'picture-image__img--error': hasError }
        ]"
        @load="onLoad"
        @error="onError"
      />
    </picture>
  </span>
</template>

<!-- No scoped styles - uses global CSS from app/assets/css/ui/content/picture-image.css -->
