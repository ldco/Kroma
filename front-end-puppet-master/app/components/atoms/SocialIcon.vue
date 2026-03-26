<script setup lang="ts">
/**
 * SocialIcon Atom
 *
 * Social media icon link with hover effects.
 * Icons are passed via slot for flexibility.
 */

defineProps<{
  /** Link URL */
  href: string
  /** Accessible label */
  label: string
  /** Platform name for styling */
  platform?:
    | 'telegram'
    | 'whatsapp'
    | 'instagram'
    | 'linkedin'
    | 'facebook'
    | 'twitter'
    | 'youtube'
    | 'github'
}>()
</script>

<template>
  <!-- Uses global .social-icon class from skeleton/footer.css -->
  <a
    :href="href"
    :aria-label="label"
    class="social-icon"
    :class="platform ? `social-icon--${platform}` : ''"
    target="_blank"
    rel="noopener noreferrer"
  >
    <slot />
  </a>
</template>

<!-- No scoped styles needed - uses skeleton/footer.css -->
