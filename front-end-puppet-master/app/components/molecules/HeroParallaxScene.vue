<script setup lang="ts">
/**
 * HeroParallaxScene Molecule Component
 *
 * Combines cloud layers and landmark cards with v-parallax directive
 * for multi-depth parallax scrolling effects in hero sections.
 *
 * @example
 * ```vue
 * <template>
 *   <section class="section section-hero">
 *     <!-- Parallax scene behind content -->
 *     <HeroParallaxScene />
 *
 *     <!-- Hero content -->
 *     <div class="container">
 *       <h1>Welcome</h1>
 *     </div>
 *   </section>
 * </template>
 * ```
 */

import { getRasterImagePath } from '~/utils/image-assets'

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

interface ParallaxConfig {
  speed: number
  axis: 'x' | 'y' | 'both'
  clamp: number
  pointer: number
  pointerAxis: 'x' | 'y' | 'both'
  pointerClamp: number
  floating: boolean
}

interface HeroLayer {
  key: string
  className: string
  file: string
  reverseFloat: boolean
  parallax: ParallaxConfig
}

// ═══════════════════════════════════════════════════════════════════════════
// LAYER CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Parallax layers configuration
 * Ordered from back (far) to front (near) for proper depth perception
 */
const layers: HeroLayer[] = [
  // Cloud layers (background atmosphere)
  {
    key: 'cloud-far',
    className: 'hero-parallax-scene__cloud hero-parallax-scene__cloud--far',
    file: 'hero/layers/clouds/far',
    reverseFloat: true,
    parallax: {
      speed: 0.014,
      axis: 'y',
      clamp: 22,
      pointer: 0.22,
      pointerAxis: 'both',
      pointerClamp: 18,
      floating: true
    }
  },
  {
    key: 'cloud-mid',
    className: 'hero-parallax-scene__cloud hero-parallax-scene__cloud--mid',
    file: 'hero/layers/clouds/mid',
    reverseFloat: false,
    parallax: {
      speed: 0.022,
      axis: 'y',
      clamp: 34,
      pointer: 0.34,
      pointerAxis: 'both',
      pointerClamp: 24,
      floating: true
    }
  },
  {
    key: 'cloud-front',
    className: 'hero-parallax-scene__cloud hero-parallax-scene__cloud--front',
    file: 'hero/layers/clouds/front',
    reverseFloat: true,
    parallax: {
      speed: 0.034,
      axis: 'y',
      clamp: 48,
      pointer: 0.48,
      pointerAxis: 'both',
      pointerClamp: 32,
      floating: true
    }
  },
  // Landmark cards (foreground interest points)
  {
    key: 'landmark-left-top',
    className: 'hero-parallax-scene__landmark hero-parallax-scene__landmark--left-top',
    file: 'hero/layers/landmarks/left-top',
    reverseFloat: false,
    parallax: {
      speed: 0.048,
      axis: 'y',
      clamp: 64,
      pointer: 0.64,
      pointerAxis: 'both',
      pointerClamp: 42,
      floating: false
    }
  },
  {
    key: 'landmark-right-bottom',
    className: 'hero-parallax-scene__landmark hero-parallax-scene__landmark--right-bottom',
    file: 'hero/layers/landmarks/right-bottom',
    reverseFloat: true,
    parallax: {
      speed: 0.056,
      axis: 'y',
      clamp: 78,
      pointer: 0.78,
      pointerAxis: 'both',
      pointerClamp: 52,
      floating: false
    }
  }
]

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Get layer image path - SVG files use direct path
 */
function getLayerSrc(file: string): string {
  // SVG files are not processed through raster pipeline
  return `/images/${file}.svg`
}

/**
 * Get parallax directive value
 */
function getParallaxValue(config: ParallaxConfig): Record<string, unknown> {
  return {
    speed: config.speed,
    axis: config.axis,
    clamp: config.clamp,
    pointer: config.pointer,
    pointerAxis: config.pointerAxis,
    pointerClamp: config.pointerClamp,
    float: config.floating
  }
}
</script>

<template>
  <!--
    Hero Parallax Scene
    Decorative parallax layers behind hero content
    aria-hidden: decorative only, content is in parent section
  -->
  <ClientOnly>
    <div
      class="hero-parallax-scene"
      aria-hidden="true"
    >
      <div
        v-for="layer in layers"
        :key="layer.key"
        v-parallax="getParallaxValue(layer.parallax)"
        :class="layer.className"
        :data-parallax-float="layer.parallax.floating ? 'true' : null"
        :data-parallax-float-reverse="layer.reverseFloat ? 'true' : null"
      >
        <img
          :src="getLayerSrc(layer.file)"
          alt=""
          loading="lazy"
          decoding="async"
          fetchpriority="low"
        />
      </div>
    </div>
  </ClientOnly>
</template>

<!-- No scoped styles - uses global CSS from app/assets/css/ui/content/hero-parallax.css -->
