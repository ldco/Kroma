import type { DirectiveBinding } from 'vue'

type ParallaxAxis = 'x' | 'y' | 'both'
type PointerAxis = 'x' | 'y' | 'both'

interface ParallaxOptions {
  speed?: number
  axis?: ParallaxAxis
  clamp?: number
  pointer?: number
  pointerAxis?: PointerAxis
  pointerClamp?: number
  floating?: boolean
}

type ParallaxValue = number | ParallaxOptions | undefined
type SceneValue = { id?: string; story?: boolean } | string | boolean | undefined

/**
 * Apply parallax data attributes to element
 * Supports scroll-driven and pointer-driven parallax
 */
function applyParallaxAttributes(el: HTMLElement, value: ParallaxValue) {
  el.setAttribute('data-parallax', 'true')

  const options = typeof value === 'number' ? { speed: value } : (value || {})

  // Scroll-driven parallax
  const speed = options.speed ?? 0.12
  el.setAttribute('data-parallax-speed', String(speed))

  const axis = options.axis ?? 'y'
  el.setAttribute('data-parallax-axis', axis)

  const clamp = options.clamp ?? 160
  el.setAttribute('data-parallax-clamp', String(clamp))

  // Pointer-driven parallax (mouse movement)
  const pointer = options.pointer ?? 0
  if (pointer > 0) {
    el.setAttribute('data-parallax-pointer', String(pointer))
  } else {
    el.removeAttribute('data-parallax-pointer')
  }

  const pointerAxis = options.pointerAxis ?? 'both'
  if (pointer > 0) {
    el.setAttribute('data-parallax-pointer-axis', pointerAxis)
  } else {
    el.removeAttribute('data-parallax-pointer-axis')
  }

  const pointerClamp = options.pointerClamp ?? 24
  if (pointer > 0) {
    el.setAttribute('data-parallax-pointer-clamp', String(pointerClamp))
  } else {
    el.removeAttribute('data-parallax-pointer-clamp')
  }

  // Floating animation
  const floating = options.floating ?? false
  if (floating) {
    el.setAttribute('data-parallax-float', 'true')
  } else {
    el.removeAttribute('data-parallax-float')
  }
}

/**
 * Clear all parallax data attributes from element
 */
function clearParallaxAttributes(el: HTMLElement) {
  el.removeAttribute('data-parallax')
  el.removeAttribute('data-parallax-speed')
  el.removeAttribute('data-parallax-axis')
  el.removeAttribute('data-parallax-clamp')
  el.removeAttribute('data-parallax-pointer')
  el.removeAttribute('data-parallax-pointer-axis')
  el.removeAttribute('data-parallax-pointer-clamp')
  el.removeAttribute('data-parallax-float')
}

/**
 * Apply scene data attributes to element
 */
function applySceneAttributes(el: HTMLElement, value: SceneValue) {
  if (value === false) {
    el.removeAttribute('data-scrolly-scene')
    el.removeAttribute('data-scrolly-scene-id')
    el.removeAttribute('data-scrolly-story')
    return
  }

  el.setAttribute('data-scrolly-scene', 'true')

  // Extract name and story mode from value
  let name: string | undefined
  let story = false

  if (typeof value === 'string' && value.length > 0) {
    name = value
  } else if (typeof value === 'object' && value) {
    name = value.id
    story = value.story ?? false
  }

  if (name) {
    el.setAttribute('data-scrolly-scene-id', name)
  } else {
    el.removeAttribute('data-scrolly-scene-id')
  }

  if (story) {
    el.setAttribute('data-scrolly-story', 'true')
  } else {
    el.removeAttribute('data-scrolly-story')
  }
}

export default defineNuxtPlugin(nuxtApp => {
  /**
   * v-parallax directive
   *
   * Usage:
   *   <div v-parallax />                      // Default: speed 0.12, axis y, clamp 160
   *   <div v-parallax="0.14" />               // Custom speed
   *   <div v-parallax="{ speed: 0.12, axis: 'x', clamp: 120, pointer: 0.5, floating: true }" />
   *
   * Options:
   *   - speed: number (default: 0.12) - Scroll parallax intensity (0-1)
   *   - axis: 'x' | 'y' | 'both' (default: 'y') - Movement axis
   *   - clamp: number (default: 160) - Max travel in pixels
   *   - pointer: number (default: 0) - Pointer parallax intensity (0-2)
   *   - pointerAxis: 'x' | 'y' | 'both' (default: 'both') - Pointer movement axis
   *   - pointerClamp: number (default: 24) - Pointer travel clamp in pixels
   *   - floating: boolean (default: false) - Enable floating animation
   */
  nuxtApp.vueApp.directive('parallax', {
    getSSRProps(binding: DirectiveBinding<ParallaxValue>) {
      const options = typeof binding.value === 'number'
        ? { speed: binding.value }
        : (binding.value || {})

      const speed = options.speed ?? 0.12
      const axis = options.axis ?? 'y'
      const clamp = options.clamp ?? 160
      const pointer = options.pointer ?? 0
      const pointerAxis = options.pointerAxis ?? 'both'
      const pointerClamp = options.pointerClamp ?? 24
      const floating = options.floating ?? false

      const props: Record<string, string | boolean> = {
        'data-parallax': 'true',
        'data-parallax-speed': String(speed),
        'data-parallax-axis': axis,
        'data-parallax-clamp': String(clamp)
      }

      if (pointer > 0) {
        props['data-parallax-pointer'] = String(pointer)
        props['data-parallax-pointer-axis'] = pointerAxis
        props['data-parallax-pointer-clamp'] = String(pointerClamp)
      }

      if (floating) {
        props['data-parallax-float'] = 'true'
      }

      return props
    },

    mounted(el: HTMLElement, binding: DirectiveBinding<ParallaxValue>) {
      applyParallaxAttributes(el, binding.value)
    },

    updated(el: HTMLElement, binding: DirectiveBinding<ParallaxValue>) {
      applyParallaxAttributes(el, binding.value)
    },

    unmounted(el: HTMLElement) {
      clearParallaxAttributes(el)
    }
  })

  /**
   * v-scrolly-scene directive
   *
   * Usage:
   *   <section id="about" v-scrolly-scene />           // Auto name from id
   *   <section v-scrolly-scene="'about'" />            // Explicit name
   *   <section v-scrolly-scene="{ id: 'about', story: true }" />  // Story mode
   *
   * Options:
   *   - id: string - Scene name (falls back to element id)
   *   - story: boolean (default: false) - Enable enhanced scene tracking
   */
  nuxtApp.vueApp.directive('scrolly-scene', {
    getSSRProps(binding: DirectiveBinding<SceneValue>) {
      if (binding.value === false) {
        return {}
      }

      if (typeof binding.value === 'string' && binding.value.length > 0) {
        return {
          'data-scrolly-scene': 'true',
          'data-scrolly-scene-id': binding.value
        }
      }

      if (typeof binding.value === 'object' && binding.value) {
        const props: Record<string, string | boolean> = {
          'data-scrolly-scene': 'true'
        }

        if (binding.value.id) {
          props['data-scrolly-scene-id'] = binding.value.id
        }

        if (binding.value.story) {
          props['data-scrolly-story'] = 'true'
        }

        return props
      }

      return { 'data-scrolly-scene': 'true' }
    },

    mounted(el: HTMLElement, binding: DirectiveBinding<SceneValue>) {
      applySceneAttributes(el, binding.value)
    },

    updated(el: HTMLElement, binding: DirectiveBinding<SceneValue>) {
      applySceneAttributes(el, binding.value)
    },

    unmounted(el: HTMLElement) {
      el.removeAttribute('data-scrolly-scene')
      el.removeAttribute('data-scrolly-scene-id')
      el.removeAttribute('data-scrolly-story')
    }
  })
})
