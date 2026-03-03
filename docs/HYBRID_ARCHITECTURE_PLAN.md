# Kroma: Hybrid Architecture Plan (Desktop Now → Web Later)

**Goal:** Build Tauri desktop app now, but design for future web deployment with PostgreSQL backend.

---

## 📋 Architecture Options

### Option 1: Pure Hybrid (Recommended ✅)

```
┌──────────────────────────────────────────────────────────────┐
│                    Frontend (React + TypeScript)              │
│                   Same codebase for both modes                │
├──────────────────────────────────────────────────────────────┤
│  Desktop Mode (Tauri)          │  Web Mode (Browser)         │
│  ─────────────────────          │  ──────────────────         │
│  Tauri WebView                  │  Standard Browser           │
│  Local API: http://localhost    │  Remote API: https://api    │
│  SQLite database                │  PostgreSQL database        │
│  Local filesystem               │  S3 storage                 │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│              Backend API (Rust - Same Codebase)               │
├──────────────────────────────────────────────────────────────┤
│  Desktop Mode                  │  Web Mode                   │
│  ─────────────────────          │  ──────────────────         │
│  SQLite (var/backend/app.db)   │  PostgreSQL (hosted)        │
│  Local file storage            │  S3 storage                 │
│  Single-user                   │  Multi-user + auth          │
│  No internet required          │  Internet required          │
└──────────────────────────────────────────────────────────────┘
```

**Key Insight:** The **backend API doesn't change** - only the storage layer!

---

### Option 2: Tauri with Remote Backend

```
┌──────────────────────────────────────────────────────────────┐
│              Tauri Desktop App (Frontend)                     │
│                   React + TypeScript                          │
├──────────────────────────────────────────────────────────────┤
│  Local Mode                    │  Cloud Mode                 │
│  ─────────────────────          │  ──────────────────         │
│  Rust backend embedded         │  Remote API server          │
│  SQLite + local files          │  PostgreSQL + S3            │
└──────────────────────────────────────────────────────────────┘
```

**User can switch between local and cloud sync.**

---

### Option 3: Separate Frontends (NOT Recommended ❌)

```
Desktop Frontend (Tauri)     Web Frontend (React SPA)
        ↓                           ↓
   Rust Backend               Rust Backend
   (same code)                (same code)
```

**Problem:** Maintaining two frontends is expensive. Avoid this.

---

## ✅ Recommended: Pure Hybrid Approach

### Phase 1: Desktop First (Now)

**Timeline:** 11-14 weeks

```
┌──────────────────────────────────────────────────────────────┐
│  Tauri Desktop App                                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  React Frontend (TypeScript)                           │  │
│  │  - API client configured for localhost:8788            │  │
│  │  - File uploads via local paths                        │  │
│  └────────────────────────────────────────────────────────┘  │
│                              ↓                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Rust Backend (embedded)                               │  │
│  │  - SQLite database                                     │  │
│  │  - Local filesystem storage                            │  │
│  │  - Single-user mode                                    │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

**Tech Stack:**
- **Frontend:** React 19 + TypeScript + Vite
- **UI Library:** React Bootstrap + Material Design
- **State:** React Query (for API caching)
- **Desktop:** Tauri 2.x (bundles Rust backend + WebView)

---

### Phase 2: Web Deployment (Future)

**Timeline:** 2-3 weeks (mostly configuration)

```
┌──────────────────────────────────────────────────────────────┐
│  Web Browser                                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  React Frontend (SAME CODEBASE)                        │  │
│  │  - API client configured for https://api.kroma.app     │  │
│  │  - File uploads via multipart/form-data                │  │
│  └────────────────────────────────────────────────────────┘  │
│                              ↓                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Rust Backend (hosted on server)                       │  │
│  │  - PostgreSQL database                                 │  │
│  │  - S3 storage                                          │  │
│  │  - Multi-user + JWT auth                               │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

**Changes Required:**
1. **Frontend:**
   - Change API base URL (environment variable)
   - Add file upload handling for web (multipart instead of local paths)
   - Add user session management (JWT tokens)

2. **Backend:**
   - Enable PostgreSQL mode (`KROMA_BACKEND_DB_URL=postgres://...`)
   - Enable multi-user auth (already implemented!)
   - Configure S3 storage (already implemented!)

**That's it!** The backend already supports both modes.

---

## 🔧 What Needs to Change

### Frontend Changes (Desktop → Web)

| Feature | Desktop (Tauri) | Web (Browser) | Change Required |
|---------|----------------|---------------|-----------------|
| **API URL** | `http://localhost:8788` | `https://api.kroma.app` | ✅ Env variable |
| **File Upload** | Local path strings | `FormData` with files | ✅ Adapter layer |
| **Auth** | Optional (dev bypass) | JWT required | ✅ Add login screen |
| **Storage** | Show local paths | Show S3 URLs | ✅ Format conversion |
| **Offline** | Full support | No offline | ✅ Feature flag |

### Backend Changes (Desktop → Web)

| Feature | Desktop Mode | Web Mode | Status |
|---------|-------------|----------|--------|
| **Database** | SQLite | PostgreSQL | ✅ Already supported |
| **Storage** | Local filesystem | S3 | ✅ Already supported |
| **Auth** | Dev bypass | JWT tokens | ✅ Already implemented |
| **Multi-user** | Single user | Multi-user | ✅ Schema ready |
| **Env Config** | `KROMA_BACKEND_DB` | `KROMA_BACKEND_DB_URL` | ✅ Already supported |

**Conclusion:** Backend is **already hybrid-ready!** ✅

---

## 📁 Recommended Project Structure

```
app/
├── src-tauri/              # Rust backend (stays the same)
│   ├── src/
│   │   ├── api/            # API routes (no change)
│   │   ├── db/             # DB layer (SQLite + PostgreSQL)
│   │   └── ...
│   └── Cargo.toml
│
├── frontend/               # React frontend (NEW)
│   ├── src/
│   │   ├── api/            # API client
│   │   │   ├── client.ts   # Base HTTP client
│   │   │   ├── desktop.ts  # Desktop-specific adapters
│   │   │   └── web.ts      # Web-specific adapters
│   │   ├── components/     # UI components (reusable)
│   │   ├── pages/          # Pages (J00-J08 journey)
│   │   ├── hooks/          # React hooks
│   │   └── App.tsx
│   ├── src-tauri/          # Tauri configuration
│   │   ├── tauri.conf.json # Desktop mode config
│   │   └── ...
│   ├── .env.desktop        # Desktop environment
│   ├── .env.web            # Web environment
│   └── package.json
│
└── docs/                   # Documentation
```

---

## 🚀 Implementation Plan

### Sprint 1-3: Desktop Frontend (J00-J03)

**Weeks 1-3:**
- Setup React + TypeScript + Vite
- Configure Tauri 2.x
- Implement J00 (onboarding) + J01 (projects)
- API client for localhost backend

**Weeks 4-6:**
- Implement J02 (references) + J03 (bootstrap)
- File picker integration (Tauri dialogs)
- Local path handling

---

### Sprint 4-6: Desktop Frontend (J04-J08)

**Weeks 7-10:**
- Implement J04-J06 (run workflow)
- Image preview + candidate selection
- Progress tracking

**Weeks 11-14:**
- Implement J07 (post-process) + J08 (export)
- QA reports UI
- Export manifest viewer

---

### Sprint 7: Web Adaptation (2-3 weeks)

**Week 1:**
- Create `.env.web` with remote API URL
- Add file upload adapter (FormData)
- Add JWT auth flow

**Week 2:**
- Deploy backend to server with PostgreSQL
- Configure S3 storage
- Test multi-user isolation

**Week 3:**
- Deploy frontend to CDN (Vercel/Netlify)
- CORS configuration
- Production testing

---

## 💡 Key Design Decisions

### 1. API Client Abstraction

```typescript
// frontend/src/api/client.ts

interface FileUpload {
  type: 'local_path' | 'file_data';
  value: string | File;
}

class ApiClient {
  // Desktop: sends { file_path: "var/projects/..." }
  // Web: sends FormData with file
  async uploadImage(endpoint: string, file: FileUpload) {
    if (file.type === 'local_path') {
      return this.post(endpoint, { file_path: file.value });
    } else {
      const formData = new FormData();
      formData.append('file', file.value);
      return this.postFormData(endpoint, formData);
    }
  }
}

// Desktop mode
const desktopClient = new ApiClient({
  baseUrl: 'http://localhost:8788',
  fileMode: 'local_path'
});

// Web mode
const webClient = new ApiClient({
  baseUrl: 'https://api.kroma.app',
  fileMode: 'file_data'
});
```

---

### 2. Environment Configuration

```typescript
// frontend/src/config.ts

export const config = {
  apiUrl: import.meta.env.VITE_API_URL || 'http://localhost:8788',
  fileMode: import.meta.env.VITE_FILE_MODE || 'local_path',
  authRequired: import.meta.env.VITE_AUTH_REQUIRED === 'true',
  s3Enabled: import.meta.env.VITE_S3_ENABLED === 'true',
};
```

```bash
# frontend/.env.desktop
VITE_API_URL=http://localhost:8788
VITE_FILE_MODE=local_path
VITE_AUTH_REQUIRED=false
VITE_S3_ENABLED=false

# frontend/.env.web
VITE_API_URL=https://api.kroma.app
VITE_FILE_MODE=file_data
VITE_AUTH_REQUIRED=true
VITE_S3_ENABLED=true
```

---

### 3. Backend Feature Flags

```rust
// Backend automatically detects mode from env vars

// Desktop mode:
// KROMA_BACKEND_DB=var/backend/app.db
// → Uses SQLite

// Web mode:
// KROMA_BACKEND_DB_URL=postgres://user:pass@host/db
// → Uses PostgreSQL

// Both modes work with same codebase!
```

---

## ⚠️ Gotchas to Avoid

### 1. File Path Handling

**Desktop:**
```json
{ "input_path": "var/projects/my-comic/outputs/image.png" }
```

**Web:**
```javascript
const formData = new FormData();
formData.append('file', fileInput.files[0]);
```

**Solution:** Create adapter layer in API client.

---

### 2. Authentication

**Desktop:** Optional (dev bypass enabled)

**Web:** Required (JWT tokens)

**Solution:** Make auth optional in backend (already done!), add login screen for web.

---

### 3. CORS

**Desktop:** No CORS (same origin)

**Web:** CORS required

**Solution:** Add CORS middleware to backend (axum has this).

---

### 4. Offline Support

**Desktop:** Full offline support

**Web:** No offline (or PWA with service workers)

**Solution:** Feature flag offline-dependent features.

---

## 📊 Comparison Table

| Feature | Desktop (Tauri) | Web (Browser) | Effort to Support Both |
|---------|----------------|---------------|------------------------|
| **Frontend Code** | React + TS | React + TS (same) | ✅ Low |
| **Backend Code** | Rust | Rust (same) | ✅ None |
| **Database** | SQLite | PostgreSQL | ✅ Env switch |
| **File Upload** | Local paths | FormData | ⚠️ Adapter needed |
| **Auth** | Optional | Required | ⚠️ Add login UI |
| **Offline** | Full | None/PWA | ⚠️ Feature flags |
| **Distribution** | .exe/.dmg/.AppImage | URL | ✅ Different build |
| **Updates** | Tauri updater | Instant deploy | ✅ Different mechanism |

---

## 🎯 Recommendation

**Start with Tauri desktop app** using the hybrid architecture:

1. **Build frontend as React SPA** (not Tauri-specific)
2. **Configure API client** for localhost backend
3. **Use Tauri only for:**
   - System tray icon
   - Native file dialogs
   - Auto-updater
   - Menu bar

4. **Keep web compatibility:**
   - Abstract file uploads
   - Env-based API URL
   - Optional auth

**Result:** You can deploy as web app later with **2-3 weeks of work** instead of rebuilding from scratch.

---

## 📦 Next Steps

1. **Decide on frontend framework** (React recommended ✅)
2. **Setup React + TypeScript + Vite** in `frontend/` folder
3. **Configure Tauri 2.x** in `frontend/src-tauri/`
4. **Start with J00-J01** (onboarding + projects)

**Ready to start frontend implementation?**
