<script setup lang="ts">
/**
 * Card Loading Placeholder Component
 *
 * Displays a card placeholder with image and text areas.
 */
withDefaults(
  defineProps<{
    imageHeight?: string
    showImage?: boolean
    textLines?: number
  }>(),
  {
    imageHeight: '200px',
    showImage: true,
    textLines: 3
  }
)
</script>

<template>
  <div class="placeholder-card">
    <LoadingBase v-if="showImage" :height="imageHeight" borderRadius="8px 8px 0 0" />
    <div class="placeholder-card__content">
      <LoadingBase class="placeholder-card__title-placeholder" height="1.5em" width="60%" />
      <LoadingText :lines="textLines" />
    </div>
  </div>
</template>
