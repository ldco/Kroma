/**
 * Image Assets Utilities
 *
 * Helper functions for working with optimized image assets.
 * Provides format-aware paths and responsive image sets.
 *
 * @packageDocumentation
 */

// ═══════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════

const IMAGES_BASE_PATH = '/images'

/**
 * Responsive image size configuration
 */
export interface ImageSizeConfig {
  maxWidth: number
  imageWidth: number
}

/**
 * Background image set result
 */
export interface BackgroundImageSet {
  /** CSS image-set() value for background-image */
  imageSet: string
  /** PNG fallback URL for older browsers */
  pngFallback: string
}

// ═══════════════════════════════════════════════════════════════════════════
// PATH HELPERS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Get the base path for an image without extension
 */
function getImageBasePath(path: string): string {
  // Remove leading slash if present
  const cleanPath = path.startsWith('/') ? path.slice(1) : path
  // Remove extension if present
  const withoutExt = cleanPath.replace(/\.(png|jpg|jpeg|webp|avif|svg)$/, '')
  return `${IMAGES_BASE_PATH}/${withoutExt}`
}

// ═══════════════════════════════════════════════════════════════════════════
// RASTER IMAGE PATHS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Get the appropriate image path based on environment and format.
 *
 * @param path - Image path without extension (e.g., 'hero/banner')
 * @param format - Optional format override ('avif' | 'webp' | 'png')
 * @returns Full image path
 *
 * @example
 * ```typescript
 * // Production: '/images/hero/banner.avif'
 * // Development: '/images/hero/banner.png'
 * const defaultPath = getRasterImagePath('hero/banner')
 *
 * // Force specific format
 * const avifPath = getRasterImagePath('hero/banner', 'avif')
 * const webpPath = getRasterImagePath('hero/banner', 'webp')
 * const pngPath = getRasterImagePath('hero/banner', 'png')
 * ```
 */
export function getRasterImagePath(path: string, format?: 'avif' | 'webp' | 'png'): string {
  const basePath = getImageBasePath(path)
  const isProduction = import.meta.env.PROD

  if (format) {
    return `${basePath}.${format}`
  }

  // Production: prefer AVIF
  // Development: use PNG for faster builds and debugging
  return isProduction ? `${basePath}.avif` : `${basePath}.png`
}

/**
 * Get multiple format paths for responsive images
 *
 * @param path - Image path without extension
 * @returns Object with all format paths
 */
export function getResponsiveImagePaths(path: string): {
  avif: string
  webp: string
  png: string
} {
  const basePath = getImageBasePath(path)
  return {
    avif: `${basePath}.avif`,
    webp: `${basePath}.webp`,
    png: `${basePath}.png`
  }
}

// ═══════════════════════════════════════════════════════════════════════════
// SRCSET & SIZES
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Generate srcset attribute value for responsive images
 *
 * @param path - Image path without extension
 * @param widths - Array of widths to generate
 * @param format - Image format (default: 'avif')
 * @returns srcset attribute value
 *
 * @example
 * ```typescript
 * const srcset = getSrcSet('hero/image', [400, 800, 1600])
 * // '/images/hero/image-400.avif 400w, /images/hero/image-800.avif 800w, ...'
 * ```
 */
export function getSrcSet(
  path: string,
  widths: number[] = [400, 800, 1200, 1600],
  format: 'avif' | 'webp' | 'png' = 'avif'
): string {
  const basePath = getImageBasePath(path)
  return widths
    .map(width => `${basePath}-${width}.${format} ${width}w`)
    .join(', ')
}

/**
 * Generate sizes attribute value for responsive images
 *
 * @param configs - Array of size configurations
 * @returns sizes attribute value
 *
 * @example
 * ```typescript
 * const sizes = getSizes([
 *   { maxWidth: 768, imageWidth: 400 },
 *   { maxWidth: 1024, imageWidth: 800 },
 *   { maxWidth: Infinity, imageWidth: 1600 }
 * ])
 * // '(max-width: 768px) 400px, (max-width: 1024px) 800px, 1600px'
 * ```
 */
export function getSizes(configs: ImageSizeConfig[]): string {
  return configs
    .map(({ maxWidth, imageWidth }) => {
      if (maxWidth === Infinity) {
        return `${imageWidth}px`
      }
      return `(max-width: ${maxWidth}px) ${imageWidth}px`
    })
    .join(', ')
}

/**
 * Get default sizes for common layouts
 */
export function getDefaultSizes(layout: 'hero' | 'thumbnail' | 'full'): string {
  switch (layout) {
    case 'hero':
      return getSizes([
        { maxWidth: 768, imageWidth: 800 },
        { maxWidth: 1024, imageWidth: 1200 },
        { maxWidth: Infinity, imageWidth: 1600 }
      ])
    case 'thumbnail':
      return getSizes([
        { maxWidth: 768, imageWidth: 200 },
        { maxWidth: 1024, imageWidth: 300 },
        { maxWidth: Infinity, imageWidth: 400 }
      ])
    case 'full':
      return getSizes([
        { maxWidth: 768, imageWidth: 640 },
        { maxWidth: 1024, imageWidth: 1024 },
        { maxWidth: Infinity, imageWidth: 1920 }
      ])
    default:
      return '100vw'
  }
}

// ═══════════════════════════════════════════════════════════════════════════
// BACKGROUND IMAGES
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Generate CSS image-set() for background images with fallbacks
 *
 * @param path - Image path without extension
 * @returns Object with image-set and PNG fallback
 *
 * @example
 * ```typescript
 * const { imageSet, pngFallback } = getBackgroundImageSet('backgrounds/site-bg')
 *
 * // Output (production):
 * // imageSet: 'image-set(url("/images/.../site-bg.avif") type("image/avif"), url("/images/.../site-bg.webp") type("image/webp"), url("/images/.../site-bg.png") type("image/png"))'
 * // pngFallback: 'url("/images/.../site-bg.png")'
 *
 * // Usage in component:
 * <div :style="{ backgroundImage: imageSet }" />
 * ```
 */
export function getBackgroundImageSet(path: string): BackgroundImageSet {
  const basePath = getImageBasePath(path)

  const avifUrl = `url("${basePath}.avif")`
  const webpUrl = `url("${basePath}.webp")`
  const pngUrl = `url("${basePath}.png")`

  const imageSet = `image-set(${avifUrl} type("image/avif"), ${webpUrl} type("image/webp"), ${pngUrl} type("image/png"))`

  return {
    imageSet,
    pngFallback: pngUrl
  }
}

/**
 * Generate background-image CSS value with progressive enhancement
 *
 * @param path - Image path without extension
 * @returns CSS background-image value
 *
 * @example
 * ```typescript
 * const bgImage = getBackgroundImage('hero/banner')
 * // Returns: 'image-set(url("/images/hero/banner.avif") type("image/avif"), ...)'
 * ```
 */
export function getBackgroundImage(path: string): string {
  return getBackgroundImageSet(path).imageSet
}

// ═══════════════════════════════════════════════════════════════════════════
// PICTURE SOURCE
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Generate picture source configuration for <picture> element
 *
 * @param path - Image path without extension
 * @param widths - Array of widths for srcset
 * @returns Array of source configurations
 *
 * @example
 * ```typescript
 * const sources = getPictureSources('hero/banner', [400, 800, 1600])
 * // [
 * //   { media: '(max-width: 768px)', srcset: '...', type: 'image/avif' },
 * //   { media: '(min-width: 769px)', srcset: '...', type: 'image/avif' },
 * //   { srcset: '...', type: 'image/png' } // fallback
 * // ]
 * ```
 */
export function getPictureSources(
  path: string,
  widths: number[] = [400, 800, 1200, 1600]
): Array<{
  media?: string
  srcset: string
  type: string
}> {
  const avifSrcSet = getSrcSet(path, widths, 'avif')
  const webpSrcSet = getSrcSet(path, widths, 'webp')
  const pngSrcSet = getSrcSet(path, widths, 'png')

  return [
    {
      media: '(max-width: 768px)',
      srcset: getSrcSet(path, widths.slice(0, 2), 'avif'),
      type: 'image/avif'
    },
    {
      media: '(min-width: 769px)',
      srcset: getSrcSet(path, widths.slice(1), 'avif'),
      type: 'image/avif'
    },
    {
      srcset: webpSrcSet,
      type: 'image/webp'
    },
    {
      srcset: pngSrcSet,
      type: 'image/png'
    }
  ]
}

// ═══════════════════════════════════════════════════════════════════════════
// LAZY LOADING
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Get loading attributes based on image position
 *
 * @param position - Image position ('above-fold' | 'below-fold')
 * @returns Loading attributes object
 *
 * @example
 * ```typescript
 * // For hero/above-fold images
 * const heroAttrs = getLoadingAttrs('above-fold')
 * // { loading: 'eager', fetchpriority: 'high', decoding: 'sync' }
 *
 * // For below-fold images
 * const lazyAttrs = getLoadingAttrs('below-fold')
 * // { loading: 'lazy', fetchpriority: 'low', decoding: 'async' }
 * ```
 */
export function getLoadingAttrs(position: 'above-fold' | 'below-fold' = 'below-fold'): {
  loading: 'eager' | 'lazy'
  fetchpriority: 'high' | 'low' | 'auto'
  decoding: 'sync' | 'async' | 'auto'
} {
  if (position === 'above-fold') {
    return {
      loading: 'eager',
      fetchpriority: 'high',
      decoding: 'sync'
    }
  }

  return {
    loading: 'lazy',
    fetchpriority: 'low',
    decoding: 'async'
  }
}
