/**
 * Providers Composable
 *
 * Provider account management for Kroma app.
 * Provides provider CRUD operations and state management.
 * All API calls are scoped to a project slug as per backend contract.
 */
import { useProvidersStore } from '~/stores/providers'
import type { ProviderAccount, CreateProviderInput } from '~/types/kroma'

export interface UseProvidersReturn {
  // State
  providers: ComputedRef<ProviderAccount[]>
  isLoading: ComputedRef<boolean>
  error: ComputedRef<string | null>

  // Getters
  getProviderByCode: (providerCode: string) => ProviderAccount | undefined
  getProvidersByType: (type: string) => ProviderAccount[]
  healthyProviders: ComputedRef<ProviderAccount[]>
  providerCount: ComputedRef<number>
  hasProviders: ComputedRef<boolean>

  // Actions - all require projectSlug parameter
  fetchProviders: (projectSlug: string) => Promise<ProviderAccount[]>
  createProvider: (projectSlug: string, input: CreateProviderInput) => Promise<ProviderAccount>
  updateProvider: (projectSlug: string, providerCode: string, input: Partial<CreateProviderInput>) => Promise<ProviderAccount>
  deleteProvider: (projectSlug: string, providerCode: string) => Promise<void>
  clearError: () => void
}

export function useProviders(): UseProvidersReturn {
  const store = useProvidersStore()
  const toast = useToast()

  // State
  const providers = computed(() => store.providers)
  const isLoading = computed(() => store.isLoading)
  const error = computed(() => store.error)

  // Getters
  const getProviderByCode = (providerCode: string) => store.getProviderByCode(providerCode)
  const getProvidersByType = (type: string) => store.getProvidersByType(type)
  const healthyProviders = computed(() => store.healthyProviders)
  const providerCount = computed(() => store.providerCount)
  const hasProviders = computed(() => store.hasProviders)

  // Actions with error handling - all require projectSlug
  async function fetchProviders(projectSlug: string): Promise<ProviderAccount[]> {
    try {
      return await store.fetchProviders(projectSlug)
    } catch (error) {
      toast.error('Failed to load providers')
      throw error
    }
  }

  async function createProvider(
    projectSlug: string,
    input: CreateProviderInput
  ): Promise<ProviderAccount> {
    try {
      const result = await store.createProvider(projectSlug, input)
      toast.success('Provider added successfully')
      return result
    } catch (error) {
      toast.error('Failed to add provider')
      throw error
    }
  }

  async function updateProvider(
    projectSlug: string,
    providerCode: string,
    input: Partial<CreateProviderInput>
  ): Promise<ProviderAccount> {
    try {
      const result = await store.updateProvider(projectSlug, providerCode, input)
      toast.success('Provider updated successfully')
      return result
    } catch (error) {
      toast.error('Failed to update provider')
      throw error
    }
  }

  async function deleteProvider(projectSlug: string, providerCode: string): Promise<void> {
    try {
      await store.deleteProvider(projectSlug, providerCode)
      toast.success('Provider deleted successfully')
    } catch (error) {
      toast.error('Failed to delete provider')
      throw error
    }
  }

  function clearError() {
    store.clearError()
  }

  return {
    // State
    providers,
    isLoading,
    error,

    // Getters
    getProviderByCode,
    getProvidersByType,
    healthyProviders,
    providerCount,
    hasProviders,

    // Actions
    fetchProviders,
    createProvider,
    updateProvider,
    deleteProvider,
    clearError
  }
}
