/**
 * useComponentHydration - Track component hydration state
 *
 * Prevents SSR hydration mismatches by deferring client-only state
 * until the component is mounted on the client.
 *
 * @example
 * ```vue
 * <script setup>
 * const { isHydrated } = useComponentHydration()
 * </script>
 *
 * <template>
 *   <div :class="{ 'active': isHydrated && isActive }">
 *     <!-- No hydration mismatch: active class applied after mount -->
 *   </div>
 * </template>
 * ```
 *
 * @see https://nuxt.com/docs/guide/going-further/experimental-features#inlinetemplate
 */

export function useComponentHydration() {
  const isHydrated = ref(false)

  onMounted(() => {
    isHydrated.value = true
  })

  return {
    isHydrated
  }
}
