<script setup lang="ts">
/**
 * HamburgerIcon Atom
 *
 * Animated hamburger menu icon.
 * Uses hamburgers.css library (Jonathan Suh).
 */

defineProps<{
  /** Whether the menu is open */
  isOpen: boolean
  /** Accessible label */
  label?: string
}>()

const emit = defineEmits<{
  toggle: []
}>()
</script>

<template>
  <button
    type="button"
    class="hamburger hamburger--squeeze"
    :class="{ 'is-active': isOpen }"
    :aria-label="label ?? 'Toggle menu'"
    :aria-expanded="isOpen"
    @click="emit('toggle')"
  >
    <span class="hamburger-box">
      <span class="hamburger-inner"></span>
    </span>
  </button>
</template>

<!--
  Uses global CSS classes from ui/hamburger.css:
  - .hamburger, .hamburger--squeeze, .is-active
  - .hamburger-box, .hamburger-inner
-->
