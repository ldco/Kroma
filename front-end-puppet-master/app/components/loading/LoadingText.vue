<script setup lang="ts">
/**
 * Text Loading Placeholder Component
 *
 * Displays placeholder lines mimicking text content.
 * Automatically varies line widths for realistic appearance.
 */
withDefaults(
  defineProps<{
    lines?: number
    lastLineWidth?: string
  }>(),
  {
    lines: 3,
    lastLineWidth: '70%'
  }
)
</script>

<template>
  <div class="placeholder-text">
    <LoadingBase
      v-for="i in lines"
      :key="i"
      height="1em"
      :width="i === lines ? lastLineWidth : '100%'"
    />
  </div>
</template>
