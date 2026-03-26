interface ScrollytellingOptions {
  enabled?: boolean
  sceneSelector?: string
  parallaxSelector?: string
}

type ParallaxAxis = 'x' | 'y' | 'both'
type PointerAxis = 'x' | 'y' | 'both'

interface ParallaxTarget {
  element: HTMLElement
  speed: number
  clamp: number
  axis: ParallaxAxis
  scene: HTMLElement | null
  // Pointer parallax
  pointer: number
  pointerAxis: PointerAxis
  pointerClamp: number
  // Floating animation
  floating: boolean
}

interface RuntimeState {
  enabled: boolean
  scenes: HTMLElement[]
  parallax: ParallaxTarget[]
  rafId: number | null
  observer: MutationObserver | null
  mediaQuery: MediaQueryList | null
  reducedMotion: boolean
  sceneSelector: string
  parallaxSelector: string
  // Pointer tracking
  pointerX: number
  pointerY: number
  finePointer: boolean
  finePointerQuery: MediaQueryList | null
}

const DEFAULT_SCENE_SELECTOR = '[data-scrolly-scene], .scene-full-bleed'
const DEFAULT_PARALLAX_SELECTOR = '[data-parallax]'

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value))
}

function parseAxis(value: string | null): ParallaxAxis {
  if (value === 'x' || value === 'y' || value === 'both') {
    return value
  }
  return 'y'
}

function parsePointerAxis(value: string | null): PointerAxis {
  if (value === 'x' || value === 'y' || value === 'both') {
    return value
  }
  return 'both'
}

function parseNumber(value: string | null, fallback: number): number {
  if (!value) return fallback
  const parsed = Number.parseFloat(value)
  return Number.isFinite(parsed) ? parsed : fallback
}

function getSceneProgress(element: HTMLElement): number {
  const raw =
    element.style.getPropertyValue('--pm-scene-progress') ||
    getComputedStyle(element).getPropertyValue('--pm-scene-progress')
  const parsed = Number.parseFloat(raw)
  return Number.isFinite(parsed) ? clamp01(parsed) : 0
}

function updateSceneProgress(element: HTMLElement): number {
  const rect = element.getBoundingClientRect()
  const viewportHeight = window.innerHeight
  const range = viewportHeight + rect.height
  const progress = range > 0 ? clamp01((viewportHeight - rect.top) / range) : 0
  element.style.setProperty('--pm-scene-progress', progress.toFixed(4))
  return progress
}

function buildParallaxTarget(element: HTMLElement, sceneSelector: string): ParallaxTarget {
  const speed = parseNumber(element.dataset.parallaxSpeed ?? null, 0.12)
  const clamp = Math.max(0, parseNumber(element.dataset.parallaxClamp ?? null, 160))
  const axis = parseAxis(element.dataset.parallaxAxis ?? null)

  // Pointer parallax (mouse movement)
  const pointer = parseNumber(element.dataset.parallaxPointer ?? null, 0)
  const pointerAxis = parsePointerAxis(element.dataset.parallaxPointerAxis ?? null)
  const pointerClamp = Math.max(0, parseNumber(element.dataset.parallaxPointerClamp ?? null, 24))

  // Floating animation
  const floating = element.dataset.parallaxFloat === 'true'

  const scene = element.closest<HTMLElement>(sceneSelector)

  return {
    element,
    speed,
    clamp,
    axis,
    scene,
    pointer,
    pointerAxis,
    pointerClamp,
    floating
  }
}

function createRuntime(
  sceneSelector: string,
  parallaxSelector: string
): RuntimeState {
  return {
    enabled: false,
    scenes: [],
    parallax: [],
    rafId: null,
    observer: null,
    mediaQuery: null,
    reducedMotion: false,
    sceneSelector,
    parallaxSelector,
    pointerX: 0,
    pointerY: 0,
    finePointer: false,
    finePointerQuery: null
  }
}

function refreshTargets(state: RuntimeState) {
  state.scenes = Array.from(document.querySelectorAll<HTMLElement>(state.sceneSelector))
  state.parallax = Array.from(document.querySelectorAll<HTMLElement>(state.parallaxSelector)).map(el =>
    buildParallaxTarget(el, state.sceneSelector)
  )
}

function applyParallax(state: RuntimeState) {
  for (const target of state.parallax) {
    const { element, axis, clamp, speed, scene, pointer, pointerAxis, pointerClamp } = target

    if (state.reducedMotion) {
      element.style.setProperty('--pm-parallax-x', '0px')
      element.style.setProperty('--pm-parallax-y', '0px')
      continue
    }

    // Scroll-driven parallax
    const sceneProgress = scene ? getSceneProgress(scene) : 0
    const centeredProgress = (sceneProgress - 0.5) * 2
    const scrollDelta = centeredProgress * clamp * speed

    // Pointer-driven parallax (only for fine-pointer devices like mouse)
    let pointerX = 0
    let pointerY = 0

    if (pointer > 0 && state.finePointer) {
      // Normalize pointer position to -1 to 1 range
      const normalizedX = state.pointerX * 2 - 1
      const normalizedY = state.pointerY * 2 - 1

      // Apply pointer intensity and clamp
      if (pointerAxis === 'x' || pointerAxis === 'both') {
        pointerX = normalizedX * pointer * pointerClamp
      }
      if (pointerAxis === 'y' || pointerAxis === 'both') {
        pointerY = normalizedY * pointer * pointerClamp
      }
    }

    // Combine scroll and pointer offsets
    const x = (axis === 'x' || axis === 'both' ? scrollDelta : 0) + pointerX
    const y = (axis === 'y' || axis === 'both' ? scrollDelta : 0) + pointerY

    element.style.setProperty('--pm-parallax-x', `${x.toFixed(2)}px`)
    element.style.setProperty('--pm-parallax-y', `${y.toFixed(2)}px`)
  }
}

function runFrame(state: RuntimeState) {
  const root = document.documentElement
  const maxScroll = Math.max(
    document.documentElement.scrollHeight - window.innerHeight,
    1
  )
  const scrollProgress = clamp01(window.scrollY / maxScroll)
  root.style.setProperty('--pm-scroll-progress', scrollProgress.toFixed(4))

  for (const scene of state.scenes) {
    updateSceneProgress(scene)
  }

  applyParallax(state)
  state.rafId = null
}

function requestFrame(state: RuntimeState) {
  if (!state.enabled) return
  if (state.rafId !== null) return
  state.rafId = window.requestAnimationFrame(() => runFrame(state))
}

function resetRuntime(state: RuntimeState) {
  document.documentElement.style.setProperty('--pm-scroll-progress', '0')

  for (const scene of state.scenes) {
    scene.style.setProperty('--pm-scene-progress', '0')
  }

  for (const target of state.parallax) {
    target.element.style.setProperty('--pm-parallax-x', '0px')
    target.element.style.setProperty('--pm-parallax-y', '0px')
  }
}

function attachRuntime(state: RuntimeState) {
  const onChange = () => {
    state.reducedMotion = !!state.mediaQuery?.matches
    requestFrame(state)
  }

  state.mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
  state.reducedMotion = state.mediaQuery.matches
  state.mediaQuery.addEventListener('change', onChange)

  // Check for fine pointer device (mouse vs touch) - persist for cleanup
  state.finePointerQuery = window.matchMedia('(pointer: fine)')
  state.finePointer = !state.reducedMotion && state.finePointerQuery.matches

  const onPointerChange = () => {
    state.finePointer = !state.reducedMotion && state.finePointerQuery!.matches
  }
  state.finePointerQuery.addEventListener('change', onPointerChange)

  // Track pointer position for pointer-driven parallax
  const onPointerMove = (event: PointerEvent) => {
    if (!state.finePointer) return
    if (event.pointerType !== 'mouse') return

    // Normalize to 0-1 range
    state.pointerX = event.clientX / window.innerWidth
    state.pointerY = event.clientY / window.innerHeight
    requestFrame(state)
  }

  window.addEventListener('pointermove', onPointerMove, { passive: true })

  const onResize = () => {
    refreshTargets(state)
    requestFrame(state)
  }

  const onScroll = () => requestFrame(state)

  window.addEventListener('scroll', onScroll, { passive: true })
  window.addEventListener('resize', onResize, { passive: true })

  // Comment 7: Observe specific mount element instead of entire document.body
  const mountElement = document.querySelector('.layout, .main, main') || document.body
  state.observer = new MutationObserver(() => {
    refreshTargets(state)
    requestFrame(state)
  })
  state.observer.observe(mountElement, { childList: true, subtree: true })

  // Store cleanup callbacks
  ;(state as RuntimeState & {
    _onScroll?: () => void
    _onResize?: () => void
    _onMediaChange?: () => void
    _onPointerChange?: () => void
    _onPointerMove?: (e: PointerEvent) => void
  })._onScroll = onScroll
  ;(state as RuntimeState & {
    _onScroll?: () => void
    _onResize?: () => void
    _onMediaChange?: () => void
    _onPointerChange?: () => void
    _onPointerMove?: (e: PointerEvent) => void
  })._onResize = onResize
  ;(state as RuntimeState & {
    _onScroll?: () => void
    _onResize?: () => void
    _onMediaChange?: () => void
    _onPointerChange?: () => void
    _onPointerMove?: (e: PointerEvent) => void
  })._onMediaChange = onChange
  ;(state as RuntimeState & {
    _onScroll?: () => void
    _onResize?: () => void
    _onMediaChange?: () => void
    _onPointerChange?: () => void
    _onPointerMove?: (e: PointerEvent) => void
  })._onPointerChange = onPointerChange
  ;(state as RuntimeState & {
    _onScroll?: () => void
    _onResize?: () => void
    _onMediaChange?: () => void
    _onPointerChange?: () => void
    _onPointerMove?: (e: PointerEvent) => void
  })._onPointerMove = onPointerMove

  refreshTargets(state)
  requestFrame(state)
}

function detachRuntime(state: RuntimeState) {
  const withCallbacks = state as RuntimeState & {
    _onScroll?: () => void
    _onResize?: () => void
    _onMediaChange?: () => void
    _onPointerChange?: () => void
    _onPointerMove?: (e: PointerEvent) => void
  }

  if (withCallbacks._onScroll) {
    window.removeEventListener('scroll', withCallbacks._onScroll)
  }
  if (withCallbacks._onResize) {
    window.removeEventListener('resize', withCallbacks._onResize)
  }
  if (state.mediaQuery && withCallbacks._onMediaChange) {
    state.mediaQuery.removeEventListener('change', withCallbacks._onMediaChange)
  }
  if (state.finePointerQuery && withCallbacks._onPointerChange) {
    state.finePointerQuery.removeEventListener('change', withCallbacks._onPointerChange)
  }
  if (withCallbacks._onPointerMove) {
    window.removeEventListener('pointermove', withCallbacks._onPointerMove)
  }

  if (state.observer) {
    state.observer.disconnect()
    state.observer = null
  }

  if (state.rafId !== null) {
    window.cancelAnimationFrame(state.rafId)
    state.rafId = null
  }

  resetRuntime(state)
}

/**
 * Global onepager scrollytelling runtime.
 * Provides CSS hooks for full-page and per-scene progress.
 */
export function useScrollytelling(options: ScrollytellingOptions = {}) {
  const {
    enabled = true,
    sceneSelector = DEFAULT_SCENE_SELECTOR,
    parallaxSelector = DEFAULT_PARALLAX_SELECTOR
  } = options

  if (!import.meta.client) {
    return { refresh: () => {} }
  }

  const runtime = useState<RuntimeState>('pm-scrollytelling-runtime', () =>
    createRuntime(sceneSelector, parallaxSelector)
  )

  function refresh() {
    refreshTargets(runtime.value)
    requestFrame(runtime.value)
  }

  onMounted(() => {
    runtime.value.sceneSelector = sceneSelector
    runtime.value.parallaxSelector = parallaxSelector
    runtime.value.enabled = enabled

    // Comment 7: Check enabled BEFORE attaching runtime to avoid unnecessary MutationObserver
    if (!runtime.value.enabled) {
      detachRuntime(runtime.value)
      return
    }

    attachRuntime(runtime.value)
  })

  onUnmounted(() => {
    detachRuntime(runtime.value)
  })

  return { refresh }
}
