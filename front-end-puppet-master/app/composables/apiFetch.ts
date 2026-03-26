export interface ApiSuccessEnvelope<T> {
  success: true
  data: T
}

type FetchInput = Parameters<typeof $fetch>[0]
type FetchOptions = Parameters<typeof $fetch>[1]

function isMutationMethod(method?: string): boolean {
  if (!method) return false
  const normalized = method.toUpperCase()
  return normalized === 'POST' || normalized === 'PUT' || normalized === 'PATCH' || normalized === 'DELETE'
}

function normalizeHeaders(headers: unknown): Record<string, string> {
  if (!headers || typeof headers !== 'object' || Array.isArray(headers)) {
    return {}
  }

  return { ...(headers as Record<string, string>) }
}

function withCsrfHeaders(method: string | undefined, options: FetchOptions | undefined): FetchOptions {
  if (import.meta.server || !isMutationMethod(method)) {
    return options || {}
  }

  const { getToken, getHeaderName } = useCsrf()
  const token = getToken()
  if (!token) {
    return options || {}
  }

  const headers = normalizeHeaders(options?.headers)
  headers[getHeaderName()] = token

  return {
    ...(options || {}),
    headers
  }
}

function getApiFetchTransport() {
  if (import.meta.server) {
    try {
      return useRequestFetch()
    } catch {
      return $fetch
    }
  }

  return $fetch
}

export function isApiSuccessEnvelope<T>(value: unknown): value is ApiSuccessEnvelope<T> {
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

export async function apiFetch<T>(input: FetchInput, options?: FetchOptions): Promise<T> {
  const method = typeof options?.method === 'string' ? options.method : undefined
  const transport = getApiFetchTransport()
  const response = await transport<ApiSuccessEnvelope<T> | T>(input, withCsrfHeaders(method, options))
  return unwrapApiEnvelope<T>(response)
}
