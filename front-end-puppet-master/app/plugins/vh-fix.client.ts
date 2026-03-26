/**
 * iOS Safari 100vh Fix
 * Sets --vh CSS variable to actual viewport height
 */

export default defineNuxtPlugin(() => {
  function setVh() {
    const vh = window.innerHeight * 0.01
    document.documentElement.style.setProperty('--vh', `${vh}px`)
  }

  setVh()
  window.addEventListener('resize', setVh)
  window.addEventListener('orientationchange', setVh)
})
