/**
 * useMediaQuery - Reactive CSS media query matching
 *
 * 2025 Responsive System Breakpoints:
 * - Mobile: < 640px
 * - Tablet: 640px - 1023px
 * - Desktop: ≥ 1024px
 * - Large: ≥ 1280px
 */

export function useMediaQuery(query: string) {
  const matches = ref(false)

  if (import.meta.client) {
    const media = window.matchMedia(query)
    matches.value = media.matches

    const handler = (e: MediaQueryListEvent) => {
      matches.value = e.matches
    }

    media.addEventListener('change', handler)

    onUnmounted(() => {
      media.removeEventListener('change', handler)
    })
  }

  return matches
}

// ══════════════════════════════════════════════════════════
// BREAKPOINT COMPOSABLES (match responsive.css)
// ══════════════════════════════════════════════════════════

/** True when viewport < 640px (phones) */
export function useIsMobile() {
  return useMediaQuery('(max-width: 639px)')
}

/** True when viewport 640px - 1023px (tablets, small laptops) */
export function useIsTablet() {
  return useMediaQuery('(min-width: 640px) and (max-width: 1023px)')
}

/** True when viewport ≥ 1024px (laptops, desktops) */
export function useIsDesktop() {
  return useMediaQuery('(min-width: 1024px)')
}

/** True when viewport ≥ 1280px (large desktops) */
export function useIsLargeDesktop() {
  return useMediaQuery('(min-width: 1280px)')
}

/** True when viewport ≥ 640px (tablet and up) */
export function useIsTabletUp() {
  return useMediaQuery('(min-width: 640px)')
}

// ══════════════════════════════════════════════════════════
// ORIENTATION COMPOSABLES
// ══════════════════════════════════════════════════════════

/** True when viewport is portrait (height > width) */
export function useIsPortrait() {
  return useMediaQuery('(orientation: portrait)')
}

/** True when viewport is landscape (width > height) */
export function useIsLandscape() {
  return useMediaQuery('(orientation: landscape)')
}

// ══════════════════════════════════════════════════════════
// USER PREFERENCE COMPOSABLES
// ══════════════════════════════════════════════════════════

/** True when user prefers reduced motion */
export function usePrefersReducedMotion() {
  return useMediaQuery('(prefers-reduced-motion: reduce)')
}

/** True when user prefers dark color scheme */
export function usePrefersDark() {
  return useMediaQuery('(prefers-color-scheme: dark)')
}

/** True when user prefers high contrast */
export function usePrefersHighContrast() {
  return useMediaQuery('(prefers-contrast: more)')
}

// ══════════════════════════════════════════════════════════
// DEVICE CAPABILITY COMPOSABLES
// ══════════════════════════════════════════════════════════

/** True when device supports hover (mouse/trackpad) */
export function useCanHover() {
  return useMediaQuery('(hover: hover)')
}

/** True when device has coarse pointer (touch) */
export function useIsTouch() {
  return useMediaQuery('(pointer: coarse)')
}
