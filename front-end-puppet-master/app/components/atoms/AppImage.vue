<script setup lang="ts">
/**
 * AppImage - Safe image component with fallback
 *
 * Drop-in replacement for <img> that:
 * - Shows themed placeholder icon when src is empty/null
 * - Shows themed placeholder on load error (never shows browser broken image icon!)
 * - Supports initials fallback for avatars
 *
 * IMPORTANT: Use this component everywhere images are displayed to ensure
 * consistent fallback behavior and prevent browser broken image icons.
 *
 * Usage Examples:
 *
 * Basic (icon fallback):
 *   <AppImage :src="feature.imageUrl" :alt="feature.title" />
 *
 * Avatar with initials fallback:
 *   <AppImage
 *     :src="user.photoUrl"
 *     :alt="user.name"
 *     fallback="initials"
 *     :initials="user.name.charAt(0)"
 *   />
 *
 * With custom class:
 *   <AppImage :src="item.image" alt="Preview" img-class="rounded-lg" />
 *
 * CSS Dependencies:
 * - Uses .content-card-placeholder from content-card.css for consistent styling
 */
import IconPhoto from '~icons/tabler/photo'

const props = withDefaults(defineProps<{
  /** Image source URL */
  src?: string | null
  /** Alt text for accessibility */
  alt?: string
  /** Fallback type: 'icon' shows photo icon, 'initials' shows text */
  fallback?: 'icon' | 'initials'
  /** Initials to show when fallback='initials' */
  initials?: string
  /** Additional classes for the image */
  imgClass?: string
  /** Loading attribute */
  loading?: 'lazy' | 'eager'
}>(), {
  src: null,
  alt: '',
  fallback: 'icon',
  initials: '',
  imgClass: '',
  loading: 'lazy'
})

const emit = defineEmits<{
  error: [event: Event]
  load: [event: Event]
}>()

// Track image state
const hasError = ref(false)
const isLoaded = ref(false)

// Determine if we should show the image
const showImage = computed(() => {
  return props.src && !hasError.value
})

// Determine if we should show placeholder
const showPlaceholder = computed(() => {
  return !props.src || hasError.value
})

// Handle image error
function onError(event: Event) {
  hasError.value = true
  emit('error', event)
}

// Handle image load
function onLoad(event: Event) {
  isLoaded.value = true
  emit('load', event)
}

// Reset state when src changes
watch(() => props.src, () => {
  hasError.value = false
  isLoaded.value = false
})
</script>

<template>
  <div class="app-image">
    <!-- Actual image - hidden until loaded, completely removed on error -->
    <img
      v-if="showImage"
      :src="src!"
      :alt="alt"
      :loading="loading"
      :class="['app-image__img', imgClass, { 'app-image__img--loaded': isLoaded }]"
      @error="onError"
      @load="onLoad"
    />

    <!-- Placeholder - shown when no src or error -->
    <div v-if="showPlaceholder" class="app-image__placeholder content-card-placeholder">
      <span v-if="fallback === 'initials' && initials" class="app-image__initials">
        {{ initials }}
      </span>
      <IconPhoto v-else />
    </div>
  </div>
</template>
