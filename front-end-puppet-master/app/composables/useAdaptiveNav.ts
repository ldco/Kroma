/**
 * useAdaptiveNav
 *
 * Detects when navigation items overflow their container
 * and signals when to collapse to hamburger menu.
 *
 * Uses ResizeObserver to detect size changes and checks
 * if nav content exceeds available space.
 */
export function useAdaptiveNav() {
  const headerRef = ref<HTMLElement | null>(null)
  const navRef = ref<HTMLElement | null>(null)
  const isCollapsed = ref(false)

  // Track if we're on client
  const isClient = import.meta.client

  function checkOverflow() {
    if (!isClient || !headerRef.value || !navRef.value) return

    const header = headerRef.value
    const nav = navRef.value

    // Get header inner container
    const headerInner = header.querySelector('.header-inner') as HTMLElement
    if (!headerInner) return

    // Calculate available space for nav
    // Header inner width - logo width - actions width - gaps
    const headerWidth = headerInner.clientWidth
    const logo = headerInner.querySelector('.header-logo') as HTMLElement
    const actions = headerInner.querySelector('.header-actions') as HTMLElement

    const logoWidth = logo?.offsetWidth || 0

    // IMPORTANT: Always measure actions width as if nav is UNCOLLAPSED
    // When collapsed, actions are hidden (width=0), causing an oscillation loop
    // So we cache the uncollapsed width and always use that
    let actionsWidth = actions?.offsetWidth || 0
    if (actionsWidth > 0) {
      // Cache the uncollapsed width
      ;(checkOverflow as any)._cachedActionsWidth = actionsWidth
    } else if ((checkOverflow as any)._cachedActionsWidth) {
      // Use cached width when actions are hidden
      actionsWidth = (checkOverflow as any)._cachedActionsWidth
    } else {
      // Fallback estimate if no cached value
      actionsWidth = 176
    }

    // Account for gaps between items (logo-nav gap + nav-actions gap)
    const gaps = 48

    // Available space for nav
    const availableWidth = headerWidth - logoWidth - actionsWidth - gaps

    // Calculate natural nav width by summing all nav items
    // NOTE: scrollWidth doesn't work because flex containers shrink items
    // instead of creating overflow. We need to measure actual item widths.
    const navLinks = nav.querySelector('.nav-links') as HTMLElement
    const navItems = nav.querySelectorAll('.nav-link')
    let navWidth = 0

    navItems.forEach(item => {
      const el = item as HTMLElement
      // offsetWidth includes padding and border
      navWidth += el.offsetWidth
    })

    // Get actual gap from computed style (nav-links uses gap)
    let navGap = 8 // default fallback
    if (navLinks) {
      const computedGap = getComputedStyle(navLinks).gap
      if (computedGap && computedGap !== 'normal') {
        navGap = parseFloat(computedGap) || 8
      }
    }

    if (navItems.length > 1) {
      navWidth += navGap * (navItems.length - 1)
    }

    // IMPORTANT: Cache navWidth when nav is visible
    // When collapsed, nav is hidden (display:none) and items have ~0 width
    // This causes oscillation: collapse -> navWidth=0 -> uncollapse -> navWidth=1085 -> collapse...
    // Also invalidate cache if item count changes (e.g., config changed)
    const cachedItemCount = (checkOverflow as any)._cachedNavItemCount || 0
    if (navItems.length !== cachedItemCount) {
      // Item count changed, invalidate cache
      ;(checkOverflow as any)._cachedNavWidth = null
      ;(checkOverflow as any)._cachedNavItemCount = navItems.length
    }

    if (navWidth > 100) {
      // Nav is visible, cache its width
      ;(checkOverflow as any)._cachedNavWidth = navWidth
    } else if ((checkOverflow as any)._cachedNavWidth) {
      // Nav is hidden, use cached width
      navWidth = (checkOverflow as any)._cachedNavWidth
    }

    // Minimum available space at desktop breakpoint (1024px)
    // At min desktop: ~1024 - padding(64) - logo(200) - actions(176) - gaps(48) ≈ 536px
    // If nav fits in this space, use standard CSS breakpoint (no adaptive collapse)
    const MIN_DESKTOP_NAV_SPACE = 500

    // Only use adaptive collapse if nav is too wide for standard desktop breakpoint
    // Otherwise, let CSS media queries handle it naturally
    if (navWidth <= MIN_DESKTOP_NAV_SPACE) {
      isCollapsed.value = false
      return
    }

    // Collapse if nav doesn't fit (with 48px buffer for comfortable spacing)
    const shouldCollapse = navWidth > availableWidth - 48

    // Debug logging (remove in production)
    if (import.meta.dev) {
      console.log(
        `[AdaptiveNav] nav=${navWidth}px, avail=${availableWidth}px, collapse=${shouldCollapse} (items=${navItems.length})`
      )
    }

    isCollapsed.value = shouldCollapse
  }

  // Set up ResizeObserver
  onMounted(() => {
    if (!isClient) return

    // Initial check after DOM settles AND fonts load
    // Fonts affect text width, so we need to wait for them
    nextTick(async () => {
      // Wait for fonts to load (affects text measurements)
      if (document.fonts?.ready) {
        await document.fonts.ready
      }
      checkOverflow()
    })

    // Watch for resize
    const resizeObserver = new ResizeObserver(() => {
      checkOverflow()
    })

    if (headerRef.value) {
      resizeObserver.observe(headerRef.value)
    }

    // Also check on window resize (for font loading, etc.)
    window.addEventListener('resize', checkOverflow)

    // Cleanup
    onUnmounted(() => {
      resizeObserver.disconnect()
      window.removeEventListener('resize', checkOverflow)
    })
  })

  return {
    headerRef,
    navRef,
    isCollapsed
  }
}
