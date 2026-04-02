/**
 * Kroma Auth Pinia Store
 *
 * Manages Kroma Bearer token authentication.
 * Stores token in localStorage for persistence across page refreshes.
 * Supports bootstrapping the first token via POST /auth/token.
 *
 * Configuration:
 * - Base URL is read from runtime config: useRuntimeConfig().public.kromaApiBaseUrl
 * - Falls back to http://127.0.0.1:8788 if not configured
 * - Set via KROMA_API_BASE_URL environment variable
 */
import { defineStore } from 'pinia'

const TOKEN_STORAGE_KEY = 'kroma_bearer_token'

/**
 * Get Kroma API base URL from runtime config
 */
function getKromaApiBase(): string {
  if (import.meta.client) {
    const config = useRuntimeConfig()
    return (config.public as any).kromaApiBaseUrl || 'http://127.0.0.1:8788'
  }
  return 'http://127.0.0.1:8788'
}

export interface KromaAuthState {
  token: string | null
  isBootstrapping: boolean
  error: string | null
}

export const useKromaAuthStore = defineStore('kromaAuth', {
  state: (): KromaAuthState => ({
    token: null,
    isBootstrapping: false,
    error: null
  }),

  getters: {
    /**
     * Check if authenticated (has token)
     */
    isAuthenticated: (state) => !!state.token
    // Note: isBootstrapping and error are accessed directly from state
  },

  actions: {
    /**
     * Initialize token from localStorage on client mount
     */
    initFromStorage() {
      if (import.meta.client) {
        const storedToken = localStorage.getItem(TOKEN_STORAGE_KEY)
        if (storedToken) {
          this.token = storedToken
        }
      }
    },

    /**
     * Set token and persist to localStorage
     */
    setToken(token: string | null) {
      this.token = token
      
      if (import.meta.client) {
        if (token) {
          localStorage.setItem(TOKEN_STORAGE_KEY, token)
        } else {
          localStorage.removeItem(TOKEN_STORAGE_KEY)
        }
      }
    },

    /**
     * Clear token (logout)
     */
    clearToken() {
      this.setToken(null)
      this.error = null
    },

    /**
     * Bootstrap the first auth token (J00 onboarding)
     * Calls POST /auth/token to create the first admin token
     * Only works when:
     * - No tokens exist yet
     * - Backend is bound to loopback (127.0.0.1)
     *
     * Response format: { ok: true, auth_token: { token: string, ... } }
     */
    async bootstrapToken(): Promise<string | null> {
      this.isBootstrapping = true
      this.error = null

      try {
        const baseUrl = getKromaApiBase()
        const response = await $fetch(`${baseUrl}/auth/token`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          }
          // No auth header needed for bootstrap
        })

        // Response format: { ok: true, auth_token: { token: string, ... } }
        const envelope = response as any
        
        // Check if response indicates success
        if (!envelope?.ok) {
          this.error = `Bootstrap failed: ${envelope?.error || 'unknown error'}`
          return null
        }
        
        // Extract token from auth_token field
        const token = envelope?.auth_token?.token
        
        if (token) {
          this.setToken(token)
          return token
        }

        this.error = 'Bootstrap response did not contain token'
        return null
      } catch (error: any) {
        this.error = error?.message || 'Failed to bootstrap token'
        console.error('Failed to bootstrap Kroma token:', error)
        return null
      } finally {
        this.isBootstrapping = false
      }
    },

    /**
     * Ensure we have a valid token
     * Bootstraps if none exists (first-run scenario)
     */
    async ensureToken(): Promise<string | null> {
      // If we already have a token, return it
      if (this.token) {
        return this.token
      }

      // Try to load from storage
      this.initFromStorage()
      if (this.token) {
        return this.token
      }

      // No token - bootstrap one
      return await this.bootstrapToken()
    },

    /**
     * Check if token exists (without bootstrapping)
     */
    hasToken(): boolean {
      return !!this.token
    }
  }
})
