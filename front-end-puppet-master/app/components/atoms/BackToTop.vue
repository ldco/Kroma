<script setup lang="ts">
/**
 * BackToTop Atom
 *
 * Fixed button that appears when scrolling down.
 * Scrolls smoothly to top of page when clicked.
 */
import IconArrowUp from '~icons/tabler/arrow-up'

const { t } = useI18n()

// Track scroll position
const isVisible = ref(false)
const scrollThreshold = 300 // Show after scrolling 300px

const handleScroll = () => {
  isVisible.value = window.scrollY > scrollThreshold
}

const scrollToTop = () => {
  window.scrollTo({
    top: 0,
    behavior: 'smooth'
  })
}

onMounted(() => {
  window.addEventListener('scroll', handleScroll, { passive: true })
  // Check initial position
  handleScroll()
})

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll)
})
</script>

<template>
  <Transition name="fade-slide">
    <button
      v-if="isVisible"
      type="button"
      class="back-to-top"
      :aria-label="t('footer.backToTop')"
      @click="scrollToTop"
    >
      <IconArrowUp />
    </button>
  </Transition>
</template>

<!--
  Uses global CSS classes from ui/content/index.css:
  - .back-to-top
  - .fade-slide-* animations
-->
