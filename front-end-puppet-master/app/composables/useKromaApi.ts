/**
 * Kroma API Composable
 *
 * Direct API client for Kroma backend.
 * Uses Bearer token authentication from localStorage.
 *
 * Unlike the Puppet Master apiFetch, this composable:
 * - Calls the Kroma backend directly (not PM's Nitro server)
 * - Uses Bearer token auth from localStorage
 * - Handles Kroma's response format (no { success, data } envelope)
 *
 * Configuration:
 * - Base URL is read from runtime config: useRuntimeConfig().public.kromaApiBaseUrl
 * - Falls back to http://127.0.0.1:8788 if not configured
 * - Set via KROMA_API_BASE_URL environment variable
 */

// Kroma backend base URL from runtime config (with fallback)
function getKromaApiBase(): string {
  if (import.meta.client) {
    const config = useRuntimeConfig()
    return (config.public as any).kromaApiBaseUrl || 'http://127.0.0.1:8788'
  }
  // SSR fallback
  return process.env.KROMA_API_BASE_URL || 'http://127.0.0.1:8788'
}

const TOKEN_STORAGE_KEY = 'kroma_bearer_token'

/**
 * Get the current Kroma Bearer token from localStorage
 */
function getKromaToken(): string | null {
  if (import.meta.client) {
    return localStorage.getItem(TOKEN_STORAGE_KEY)
  }
  return null
}

/**
 * Set the Kroma Bearer token in localStorage
 */
function setKromaToken(token: string | null): void {
  if (import.meta.client) {
    if (token) {
      localStorage.setItem(TOKEN_STORAGE_KEY, token)
    } else {
      localStorage.removeItem(TOKEN_STORAGE_KEY)
    }
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
export async function bootstrapKromaToken(): Promise<string | null> {
  try {
    const baseUrl = getKromaApiBase()
    const response = await $fetch(`${baseUrl}/auth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      }
    })

    const envelope = response as any
    if (!envelope?.ok) {
      console.error('Bootstrap failed:', envelope?.error)
      return null
    }

    const token = envelope?.auth_token?.token
    if (token) {
      setKromaToken(token)
      return token
    }

    return null
  } catch (error) {
    console.error('Failed to bootstrap token:', error)
    return null
  }
}

/**
 * Ensure we have a valid Kroma token
 * Bootstraps if none exists (first-run scenario)
 */
async function ensureKromaToken(): Promise<string | null> {
  let token = getKromaToken()
  if (!token) {
    token = await bootstrapKromaToken()
  }
  return token
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
      const baseUrl = getKromaApiBase()
      const url = `${baseUrl}${endpoint}`
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
  const getProjects = async () => {
    const response = await kromaFetch<any>('/api/projects', { method: 'GET' }, { projects: [] })
    return (response as any)?.projects ?? []
  }
  const getProject = async (slug: string) => {
    const response = await kromaFetch<any>(`/api/projects/${slug}`, { method: 'GET' })
    return (response as any)?.project ?? response
  }
  const createProject = async (input: any) => {
    const response = await kromaFetch<any>('/api/projects', { method: 'POST', body: input })
    const envelope = response as any
    if (envelope?.ok) {
      return envelope.project ?? response
    }
    return response
  }
  const updateProject = async (slug: string, input: any) => {
    const response = await kromaFetch<any>(`/api/projects/${slug}`, { method: 'PUT', body: input })
    const envelope = response as any
    if (envelope?.ok) {
      return envelope.project ?? response
    }
    return response
  }

  // Providers API
  const getProviders = async (projectSlug: string) => {
    const response = await kromaFetch<any>(`/api/projects/${projectSlug}/provider-accounts`, { method: 'GET' }, { providers: [] })
    return (response as any)?.providers ?? []
  }
  const createProvider = async (projectSlug: string, input: any) => {
    const response = await kromaFetch<any>(`/api/projects/${projectSlug}/provider-accounts`, { method: 'POST', body: input })
    const envelope = response as any
    if (envelope?.ok) {
      return envelope.provider ?? response
    }
    return response
  }
  const updateProvider = async (projectSlug: string, providerCode: string, input: any) => {
    const response = await kromaFetch<any>(`/api/projects/${projectSlug}/provider-accounts/${providerCode}`, { method: 'PUT', body: input })
    const envelope = response as any
    if (envelope?.ok) {
      return envelope.provider ?? response
    }
    return response
  }
  const deleteProvider = async (projectSlug: string, providerCode: string) => {
    await kromaFetch<void>(`/api/projects/${projectSlug}/provider-accounts/${providerCode}`, { method: 'DELETE' })
  }

  // Runs API
  const getRuns = async (projectSlug: string) => {
    const response = await kromaFetch<any>(`/api/projects/${projectSlug}/runs`, { method: 'GET' }, { runs: [] })
    return (response as any)?.runs ?? []
  }
  const getRun = async (projectSlug: string, runId: string) => {
    const response = await kromaFetch<any>(`/api/projects/${projectSlug}/runs/${runId}`, { method: 'GET' })
    return (response as any)?.run ?? response
  }
  const triggerRun = async (projectSlug: string, config: any) => {
    const response = await kromaFetch<any>(`/api/projects/${projectSlug}/runs/trigger`, { method: 'POST', body: config })
    const envelope = response as any
    if (envelope?.ok) {
      return envelope.run ?? response
    }
    return response
  }
  
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
