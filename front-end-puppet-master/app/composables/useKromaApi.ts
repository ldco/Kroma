/**
 * Kroma API Composable
 *
 * Direct API client for Kroma backend (http://127.0.0.1:8788)
 * Uses Bearer token authentication from kromaAuth store and handles Kroma-specific response format.
 *
 * Unlike the Puppet Master apiFetch, this composable:
 * - Calls the Kroma backend directly (not PM's Nitro server)
 * - Uses Bearer token auth from kromaAuth store (persisted in localStorage)
 * - Handles Kroma's response format (no { success, data } envelope)
 */

// Kroma backend base URL
const KROMA_API_BASE = 'http://127.0.0.1:8788'

/**
 * Get the current Kroma Bearer token from kromaAuth store
 */
function getKromaToken(): string | null {
  if (import.meta.client) {
    const authStore = useKromaAuthStore()
    return authStore.token
  }
  return null
}

/**
 * Set the Kroma Bearer token via kromaAuth store (persists to localStorage)
 */
function setKromaToken(token: string | null): void {
  if (import.meta.client) {
    const authStore = useKromaAuthStore()
    authStore.setToken(token)
  }
}

/**
 * Build fetch options with Kroma Bearer auth
 */
function buildKromaFetchOptions(options?: RequestInit & { body?: any }): RequestInit & { body?: any } {
  const token = getKromaToken()
  
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options?.headers as Record<string, string>),
  }
  
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }
  
  return {
    ...options,
    headers,
  }
}

/**
 * Handle API errors with user-friendly messages
 */
function handleKromaApiError(error: any, fallbackValue?: any): any {
  console.error('Kroma API Error:', error)
  
  // Return fallback value if provided (for list endpoints)
  if (fallbackValue !== undefined) {
    return fallbackValue
  }
  
  // Re-throw for mutations and other critical calls
  throw error
}

/**
 * Bootstrap the first auth token (J00 onboarding)
 * Calls POST /auth/token to create the first admin token
 */
async function bootstrapKromaToken(): Promise<string | null> {
  const authStore = useKromaAuthStore()
  return await authStore.bootstrapToken()
}

/**
 * Ensure we have a valid Kroma token
 * Bootstraps if none exists (first-run scenario)
 */
async function ensureKromaToken(): Promise<string | null> {
  const authStore = useKromaAuthStore()
  return await authStore.ensureToken()
}

/**
 * Kroma API client interface
 */
export interface UseKromaApiReturn {
  // Token management
  getToken: () => string | null
  setToken: (token: string | null) => void
  clearToken: () => void
  ensureToken: () => Promise<string | null>
  bootstrapToken: () => Promise<string | null>
  
  // Projects API
  getProjects: () => Promise<any[]>
  getProject: (slug: string) => Promise<any>
  createProject: (input: any) => Promise<any>
  updateProject: (slug: string, input: any) => Promise<any>
  
  // Providers API
  getProviders: (projectSlug: string) => Promise<any[]>
  createProvider: (projectSlug: string, input: any) => Promise<any>
  updateProvider: (projectSlug: string, providerCode: string, input: any) => Promise<any>
  deleteProvider: (projectSlug: string, providerCode: string) => Promise<void>
  
  // Runs API
  getRuns: (projectSlug: string) => Promise<any[]>
  getRun: (projectSlug: string, runId: string) => Promise<any>
  triggerRun: (projectSlug: string, config: any) => Promise<any>
}

/**
 * Create Kroma API composable
 */
export function useKromaApi(): UseKromaApiReturn {
  /**
   * Execute a Kroma API call with error handling
   */
  async function kromaFetch<T>(
    endpoint: string,
    options?: RequestInit & { body?: any },
    fallbackValue?: T
  ): Promise<T> {
    try {
      const url = `${KROMA_API_BASE}${endpoint}`
      const fetchOptions = buildKromaFetchOptions(options)
      
      const response = await $fetch<T>(url, fetchOptions)
      return response
    } catch (error) {
      return handleKromaApiError(error, fallbackValue) as T
    }
  }
  
  // Token management
  const getToken = () => getKromaToken()
  const setToken = (token: string | null) => setKromaToken(token)
  const clearToken = () => setKromaToken(null)
  const ensureToken = () => ensureKromaToken()
  const bootstrapToken = () => bootstrapKromaToken()
  
  // Projects API
  const getProjects = () => kromaFetch<any[]>('/api/projects', { method: 'GET' }, [])
  const getProject = (slug: string) => kromaFetch<any>(`/api/projects/${slug}`, { method: 'GET' })
  const createProject = (input: any) => kromaFetch<any>('/api/projects', { method: 'POST', body: input })
  const updateProject = (slug: string, input: any) => kromaFetch<any>(`/api/projects/${slug}`, { method: 'PUT', body: input })
  
  // Providers API
  const getProviders = (projectSlug: string) => 
    kromaFetch<any[]>(`/api/projects/${projectSlug}/provider-accounts`, { method: 'GET' }, [])
  const createProvider = (projectSlug: string, input: any) =>
    kromaFetch<any>(`/api/projects/${projectSlug}/provider-accounts`, { method: 'POST', body: input })
  const updateProvider = (projectSlug: string, providerCode: string, input: any) =>
    kromaFetch<any>(`/api/projects/${projectSlug}/provider-accounts/${providerCode}`, { method: 'PUT', body: input })
  const deleteProvider = (projectSlug: string, providerCode: string) =>
    kromaFetch<void>(`/api/projects/${projectSlug}/provider-accounts/${providerCode}`, { method: 'DELETE' })
  
  // Runs API
  const getRuns = (projectSlug: string) =>
    kromaFetch<any[]>(`/api/projects/${projectSlug}/runs`, { method: 'GET' }, [])
  const getRun = (projectSlug: string, runId: string) =>
    kromaFetch<any>(`/api/projects/${projectSlug}/runs/${runId}`, { method: 'GET' })
  const triggerRun = (projectSlug: string, config: any) =>
    kromaFetch<any>(`/api/projects/${projectSlug}/runs/trigger`, { method: 'POST', body: config })
  
  return {
    // Token management
    getToken,
    setToken,
    clearToken,
    ensureToken,
    bootstrapToken,
    
    // Projects API
    getProjects,
    getProject,
    createProject,
    updateProject,
    
    // Providers API
    getProviders,
    createProvider,
    updateProvider,
    deleteProvider,
    
    // Runs API
    getRuns,
    getRun,
    triggerRun,
  }
}
