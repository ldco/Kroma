/**
 * Kroma Auth Pinia Store
 *
 * Manages Kroma Bearer token authentication.
 * Stores token in localStorage for persistence across page refreshes.
 * Supports bootstrapping the first token via POST /auth/token.
 */
import { defineStore } from 'pinia'

const TOKEN_STORAGE_KEY = 'kroma_bearer_token'
const KROMA_API_BASE = 'http://127.0.0.1:8788'

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
    isAuthenticated: (state) => !!state.token,

    /**
     * Check if currently bootstrapping
     */
    isBootstrapping: (state) => state.isBootstrapping,

    /**
     * Get current error message
     */
    error: (state) => state.error
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
     */
    async bootstrapToken(): Promise<string | null> {
      this.isBootstrapping = true
      this.error = null

      try {
        const response = await $fetch(`${KROMA_API_BASE}/auth/token`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          }
          // No auth header needed for bootstrap
        })

        // Response format: { token: string, ... }
        const token = (response as any)?.token
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
