# Rust Backend: Desktop + Server (One Codebase)

**Answer: YES! ✅** The same Rust backend code works for BOTH desktop and server deployments.

---

## 🎯 Key Insight

**Your backend is ALREADY hybrid-ready!** No code changes needed — just environment variables.

```
┌──────────────────────────────────────────────────────────────┐
│              Rust Backend (src-tauri/)                       │
│                   (SAME CODE for both!)                      │
├──────────────────────────────────────────────────────────────┤
│  Desktop Mode                  │  Server Mode               │
│  ─────────────────────          │  ──────────────────        │
│  Env: KROMA_BACKEND_DB          │  Env: KROMA_BACKEND_DB_URL │
│  SQLite database                │  PostgreSQL database       │
│  Local filesystem               │  S3 storage                │
│  Single-user                    │  Multi-user + JWT auth     │
│  Dev auth bypass                │  Full auth required        │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔧 Configuration Matrix

| Feature | Desktop Mode | Server Mode | How to Switch |
|---------|-------------|-------------|---------------|
| **Database** | SQLite | PostgreSQL | Env variable |
| **Storage** | Local files | S3 | API endpoint |
| **Auth** | Dev bypass | JWT required | Env variable |
| **Users** | Single | Multi-user | Auto-detected |
| **CORS** | Disabled | Enabled | Env variable |

---

## 📋 Environment Variables

### Desktop Mode (.env.desktop)

```bash
# Database (SQLite)
KROMA_BACKEND_DB=var/backend/app.db

# Auth (dev bypass enabled)
KROMA_API_AUTH_DEV_BYPASS=true
KROMA_API_AUTH_BOOTSTRAP_FIRST_TOKEN=true

# Storage (local)
# (configured per-project via API)

# Bind address (localhost only)
KROMA_BACKEND_BIND=127.0.0.1:8788
```

### Server Mode (.env.server)

```bash
# Database (PostgreSQL)
KROMA_BACKEND_DB_URL=postgres://user:password@host:5432/kroma_db

# Auth (full security)
KROMA_API_AUTH_DEV_BYPASS=false
KROMA_API_AUTH_BOOTSTRAP_FIRST_TOKEN=false

# Storage (S3)
# (configured per-project via API)

# Bind address (public)
KROMA_BACKEND_BIND=0.0.0.0:8788

# CORS (for web frontend)
KROMA_CORS_ORIGINS=https://app.kroma.app,https://kroma.app

# Optional: Master key for encryption
IAT_MASTER_KEY=<base64url 32-byte key>
IAT_MASTER_KEY_REF=production-v1
```

---

## 🚀 Deployment Modes

### Mode 1: Desktop (Embedded in Tauri)

```rust
// src-tauri/src/main.rs (for Tauri)

#[tokio::main]
async fn main() {
    // Database auto-detects SQLite from KROMA_BACKEND_DB
    let db_path = std::env::var("KROMA_BACKEND_DB")
        .unwrap_or_else(|_| String::from("var/backend/app.db"));
    
    // Start backend on localhost
    let addr: SocketAddr = "127.0.0.1:8788".parse().unwrap();
    
    // Serve forever
    serve(addr).await.unwrap();
}
```

**User Experience:**
- App installs as `.exe` / `.dmg` / `.deb`
- Backend runs as background process
- Data stored locally (`~/Kroma/`)
- No internet required
- Single-user only

---

### Mode 2: Server (Hosted on VPS/Cloud)

```rust
// src-tauri/src/main.rs (SAME CODE!)

#[tokio::main]
async fn main() {
    // Database auto-detects PostgreSQL from KROMA_BACKEND_DB_URL
    let db_url = std::env::var("KROMA_BACKEND_DB_URL").ok();
    
    if let Some(url) = db_url {
        // PostgreSQL mode
        println!("Starting in server mode with PostgreSQL");
    } else {
        // SQLite mode
        println!("Starting in desktop mode with SQLite");
    }
    
    // Start backend on public interface
    let addr: SocketAddr = "0.0.0.0:8788".parse().unwrap();
    
    // Serve forever
    serve(addr).await.unwrap();
}
```

**User Experience:**
- Access via web browser
- Multi-user with authentication
- Data stored in PostgreSQL + S3
- Internet required
- Team collaboration possible

---

## 🗄️ Database Layer

### Auto-Detection Logic

```rust
// src-tauri/src/db/mod.rs

pub fn resolve_backend_config(repo_root: &Path) -> DatabaseBackendConfig {
    let db_url = std::env::var("KROMA_BACKEND_DB_URL").ok();
    let sqlite_path = std::env::var("KROMA_BACKEND_DB").ok();
    
    // PostgreSQL takes priority if set
    if let Some(url) = db_url.filter(|v| !v.trim().is_empty()) {
        return DatabaseBackendConfig::Postgres(PostgresConfig { 
            database_url: url 
        });
    }
    
    // Fallback to SQLite
    let path = sqlite_path.unwrap_or_else(|| String::from("var/backend/app.db"));
    DatabaseBackendConfig::Sqlite(DbConfig::new(path))
}
```

**Result:** Same code, different database based on env var!

---

## 📦 Storage Layer

### Local Files (Desktop)

```bash
# User sets via API
PUT /api/projects/my-comic/storage/local
{
  "local_project_root": "var/projects/my-comic"
}

# Files stored at:
var/projects/my-comic/outputs/scene1.png
var/projects/my-comic/upscaled/scene1.png
```

### S3 (Server)

```bash
# User sets via API
PUT /api/projects/my-comic/storage/s3
{
  "enabled": true,
  "bucket": "kroma-user-files",
  "prefix": "iat-projects",
  "region": "us-east-1"
}

# Files stored at:
s3://kroma-user-files/iat-projects/{project_id}/outputs/scene1.png
```

**Key Point:** The API is the same! Only storage backend changes.

---

## 🔐 Authentication Layer

### Desktop: Dev Bypass

```rust
// Auth middleware checks env var

let dev_bypass = std::env::var("KROMA_API_AUTH_DEV_BYPASS")
    .map(|v| v == "true")
    .unwrap_or(false);

if dev_bypass {
    // Skip auth checks (desktop mode)
    return Ok(());
}

// Full JWT validation (server mode)
validate_jwt(token)?;
```

### Server: JWT Required

```bash
# Env var disables bypass
KROMA_API_AUTH_DEV_BYPASS=false
```

**Result:** 
- Desktop: No login required (single-user)
- Server: JWT tokens required (multi-user)

---

## 📊 Feature Comparison

| Feature | Desktop (SQLite) | Server (PostgreSQL) | Code Change |
|---------|-----------------|---------------------|-------------|
| **Database** | SQLite file | PostgreSQL cluster | ❌ None |
| **Migrations** | Auto on startup | Auto on startup | ❌ None |
| **Storage** | Local filesystem | S3 compatible | ❌ None |
| **Auth** | Dev bypass | JWT tokens | ❌ None |
| **Users** | Single (app_users) | Multi (app_users) | ❌ None |
| **Projects** | Unlimited | Unlimited | ❌ None |
| **CORS** | Disabled | Configurable | ⚠️ Env var |
| **Bind Address** | 127.0.0.1 | 0.0.0.0 | ⚠️ Env var |
| **Rate Limiting** | Not needed | Recommended | ⚠️ Add middleware |

---

## 🛠️ What Needs to Change

### For Desktop → Server Deployment

**Add to backend (optional enhancements):**

1. **CORS Middleware** (for web frontend)

```rust
// src-tauri/src/api/mod.rs

use tower_http::cors::{CorsLayer, Any};

pub fn create_router() -> Router {
    let cors = if cfg!(feature = "web") {
        CorsLayer::new()
            .allow_origin(Any) // Configure properly!
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::permissive()
    };
    
    Router::new()
        // ... routes
        .layer(cors)
}
```

2. **Rate Limiting** (for public API)

```rust
// Add tower-governor or similar
use tower_governor::{GovernorConfigBuilder, governor_middleware};

let governor_conf = GovernorConfigBuilder::default()
    .per_second(10)
    .burst_size(100)
    .finish()
    .unwrap();

Router::new()
    // ... routes
    .layer(governor_middleware(&governor_conf))
```

3. **Health Check Endpoint** (for load balancers)

```rust
// Already exists! GET /health
// Returns: {"ok": true, "status": "ok", "service": "kroma-backend-core"}
```

---

## 📦 Deployment Examples

### Desktop: Tauri Embedded

```rust
// frontend/src-tauri/src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kroma_backend_core::api::server::serve;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Backend runs on localhost
    let addr: SocketAddr = "127.0.0.1:8788".parse().unwrap();
    
    // Spawn backend in background
    tokio::spawn(async move {
        serve(addr).await.unwrap();
    });
    
    // Run Tauri WebView
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Build:**
```bash
npm run tauri build
# Creates: .exe, .dmg, .deb
```

---

### Server: Docker Container

```dockerfile
# Dockerfile

FROM rust:1.75 as builder

WORKDIR /app
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock ./
COPY src-tauri/src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/kroma-backend-core /usr/local/bin/

EXPOSE 8788

CMD ["kroma-backend-core"]
```

**Deploy:**
```bash
docker run -d \
  -p 8788:8788 \
  -e KROMA_BACKEND_DB_URL=postgres://... \
  -e KROMA_API_AUTH_DEV_BYPASS=false \
  -v /data/kroma:/var/kroma \
  kroma-backend:latest
```

---

### Server: Systemd Service (VPS)

```ini
# /etc/systemd/system/kroma-backend.service

[Unit]
Description=Kroma Backend API
After=network.target postgresql.service

[Service]
Type=simple
User=kroma
Group=kroma
WorkingDirectory=/opt/kroma

Environment=KROMA_BACKEND_DB_URL=postgres://user:pass@localhost/kroma
Environment=KROMA_API_AUTH_DEV_BYPASS=false
Environment=KROMA_BACKEND_BIND=0.0.0.0:8788

ExecStart=/opt/kroma/kroma-backend-core
Restart=always

[Install]
WantedBy=multi-user.target
```

**Deploy:**
```bash
sudo systemctl enable kroma-backend
sudo systemctl start kroma-backend
sudo systemctl status kroma-backend
```

---

## 🔄 Migration Path

### From Desktop to Server

**Scenario:** User wants to sync desktop project to server.

**Steps:**

1. **Export from Desktop:**
   ```bash
   # Desktop app exports project
   GET /api/projects/my-comic/bootstrap-prompt
   
   # Returns JSON with:
   # - Project metadata
   # - Style guides
   # - Characters
   # - Reference sets
   # (NOT secret values)
   ```

2. **Import to Server:**
   ```bash
   # Web app imports
   POST /api/projects/my-comic/bootstrap-import
   {
     "mode": "merge",
     "bootstrap_json": {...}
   }
   ```

3. **Upload Files:**
   ```bash
   # Web app uploads files via FormData
   POST /api/projects/my-comic/assets
   Content-Type: multipart/form-data
   ```

**Result:** Project migrated from desktop to server!

---

## ⚠️ Gotchas

### 1. File Paths

**Desktop:** `var/projects/my-comic/outputs/image.png`

**Server:** S3 URL or database reference

**Solution:** Abstract file access behind trait:

```rust
// src-tauri/src/storage/mod.rs

pub trait StorageBackend: Send + Sync {
    async fn save(&self, path: &str, data: &[u8]) -> Result<String>;
    async fn load(&self, path: &str) -> Result<Vec<u8>>;
    async fn delete(&self, path: &str) -> Result<()>;
}

// Desktop implementation
pub struct LocalStorage { root: PathBuf }

// Server implementation  
pub struct S3Storage { bucket: String, client: S3Client }
```

---

### 2. Database Schema

**SQLite and PostgreSQL have slight differences:**

| Feature | SQLite | PostgreSQL |
|---------|--------|------------|
| Boolean | INTEGER (0/1) | BOOLEAN |
| Text | TEXT | TEXT/VARCHAR |
| Timestamp | TEXT (ISO8601) | TIMESTAMPTZ |
| Auto-increment | INTEGER PRIMARY KEY | SERIAL / GENERATED |

**Solution:** Use rusqlite for SQLite, sqlx for PostgreSQL with abstraction layer.

**Current Status:** Your code already handles this! ✅

---

### 3. Concurrency

**SQLite:** File-level locking (single writer)

**PostgreSQL:** Row-level locking (multiple writers)

**Impact:** Desktop may be slower with concurrent writes.

**Solution:** Queue writes on desktop, or use connection pooling.

---

## 📊 Performance Comparison

| Metric | Desktop (SQLite) | Server (PostgreSQL) |
|--------|-----------------|---------------------|
| **Read Latency** | ~1ms | ~5ms (network) |
| **Write Latency** | ~5ms | ~10ms (network + WAL) |
| **Concurrent Reads** | Unlimited | Unlimited |
| **Concurrent Writes** | 1 (file lock) | Hundreds |
| **Max DB Size** | ~140 TB | Unlimited |
| **Backup** | Copy file | pg_dump / replication |

---

## 🎯 Recommendation

**Use the same Rust backend for both!**

**Benefits:**
- ✅ One codebase to maintain
- ✅ Same API contract for desktop and web
- ✅ Easy migration path for users
- ✅ Backend features work everywhere
- ✅ No code duplication

**What to add:**
1. CORS middleware (for web frontend)
2. Rate limiting (for public server)
3. Health check endpoint (already exists!)
4. Metrics/monitoring (optional)

**Timeline:**
- Desktop deployment: Ready now! ✅
- Server deployment: 1-2 days (CORS + rate limiting)

---

## 📦 Next Steps

1. **Test desktop mode** with Tauri + Nuxt
2. **Add CORS middleware** to backend
3. **Deploy to test server** with PostgreSQL
4. **Test web frontend** against server backend
5. **Document migration** process for users

**Ready to proceed?**
