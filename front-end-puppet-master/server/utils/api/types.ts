/**
 * API Client Types
 * Type definitions for the external API integration
 */

/**
 * Authentication configuration
 */
export interface AuthConfig {
  type: 'oauth2' | 'jwt' | 'apikey'
  clientId?: string
  clientSecret?: string
  tokenUrl?: string
  staticToken?: string // For JWT
  apiKey?: string
  refreshBuffer?: number // Seconds before expiry to refresh
}

/**
 * OAuth 2.0 token response
 */
export interface TokenResponse {
  access_token: string
  expires_in: number
  token_type: string
  refresh_token?: string
}

/**
 * Retry configuration
 */
export interface RetryConfig {
  maxAttempts: number
  initialDelay: number
  maxDelay: number
  backoffMultiplier: number
}

/**
 * Circuit breaker configuration
 */
export interface CircuitBreakerConfig {
  enabled: boolean
  failureThreshold: number
  resetTimeout: number
}

/**
 * Cache configuration
 */
export interface CacheConfig {
  enabled: boolean
  ttl: Record<string, number> // Per-resource TTL in seconds
}

/**
 * Complete API client configuration
 */
export interface APIClientConfig {
  baseUrl: string
  timeout: number
  auth: AuthConfig
  retry: RetryConfig & { circuitBreaker?: CircuitBreakerConfig }
  cache: CacheConfig
}

/**
 * API request options
 */
export interface APIRequestOptions {
  body?: unknown
  query?: Record<string, string | number | boolean>
  headers?: Record<string, string>
  cache?: {
    key?: string
    ttl?: number
  }
  skipAuth?: boolean
}

/**
 * API error
 */
export interface APIError extends Error {
  status: number
  message: string
  isRetryable: boolean
}

/**
 * Generic API response wrapper
 */
export interface APIResponse<T> {
  data?: T
  error?: APIError
  status: number
}

/**
 * Cache entry
 */
export interface CacheEntry<T> {
  data: T
  expiresAt: number
}
