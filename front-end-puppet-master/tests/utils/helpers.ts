/**
 * Test Helpers
 *
 * Common utility functions for tests.
 * Includes authentication helpers, assertion utilities, and test setup/teardown helpers.
 */

import { $fetch } from '@nuxt/test-utils/e2e'

export interface ApiSuccessEnvelope<T> {
  success: true
  data: T
}

function isApiSuccessEnvelope<T>(value: unknown): value is ApiSuccessEnvelope<T> {
  if (!value || typeof value !== 'object') {
    return false
  }

  const candidate = value as { success?: unknown; data?: unknown }
  return candidate.success === true && 'data' in candidate
}

export function unwrapApiEnvelope<T>(value: ApiSuccessEnvelope<T> | T): T {
  if (isApiSuccessEnvelope<T>(value)) {
    return value.data
  }

  return value as T
}

/**
 * Error type for fetch errors
 */
export interface FetchError {
  statusCode: number
  data?: {
    message?: string
    [key: string]: unknown
  }
  message?: string
}

/**
 * Login response type
 */
export interface LoginResponse {
  user?: {
    id: number
    email: string
    name: string | null
    role: string
  }
  csrfToken?: string
  requires2fa?: boolean
  message?: string
}

function buildCookieHeader(setCookie: string[] | undefined): string {
  if (!setCookie?.length) {
    return ''
  }

  return setCookie
    .map(cookie => cookie.split(';', 1)[0]?.trim())
    .filter((cookie): cookie is string => Boolean(cookie))
    .join('; ')
}

function extractSetCookies(headers: Headers): string[] {
  const headerWithSetCookie = headers as Headers & { getSetCookie?: () => string[] }

  if (typeof headerWithSetCookie.getSetCookie === 'function') {
    return headerWithSetCookie.getSetCookie()
  }

  const combined = headers.get('set-cookie')
  if (!combined) {
    return []
  }

  // Split cookies while preserving commas inside Expires attributes.
  return combined.split(/,(?=\s*[^;=]+=[^;]+)/)
}

/**
 * Login and get session cookie + CSRF token
 * Returns headers to use in subsequent requests
 */
export async function loginAsUser(
  email = 'master@example.com',
  password = 'master123'
): Promise<{ headers: Record<string, string>; csrfToken: string; user: LoginResponse['user'] }> {
  let cookieHeader = ''

  const payload = unwrapApiEnvelope(
    await $fetch<ApiSuccessEnvelope<LoginResponse> | LoginResponse>('/api/auth/login', {
      method: 'POST',
      body: { email, password },
      onResponse({ response }) {
        cookieHeader = buildCookieHeader(extractSetCookies(response.headers))
      }
    })
  )

  if (!payload.csrfToken || !payload.user) {
    throw new Error(`Failed to login as ${email}`)
  }

  return {
    headers: {
      'x-csrf-token': payload.csrfToken,
      ...(cookieHeader ? { cookie: cookieHeader } : {})
    },
    csrfToken: payload.csrfToken,
    user: payload.user
  }
}

/**
 * Login as master user (full access)
 */
export async function loginAsMaster() {
  return loginAsUser('master@example.com', 'master123')
}

/**
 * Login as admin user
 */
export async function loginAsAdmin() {
  return loginAsUser('admin@example.com', 'admin123')
}

/**
 * Login as editor user
 */
export async function loginAsEditor() {
  return loginAsUser('editor@example.com', 'editor123')
}

/**
 * Make an authenticated request
 */
export async function authenticatedFetch<T>(
  url: string,
  options: {
    method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'
    body?: unknown
    headers?: Record<string, string>
    auth?: { headers: Record<string, string> }
  } = {}
): Promise<T> {
  const { method = 'GET', body, headers = {}, auth } = options

  const authHeaders = auth?.headers || (await loginAsMaster()).headers

  const response = await $fetch<ApiSuccessEnvelope<T> | T>(url, {
    method,
    body,
    headers: {
      ...authHeaders,
      ...headers
    }
  })

  return unwrapApiEnvelope<T>(response)
}

/**
 * Assert that a fetch call throws an error with expected status code
 */
export async function expectFetchError(
  fetchPromise: Promise<unknown>,
  expectedStatus: number,
  expectedMessage?: string
): Promise<FetchError> {
  try {
    await fetchPromise
    throw new Error(`Expected fetch to throw with status ${expectedStatus}`)
  } catch (error: unknown) {
    const err = error as FetchError
    if (err.statusCode !== expectedStatus) {
      throw new Error(`Expected status ${expectedStatus}, got ${err.statusCode}`)
    }
    if (expectedMessage && err.data?.message !== expectedMessage) {
      throw new Error(`Expected message "${expectedMessage}", got "${err.data?.message}"`)
    }
    return err
  }
}

/**
 * Assert that an error contains expected message
 */
export function assertErrorMessage(error: FetchError, expectedMessage: string): void {
  const actualMessage = error.data?.message || error.message || ''
  if (!actualMessage.includes(expectedMessage)) {
    throw new Error(`Expected error to contain "${expectedMessage}", got "${actualMessage}"`)
  }
}

/**
 * Wait for a condition to be true
 * Useful for async operations that need polling
 */
export async function waitFor(
  condition: () => boolean | Promise<boolean>,
  options: { timeout?: number; interval?: number } = {}
): Promise<void> {
  const { timeout = 5000, interval = 100 } = options
  const startTime = Date.now()

  while (Date.now() - startTime < timeout) {
    if (await condition()) {
      return
    }
    await sleep(interval)
  }

  throw new Error(`Condition not met within ${timeout}ms`)
}

/**
 * Sleep for a specified duration
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * Generate a random string
 */
export function randomString(length = 10): string {
  const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  let result = ''
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  return result
}

/**
 * Generate a random email
 */
export function randomEmail(): string {
  return `test-${randomString(8)}@example.com`
}

/**
 * Generate a random slug
 */
export function randomSlug(): string {
  return `test-${randomString(8)}`.toLowerCase()
}

/**
 * Check if an object has all expected keys
 */
export function hasKeys(obj: Record<string, unknown>, keys: string[]): boolean {
  return keys.every(key => key in obj)
}

/**
 * Deep clone an object
 */
export function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj))
}

/**
 * Create a test timeout error
 */
export function createTimeoutError(operation: string, timeout: number): Error {
  return new Error(`${operation} timed out after ${timeout}ms`)
}
