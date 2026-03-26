/**
 * Admin Store (Composition API)
 *
 * Pinia store for admin panel UI state.
 * Handles: sidebar, unsaved changes, navigation warnings.
 *
 * When to use Pinia vs useState():
 * - Pinia: Complex state, DevTools debugging, multiple related values
 * - useState(): Simple shared state, single values
 */
import { defineStore } from 'pinia'

export const useAdminStore = defineStore('admin', () => {
  // ═══════════════════════════════════════════════════════════════
  // STATE
  // ═══════════════════════════════════════════════════════════════

  // Sidebar collapsed state (persists in localStorage)
  const sidebarCollapsed = ref(false)

  // Track unsaved changes for navigation warning
  const unsavedChanges = ref(false)
  const unsavedFormId = ref<string | null>(null)

  // Current admin section for breadcrumbs/highlighting
  const currentSection = ref<string>('dashboard')

  // Mobile nav open state
  const mobileNavOpen = ref(false)

  // ═══════════════════════════════════════════════════════════════
  // GETTERS (Computed)
  // ═══════════════════════════════════════════════════════════════

  const hasUnsavedChanges = computed(() => unsavedChanges.value)

  const sidebarWidth = computed(() => (sidebarCollapsed.value ? '64px' : '240px'))

  // ═══════════════════════════════════════════════════════════════
  // ACTIONS
  // ═══════════════════════════════════════════════════════════════

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
    // Persist preference
    if (import.meta.client) {
      localStorage.setItem('admin-sidebar-collapsed', String(sidebarCollapsed.value))
    }
  }

  function collapseSidebar() {
    sidebarCollapsed.value = true
  }

  function expandSidebar() {
    sidebarCollapsed.value = false
  }

  function setCurrentSection(section: string) {
    currentSection.value = section
  }

  function markUnsaved(formId: string) {
    unsavedChanges.value = true
    unsavedFormId.value = formId
  }

  function markSaved() {
    unsavedChanges.value = false
    unsavedFormId.value = null
  }

  function toggleMobileNav() {
    mobileNavOpen.value = !mobileNavOpen.value
  }

  function closeMobileNav() {
    mobileNavOpen.value = false
  }

  // Initialize from localStorage on client
  function hydrate() {
    if (import.meta.client) {
      const saved = localStorage.getItem('admin-sidebar-collapsed')
      if (saved !== null) {
        sidebarCollapsed.value = saved === 'true'
      }
    }
  }

  return {
    // State
    sidebarCollapsed,
    unsavedChanges,
    unsavedFormId,
    currentSection,
    mobileNavOpen,

    // Getters
    hasUnsavedChanges,
    sidebarWidth,

    // Actions
    toggleSidebar,
    collapseSidebar,
    expandSidebar,
    setCurrentSection,
    markUnsaved,
    markSaved,
    toggleMobileNav,
    closeMobileNav,
    hydrate
  }
})
