/**
 * Providers Pinia Store
 *
 * Manages provider account state for Kroma app.
 * All API calls are scoped to a project slug as per backend contract.
 */
import { defineStore } from 'pinia'
import type { ProviderAccount, CreateProviderInput } from '~/types/kroma'

export interface ProvidersState {
  providers: ProviderAccount[]
  isLoading: boolean
  error: string | null
}

export const useProvidersStore = defineStore('providers', {
  state: (): ProvidersState => ({
    providers: [],
    isLoading: false,
    error: null
  }),

  getters: {
    /**
     * Get provider by providerCode
     */
    getProviderByCode: (state) => (providerCode: string) => {
      return state.providers.find(p => p.provider_code === providerCode)
    },

    /**
     * Get providers by type
     */
    getProvidersByType: (state) => (providerType: string) => {
      return state.providers.filter(p => p.provider_type === providerType)
    },

    /**
     * Get healthy providers
     */
    healthyProviders: (state) => {
      return state.providers.filter(p => p.health_status.status === 'healthy')
    },

    /**
     * Get total provider count
     */
    providerCount: (state) => state.providers.length,

    /**
     * Check if store has any providers
     */
    hasProviders: (state) => state.providers.length > 0
  },

  actions: {
    /**
     * Fetch all providers for a project
     */
    async fetchProviders(projectSlug: string) {
      this.isLoading = true
      this.error = null

      try {
        const response = await apiFetch<{ providers: ProviderAccount[] }>(
          `/api/projects/${projectSlug}/provider-accounts`
        )
        this.providers = response.providers
        return response.providers
      } catch (error) {
        this.error = 'Failed to load providers'
        console.error('Error fetching providers:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Create new provider for a project
     */
    async createProvider(projectSlug: string, input: CreateProviderInput) {
      this.isLoading = true
      this.error = null

      try {
        const response = await apiFetch<ProviderAccount>(
          `/api/projects/${projectSlug}/provider-accounts`,
          {
            method: 'POST',
            body: input
          }
        )

        // Add to local state
        this.providers.push(response)

        return response
      } catch (error) {
        this.error = 'Failed to create provider'
        console.error('Error creating provider:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Update provider (name only - API key changes require re-creation)
     * Backend uses providerCode as identifier, not numeric ID
     */
    async updateProvider(projectSlug: string, providerCode: string, input: Partial<CreateProviderInput>) {
      this.isLoading = true
      this.error = null

      try {
        const response = await apiFetch<ProviderAccount>(
          `/api/projects/${projectSlug}/provider-accounts/${providerCode}`,
          {
            method: 'PUT',
            body: input
          }
        )

        // Update in local state
        const index = this.providers.findIndex(p => p.provider_code === providerCode)
        if (index !== -1) {
          this.providers[index] = { ...this.providers[index], ...response }
        }

        return response
      } catch (error) {
        this.error = 'Failed to update provider'
        console.error('Error updating provider:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Delete provider
     * Backend uses providerCode as identifier
     */
    async deleteProvider(projectSlug: string, providerCode: string) {
      this.isLoading = true
      this.error = null

      try {
        await apiFetch(
          `/api/projects/${projectSlug}/provider-accounts/${providerCode}`,
          {
            method: 'DELETE'
          }
        )

        // Remove from local state
        this.providers = this.providers.filter(p => p.provider_code !== providerCode)
      } catch (error) {
        this.error = 'Failed to delete provider'
        console.error('Error deleting provider:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Clear error
     */
    clearError() {
      this.error = null
    }
  }
})
