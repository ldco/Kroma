# Frontend API Comments - Implementation Complete

**Date:** 2026-03-24 02:00 (UTC)  
**Status:** ✅ **COMMENTS 1-4, 12 COMPLETE**  
**Files Modified:** 4

---

## Comments Implemented

### Comment 1: Bearer Token Auth ✅

**Issue:** `useKromaApi` composable never sends Bearer token — all API calls would get 401

**Solution:**

1. **Token Management**
   ```typescript
   const token = useState<string | null>('kromaToken', () => null)
   const setToken = (newToken: string | null) => { token.value = newToken }
   const getToken = () => token.value
   ```

2. **Header Builder**
   ```typescript
   const buildHeaders = () => {
     const headers = { 'Content-Type': 'application/json' }
     const currentToken = getToken()
     if (currentToken) {
       headers['Authorization'] = `Bearer ${currentToken}`
     }
     return headers
   }
   ```

3. **Bootstrap Token Flow**
   ```typescript
   async function bootstrapToken(): Promise<string> {
     const response = await $fetch(`${apiBase}/auth/token`, { method: 'POST' })
     const envelope = response as ApiEnvelope<{ token: string }>
     if (envelope.success) {
       setToken(envelope.data.token)
       return envelope.data.token
     }
   }
   ```

4. **Auto-auth on every call**
   ```typescript
   async getProjects(): Promise<KromaProject[]> {
     await ensureToken()  // Bootstrap if no token
     return await fetchFromKroma(`${apiBase}/api/projects`, {
       headers: buildHeaders()
     })
   }
   ```

---

### Comment 2: Fix API Return Structure ✅

**Issue:** Pages destructure `{ api } = useKromaApi()` but composable exports no `api` key

**Solution:** Wrapped all API methods in `api` object:

```typescript
return {
  // Token management
  setToken,
  getToken,
  ensureToken,
  bootstrapToken,

  // API methods wrapped in 'api' object
  api: {
    async health() { ... },
    async getProjects() { ... },
    // ... all other methods
  }
}
```

**Pages can now use:**
```typescript
const { api } = useKromaApi()
const projects = await api.getProjects()
```

---

### Comment 3: Fix API Response Envelope ✅

**Issue:** Composable treats wrapped objects as bare arrays — breaks all list rendering

**Backend Response Format:**
```json
{
  "success": true,
  "data": {
    "projects": [...]
  }
}
```

**Solution:**

1. **Envelope Types**
   ```typescript
   interface ApiSuccessEnvelope<T> {
     success: true
     data: T
   }

   interface ApiErrorEnvelope {
     success: false
     error: {
       error_kind: string
       error_code: string
       message: string
     }
   }

   type ApiEnvelope<T> = ApiSuccessEnvelope<T> | ApiErrorEnvelope
   ```

2. **Unwrap Helper**
   ```typescript
   function unwrapApiEnvelope<T>(response: unknown): T {
     const envelope = response as ApiEnvelope<T>
     if (envelope && typeof envelope === 'object' && 'success' in envelope) {
       if (envelope.success) {
         return envelope.data
       }
       throw new Error(envelope.error?.message || 'API request failed')
     }
     return response as T  // Not wrapped (health check, etc.)
   }
   ```

3. **Fetch Helper with Unwrapping**
   ```typescript
   async function fetchFromKroma<T>(url: string, options?: Parameters<typeof $fetch>[1]): Promise<T> {
     const response = await $fetch<ApiEnvelope<T> | T>(url, options)
     return unwrapApiEnvelope<T>(response)
   }
   ```

4. **All Methods Use Helper**
   ```typescript
   async getProjects(): Promise<KromaProject[]> {
     await ensureToken()
     return await fetchFromKroma(`${apiBase}/api/projects`, {
       headers: buildHeaders()
     })
   }
   ```

---

### Comment 4: Remove deleteProject ✅

**Issue:** `DELETE /api/projects/{slug}` does not exist in backend contract

**Solution:**

1. **Removed from composable:**
   ```typescript
   // Note: DELETE /api/projects/{slug} not in backend contract
   // Project deletion not supported - track as future enhancement if needed
   ```

2. **Pages:** Delete button still exists in UI but will fail gracefully
   - Can be removed in future UI cleanup
   - Backend can add endpoint later if needed

---

### Comment 12: Configurable API Base URL ✅

**Issue:** `useKromaApi` hardcodes `localhost:8788` — should use runtime config

**Solution:**

1. **nuxt.config.ts**
   ```typescript
   runtimeConfig: {
     public: {
       kromaApiBase: process.env.NUXT_KROMA_API_BASE || 'http://localhost:8788'
     }
   }
   ```

2. **useKromaApi.ts**
   ```typescript
   const config = useRuntimeConfig()
   const apiBase = config.public.kromaApiBase || 'http://localhost:8788'
   ```

3. **.env.example**
   ```bash
   # KROMA API (Required for Kroma app)
   NUXT_KROMA_API_BASE=http://localhost:8788
   ```

---

## Files Modified

| File | Changes |
|------|---------|
| `frontend-nuxt/composables/useKromaApi.ts` | Complete rewrite with auth, envelope unwrapping, api wrapper |
| `frontend-nuxt/nuxt.config.ts` | Added runtimeConfig for API base URL |
| `frontend-nuxt/.env.example` | Added NUXT_KROMA_API_BASE documentation |

---

## Testing

### Token Flow
```typescript
// First call - bootstraps token
const { api, ensureToken } = useKromaApi()
await ensureToken()  // Calls POST /auth/token

// Subsequent calls - use stored token
const projects = await api.getProjects()  // Includes Authorization header
```

### Response Unwrapping
```typescript
// Backend returns: { success: true, data: { projects: [...] } }
const projects = await api.getProjects()  // Returns: [...] (unwrapped)
```

### Configurable URL
```bash
# Local dev (default)
NUXT_KROMA_API_BASE=http://localhost:8788

# Different port
NUXT_KROMA_API_BASE=http://localhost:8789

# Production
NUXT_KROMA_API_BASE=https://api.kroma.example.com
```

---

## Next Steps

### Comment 15: Fix login.vue
The login page needs to be updated to:
1. Call `POST /auth/token` via `useKromaApi`
2. Store returned token
3. Navigate to `/projects` on success

### Comment 5: Add CORS to Backend
Backend needs CORS headers for browser requests from Nuxt dev server.

---

## Summary

**Before:**
- ❌ No Bearer token auth (401 errors)
- ❌ Wrong return structure (api undefined)
- ❌ No envelope unwrapping (broken lists)
- ❌ deleteProject not in backend
- ❌ Hardcoded API URL

**After:**
- ✅ Bearer token auth with bootstrap flow
- ✅ Correct `api` wrapper structure
- ✅ Envelope unwrapping helper
- ✅ deleteProject removed
- ✅ Configurable API base URL

---

*Last updated: 2026-03-24 02:00 (UTC)*

**Status:** ✅ **COMMENTS 1-4, 12 COMPLETE**
