# API Reference

Complete reference for all Puppet Master API endpoints.

## Authentication

### Session-Based Auth

Puppet Master uses session-based authentication with HTTP-only cookies:

1. **Login** - POST `/api/auth/login` returns session cookie + CSRF token
2. **Include cookies** - All requests must include `credentials: 'include'`
3. **CSRF header** - State-changing requests require `X-CSRF-Token` header

### Making Authenticated Requests

```typescript
async function apiRequest(url: string, options: RequestInit = {}) {
  return fetch(url, {
    ...options,
    credentials: 'include',
    headers: {
      ...options.headers,
      'X-CSRF-Token': csrfToken,
      'Content-Type': 'application/json'
    }
  })
}
```

### Error Response Format

```json
{
  "statusCode": 401,
  "message": "Authentication required",
  "data": {}
}
```

| Code | Meaning |
|------|---------|
| 400 | Validation error |
| 401 | Not authenticated |
| 403 | Forbidden (role or CSRF) |
| 404 | Resource not found |
| 409 | Conflict (duplicate) |
| 423 | Account locked |
| 429 | Rate limited |
| 503 | Service unhealthy |

---

## Public Endpoints

### Health Check

```
GET /api/health
```

Returns server health status. Used by load balancers and monitoring.

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00.000Z",
  "uptime": 3600,
  "version": "1.0.0",
  "checks": [
    { "name": "database", "status": "ok", "latency": 5 },
    { "name": "memory", "status": "ok", "message": "150MB / 500MB" }
  ],
  "responseTime": 12
}
```

| Status | HTTP Code |
|--------|-----------|
| `ok` | 200 |
| `degraded` | 200 |
| `unhealthy` | 503 |

---

### Settings

```
GET /api/settings
```

Returns public site settings grouped by category.

**Response:**
```json
{
  "contact": {
    "email": "hello@example.com",
    "phone": "+1234567890",
    "location": "New York, NY"
  },
  "social": {
    "instagram": "https://instagram.com/example",
    "telegram": "https://t.me/example"
  },
  "seo": {
    "title": "My Site",
    "description": "Site description"
  }
}
```

---

### Translations

```
GET /api/i18n/{locale}
```

Returns all translations for a specific locale.

**Parameters:**
- `locale` - Language code (e.g., `en`, `ru`, `he`)

**Response:**
```json
{
  "nav": {
    "home": "Home",
    "about": "About",
    "contact": "Contact"
  },
  "hero": {
    "title": "Welcome",
    "subtitle": "..."
  }
}
```

---

### Contact Form

```
POST /api/contact/submit
```

Submit a contact form. Rate limited: 5 per hour per IP.

**Body:**
```json
{
  "name": "John Doe",
  "email": "john@example.com",
  "message": "Hello, I'd like to inquire about..."
}
```

**Response:**
```json
{
  "success": true,
  "message": "Message sent successfully"
}
```

---

### Portfolios (Public)

```
GET /api/portfolios
```

Returns published portfolios.

**Query Parameters:**
- `type` - Filter by type: `gallery` | `case_study`

**Response:**
```json
[
  {
    "id": 1,
    "slug": "web-projects",
    "name": "Web Projects",
    "description": "...",
    "type": "gallery",
    "coverImageUrl": "/uploads/cover.webp",
    "coverThumbnailUrl": "/uploads/cover-thumb.webp"
  }
]
```

---

```
GET /api/portfolios/{id}
```

Returns single portfolio with items.

**Response:**
```json
{
  "id": 1,
  "slug": "web-projects",
  "name": "Web Projects",
  "type": "gallery",
  "items": [
    {
      "id": 1,
      "itemType": "image",
      "mediaUrl": "/uploads/image.webp",
      "thumbnailUrl": "/uploads/image-thumb.webp",
      "caption": "Project screenshot"
    }
  ]
}
```

---

## Authentication Endpoints

### Login

```
POST /api/auth/login
```

**Body:**
```json
{
  "email": "user@example.com",
  "password": "password123",
  "rememberMe": false
}
```

**Response (2FA disabled):**
```json
{
  "success": true,
  "user": {
    "id": 1,
    "email": "user@example.com",
    "name": "User Name",
    "role": "admin"
  },
  "csrfToken": "abc123..."
}
```

**Response (2FA enabled):**
```json
{
  "success": true,
  "requires2fa": true,
  "message": "Two-factor authentication required",
  "csrfToken": "abc123..."
}
```

When 2FA is required, the user must call `/api/user/2fa/verify` with their TOTP code.

**Cookies Set:**
- `pm-session` - Session ID (httpOnly, secure) — only when 2FA disabled or after 2FA verification
- `pm-csrf` - CSRF token (httpOnly, secure)
- `pm-2fa-pending` - Pending 2FA token (httpOnly, secure) — only when 2FA required

---

### Logout

```
POST /api/auth/logout
```

Clears session and cookies.

**Response:**
```json
{
  "success": true
}
```

---

### Get Session

```
GET /api/auth/session
```

Returns current session and CSRF token.

**Response:**
```json
{
  "user": {
    "id": 1,
    "email": "user@example.com",
    "name": "User Name",
    "role": "admin"
  },
  "csrfToken": "abc123..."
}
```

---

## User Endpoints

### Change Password

```
PUT /api/user/change-password
```

**Requires:** Authentication

**Body:**
```json
{
  "currentPassword": "old-password",
  "newPassword": "new-password"
}
```

**Response:**
```json
{
  "success": true
}
```

---

## Two-Factor Authentication Endpoints

All 2FA endpoints require authentication except `/api/user/2fa/verify` (which requires a pending 2FA session).

### 2FA Status

```
GET /api/user/2fa/status
```

**Requires:** Authentication

**Response:**
```json
{
  "enabled": false
}
```

---

### Setup 2FA

```
POST /api/user/2fa/setup
```

**Requires:** Authentication

Generates a new TOTP secret and returns QR code for authenticator app setup. Does NOT enable 2FA — user must verify with `/enable` endpoint.

**Response:**
```json
{
  "qrCode": "data:image/png;base64,...",
  "uri": "otpauth://totp/PuppetMaster:user@example.com?secret=...",
  "backupCodes": ["ABC123", "DEF456", "..."]
}
```

**Note:** Backup codes are only shown once during setup. Rate limited: 5 requests per 15 minutes.

---

### Enable 2FA

```
POST /api/user/2fa/enable
```

**Requires:** Authentication + pending setup

**Body:**
```json
{
  "code": "123456"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Two-factor authentication enabled"
}
```

Verifies the TOTP code and enables 2FA for the user.

---

### Disable 2FA

```
POST /api/user/2fa/disable
```

**Requires:** Authentication

**Body:**
```json
{
  "code": "123456"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Two-factor authentication disabled"
}
```

---

### Verify 2FA (Login)

```
POST /api/user/2fa/verify
```

**Requires:** Pending 2FA session (after login with 2FA enabled)

**Body:**
```json
{
  "code": "123456"
}
```

Or with backup code:
```json
{
  "code": "ABC123",
  "isBackupCode": true
}
```

**Response:**
```json
{
  "success": true,
  "user": {
    "id": 1,
    "email": "user@example.com",
    "name": "User Name",
    "role": "admin"
  },
  "csrfToken": "abc123..."
}
```

---

## Admin Endpoints

All admin endpoints require authentication and appropriate role.

### Statistics

```
GET /api/admin/stats
```

**Requires:** Editor+

**Response:**
```json
{
  "portfolioItems": 5,
  "contactSubmissions": 100,
  "unreadMessages": 3
}
```

---

### Settings

```
PUT /api/admin/settings
```

**Requires:** Editor+

**Body:**
```json
{
  "contact.email": "new@example.com",
  "social.instagram": "https://instagram.com/new"
}
```

**Response:**
```json
{
  "success": true
}
```

---

### Contacts

```
GET /api/admin/contacts
```

**Requires:** Editor+

**Query:**
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20, max: 100)

**Response:**
```json
{
  "items": [
    {
      "id": 1,
      "name": "John Doe",
      "email": "john@example.com",
      "message": "...",
      "read": false,
      "createdAt": 1705315800000
    }
  ],
  "unreadCount": 3
}
```

---

```
PUT /api/admin/contacts/{id}
```

Update contact (mark as read).

**Body:**
```json
{
  "read": true
}
```

---

```
DELETE /api/admin/contacts/{id}
```

Delete contact submission.

---

### Users

```
GET /api/admin/users
```

**Requires:** Admin+

**Response:**
```json
{
  "users": [
    {
      "id": 1,
      "email": "user@example.com",
      "name": "User Name",
      "role": "admin",
      "createdAt": "2024-01-01T00:00:00.000Z"
    }
  ]
}
```

---

```
POST /api/admin/users
```

**Requires:** Admin+

**Body:**
```json
{
  "email": "new@example.com",
  "password": "password123",
  "name": "New User",
  "role": "editor"
}
```

**Response:**
```json
{
  "success": true,
  "id": 5
}
```

---

```
PUT /api/admin/users/{id}
```

**Requires:** Admin+

**Body:**
```json
{
  "name": "Updated Name",
  "role": "admin"
}
```

---

```
DELETE /api/admin/users/{id}
```

**Requires:** Admin+

---

### Translations

```
GET /api/admin/translations
```

**Requires:** Editor+

**Response:**
```json
{
  "locales": ["en", "ru", "he"],
  "content": {
    "en": [
      { "id": 1, "key": "hero.title", "value": "Welcome" }
    ]
  },
  "system": {
    "en": [
      { "id": 100, "key": "nav.home", "value": "Home" }
    ]
  },
  "canEditSystem": true
}
```

---

```
POST /api/admin/translations
```

Create or update translation.

**Body:**
```json
{
  "locale": "en",
  "key": "hero.title",
  "value": "New Title"
}
```

**Response:**
```json
{
  "success": true,
  "action": "updated",
  "id": 1
}
```

---

```
DELETE /api/admin/translations/{id}
```

Delete translation.

---

### Logs (Master Only)

```
GET /api/admin/logs
```

**Requires:** Master

Returns in-memory application logs (last 500 entries).

---

```
GET /api/admin/audit-logs
```

**Requires:** Master

**Query:**
- `limit` - Max entries (default: 50, max: 200)
- `offset` - Pagination offset
- `action` - Filter by action type
- `userId` - Filter by actor

**Response:**
```json
{
  "logs": [
    {
      "id": 1,
      "action": "login",
      "userId": 1,
      "ipAddress": "192.168.1.1",
      "userAgent": "Mozilla/5.0...",
      "success": true,
      "createdAt": "2024-01-15T10:30:00.000Z"
    }
  ],
  "total": 100
}
```

---

### Health (Master Only)

```
GET /api/admin/health
```

**Requires:** Master

Extended health information including memory, CPU, database stats.

---

## Portfolio Management

### List Portfolios (Admin)

```
GET /api/portfolios
```

When authenticated, returns all portfolios including unpublished.

---

### Create Portfolio

```
POST /api/portfolios
```

**Requires:** Editor+

**Body:**
```json
{
  "name": "New Portfolio",
  "slug": "new-portfolio",
  "description": "Description",
  "type": "gallery",
  "published": false
}
```

---

### Update Portfolio

```
PUT /api/portfolios/{id}
```

**Requires:** Editor+

**Body:**
```json
{
  "name": "Updated Name",
  "published": true
}
```

---

### Delete Portfolio

```
DELETE /api/portfolios/{id}
```

**Requires:** Editor+

---

### Portfolio Items

```
GET /api/portfolios/{id}/items
```

**Query:**
- `type` - Filter by item type

---

```
POST /api/portfolios/{id}/items
```

**Body (Gallery Item):**
```json
{
  "itemType": "image",
  "mediaUrl": "/uploads/image.webp",
  "thumbnailUrl": "/uploads/thumb.webp",
  "caption": "Image caption"
}
```

**Body (Case Study):**
```json
{
  "itemType": "case_study",
  "slug": "project-name",
  "title": "Project Name",
  "description": "Short description",
  "content": "Full markdown content...",
  "category": "Web Design",
  "tags": "design, web, responsive"
}
```

---

```
PUT /api/portfolios/{id}/items/{itemId}
```

Update portfolio item.

---

```
DELETE /api/portfolios/{id}/items/{itemId}
```

Delete portfolio item.

---

```
POST /api/portfolios/{id}/items/reorder
```

Reorder portfolio items.

**Body:**
```json
{
  "items": [
    { "id": 3, "order": 0 },
    { "id": 1, "order": 1 },
    { "id": 2, "order": 2 }
  ]
}
```

---

## File Upload

### Upload Image

```
POST /api/upload/image
```

**Requires:** Authentication

**Body:** `multipart/form-data`
- `image` - Image file (max 10MB, validated by magic bytes)

**Response:**
```json
{
  "success": true,
  "url": "/uploads/image-1705315800.webp",
  "thumbnailUrl": "/uploads/image-1705315800-thumb.webp"
}
```

**Note:** Files are validated by magic bytes (not MIME type) and scanned for viruses.

---

### Upload Video

```
POST /api/upload/video
```

**Requires:** Editor+

**Body:** `multipart/form-data`
- `file` - Video file (max 100MB, max 5 minutes)

**Response:**
```json
{
  "url": "/uploads/video-1705315800.mp4",
  "thumbnailUrl": "/uploads/video-1705315800-thumb.webp"
}
```

---

## WebSocket

```
WS /api/ws
```

WebSocket endpoint for real-time communication. See [WebSocket Reference](websocket.md) for details.

---

## Pagination

Paginated endpoints accept:
- `page` - Page number (1-indexed)
- `limit` - Items per page (max 100)

Response includes:
```json
{
  "items": [...],
  "meta": {
    "total": 100,
    "page": 1,
    "limit": 20,
    "totalPages": 5,
    "hasNextPage": true,
    "hasPrevPage": false
  }
}
```

---

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/api/auth/login` | 5 attempts | 15 minutes |
| `/api/contact/submit` | 5 submissions | 60 minutes |
| `/api/user/2fa/setup` | 5 requests | 15 minutes |

---

## Setup Endpoints

Used by the configuration wizard at `/init`.

### Get Config

```
GET /api/setup/config
```

Returns current configuration state.

---

### Save Config

```
POST /api/setup/config
```

Saves configuration from the wizard.

---

### Import ZIP (Brownfield)

```
POST /api/setup/import-zip
```

**Body:** `multipart/form-data`
- `file` - ZIP file with existing project code

Uploads and extracts existing project code for brownfield migration.

---

### Delete Import

```
DELETE /api/setup/import-zip
```

Removes previously uploaded import files.

---

## Analytics Endpoints

### Performance Metrics

```
POST /api/analytics/performance
```

Records client-side performance metrics (Core Web Vitals).

---

## Role Management Endpoints

### List Roles

```
GET /api/admin/roles
```

**Requires:** Master

Returns custom role definitions.

---

### Create Role

```
POST /api/admin/roles
```

**Requires:** Master

Creates a new custom role with page-level permissions.

---

### Update Role

```
PUT /api/admin/roles/{id}
```

**Requires:** Master

---

### Delete Role

```
DELETE /api/admin/roles/{id}
```

**Requires:** Master

---

## Additional Content Module Endpoints

The following content modules have CRUD endpoints following the same pattern:

| Module | Public Endpoint | Admin Endpoint |
|--------|-----------------|----------------|
| Blog Posts | `GET /api/blog/posts` | `POST/PUT/DELETE /api/admin/blog/posts/*` |
| Blog Categories | `GET /api/blog/categories` | `POST/PUT/DELETE /api/admin/blog/categories/*` |
| Blog Tags | — | `GET/POST/PUT/DELETE /api/admin/blog/tags/*` |
| Team Members | `GET /api/team` | `POST/PUT/DELETE /api/admin/team/*` |
| Testimonials | `GET /api/testimonials` | `POST/PUT/DELETE /api/admin/testimonials/*` |
| Features | `GET /api/features` | `POST/PUT/DELETE /api/admin/features/*` |
| Clients | `GET /api/clients` | `POST/PUT/DELETE /api/admin/clients/*` |
| FAQ | `GET /api/faq` | `POST/PUT/DELETE /api/admin/faq/*` |
| Pricing | `GET /api/pricing` | `POST/PUT/DELETE /api/admin/pricing/*` |

Each module supports translations via `PUT /api/admin/{module}/{id}/translations`.

---

## OpenAPI Specification

The canonical API documentation is available at:
- **Swagger UI**: `/api/docs/swagger` (interactive)
- **OpenAPI JSON**: `/api/docs/openapi.json` (machine-readable)

The OpenAPI spec is defined in `server/openapi.ts`.
