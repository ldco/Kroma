/**
 * Fonts Plugin (Client-side)
 *
 * Loads Google Fonts asynchronously without render-blocking.
 * This is CSP-compliant because it's a bundled script, not an inline event handler.
 *
 * Comment 8: Font CSS is preloaded in nuxt.config.ts for SSR.
 * This plugin ensures the stylesheet is applied non-blockingly.
 * Uses media="print" trick to prevent render-blocking, then switches to "all".
 */
export default defineNuxtPlugin(() => {
  const fontUrl = 'https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;600;700;900&display=swap'

  // Check if preload link already exists (from SSR)
  const existingPreload = document.querySelector(`link[rel="preload"][href="${fontUrl}"]`)
  
  // Create stylesheet with media="print" - browser loads but doesn't block render
  const link = document.createElement('link')
  link.rel = 'stylesheet'
  link.href = fontUrl
  link.media = 'print'

  // When loaded, switch to media="all" to apply styles
  link.onload = function () {
    link.media = 'all'
    // Remove preload link once stylesheet is loaded
    if (existingPreload) {
      existingPreload.remove()
    }
  }

  // Insert immediately - the media="print" ensures it's non-blocking
  document.head.appendChild(link)
})
