<script setup lang="ts">
/**
 * Theme Toggle Atom
 *
 * Toggles between light and dark mode with modern View Transitions API animation.
 * Uses @nuxtjs/color-mode under the hood.
 * Icons from Tabler via unplugin-icons.
 *
 * ANIMATION: Circle reveal effect using View Transitions API (2025 best practice)
 * - Animates from click position outward
 * - Falls back gracefully for unsupported browsers
 * - Respects prefers-reduced-motion
 *
 * Uses ClientOnly to avoid SSR hydration mismatch since
 * colorMode.value can differ between server and client.
 */
import IconSun from '~icons/tabler/sun'
import IconMoon from '~icons/tabler/moon'

const colorMode = useColorMode()
const { t } = useI18n()

/**
 * Toggle theme with View Transitions API animation
 * Creates a circle reveal effect from the click position
 */
async function toggleTheme(event: MouseEvent) {
  const newTheme = colorMode.value === 'dark' ? 'light' : 'dark'

  // Check if View Transitions API is supported and motion is allowed
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
  const supportsViewTransitions = 'startViewTransition' in document

  if (!supportsViewTransitions || prefersReducedMotion) {
    // Fallback: instant switch
    colorMode.preference = newTheme
    return
  }

  // Get click coordinates for circle origin
  const x = event.clientX
  const y = event.clientY

  // Calculate the maximum radius needed to cover the entire screen
  const endRadius = Math.hypot(
    Math.max(x, window.innerWidth - x),
    Math.max(y, window.innerHeight - y)
  )

  // Set CSS custom properties for the animation origin
  document.documentElement.style.setProperty('--theme-toggle-x', `${x}px`)
  document.documentElement.style.setProperty('--theme-toggle-y', `${y}px`)
  document.documentElement.style.setProperty('--theme-toggle-radius', `${endRadius}px`)

  // Start the view transition
  const transition = (document as any).startViewTransition(() => {
    colorMode.preference = newTheme
  })

  // Wait for transition to be ready, then run animation
  await transition.ready

  // Animate the new view with a circle clip-path
  document.documentElement.animate(
    {
      clipPath: [`circle(0px at ${x}px ${y}px)`, `circle(${endRadius}px at ${x}px ${y}px)`]
    },
    {
      duration: 400,
      easing: 'cubic-bezier(0.4, 0, 0.2, 1)',
      pseudoElement: '::view-transition-new(root)'
    }
  )
}

const isDark = computed(() => colorMode.value === 'dark')
</script>

<template>
  <ClientOnly>
    <button
      type="button"
      class="btn btn-icon btn-ghost theme-toggle"
      :aria-label="t('theme.' + colorMode.preference)"
      @click="toggleTheme"
    >
      <Transition name="theme-icon" mode="out-in">
        <IconMoon v-if="isDark" key="moon" class="theme-toggle-icon" />
        <IconSun v-else key="sun" class="theme-toggle-icon" />
      </Transition>
    </button>
    <!-- Fallback shows nothing during SSR - button appears on hydration -->
    <template #fallback>
      <button type="button" class="btn btn-icon btn-ghost theme-toggle" aria-label="Theme">
        <IconSun class="theme-toggle-icon" />
      </button>
    </template>
  </ClientOnly>
</template>

<!--
  Uses global CSS classes from animations/keyframes.css
  View Transition styles are in animations/transitions.css
-->
