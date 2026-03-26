/**
 * Performance Monitoring Composable
 *
 * Client-side performance tracking using Web Vitals and Performance API.
 * Provides metrics for Core Web Vitals, resource timing, and custom measurements.
 *
 * Features:
 * - Core Web Vitals (LCP, FID, CLS, TTFB, FCP)
 * - Resource timing analysis
 * - Custom performance marks
 * - Server-Timing header parsing
 * - Performance reporting
 */

export interface WebVitalsMetric {
  name: string
  value: number
  rating: 'good' | 'needs-improvement' | 'poor'
  delta: number
}

export interface ResourceTiming {
  name: string
  type: string
  duration: number
  transferSize: number
  encodedBodySize: number
  decodedBodySize: number
}

export interface PerformanceReport {
  url: string
  timestamp: number
  webVitals: Record<string, WebVitalsMetric>
  resourceTimings: ResourceTiming[]
  serverTimings: Record<string, number>
  customMarks: Record<string, number>
  navigationTiming: {
    domContentLoaded: number
    loadComplete: number
    firstByte: number
  }
}

// Web Vitals thresholds (based on Google's recommendations)
const VITALS_THRESHOLDS = {
  LCP: { good: 2500, poor: 4000 },
  FID: { good: 100, poor: 300 },
  CLS: { good: 0.1, poor: 0.25 },
  TTFB: { good: 800, poor: 1800 },
  FCP: { good: 1800, poor: 3000 },
  INP: { good: 200, poor: 500 }
}

function getRating(metricName: string, value: number): 'good' | 'needs-improvement' | 'poor' {
  const thresholds = VITALS_THRESHOLDS[metricName as keyof typeof VITALS_THRESHOLDS]
  if (!thresholds) return 'good'

  if (value <= thresholds.good) return 'good'
  if (value > thresholds.poor) return 'poor'
  return 'needs-improvement'
}

export function usePerformance() {
  const webVitals = ref<Record<string, WebVitalsMetric>>({})
  const resourceTimings = ref<ResourceTiming[]>([])
  const serverTimings = ref<Record<string, number>>({})
  const customMarks = ref<Record<string, number>>({})
  const isSupported = ref(typeof window !== 'undefined' && 'performance' in window)

  /**
   * Record a Core Web Vital
   */
  function recordWebVital(name: string, value: number, delta: number = 0) {
    webVitals.value[name] = {
      name,
      value,
      rating: getRating(name, value),
      delta
    }
  }

  /**
   * Initialize Core Web Vitals observers
   */
  function initWebVitals() {
    if (!isSupported.value) return

    // Largest Contentful Paint (LCP)
    if ('PerformanceObserver' in window) {
      try {
        const lcpObserver = new PerformanceObserver(entryList => {
          const entries = entryList.getEntries()
          const lastEntry = entries[entries.length - 1] as PerformanceEntry & { startTime: number }
          recordWebVital('LCP', lastEntry.startTime)
        })
        lcpObserver.observe({ type: 'largest-contentful-paint', buffered: true })
      } catch {
        // LCP not supported
      }

      // First Input Delay (FID)
      try {
        const fidObserver = new PerformanceObserver(entryList => {
          const entries = entryList.getEntries()
          const firstEntry = entries[0] as PerformanceEntry & { processingStart: number; startTime: number }
          if (firstEntry) {
            recordWebVital('FID', firstEntry.processingStart - firstEntry.startTime)
          }
        })
        fidObserver.observe({ type: 'first-input', buffered: true })
      } catch {
        // FID not supported
      }

      // Cumulative Layout Shift (CLS)
      try {
        let clsValue = 0
        const clsObserver = new PerformanceObserver(entryList => {
          for (const entry of entryList.getEntries() as Array<PerformanceEntry & { hadRecentInput: boolean; value: number }>) {
            if (!entry.hadRecentInput) {
              clsValue += entry.value
            }
          }
          recordWebVital('CLS', clsValue)
        })
        clsObserver.observe({ type: 'layout-shift', buffered: true })
      } catch {
        // CLS not supported
      }
    }

    // First Contentful Paint (FCP) & Time to First Byte (TTFB) from Navigation Timing
    if ('getEntriesByType' in performance) {
      const navigationEntries = performance.getEntriesByType('navigation') as PerformanceNavigationTiming[]
      if (navigationEntries.length > 0) {
        const nav = navigationEntries[0]
        recordWebVital('TTFB', nav.responseStart - nav.requestStart)
      }

      // FCP from paint entries
      const paintEntries = performance.getEntriesByType('paint')
      const fcp = paintEntries.find(entry => entry.name === 'first-contentful-paint')
      if (fcp) {
        recordWebVital('FCP', fcp.startTime)
      }
    }
  }

  /**
   * Collect resource timing information
   */
  function collectResourceTimings(): ResourceTiming[] {
    if (!isSupported.value || !('getEntriesByType' in performance)) return []

    const resources = performance.getEntriesByType('resource') as PerformanceResourceTiming[]

    return resources.map(resource => ({
      name: resource.name,
      type: resource.initiatorType,
      duration: resource.duration,
      transferSize: resource.transferSize || 0,
      encodedBodySize: resource.encodedBodySize || 0,
      decodedBodySize: resource.decodedBodySize || 0
    }))
  }

  /**
   * Parse Server-Timing header from a response
   */
  function parseServerTiming(response: Response): Record<string, number> {
    const header = response.headers.get('Server-Timing')
    if (!header) return {}

    const timings: Record<string, number> = {}
    const entries = header.split(',')

    for (const entry of entries) {
      const match = entry.trim().match(/^(\w+);dur=([0-9.]+)/)
      if (match) {
        timings[match[1]] = parseFloat(match[2])
      }
    }

    serverTimings.value = { ...serverTimings.value, ...timings }
    return timings
  }

  /**
   * Create a custom performance mark
   */
  function mark(name: string) {
    if (!isSupported.value) return

    performance.mark(name)
    customMarks.value[name] = performance.now()
  }

  /**
   * Measure time between two marks
   */
  function measure(name: string, startMark: string, endMark?: string): number | null {
    if (!isSupported.value) return null

    try {
      const measureEntry = performance.measure(name, startMark, endMark)
      customMarks.value[name] = measureEntry.duration
      return measureEntry.duration
    } catch {
      return null
    }
  }

  /**
   * Start a timer and return a function to stop it
   */
  function startTimer(name: string) {
    const startMark = `${name}-start`
    mark(startMark)

    return {
      stop: () => {
        const endMark = `${name}-end`
        mark(endMark)
        return measure(name, startMark, endMark)
      }
    }
  }

  /**
   * Get navigation timing metrics
   */
  function getNavigationTiming() {
    if (!isSupported.value || !('getEntriesByType' in performance)) {
      return { domContentLoaded: 0, loadComplete: 0, firstByte: 0 }
    }

    const navigationEntries = performance.getEntriesByType('navigation') as PerformanceNavigationTiming[]
    if (navigationEntries.length === 0) {
      return { domContentLoaded: 0, loadComplete: 0, firstByte: 0 }
    }

    const nav = navigationEntries[0]
    return {
      domContentLoaded: nav.domContentLoadedEventEnd - nav.startTime,
      loadComplete: nav.loadEventEnd - nav.startTime,
      firstByte: nav.responseStart - nav.requestStart
    }
  }

  /**
   * Generate a complete performance report
   */
  function generateReport(): PerformanceReport {
    resourceTimings.value = collectResourceTimings()

    return {
      url: window.location.href,
      timestamp: Date.now(),
      webVitals: { ...webVitals.value },
      resourceTimings: resourceTimings.value,
      serverTimings: { ...serverTimings.value },
      customMarks: { ...customMarks.value },
      navigationTiming: getNavigationTiming()
    }
  }

  /**
   * Send performance report to analytics endpoint
   */
  async function sendReport(endpoint: string = '/api/analytics/performance') {
    const report = generateReport()

    try {
      // Use sendBeacon for reliability
      if ('sendBeacon' in navigator) {
        navigator.sendBeacon(endpoint, JSON.stringify(report))
      } else {
        await fetch(endpoint, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(report),
          keepalive: true
        })
      }
    } catch {
      // Silently fail - analytics shouldn't break the app
    }
  }

  /**
   * Get a summary of current performance
   */
  const summary = computed(() => {
    const vitals = Object.values(webVitals.value)
    const goodCount = vitals.filter(v => v.rating === 'good').length
    const poorCount = vitals.filter(v => v.rating === 'poor').length

    let overallRating: 'good' | 'needs-improvement' | 'poor'
    if (poorCount > 0) {
      overallRating = 'poor'
    } else if (goodCount === vitals.length) {
      overallRating = 'good'
    } else {
      overallRating = 'needs-improvement'
    }

    return {
      overallRating,
      vitalsCount: vitals.length,
      goodCount,
      poorCount,
      needsImprovementCount: vitals.length - goodCount - poorCount
    }
  })

  // Initialize on mount
  onMounted(() => {
    if (isSupported.value) {
      initWebVitals()
    }
  })

  return {
    // State
    webVitals: readonly(webVitals),
    resourceTimings: readonly(resourceTimings),
    serverTimings: readonly(serverTimings),
    customMarks: readonly(customMarks),
    isSupported: readonly(isSupported),
    summary,

    // Methods
    recordWebVital,
    collectResourceTimings,
    parseServerTiming,
    mark,
    measure,
    startTimer,
    getNavigationTiming,
    generateReport,
    sendReport
  }
}
