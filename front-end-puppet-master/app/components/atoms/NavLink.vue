<script setup lang="ts">
/**
 * NavLink Atom
 *
 * Navigation link with active state styling.
 * Supports both internal routes and anchor links.
 *
 * For anchor links (#section), uses plain <a> to avoid Vue Router warnings.
 * Active state for anchors is managed by scrollspy (passed via isActive prop).
 * For routes (/page), uses NuxtLink which auto-applies router-link-active.
 */

defineProps<{
  /** Link destination */
  to: string
  /** Link text (slot is preferred) */
  label?: string
  /** Whether this is an anchor link (for one-pager) */
  isAnchor?: boolean
  /** Whether this link is currently active (for anchor links with scrollspy) */
  isActive?: boolean
}>()

const emit = defineEmits<{
  click: []
}>()
</script>

<template>
  <!-- Uses global .nav-link class from skeleton/nav.css -->

  <!-- Anchor links: use plain <a> to avoid Vue Router warnings -->
  <!-- Active state is managed by scrollspy and passed via isActive prop -->
  <a
    v-if="isAnchor"
    :href="to"
    class="nav-link nav-link--anchor"
    :class="{ active: isActive }"
    :aria-current="isActive ? 'true' : undefined"
    @click="emit('click')"
  >
    <slot>{{ label }}</slot>
  </a>

  <!-- Route links: use NuxtLink for client-side navigation -->
  <!-- NuxtLink auto-applies router-link-active and aria-current="page" -->
  <NuxtLink v-else :to="to" class="nav-link" @click="emit('click')">
    <slot>{{ label }}</slot>
  </NuxtLink>
</template>

<!-- No scoped styles needed - uses skeleton/nav.css -->
