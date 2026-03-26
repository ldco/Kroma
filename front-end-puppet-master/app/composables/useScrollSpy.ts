/**
 * useScrollSpy - Active Section Detection for Onepager Navigation
 *
 * Uses IntersectionObserver to detect which section is currently in view.
 * Provides the active section ID for nav link highlighting.
 *
 * @example
 * ```vue
 * <script setup>
 * const { activeSection } = useScrollSpy(['home', 'about', 'portfolio', 'services', 'contact'])
 * </script>
 *
 * <template>
 *   <a :class="{ active: activeSection === 'about' }" href="#about">About</a>
 * </template>
 * ```
 */

interface ScrollSpyOptions {
  /** Root margin for intersection (default: "-20% 0px -70% 0px" - triggers when section is in top 30% of viewport) */
  rootMargin?: string
  /** Threshold for intersection (default: 0) */
  threshold?: number | number[]
  /** Default section when at top of page (default: first section) */
  defaultSection?: string
}

interface ScrollSpyReturn {
  /** The currently active section ID */
  activeSection: Ref<string>
  /** Manually set active section (e.g., when clicking a nav link) */
  setActiveSection: (id: string) => void
}

export function useScrollSpy(
  sectionIds: string[],
  options: ScrollSpyOptions = {}
): ScrollSpyReturn {
  const {
    rootMargin = '-20% 0px -70% 0px',
    threshold = 0,
    defaultSection = sectionIds[0] || ''
  } = options

  const activeSection = ref(defaultSection)
  const route = useRoute()
  let observer: IntersectionObserver | null = null

  function setActiveSection(id: string) {
    activeSection.value = id
  }

  /**
   * Manually detect which section is currently in view.
   * Used on mount and after locale changes when observer doesn't fire.
   */
  function detectActiveSection() {
    if (!import.meta.client) return

    // If at top of page, use first section
    if (window.scrollY < 100) {
      activeSection.value = defaultSection
      return
    }

    // Find which section is currently in the "active zone" (top 20-30% of viewport)
    const viewportHeight = window.innerHeight
    const activeZoneTop = viewportHeight * 0.2
    const activeZoneBottom = viewportHeight * 0.3

    for (const id of sectionIds) {
      const element = document.getElementById(id)
      if (element) {
        const rect = element.getBoundingClientRect()
        // Section is "active" if its top is in the active zone
        if (rect.top <= activeZoneBottom && rect.bottom >= activeZoneTop) {
          activeSection.value = id
          return
        }
      }
    }
  }

  onMounted(() => {
    if (!import.meta.client) return

    // Create observer
    observer = new IntersectionObserver(
      entries => {
        // Find the entry that is intersecting
        for (const entry of entries) {
          if (entry.isIntersecting) {
            activeSection.value = entry.target.id
            break
          }
        }
      },
      { rootMargin, threshold }
    )

    // Observe all sections
    for (const id of sectionIds) {
      const element = document.getElementById(id)
      if (element) {
        observer.observe(element)
      }
    }

    // Detect initial active section based on current scroll position
    // IntersectionObserver only fires on changes, not on initial setup
    detectActiveSection()
  })

  // Watch for route changes (locale change triggers route change)
  // Re-detect active section since component doesn't remount
  watch(
    () => route.fullPath,
    () => {
      nextTick(() => {
        // If route has a hash, use it directly as active section
        // (smooth scroll hasn't completed yet, so we can't detect from position)
        if (route.hash) {
          const targetSection = route.hash.slice(1) // Remove #
          if (sectionIds.includes(targetSection)) {
            activeSection.value = targetSection
            return
          }
        }
        // No hash - detect from scroll position (e.g., at home with no hash)
        detectActiveSection()
      })
    }
  )

  onUnmounted(() => {
    if (observer) {
      observer.disconnect()
      observer = null
    }
  })

  return {
    activeSection,
    setActiveSection
  }
}
