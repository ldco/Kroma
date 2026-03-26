/**
 * useReveal - Scroll Reveal Animations
 *
 * Provides IntersectionObserver-based reveal animations for page elements.
 * Works with the v-reveal directive and data-reveal CSS classes.
 *
 * @example
 * ```vue
 * <script setup>
 * // Activate reveals on the page (call once in page/layout)
 * useReveal()
 * </script>
 *
 * <template>
 *   <div v-reveal>Fades up on scroll</div>
 *   <div v-reveal="'fade-left'">Slides from left</div>
 *   <div v-reveal="{ animation: 'scale', delay: 200 }">Scale with delay</div>
 * </template>
 * ```
 */

export interface RevealOptions {
  /** Enable/disable reveal functionality (default: true) */
  enabled?: boolean
  /** Root margin for IntersectionObserver (default: '0px 0px -10% 0px') */
  rootMargin?: string
  /** Threshold for triggering reveal (default: 0.1) */
  threshold?: number
  /** Whether to re-hide elements when scrolling back up (default: false) */
  resetOnExit?: boolean
  /** Selector for elements to observe (default: '[data-reveal]') */
  selector?: string
}

interface RevealReturn {
  /** Manually trigger reveal check */
  refresh: () => void
  /** Pause observation */
  pause: () => void
  /** Resume observation */
  resume: () => void
}

/**
 * Activate scroll reveal animations on the page.
 * Call once in a page or layout component.
 */
export function useReveal(options: RevealOptions = {}): RevealReturn {
  const {
    enabled = true,
    rootMargin = '0px 0px -10% 0px',
    threshold = 0.1,
    resetOnExit = false,
    selector = '[data-reveal]'
  } = options

  // No-op functions for when disabled - Comment 7: Return early before any observer is created
  const noop = () => {}
  if (!enabled) {
    return { refresh: noop, pause: noop, resume: noop }
  }

  let observer: IntersectionObserver | null = null
  let mutationObserver: MutationObserver | null = null
  let isPaused = false
  let observedRoot: Element | null = null

  /**
   * Handle intersection changes
   */
  function handleIntersect(entries: IntersectionObserverEntry[]) {
    if (isPaused) return

    for (const entry of entries) {
      if (entry.isIntersecting) {
        entry.target.classList.add('revealed')
      } else if (resetOnExit) {
        entry.target.classList.remove('revealed')
      }
    }
  }

  /**
   * Find and observe all reveal elements
   * Comment 7: Observe from specific mount element instead of entire document.body
   */
  function observeElements(root: Element = document) {
    if (!observer) return

    const elements = root.querySelectorAll(selector)
    for (const el of elements) {
      // Skip already revealed elements (unless resetOnExit is true)
      if (!resetOnExit && el.classList.contains('revealed')) continue
      observer.observe(el)
    }
  }

  /**
   * Refresh - re-scan DOM for new reveal elements
   */
  function refresh() {
    if (observedRoot) {
      observeElements(observedRoot)
    } else {
      observeElements()
    }
  }

  /**
   * Pause observation (useful during route transitions)
   */
  function pause() {
    isPaused = true
  }

  /**
   * Resume observation
   */
  function resume() {
    isPaused = false
  }

  onMounted(() => {
    if (!import.meta.client) return

    // Comment 7: Get specific mount element (layout main) instead of observing entire body
    const mountElement = document.querySelector('.layout, .main, main') as Element | null

    // Check for reduced motion preference
    const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    if (prefersReducedMotion) {
      // Reveal everything immediately for accessibility
      const root = mountElement || document
      const elements = root.querySelectorAll(selector)
      for (const el of elements) {
        el.classList.add('revealed')
      }
      return
    }

    // Create observer
    observer = new IntersectionObserver(handleIntersect, {
      rootMargin,
      threshold
    })

    // Initial observation from mount element
    observedRoot = mountElement || document
    observeElements(observedRoot)

    // Comment 7: Watch for dynamic content with MutationObserver on specific root
    // Only create if we have a specific mount element to avoid body-level observation
    if (mountElement) {
      mutationObserver = new MutationObserver(mutations => {
        let hasNewElements = false
        for (const mutation of mutations) {
          if (mutation.type === 'childList' && mutation.addedNodes.length > 0) {
            hasNewElements = true
            break
          }
        }
        if (hasNewElements) {
          // Debounce refresh
          setTimeout(refresh, 100)
        }
      })

      mutationObserver.observe(mountElement, {
        childList: true,
        subtree: true
      })
    }
  })

  // Consolidated cleanup
  onUnmounted(() => {
    if (observer) {
      observer.disconnect()
      observer = null
    }
    if (mutationObserver) {
      mutationObserver.disconnect()
      mutationObserver = null
    }
  })

  return {
    refresh,
    pause,
    resume
  }
}
