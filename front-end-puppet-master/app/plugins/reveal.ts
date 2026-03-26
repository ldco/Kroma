/**
 * Reveal Directive Plugin
 *
 * Registers the v-reveal directive for scroll-triggered animations.
 * Works on both server (SSR) and client.
 *
 * In onepager mode: Elements start hidden, revealed on scroll via useReveal()
 * In SPA mode: Elements are immediately visible (no scroll reveal)
 *
 * Usage:
 *   <div v-reveal>Fade up (default)</div>
 *   <div v-reveal="'fade-left'">Slide from left</div>
 *   <div v-reveal="{ animation: 'scale', delay: 200, duration: 'slow' }">Options</div>
 *
 * Animation types: fade-up, fade-down, fade-left, fade-right, scale, zoom, flip, slide-up, fade
 * Delay: number in ms (100, 200, 300, 400, 500, 600)
 * Duration: 'fast' | 'slow' | 'slower'
 * Ease: 'bounce' | 'smooth'
 */
import type { DirectiveBinding } from 'vue'
import config from '~/puppet-master.config'

interface RevealBinding {
  animation?: string
  delay?: number
  duration?: 'fast' | 'slow' | 'slower'
  ease?: 'bounce' | 'smooth'
}

type RevealValue = string | RevealBinding | undefined

/**
 * Parse directive binding value into animation config
 */
function parseBinding(value: RevealValue) {
  let animation = 'fade-up'
  let delay: number | undefined
  let duration: string | undefined
  let ease: string | undefined

  if (typeof value === 'string') {
    animation = value
  } else if (typeof value === 'object' && value !== null) {
    animation = value.animation || 'fade-up'
    delay = value.delay
    duration = value.duration
    ease = value.ease
  }

  return { animation, delay, duration, ease }
}

export default defineNuxtPlugin(nuxtApp => {
  nuxtApp.vueApp.directive('reveal', {
    // SSR: Return attributes to render on server
    getSSRProps(binding: DirectiveBinding<RevealValue>) {
      // Comment 12: Check config at call time for SSR/CSR consistency
      // config.features.onepager is a build-time constant
      const isRevealEnabled = config.features.onepager

      // In SPA mode, don't add any attributes
      if (!isRevealEnabled) {
        return {}
      }

      const { animation, delay, duration, ease } = parseBinding(binding.value)
      const attrs: Record<string, string> = {
        'data-reveal': animation
      }

      if (delay) {
        const roundedDelay = Math.round(delay / 100) * 100
        attrs['data-reveal-delay'] = String(roundedDelay)
      }

      if (duration) {
        attrs['data-reveal-duration'] = duration
      }

      if (ease) {
        attrs['data-reveal-ease'] = ease
      }

      return attrs
    },

    // Client: Set attributes after mount
    mounted(el: HTMLElement, binding: DirectiveBinding<RevealValue>) {
      // Comment 12: Check config at call time for SSR/CSR consistency
      const isRevealEnabled = config.features.onepager

      // In SPA mode, skip reveal - elements stay visible
      if (!isRevealEnabled) {
        return
      }

      const { animation, delay, duration, ease } = parseBinding(binding.value)

      // Set data attributes for CSS
      el.setAttribute('data-reveal', animation)

      if (delay) {
        const roundedDelay = Math.round(delay / 100) * 100
        el.setAttribute('data-reveal-delay', String(roundedDelay))
      }

      if (duration) {
        el.setAttribute('data-reveal-duration', duration)
      }

      if (ease) {
        el.setAttribute('data-reveal-ease', ease)
      }
    },

    unmounted(el: HTMLElement) {
      // Clean up data attributes (only if they were set)
      if (el.hasAttribute('data-reveal')) {
        el.removeAttribute('data-reveal')
        el.removeAttribute('data-reveal-delay')
        el.removeAttribute('data-reveal-duration')
        el.removeAttribute('data-reveal-ease')
      }
    }
  })
})
