/**
 * Type declarations for unplugin-icons
 *
 * These icons are loaded at build time by unplugin-icons.
 * This file provides TypeScript type safety.
 */

declare module '~icons/tabler/*' {
  import type { FunctionalComponent, SVGAttributes } from 'vue'
  const component: FunctionalComponent<SVGAttributes>
  export default component
}
