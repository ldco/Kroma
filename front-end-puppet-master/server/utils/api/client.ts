/**
 * API Client
 * Centralized HTTP client with authentication, retries, and caching
 */
import type { APIClientConfig, APIRequestOptions, APIError } from './types'
import { useAuthManager } from './auth'
import { useCache } from './cache'
import { useRetry } from './retry'
import { logger } from '../logger'
import config from '~/puppet-master.config'

export class APIClient {
  private baseUrl: string
  private timeout: number
  private authManager: ReturnType<typeof useAuthManager>
  private cache: ReturnType<typeof useCache>
  private retry: ReturnType<typeof useRetry>

  constructor(clientConfig: APIClientConfig) {
    this.baseUrl = clientConfig.baseUrl
    this.timeout = clientConfig.timeout || 30000
    this.authManager = useAuthManager(clientConfig.auth)
    this.cache = useCache(clientConfig.cache)
    this.retry = useRetry({
      ...clientConfig.retry,
      circuitBreaker: clientConfig.retry.circuitBreaker
    })
  }

  /**
   * Generic request method with auth, cache, and retry
   */
  async request<T>(
    method: 'GET' | 'POST' | 'PUT' | 'DELETE',
    path: string,
    options?: APIRequestOptions
  ): Promise<T> {
    const cacheKey =
      options?.cache?.key || `${method}:${path}:${JSON.stringify(options?.query || {})}`

    // Try cache first for GET requests
    if (method === 'GET' && options?.cache && config.dataSource.api.cache.enabled) {
      const cached = await this.cache.get<T>(cacheKey)
      if (cached) {
        return cached
      }
    }

    // Execute request with retry
    const result = await this.retry.execute<T>(async () => {
      const url = this.buildUrl(path, options?.query)
      const headers = await this.buildHeaders(options?.headers, options?.skipAuth)

      const response = await fetch(url, {
        method,
        headers,
        body: options?.body ? JSON.stringify(options.body) : undefined,
        signal: AbortSignal.timeout(this.timeout)
      })

      if (!response.ok) {
        throw await this.handleError(response)
      }

      const contentType = response.headers.get('content-type')
      if (contentType?.includes('application/json')) {
        return (await response.json()) as T
      }

      // Return text for non-JSON responses
      return (await response.text()) as T
    })

    // Cache successful GET responses
    if (method === 'GET' && options?.cache && result && config.dataSource.api.cache.enabled) {
      await this.cache.set(cacheKey, result, options.cache.ttl)
    }

    return result
  }

  /**
   * GET request
   */
  get<T>(path: string, options?: Omit<APIRequestOptions, 'body'>): Promise<T> {
    return this.request<T>('GET', path, options)
  }

  /**
   * POST request
   */
  post<T>(path: string, body?: unknown, options?: Omit<APIRequestOptions, 'body'>): Promise<T> {
    return this.request<T>('POST', path, { ...options, body })
  }

  /**
   * PUT request
   */
  put<T>(path: string, body?: unknown, options?: Omit<APIRequestOptions, 'body'>): Promise<T> {
    return this.request<T>('PUT', path, { ...options, body })
  }

  /**
   * DELETE request
   */
  delete<T>(path: string, options?: APIRequestOptions): Promise<T> {
    return this.request<T>('DELETE', path, options)
  }

  /**
   * GET request with automatic TTL based on resource type
   * Extracts resource name from path (e.g., '/portfolio/123' → 'portfolio')
   * and applies configured TTL from puppet-master.config.ts
   */
  getResource<T>(
    resource: keyof typeof config.dataSource.api.cache.ttl,
    path: string,
    options?: Omit<APIRequestOptions, 'body' | 'cache'>
  ): Promise<T> {
    const ttl = config.dataSource.api.cache.ttl[resource] ?? 300
    return this.get<T>(path, {
      ...options,
      cache: ttl > 0 ? { ttl } : undefined
    })
  }

  /**
   * Build full URL with query parameters
   */
  private buildUrl(path: string, query?: Record<string, string | number | boolean>): string {
    // Handle absolute vs relative paths
    const url = path.startsWith('http') ? new URL(path) : new URL(path, this.baseUrl)

    // Add query parameters
    if (query) {
      Object.entries(query).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          url.searchParams.append(key, String(value))
        }
      })
    }

    return url.toString()
  }

  /**
   * Build request headers with authentication
   */
  private async buildHeaders(
    customHeaders?: Record<string, string>,
    skipAuth?: boolean
  ): Promise<Record<string, string>> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      Accept: 'application/json',
      ...customHeaders
    }

    // Add authentication
    if (!skipAuth) {
      const token = await this.authManager.getToken()
      if (token) {
        headers['Authorization'] = `Bearer ${token}`
      }
    }

    return headers
  }

  /**
   * Handle HTTP errors
   */
  private async handleError(response: Response): Promise<APIError> {
    let message = response.statusText

    try {
      const contentType = response.headers.get('content-type')
      if (contentType?.includes('application/json')) {
        const error = await response.json()
        message = error.message || error.error || message
      } else {
        message = await response.text()
      }
    } catch {
      // Use statusText if parsing fails
    }

    const error = new Error(message) as APIError
    error.status = response.status
    error.message = message
    error.isRetryable = response.status >= 500 || response.status === 429

    return error
  }

  /**
   * Invalidate cache for a specific key or prefix
   */
  async invalidateCache(keyOrPrefix: string): Promise<void> {
    await this.cache.clearPrefix(keyOrPrefix)
  }

  /**
   * Get cache statistics
   */
  getCacheStats() {
    return this.cache.getStats()
  }

  /**
   * Get circuit breaker state
   */
  getCircuitState() {
    return this.retry.getState()
  }

  /**
   * Reset circuit breaker (for admin/testing)
   */
  resetCircuit() {
    this.retry.reset()
  }
}

// Singleton factory with initialization guard
let apiClient: APIClient | null = null
let isInitializing = false

export function useAPIClient(): APIClient {
  // Return existing instance if available
  if (apiClient) return apiClient

  // Prevent concurrent initialization (very rare in single-threaded Node.js)
  if (isInitializing) {
    throw new Error(
      '[APIClient] Concurrent initialization detected - this should not happen in production'
    )
  }

  isInitializing = true

  try {
    const runtimeConfig = useRuntimeConfig()

    apiClient = new APIClient({
      baseUrl: runtimeConfig.apiBaseUrl,
      timeout: config.dataSource.api.timeout,
      auth: {
        type: runtimeConfig.apiJwtToken ? 'jwt' : runtimeConfig.apiKey ? 'apikey' : 'oauth2',
        clientId: runtimeConfig.apiClientId,
        clientSecret: runtimeConfig.apiClientSecret,
        tokenUrl: runtimeConfig.apiTokenUrl,
        staticToken: runtimeConfig.apiJwtToken,
        apiKey: runtimeConfig.apiKey,
        refreshBuffer: parseInt(runtimeConfig.apiTokenRefreshBuffer)
      },
      retry: {
        ...config.dataSource.api.retry,
        circuitBreaker: config.dataSource.api.circuitBreaker
      },
      cache: config.dataSource.api.cache
    })

    logger.info({ baseUrl: runtimeConfig.apiBaseUrl }, 'APIClient initialized')
    return apiClient
  } finally {
    isInitializing = false
  }
}
