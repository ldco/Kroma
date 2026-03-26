# External API Integration Guide

**Version:** 1.0
**Last Updated:** 2025-12-19

---

## Table of Contents

1. [Overview](#overview)
2. [Why Backend Proxy?](#why-backend-proxy)
3. [Configuration](#configuration)
4. [Architecture](#architecture)
5. [Features](#features)
6. [Usage Examples](#usage-examples)
7. [Production Considerations](#production-considerations)
8. [Troubleshooting](#troubleshooting)

---

## Overview

Puppet Master supports using an **external REST API** as an alternative to the built-in SQLite database. This feature allows you to:

- **Integrate with headless CMS** (Strapi, Contentful, Sanity, etc.)
- **Build microservices architectures** with shared data layers
- **Deploy multi-instance applications** with centralized data
- **Migrate to cloud-native** infrastructure gradually

### Key Capabilities

✅ **Three Provider Modes**:

- `database` - Local SQLite (default, zero configuration)
- `api` - All data from external REST API
- `hybrid` - Mix & match per resource (e.g., auth in DB, content from API)

✅ **Production-Grade Resilience**:

- Circuit breaker pattern prevents cascading failures
- Exponential backoff retry logic
- Intelligent caching with per-resource TTL
- OAuth 2.0 / JWT / API Key authentication

✅ **Zero Frontend Changes**:

- Frontend always uses same `/api/*` endpoints
- Data source is completely transparent
- Toggle providers without touching Vue components

---

## Why Backend Proxy?

### ✅ Our Approach (Backend Proxy)

All API calls go through your Nuxt backend, which proxies to the external API:

```
Frontend → /api/portfolio → Nuxt Backend → External API → Data
```

**Benefits:**

| **Security**                               | **Performance**                          | **Reliability**                              |
| ------------------------------------------ | ---------------------------------------- | -------------------------------------------- |
| ✅ Credentials never exposed to browser    | ✅ Server-side cache shared across users | ✅ Centralized retry logic + circuit breaker |
| ✅ No CORS configuration needed            | ✅ Request deduplication                 | ✅ Graceful degradation on failures          |
| ✅ Request signing without client exposure | ✅ Connection pooling                    | ✅ Automatic OAuth token refresh             |
| ✅ Centralized rate limiting               | ✅ Reduced client bundle size            | ✅ Fallback to cached data                   |

### ❌ Alternative (Frontend Direct API)

**Problems with direct frontend-to-API calls:**

- 🔴 API credentials exposed in browser (DevTools can extract them)
- 🔴 CORS configuration complexity
- 🔴 Per-user caching (inefficient, browser storage limits)
- 🔴 Larger frontend bundle (HTTP client + auth logic)
- 🔴 Token refresh UI complexity
- 🔴 Can't share cache between users
- 🔴 Harder to implement circuit breaker patterns

**Only use frontend direct API for:**

- Public APIs (no credentials needed)
- User-specific OAuth (e.g., "Login with Google")
- Real-time features (WebSocket/SSE)
- Client-side file uploads to signed URLs

---

## Configuration

### 1. puppet-master.config.ts (Build-Time)

Choose your provider mode:

```typescript
// app/puppet-master.config.ts
export default {
  dataSource: {
    // Provider mode
    provider: 'database', // 'database' | 'api' | 'hybrid'

    // Hybrid mode: per-resource configuration
    resources: {
      users: 'database', // Keep auth local (recommended)
      sessions: 'database', // Keep sessions local (recommended)
      settings: 'api', // Fetch settings from API
      portfolio: 'api', // Fetch content from API
      contacts: 'api', // Send contacts to API
      translations: 'database' // Or 'api' depending on your CMS
    },

    // API client configuration
    api: {
      timeout: 30000, // Request timeout (30 seconds)

      // Retry with exponential backoff
      retry: {
        maxAttempts: 3, // Try up to 3 times
        initialDelay: 1000, // Start with 1 second
        maxDelay: 10000, // Cap at 10 seconds
        backoffMultiplier: 2 // Double delay each time
      },

      // Circuit breaker (prevent cascading failures)
      circuitBreaker: {
        enabled: true,
        failureThreshold: 5, // Open after 5 failures
        resetTimeout: 60000 // Try again after 60 seconds
      },

      // Response caching (seconds)
      cache: {
        enabled: true,
        ttl: {
          users: 300, // 5 minutes
          sessions: 60, // 1 minute
          settings: 600, // 10 minutes
          portfolio: 180, // 3 minutes
          contacts: 0, // No cache (always fresh)
          translations: 3600 // 1 hour
        }
      }
    }
  }
}
```

### 2. .env (Runtime Credentials)

**Option A: OAuth 2.0 Client Credentials (Recommended)**

```bash
# API base URL
API_BASE_URL=https://api.example.com/v1

# OAuth 2.0 credentials
API_CLIENT_ID=your-client-id
API_CLIENT_SECRET=your-client-secret
API_TOKEN_URL=https://auth.example.com/oauth/token

# Token refresh timing (optional)
API_TOKEN_REFRESH_BUFFER=300  # Refresh 5 min before expiry
```

**Option B: Static JWT Token (Simpler for Development)**

```bash
API_BASE_URL=https://api.example.com/v1
API_JWT_TOKEN=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Option C: API Key (Least Secure, Internal Only)**

```bash
API_BASE_URL=https://api.example.com/v1
API_KEY=your-api-key-here
```

**Optional: Redis for Multi-Instance Deployments**

```bash
# For shared cache across multiple app instances
REDIS_URL=redis://localhost:6379
REDIS_PREFIX=pm-cache:
```

---

## Architecture

> **Note:** The API client, authentication manager, cache layer, and retry logic are fully implemented. The Repository Factory Layer shown below is a **planned future enhancement** — currently, API routes interact directly with the database (Drizzle) or API client.

### System Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    Frontend (Vue/Nuxt)                        │
│              Always uses /api/* endpoints                     │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│              Backend API Routes (25 endpoints)                │
│         /api/portfolio, /api/auth, /api/settings...           │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│              Repository Factory Layer (FUTURE)                │
│    usePortfolioRepository(), useUsersRepository(), etc.       │
│         Returns appropriate implementation per config         │
└────────────┬───────────────────────────────┬─────────────────┘
             │                               │
    ┌────────▼────────┐             ┌────────▼─────────┐
    │   Database      │             │   API Client     │
    │ Implementation  │             │  Implementation  │
    │   (Drizzle)     │             │  (REST + JWT)    │
    └────────┬────────┘             └────────┬─────────┘
             │                               │
    ┌────────▼────────┐             ┌────────▼─────────┐
    │  SQLite + WAL   │             │  External API    │
    │  (current DB)   │             │  + Cache Layer   │
    └─────────────────┘             └──────────────────┘
```

### Component Breakdown

**1. API Client** (`server/utils/api/client.ts`)

- Centralized HTTP client with fetch API
- Handles all external API communication
- Singleton pattern for efficiency

**2. Authentication Manager** (`server/utils/api/auth.ts`)

- OAuth 2.0 client credentials flow
- JWT token management
- Automatic token refresh
- Support for static tokens and API keys

**3. Cache Layer** (`server/utils/api/cache.ts`)

- In-memory cache (Map-based)
- Automatic cleanup of expired entries
- Redis-ready for multi-instance deployments
- Per-resource TTL configuration

**4. Retry Logic + Circuit Breaker** (`server/utils/api/retry.ts`)

- Exponential backoff algorithm
- Configurable max attempts and delays
- Circuit breaker states: CLOSED → OPEN → HALF_OPEN
- Smart retryable error detection (5xx, 429, network errors)

---

## Features

### Circuit Breaker Pattern

**Purpose:** Prevent cascading failures when external API is down

**How it works:**

1. **CLOSED** (Normal): All requests go through
2. After **5 consecutive failures** → Circuit opens
3. **OPEN** (Failing): Requests immediately rejected (no API calls)
4. After **60 seconds** → Circuit goes half-open
5. **HALF_OPEN** (Testing): Allow one request to test recovery
6. If successful → Circuit closes, back to normal
7. If fails → Circuit opens again

**Example:**

```
[10:00:00] Request 1 fails (500) - Retry 3 times - Failure count: 1
[10:00:05] Request 2 fails (500) - Retry 3 times - Failure count: 2
[10:00:10] Request 3 fails (500) - Retry 3 times - Failure count: 3
[10:00:15] Request 4 fails (500) - Retry 3 times - Failure count: 4
[10:00:20] Request 5 fails (500) - Retry 3 times - Failure count: 5
[10:00:25] Circuit OPEN - Service unavailable
[10:00:30] Request 6 - Immediately rejected (no API call)
[10:01:25] Circuit HALF_OPEN - Testing recovery
[10:01:30] Request 7 succeeds - Circuit CLOSED - Back to normal
```

### Exponential Backoff Retry

**Purpose:** Handle transient failures gracefully

**Algorithm:**

```
delay = initialDelay × backoffMultiplier ^ attemptNumber
delay = min(delay, maxDelay)
```

**Example with default config:**

```
Attempt 1: 0ms    (immediate)
Attempt 2: 1000ms (1 second)
Attempt 3: 2000ms (2 seconds)
Max reached: 10000ms (10 seconds)
```

**Retryable errors:**

- Network errors (connection refused, timeout)
- HTTP 5xx (server errors)
- HTTP 429 (rate limit)

**Non-retryable errors:**

- HTTP 4xx (except 429) - client errors
- Validation failures
- Authentication failures (401)

### Intelligent Caching

**Cache Strategy:**

- **GET requests only** (no caching of mutations)
- **Per-resource TTL** (configure in config)
- **Prefix-based invalidation** (clear all portfolio cache)
- **Automatic expiry cleanup** (every 5 minutes)

**Cache Key Format:**

```
GET:/api/portfolio:{"category":"web","limit":10}
```

**Cache Hit Example:**

```typescript
// First request - cache MISS
const items1 = await repo.findAll({ category: 'web', limit: 10 })
// → Hits external API
// → Stores in cache with TTL=180s

// Second request within 3 minutes - cache HIT
const items2 = await repo.findAll({ category: 'web', limit: 10 })
// → Returns from cache (no API call)
// → Much faster response
```

---

## Usage Examples

### Example 1: Strapi Headless CMS

**Configuration:**

```typescript
// puppet-master.config.ts
dataSource: {
  provider: 'hybrid',
  resources: {
    users: 'database',      // Keep auth local
    sessions: 'database',   // Keep sessions local
    portfolio: 'api',       // Use Strapi
    settings: 'api',        // Use Strapi
    contacts: 'api',        // Use Strapi
    translations: 'database' // Or 'api' if Strapi has i18n
  }
}
```

```.env
# Strapi configuration
API_BASE_URL=https://your-strapi.com/api
API_JWT_TOKEN=your-strapi-api-token
```

**API Contract:** Your Strapi API must provide:

- `GET /portfolio` - List items
- `GET /portfolio/:id` - Get single item
- `POST /portfolio` - Create item
- `PUT /portfolio/:id` - Update item
- `DELETE /portfolio/:id` - Delete item

### Example 2: Contentful CMS

**Configuration:**

```.env
API_BASE_URL=https://cdn.contentful.com/spaces/YOUR_SPACE
API_KEY=YOUR_DELIVERY_API_KEY
```

### Example 3: Custom Microservice

**Configuration:**

```.env
# OAuth 2.0 with custom service
API_BASE_URL=https://api.yourservice.com/v1
API_CLIENT_ID=puppet-master-client
API_CLIENT_SECRET=super-secret-key
API_TOKEN_URL=https://auth.yourservice.com/oauth/token
```

---

## Production Considerations

### Security Checklist

✅ **Never commit credentials** - Use environment variables
✅ **Use OAuth 2.0** in production (not static tokens)
✅ **Enable HTTPS** for external API
✅ **Rotate tokens regularly** - Set up auto-rotation
✅ **Audit API logs** - Monitor for suspicious activity
✅ **Rate limit** your API endpoints
✅ **Validate responses** - Don't trust external data blindly

### Performance Optimization

✅ **Enable caching** - Reduce API calls by 70%+
✅ **Use appropriate TTLs** - Balance freshness vs performance
✅ **Monitor cache hit rates** - Aim for >70%
✅ **Use Redis** for multi-instance deployments
✅ **Configure timeout** - Don't wait forever for slow APIs
✅ **Optimize retries** - Balance resilience vs latency

### Monitoring & Observability

**Recommended metrics to track:**

```typescript
// Circuit breaker state
const { circuitState, failureCount } = apiClient.getCircuitState()

// Cache statistics
const { total, active, expired } = apiClient.getCacheStats()

// API call metrics
- Request count (success/failure)
- Average response time
- Cache hit rate
- Retry count
- Circuit breaker opens
```

**Health Check Endpoint (TODO):**

```typescript
// server/api/health.get.ts
export default defineEventHandler(() => {
  const apiClient = useAPIClient()

  return {
    status: 'healthy',
    circuit: apiClient.getCircuitState(),
    cache: apiClient.getCacheStats()
  }
})
```

### Scaling Considerations

**Single Instance:**

- ✅ In-memory cache works great
- ✅ Circuit breaker per instance
- ✅ Simple deployment

**Multi-Instance (Load Balanced):**

- ⚠️ Need Redis for shared cache
- ⚠️ Each instance has separate circuit state
- ⚠️ Consider centralizing circuit breaker in Redis
- ✅ Better fault tolerance

### Cost Considerations

**API Calls:**

- Caching reduces API calls by 70%+
- Circuit breaker prevents wasted calls when API is down
- Retries multiply your costs (1 request = up to 3 API calls)

**Optimization:**

- Increase cache TTLs for static content
- Use `provider: 'hybrid'` to keep frequently accessed data in DB
- Monitor and optimize slow endpoints

---

## Troubleshooting

### Common Issues

#### 1. "Circuit is OPEN - service unavailable"

**Cause:** External API has failed 5+ times

**Solution:**

1. Check if external API is online
2. Verify credentials in `.env`
3. Check API logs for errors
4. Wait 60 seconds for circuit to half-open
5. Or manually reset: `apiClient.resetCircuit()`

#### 2. "Token refresh failed"

**Cause:** OAuth credentials invalid or token endpoint unreachable

**Solution:**

1. Verify `API_CLIENT_ID` and `API_CLIENT_SECRET`
2. Check `API_TOKEN_URL` is correct
3. Test OAuth endpoint manually:
   ```bash
   curl -X POST $API_TOKEN_URL \
     -d "grant_type=client_credentials" \
     -d "client_id=$API_CLIENT_ID" \
     -d "client_secret=$API_CLIENT_SECRET"
   ```

#### 3. "Max retries exceeded"

**Cause:** All 3 retry attempts failed

**Solution:**

1. Check external API health
2. Verify network connectivity
3. Increase `retry.maxAttempts` if API is flaky
4. Check if error is retryable (5xx, 429, network errors)

#### 4. Stale cache data

**Cause:** Cache TTL too long or cache not invalidated

**Solution:**

1. Lower `cache.ttl` for that resource
2. Manually invalidate cache:
   ```typescript
   apiClient.invalidateCache('portfolio')
   ```
3. Disable cache for that resource: `ttl: 0`

#### 5. Performance issues

**Symptoms:** Slow response times

**Diagnosis:**

1. Check cache hit rate:
   ```typescript
   const stats = apiClient.getCacheStats()
   console.log('Hit rate:', (stats.active / stats.total) * 100 + '%')
   ```
2. Monitor external API response times
3. Check if retry logic is triggering often

**Solutions:**

- Increase cache TTLs
- Use `provider: 'hybrid'` for frequently accessed data
- Optimize external API endpoints
- Add request deduplication (TODO)

---

## Roadmap (Future Enhancements)

### Planned Features

1. **Repository Pattern Implementation** - Fully abstract data layer
2. **Request Deduplication** - Coalesce simultaneous identical requests
3. **GraphQL Support** - Add GraphQL client alongside REST
4. **Webhook Cache Invalidation** - Real-time cache updates
5. **Health Monitoring UI** - Admin panel for circuit breaker / cache stats
6. **Redis Cache Support** - Distributed cache for multi-instance
7. **Request/Response Logging** - Debug mode for API calls
8. **Metrics Endpoint** - Prometheus-compatible metrics

### Contributing

Found a bug or have a feature request? Please open an issue on GitHub.

---

## API Contract Specification

For external APIs to work with Puppet Master, they must implement these endpoints:

### Portfolio Resource

```
GET    /portfolios             - List portfolios
GET    /portfolios/:id         - Get single portfolio
POST   /portfolios             - Create portfolio
PUT    /portfolios/:id         - Update portfolio
DELETE /portfolios/:id         - Delete portfolio
```

**Request/Response Formats:** See `server/repositories/types.ts` for base TypeScript interfaces. Full portfolio types are defined inline in `server/api/portfolios/*.ts`.

### Settings Resource

```
GET /settings - Get all settings (grouped)
PUT /settings - Update settings (batch)
```

### Contacts Resource

```
GET    /contacts        - List submissions
POST   /contacts        - Create submission
PUT    /contacts/:id    - Mark read/unread
DELETE /contacts/:id    - Delete submission
```

### Users Resource

```
GET    /users      - List users
POST   /users      - Create user
PUT    /users/:id  - Update user
DELETE /users/:id  - Delete user
```

### Sessions Resource

```
GET    /sessions/:id - Get session
POST   /sessions     - Create session
DELETE /sessions/:id - Delete session
```

### Translations Resource

```
GET    /translations/:locale  - Get all translations for locale
POST   /translations          - Create/update translation
DELETE /translations/:id      - Delete translation
```

---

## Additional Resources

- [Nuxt Documentation](https://nuxt.com/docs)
- [OAuth 2.0 Client Credentials Flow](https://oauth.net/2/grant-types/client-credentials/)
- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
- [HTTP Caching Best Practices](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)

---

**Last Updated:** 2025-12-19
**Maintained by:** Puppet Master Team
