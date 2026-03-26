# Security Guide

Comprehensive security documentation for Puppet Master.

## Security Architecture Overview

Puppet Master implements defense in depth with multiple security layers:

```
Request → Security Headers → Rate Limit → CSRF Check → Auth → RBAC → Handler
                                                                      ↓
                                                               Audit Logging
```

| Layer | Protection |
|-------|------------|
| Security Headers | XSS, clickjacking, MIME sniffing |
| Rate Limiting | Brute force, DoS |
| CSRF Protection | Cross-site request forgery |
| Authentication | Session-based with HTTP-only cookies |
| Authorization | Role-based access control (RBAC) |
| File Validation | Magic byte verification |
| Audit Logging | Security event tracking |

---

## Authentication

### Session-Based Auth

Puppet Master uses session-based authentication with HTTP-only cookies:

1. **Login** - User submits credentials
2. **Session Created** - Server creates session in database
3. **Cookies Set** - HTTP-only session and CSRF cookies returned
4. **Subsequent Requests** - Browser sends cookies automatically

**Cookies:**

| Cookie | Purpose | Flags |
|--------|---------|-------|
| `pm-session` | Session ID | httpOnly, secure, sameSite=strict |
| `pm-csrf` | CSRF token | httpOnly, secure, sameSite=strict |

### Session Management

- **Session Duration**: 24 hours (default), 7 days (remember me)
- **Session Storage**: SQLite database
- **Session Cleanup**: Hourly scheduled task removes expired sessions
- **Single Logout**: Clears session from database and cookies

### Password Security

- **Hashing**: bcrypt with automatic salt
- **Minimum Length**: 8 characters (enforced on creation/change)
- **Change Requires**: Current password verification

---

## Authorization (RBAC)

Three-tier role hierarchy:

```
Master (level 2) > Admin (level 1) > Editor (level 0)
```

### Role Permissions

| Permission | Editor | Admin | Master |
|------------|--------|-------|--------|
| View/edit settings | Yes | Yes | Yes |
| Manage portfolios | Yes | Yes | Yes |
| Manage translations | Yes | Yes | Yes |
| View contacts | Yes | Yes | Yes |
| Manage users | No | Yes | Yes |
| View audit logs | No | No | Yes |
| View system health | No | No | Yes |

### Role Descriptions

- **Editor**: Content editing only - portfolios, translations, settings
- **Admin**: Full client access - can manage users (except master accounts)
- **Master**: Developer access - full system access including logs and health

### Role Enforcement

Roles are checked at two levels:

1. **Middleware Level**: Routes like `/api/admin/users/*` require admin+
2. **Handler Level**: Individual endpoints check roles as needed

```typescript
// Middleware example
if (path.startsWith('/api/admin/users')) {
  if (!hasRole(user.role, 'admin')) {
    throw createError({ statusCode: 403 })
  }
}

// Handler example
if (!hasRole(user.role, 'master')) {
  throw createError({ statusCode: 403, message: 'Master access required' })
}
```

---

## CSRF Protection

Implements double-submit cookie pattern:

1. **Token Generation**: Secure random 32-byte token
2. **Cookie Storage**: Token stored in HTTP-only cookie
3. **Header Validation**: State-changing requests must include `X-CSRF-Token` header
4. **Timing-Safe Compare**: Prevents timing attacks

### Protected Methods

CSRF validation applies to:
- POST
- PUT
- DELETE
- PATCH

GET and HEAD requests are exempt (should be side-effect free).

### Client Usage

```typescript
// Get CSRF token from login response
const { csrfToken } = await login(credentials)

// Include in state-changing requests
await fetch('/api/admin/settings', {
  method: 'PUT',
  credentials: 'include',
  headers: {
    'X-CSRF-Token': csrfToken,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(data)
})
```

---

## Security Headers

All responses include security headers:

### X-Frame-Options

```
X-Frame-Options: DENY
```

Prevents clickjacking by disabling iframe embedding.

### X-Content-Type-Options

```
X-Content-Type-Options: nosniff
```

Prevents MIME type sniffing attacks.

### Referrer-Policy

```
Referrer-Policy: strict-origin-when-cross-origin
```

Controls referrer information sent with requests.

### HSTS (Production Only)

```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
```

Forces HTTPS for 1 year, includes subdomains, eligible for preload list.

### Content Security Policy

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'unsafe-inline';
  style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
  img-src 'self' data: blob: https:;
  font-src 'self' https://fonts.gstatic.com;
  connect-src 'self';
  media-src 'self' https:;
  object-src 'none';
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self';
  upgrade-insecure-requests
```

| Directive | Purpose |
|-----------|---------|
| `default-src 'self'` | Only allow same-origin by default |
| `script-src 'unsafe-inline'` | Required for Vue hydration |
| `object-src 'none'` | Disable Flash, Java plugins |
| `frame-ancestors 'none'` | Clickjacking protection |
| `upgrade-insecure-requests` | Auto-upgrade HTTP to HTTPS |

### Permissions-Policy

```
Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=(), usb=()
```

Disables access to sensitive browser features.

### Cross-Origin Policies

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Resource-Policy: same-origin
```

Isolates browsing context for additional protection.

---

## Rate Limiting

In-memory rate limiting for abuse prevention.

### Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/api/auth/login` | 5 attempts | 15 minutes |
| `/api/contact/submit` | 5 submissions | 60 minutes |

### Implementation

```typescript
// Rate limiter checks IP address
if (!loginRateLimiter.checkRateLimit(clientIp)) {
  throw createError({
    statusCode: 429,
    message: 'Too many login attempts. Try again later.'
  })
}
```

### IP Detection

Handles proxied requests:
1. `X-Forwarded-For` header (first IP)
2. `X-Real-IP` header
3. Socket remote address

---

## Account Lockout

Protects against brute force attacks at the account level.

### Configuration

| Setting | Value |
|---------|-------|
| Max failed attempts | 5 |
| Lockout duration | 30 minutes |
| Attempt reset period | 1 hour |

### Behavior

1. Failed login increments attempt counter
2. After 5 failures, account is locked for 30 minutes
3. Counter resets after 1 hour of no failures
4. Successful login resets all counters

### Lockout Response

```json
{
  "statusCode": 423,
  "message": "Account locked. Try again in 28 minutes."
}
```

### Admin Unlock

Master/Admin users can manually unlock accounts via the admin panel.

---

## File Upload Security

### Magic Byte Validation

Never trusts client-provided MIME types. Validates files by checking magic bytes (file signatures).

**Supported Image Types:**
- JPEG (`FF D8 FF`)
- PNG (`89 50 4E 47`)
- GIF (`47 49 46 38`)
- WebP (`52 49 46 46` + `57 45 42 50`)

**Supported Video Types:**
- MP4 (`66 74 79 70` at offset 4)
- WebM (`1A 45 DF A3`)
- MOV (`66 74 79 70 71 74` at offset 4)
- AVI (`52 49 46 46` + `41 56 49 20`)

### Size Limits

| Type | Max Size | Max Duration |
|------|----------|--------------|
| Image | 10 MB | N/A |
| Video | 100 MB | 5 minutes |

### Processing

- **Images**: Converted to WebP, resized, thumbnails generated
- **Videos**: Compressed with FFmpeg (if available)

---

## Audit Logging

Security events are logged to the database for forensic analysis.

### Logged Events

| Event | Description |
|-------|-------------|
| `login` | Successful login |
| `login_failed` | Failed login attempt |
| `logout` | User logout |
| `password_change` | Password changed |
| `role_change` | User role modified |
| `user_create` | New user created |
| `user_update` | User details modified |
| `user_delete` | User deleted |
| `account_locked` | Account locked due to failed attempts |
| `account_unlocked` | Account manually unlocked |
| `session_expired` | Session timed out |

### Log Entry Fields

```typescript
{
  action: 'login_failed',
  userId: null,           // Actor (null if not logged in)
  targetUserId: 5,        // Affected user
  ipAddress: '192.168.1.1',
  userAgent: 'Mozilla/5.0...',
  details: { email: 'user@example.com', reason: 'invalid_password' },
  success: false,
  createdAt: '2024-01-15T10:30:00.000Z'
}
```

### Accessing Logs

Only Master role can access audit logs:

```
GET /api/admin/audit-logs?limit=50&action=login_failed
```

---

## Production Checklist

### Required Before Deployment

- [ ] **Change default credentials** - All seeded users have default passwords
- [ ] **Set NODE_ENV=production** - Enables HSTS, tighter CSP
- [ ] **Use HTTPS** - Required for secure cookies, HSTS
- [ ] **Secure .env file** - Never commit, restrict file permissions
- [ ] **Configure real SMTP** - For password reset emails
- [ ] **Set strong session secret** - If using external session store

### Recommended

- [ ] **Enable WAF** - Web Application Firewall at edge
- [ ] **Configure backup** - Regular database backups
- [ ] **Set up monitoring** - Uptime Kuma or similar
- [ ] **Enable access logs** - Nginx/Traefik access logs
- [ ] **Review audit logs** - Regular security review

### Database Security

- [ ] **Restrict file permissions** - SQLite file should be 600
- [ ] **Place outside web root** - Default `./data/` is gitignored
- [ ] **Regular backups** - Automated backup strategy

### Network Security

- [ ] **Firewall rules** - Only expose ports 80, 443
- [ ] **Disable SSH password auth** - Use key-based only
- [ ] **Keep dependencies updated** - Regular `pnpm update`

---

## Security Headers Testing

Use these tools to verify security headers:

1. **Mozilla Observatory**: https://observatory.mozilla.org
2. **Security Headers**: https://securityheaders.com
3. **CSP Evaluator**: https://csp-evaluator.withgoogle.com

### Expected Scores

| Tool | Expected Grade |
|------|----------------|
| Mozilla Observatory | A+ |
| Security Headers | A |

---

## Reporting Vulnerabilities

If you discover a security vulnerability:

1. **Do not** open a public issue
2. Email security concerns to the project maintainers
3. Include detailed reproduction steps
4. Allow time for a fix before public disclosure

---

## Security Architecture Decisions

### Why Session-Based Over JWT?

1. **Revocability** - Sessions can be invalidated immediately
2. **Server Control** - No token refresh complexity
3. **Smaller Payload** - Session ID vs full JWT
4. **No Client Storage** - HTTP-only cookies are safer

### Why SQLite for Sessions?

1. **Simplicity** - No Redis dependency for small deployments
2. **Persistence** - Sessions survive server restarts
3. **Cleanup** - Easy scheduled cleanup via SQL
4. **Migration Path** - Can move to Redis for scale

### Why In-Memory Rate Limiting?

1. **Zero Dependencies** - No Redis required
2. **Sufficient** - Works for single-instance deployments
3. **Migration Path** - Can add Redis for multi-instance
