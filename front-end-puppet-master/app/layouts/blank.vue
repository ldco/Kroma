<script setup lang="ts">
/**
 * Blank Layout
 *
 * Minimal layout with optional header/footer slots.
 * Use for login pages, error pages, standalone pages.
 *
 * SLOTS:
 * - header: Fixed/sticky header (OUTSIDE .main - position:fixed works)
 * - default: Main page content (inside .main with container queries)
 * - footer: Footer content (OUTSIDE .main)
 *
 * WHY SLOTS MATTER:
 * The .main element has `container-type: inline-size` which creates a
 * containing block for fixed-positioned descendants. Any position:fixed
 * element inside .main will be fixed relative to .main, not the viewport.
 *
 * By providing header/footer slots OUTSIDE .main, fixed positioning works
 * correctly - same grid structure as default.vue layout.
 *
 * Uses .layout from layout/page.css (same as default.vue)
 */
</script>

<template>
  <div class="layout">
    <!-- Header slot: OUTSIDE .main so position:fixed works correctly -->
    <slot name="header" />

    <main class="main">
      <slot />
    </main>

    <!-- Footer slot: OUTSIDE .main -->
    <slot name="footer" />
  </div>
</template>
