/**
 * Pagination Composable
 *
 * Client-side pagination state management.
 * Works with the server-side pagination utility.
 *
 * Features:
 * - Reactive pagination state
 * - URL query sync
 * - Keyboard navigation
 * - Infinite scroll support
 */

export interface PaginationMeta {
  total: number
  page: number
  limit: number
  totalPages: number
  hasNextPage: boolean
  hasPreviousPage: boolean
  nextCursor?: number | string
  previousCursor?: number | string
}

export interface PaginationState {
  page: number
  limit: number
  sortBy: string
  sortOrder: 'asc' | 'desc'
  filters: Record<string, unknown>
}

export interface UsePaginationOptions {
  defaultLimit?: number
  defaultSortBy?: string
  defaultSortOrder?: 'asc' | 'desc'
  syncWithUrl?: boolean
  onPageChange?: (page: number) => void
}

export function usePagination(options: UsePaginationOptions = {}) {
  const {
    defaultLimit = 20,
    defaultSortBy = 'createdAt',
    defaultSortOrder = 'desc',
    syncWithUrl = true,
    onPageChange
  } = options

  const route = useRoute()
  const router = useRouter()

  // State
  const page = ref(1)
  const limit = ref(defaultLimit)
  const sortBy = ref(defaultSortBy)
  const sortOrder = ref<'asc' | 'desc'>(defaultSortOrder)
  const filters = ref<Record<string, unknown>>({})
  const meta = ref<PaginationMeta | null>(null)
  const isLoading = ref(false)

  // Computed
  const offset = computed(() => (page.value - 1) * limit.value)
  const hasNextPage = computed(() => meta.value?.hasNextPage ?? false)
  const hasPreviousPage = computed(() => meta.value?.hasPreviousPage ?? false)
  const totalPages = computed(() => meta.value?.totalPages ?? 0)
  const total = computed(() => meta.value?.total ?? 0)

  // Initialize from URL
  if (syncWithUrl) {
    const initFromUrl = () => {
      const query = route.query
      if (query.page) page.value = parseInt(String(query.page), 10) || 1
      if (query.limit) limit.value = parseInt(String(query.limit), 10) || defaultLimit
      if (query.sortBy) sortBy.value = String(query.sortBy)
      if (query.sortOrder && (query.sortOrder === 'asc' || query.sortOrder === 'desc')) {
        sortOrder.value = query.sortOrder
      }

      // Parse filters
      for (const [key, value] of Object.entries(query)) {
        const filterMatch = key.match(/^filter\[(\w+)\]$/)
        if (filterMatch) {
          filters.value[filterMatch[1]] = value
        }
      }
    }

    initFromUrl()

    // Watch route changes
    watch(
      () => route.query,
      () => initFromUrl(),
      { deep: true }
    )
  }

  // Sync to URL
  const syncToUrl = () => {
    if (!syncWithUrl) return

    const query: Record<string, string> = {
      page: String(page.value)
    }

    if (limit.value !== defaultLimit) {
      query.limit = String(limit.value)
    }
    if (sortBy.value !== defaultSortBy) {
      query.sortBy = sortBy.value
    }
    if (sortOrder.value !== defaultSortOrder) {
      query.sortOrder = sortOrder.value
    }

    // Add filters
    for (const [key, value] of Object.entries(filters.value)) {
      if (value !== undefined && value !== null && value !== '') {
        query[`filter[${key}]`] = String(value)
      }
    }

    router.replace({ query })
  }

  // Navigation methods
  const goToPage = (newPage: number) => {
    if (newPage < 1 || newPage > totalPages.value) return
    page.value = newPage
    syncToUrl()
    onPageChange?.(newPage)
  }

  const nextPage = () => {
    if (hasNextPage.value) {
      goToPage(page.value + 1)
    }
  }

  const previousPage = () => {
    if (hasPreviousPage.value) {
      goToPage(page.value - 1)
    }
  }

  const firstPage = () => {
    goToPage(1)
  }

  const lastPage = () => {
    if (totalPages.value > 0) {
      goToPage(totalPages.value)
    }
  }

  // Sorting
  const setSort = (field: string, order?: 'asc' | 'desc') => {
    if (sortBy.value === field && !order) {
      // Toggle order if same field
      sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
    } else {
      sortBy.value = field
      sortOrder.value = order || 'desc'
    }
    page.value = 1 // Reset to first page
    syncToUrl()
  }

  // Filtering
  const setFilter = (key: string, value: unknown) => {
    filters.value[key] = value
    page.value = 1 // Reset to first page
    syncToUrl()
  }

  const clearFilter = (key: string) => {
    delete filters.value[key]
    page.value = 1
    syncToUrl()
  }

  const clearAllFilters = () => {
    filters.value = {}
    page.value = 1
    syncToUrl()
  }

  // Update meta from API response
  const updateMeta = (newMeta: PaginationMeta) => {
    meta.value = newMeta
    // Ensure page is valid
    if (page.value > newMeta.totalPages && newMeta.totalPages > 0) {
      goToPage(newMeta.totalPages)
    }
  }

  // Build query params for API request
  const buildQueryParams = (): Record<string, string> => {
    const params: Record<string, string> = {
      page: String(page.value),
      limit: String(limit.value),
      sortBy: sortBy.value,
      sortOrder: sortOrder.value
    }

    for (const [key, value] of Object.entries(filters.value)) {
      if (value !== undefined && value !== null && value !== '') {
        params[`filter[${key}]`] = String(value)
      }
    }

    return params
  }

  // Generate page numbers for pagination UI
  const pageNumbers = computed(() => {
    const pages: (number | '...')[] = []
    const currentPage = page.value
    const total = totalPages.value

    if (total <= 7) {
      // Show all pages
      for (let i = 1; i <= total; i++) {
        pages.push(i)
      }
    } else {
      // Always show first page
      pages.push(1)

      if (currentPage > 3) {
        pages.push('...')
      }

      // Show pages around current
      const start = Math.max(2, currentPage - 1)
      const end = Math.min(total - 1, currentPage + 1)

      for (let i = start; i <= end; i++) {
        pages.push(i)
      }

      if (currentPage < total - 2) {
        pages.push('...')
      }

      // Always show last page
      pages.push(total)
    }

    return pages
  })

  return {
    // State
    page: readonly(page),
    limit: readonly(limit),
    sortBy: readonly(sortBy),
    sortOrder: readonly(sortOrder),
    filters: readonly(filters),
    meta: readonly(meta),
    isLoading,

    // Computed
    offset,
    hasNextPage,
    hasPreviousPage,
    totalPages,
    total,
    pageNumbers,

    // Navigation
    goToPage,
    nextPage,
    previousPage,
    firstPage,
    lastPage,

    // Sorting
    setSort,

    // Filtering
    setFilter,
    clearFilter,
    clearAllFilters,

    // API helpers
    updateMeta,
    buildQueryParams
  }
}
