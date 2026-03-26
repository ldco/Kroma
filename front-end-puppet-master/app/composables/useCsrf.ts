/**
 * CSRF Composable
 *
 * Client-side CSRF token management.
 * Token is received from server after login and stored in state.
 * Must be sent in X-CSRF-Token header on all state-changing requests.
 */

const CSRF_HEADER_NAME = 'x-csrf-token'

export function useCsrf() {
  // Store CSRF token in state (survives navigation, cleared on refresh)
  const csrfToken = useState<string | null>('csrf-token', () => null)

  /**
   * Set the CSRF token (called after login)
   */
  function setToken(token: string): void {
    csrfToken.value = token
  }

  /**
   * Clear the CSRF token (called on logout)
   */
  function clearToken(): void {
    csrfToken.value = null
  }

  /**
   * Get current CSRF token
   */
  function getToken(): string | null {
    return csrfToken.value
  }

  /**
   * Get headers object with CSRF token for fetch requests
   */
  function getHeaders(): Record<string, string> {
    if (!csrfToken.value) {
      return {}
    }
    return {
      [CSRF_HEADER_NAME]: csrfToken.value
    }
  }

  /**
   * Get the header name for CSRF token
   */
  function getHeaderName(): string {
    return CSRF_HEADER_NAME
  }

  return {
    csrfToken: readonly(csrfToken),
    setToken,
    clearToken,
    getToken,
    getHeaders,
    getHeaderName
  }
}
