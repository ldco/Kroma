# Additional Comments Implementation

**Date:** 2026-03-24 05:00 (UTC)  
**Status:** ✅ **COMMENTS 1-3 COMPLETE**  
**Remaining:** Comments 4-8 (CORS, CI, docs)

---

## Implemented Comments

### ✅ Comment 1: Fix Auth Token Response Format

**Files:** `frontend-nuxt/composables/useKromaApi.ts`, `frontend-nuxt/pages/login.vue`

**Changes:**
- Changed envelope format from `{ success, data }` to `{ ok, auth_token }`
- Updated `bootstrapToken()` to read `envelope.ok` and `envelope.auth_token.token`
- Updated `login.vue` to read `envelope.ok && envelope.auth_token?.token`

**Before:**
```typescript
if (envelope.success && envelope.data.token) {
  setToken(envelope.data.token)
}
```

**After:**
```typescript
if (envelope.ok && envelope.auth_token?.token) {
  setToken(envelope.auth_token.token)
}
```

---

### ✅ Comment 2: Fix List Endpoint Envelope Unwrapping

**File:** `frontend-nuxt/composables/useKromaApi.ts`

**Changes:**
- Added `extractKey` parameter to `fetchFromKroma<T>()`
- Updated `unwrapKromaEnvelope()` to extract specific fields
- All list methods now pass correct key:
  - `getProjects()` → `'projects'`
  - `getRuns()` → `'runs'`
  - `getStyleGuides()` → `'style_guides'`
  - `getPromptTemplates()` → `'prompt_templates'`
  - `getReferenceSets()` → `'reference_sets'`
  - `getAssets()` → `'assets'`
  - `getQualityReports()` → `'quality_reports'`
  - `getCostEvents()` → `'cost_events'`
  - `getInstructions()` → `'instructions'`

**Backend Format:**
```json
{
  "ok": true,
  "projects": [...]
}
```

**Frontend Usage:**
```typescript
async getProjects(): Promise<KromaProject[]> {
  return await fetchFromKroma(`${apiBase}/api/projects`, 'projects', {
    headers: buildHeaders()
  })
}
```

---

### ✅ Comment 3: Remove deleteProject Calls

**Files:** `frontend-nuxt/pages/index.vue`, `frontend-nuxt/pages/projects.vue`

**Changes:**
- Removed `api.deleteProject()` calls
- Removed delete buttons from templates
- Added placeholder functions with "Not Implemented" messages
- Added comments noting backend doesn't support DELETE /api/projects/{slug}

**Before:**
```typescript
async function deleteProject(slug: string, name: string) {
  await api.deleteProject(slug)
  toast.add({ title: 'Deleted', ... })
}
```

**After:**
```typescript
async function deleteProject(slug: string, name: string) {
  // Note: Project deletion not yet supported by backend
  toast.add({ 
    title: 'Not Implemented', 
    description: 'Project deletion is not yet supported', 
    color: 'yellow' 
  })
}
```

---

## Remaining Comments (4-8)

### Comment 4: Fix CORS Layer
**File:** `src-tauri/src/api/server.rs`  
**Issue:** Fold pattern only keeps last origin  
**Fix Needed:** Use `AllowOrigin::list(origins)` with collected Vec

### Comment 5: Fix CI Workflow Manifest Path
**File:** `.github/workflows/backend-rust.yml`  
**Issue:** `--manifest-path` conflicts with `working-directory`  
**Fix:** Remove `--manifest-path` flags (cargo finds Cargo.toml in working dir)

### Comment 6: Add #[ignore] to Failing Tests
**File:** `src-tauri/src/pipeline/runtime.rs`  
**Issue:** CI comment claims 7 tests are #[ignore] but none exist  
**Fix:** Add `#[ignore = "requires image processing infrastructure"]` to listed tests

### Comment 7: Update Stale Documentation
**Files:** `docs/MIGRATION_STATUS.md`, `docs/TECH_SPEC.md`  
**Fix:**
- Update MIGRATION_STATUS.md: Mark migration COMPLETE, remove scripts table
- Update TECH_SPEC.md: Change "PostgreSQL in production" to "SQLite (desktop-first)"

### Comment 8: Fix Route Count in RELEASE_NOTES
**Files:** `docs/RELEASE_NOTES_v0.2.0.md`, `docs/STEP_B_COMPLETE_RU.md`  
**Fix:** Change "68 endpoints" to "74 endpoints"

---

## Files Modified (6)

1. `frontend-nuxt/composables/useKromaApi.ts` - Auth format + envelope unwrapping
2. `frontend-nuxt/pages/login.vue` - Auth format
3. `frontend-nuxt/pages/index.vue` - Remove deleteProject
4. `frontend-nuxt/pages/projects.vue` - Remove deleteProject

---

## Testing

### Auth Token
```typescript
// Backend returns: { ok: true, auth_token: { token: '...', ... } }
const { bootstrapToken } = useKromaApi()
await bootstrapToken()  // Now correctly reads auth_token.token
```

### List Endpoints
```typescript
// Backend returns: { ok: true, projects: [...] }
const projects = await api.getProjects()  // Now correctly extracts 'projects' field
```

### Delete Project
```typescript
// Shows "Not Implemented" message instead of failing
await api.deleteProject(slug)  // Function removed, calls show placeholder
```

---

*Last updated: 2026-03-24 05:00 (UTC)*

**Status:** ✅ **3/8 NEW COMMENTS COMPLETE**
