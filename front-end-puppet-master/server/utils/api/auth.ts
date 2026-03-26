/**
 * Authentication Manager for OAuth 2.0 and JWT
 * Handles token acquisition, refresh, and validation
 */
import type { AuthConfig, TokenResponse } from './types'
import { logger } from '../logger'

export function useAuthManager(config: AuthConfig) {
  let currentToken: string | null = null
  let tokenExpiresAt: Date | null = null
  let refreshPromise: Promise<void> | null = null

  /**
   * Get valid access token (refresh if needed)
   */
  async function getToken(): Promise<string | null> {
    // Static token (JWT)
    if (config.type === 'jwt' && config.staticToken) {
      return config.staticToken
    }

    // API key
    if (config.type === 'apikey' && config.apiKey) {
      return config.apiKey
    }

    // OAuth 2.0 - check if token needs refresh
    if (config.type === 'oauth2') {
      const now = new Date()
      const refreshThreshold = new Date(now.getTime() + (config.refreshBuffer || 300) * 1000)

      // Refresh if no token or expiring soon
      if (!currentToken || !tokenExpiresAt || tokenExpiresAt <= refreshThreshold) {
        // Deduplicate concurrent refresh calls using shared promise
        if (!refreshPromise) {
          refreshPromise = refreshToken().finally(() => {
            refreshPromise = null
          })
        }
        await refreshPromise
      }

      return currentToken
    }

    return null
  }

  /**
   * Fetch new OAuth 2.0 token using client credentials flow
   */
  async function refreshToken(): Promise<void> {
    if (!config.tokenUrl || !config.clientId || !config.clientSecret) {
      throw new Error('[AuthManager] OAuth 2.0 credentials not configured')
    }

    try {
      const response = await fetch(config.tokenUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: new URLSearchParams({
          grant_type: 'client_credentials',
          client_id: config.clientId,
          client_secret: config.clientSecret
        })
      })

      if (!response.ok) {
        const errorText = await response.text()
        throw new Error(`Token refresh failed (${response.status}): ${errorText}`)
      }

      const data: TokenResponse = await response.json()

      currentToken = data.access_token
      tokenExpiresAt = new Date(Date.now() + data.expires_in * 1000)

      logger.debug({ expiresAt: tokenExpiresAt.toISOString() }, 'AuthManager: Token refreshed')
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error)
      logger.error({ error: message }, 'AuthManager: Token refresh failed')
      throw error
    }
  }

  /**
   * Invalidate current token (force refresh on next request)
   */
  function invalidate(): void {
    currentToken = null
    tokenExpiresAt = null
    refreshPromise = null
    logger.debug('AuthManager: Token invalidated')
  }

  /**
   * Check if current token is valid (not expired)
   */
  function isValid(): boolean {
    if (!currentToken || !tokenExpiresAt) {
      return false
    }
    return tokenExpiresAt > new Date()
  }

  return {
    getToken,
    refreshToken,
    invalidate,
    isValid
  }
}
