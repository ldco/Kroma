# PuppetMaster2 - Monitoring Stack

This document explains our monitoring tool choices with analysis of alternatives.

---

## Stack Overview

| Component | Tool | Purpose |
|-----------|------|---------|
| **Uptime Monitoring** | Uptime Kuma | External availability checks |
| **Application Health** | Built-in health page | Internal system details |
| **Logging** | Pino (JSON) + in-memory buffer | Application logs |
| **Audit Trail** | SQLite audit_logs table | Security events |

---

## Tool Selection: Uptime Monitoring

### Chosen: Uptime Kuma

**Why Uptime Kuma?**

| Criterion | Score | Notes |
|-----------|-------|-------|
| RAM usage | ⭐⭐⭐⭐⭐ | ~50MB |
| Setup complexity | ⭐⭐⭐⭐⭐ | Single Docker container |
| UI quality | ⭐⭐⭐⭐⭐ | Modern, beautiful |
| Alerting | ⭐⭐⭐⭐⭐ | 90+ notification types |
| Maintenance | ⭐⭐⭐⭐⭐ | Self-contained, auto-updates |

### Alternatives Considered

#### 1. Prometheus + Grafana

| Aspect | Assessment |
|--------|------------|
| **What it does** | Metrics collection + visualization |
| **RAM required** | 500MB-1GB |
| **Best for** | Multiple services, detailed metrics |
| **Why not chosen** | Overkill for single app, heavy resource usage |

#### 2. Zabbix

| Aspect | Assessment |
|--------|------------|
| **What it does** | Enterprise monitoring for servers/networks |
| **RAM required** | 1-2GB + database |
| **Best for** | 10+ servers, enterprise compliance |
| **Why not chosen** | Way too heavy, complex setup, dated UI |

#### 3. Netdata

| Aspect | Assessment |
|--------|------------|
| **What it does** | Real-time system metrics |
| **RAM required** | 200-400MB |
| **Best for** | System-level monitoring |
| **Why not chosen** | More than needed, no uptime focus |

#### 4. UptimeRobot (SaaS)

| Aspect | Assessment |
|--------|------------|
| **What it does** | External uptime monitoring |
| **Cost** | Free tier: 50 monitors, 5-min intervals |
| **Best for** | No self-hosting wanted |
| **Why not chosen** | Third-party dependency, less control |

#### 5. Gatus

| Aspect | Assessment |
|--------|------------|
| **What it does** | Lightweight health checks |
| **RAM required** | ~30MB |
| **Best for** | Minimalists, config-file approach |
| **Why not chosen** | Less polished UI, fewer notification options |

### Decision Matrix

| Tool | RAM | UI | Alerts | Self-Hosted | Winner? |
|------|-----|-----|--------|-------------|---------|
| Uptime Kuma | 50MB | ⭐⭐⭐⭐⭐ | 90+ | ✅ | ✅ |
| Prometheus+Grafana | 500MB+ | ⭐⭐⭐⭐⭐ | Good | ✅ | ❌ Heavy |
| Zabbix | 1GB+ | ⭐⭐⭐ | Good | ✅ | ❌ Overkill |
| Netdata | 300MB | ⭐⭐⭐⭐ | Limited | ✅ | ❌ Wrong focus |
| UptimeRobot | 0 | ⭐⭐⭐⭐ | Good | ❌ | ❌ SaaS |
| Gatus | 30MB | ⭐⭐⭐ | Limited | ✅ | ❌ Less features |

---

## Tool Selection: Application Logging

### Chosen: Pino + In-Memory Buffer

**Why Pino?**

| Criterion | Score | Notes |
|-----------|-------|-------|
| Performance | ⭐⭐⭐⭐⭐ | Fastest Node.js logger |
| JSON format | ⭐⭐⭐⭐⭐ | Perfect for aggregators |
| Bundle size | ⭐⭐⭐⭐⭐ | Minimal dependencies |
| Ecosystem | ⭐⭐⭐⭐⭐ | pino-pretty, transports |

### Why In-Memory Buffer?

Reading Docker logs from Node.js is complex. Instead:

```typescript
// Ring buffer keeps last 500 log entries in RAM
// ~50KB memory for 500 entries
const LOG_BUFFER_SIZE = 500
```

| Approach | Pros | Cons |
|----------|------|------|
| **In-memory buffer** ✅ | Fast, simple, no deps | Lost on restart |
| Docker API | Access all logs | Complex, needs socket |
| File tailing | Persistent | I/O overhead |
| External (Loki) | Full history | Extra service |

### Alternatives Considered

#### 1. Winston

| Aspect | Assessment |
|--------|------------|
| **Why not** | Slower than Pino, larger bundle |

#### 2. Bunyan

| Aspect | Assessment |
|--------|------------|
| **Why not** | Less maintained, Pino is successor |

#### 3. Loki + Grafana

| Aspect | Assessment |
|--------|------------|
| **What it does** | Log aggregation with search |
| **RAM required** | 200MB+ |
| **Why not chosen** | Extra service to maintain |

---

## Tool Selection: Audit Logging

### Chosen: SQLite audit_logs Table

**Why database-backed?**

| Criterion | Score | Notes |
|-----------|-------|-------|
| Persistence | ⭐⭐⭐⭐⭐ | Survives restarts |
| Query capability | ⭐⭐⭐⭐⭐ | Filter by user, action, date |
| No extra deps | ⭐⭐⭐⭐⭐ | Uses existing SQLite |
| Compliance | ⭐⭐⭐⭐⭐ | Audit trail for security |

Already implemented in `server/utils/audit.ts`.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       VPS (2GB RAM)                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐        ┌─────────────────┐            │
│  │   Uptime Kuma   │        │   PuppetMaster  │            │
│  │   (port 3001)   │───────▶│   (port 3000)   │            │
│  │    ~50MB RAM    │ checks │   ~300MB RAM    │            │
│  └─────────────────┘        └────────┬────────┘            │
│          │                           │                      │
│          │                    ┌──────┴──────┐               │
│          │                    │             │               │
│          ▼                    ▼             ▼               │
│  ┌─────────────────┐   ┌───────────┐  ┌───────────┐        │
│  │  Status Page    │   │ SQLite DB │  │ Log Buffer│        │
│  │  (optional)     │   │ +audit_logs│ │ (in-memory)│       │
│  └─────────────────┘   └───────────┘  └───────────┘        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Health Page Integration

### Data Sources

| Section | Source | API Endpoint |
|---------|--------|--------------|
| System Status | Built-in | `GET /api/admin/health` |
| Recent Logs | In-memory buffer | `GET /api/admin/logs` |
| Audit Trail | SQLite | `GET /api/admin/audit-logs` |
| Uptime Status | Uptime Kuma | External link or embed |

### Integration Options for Uptime Kuma

| Option | Implementation | Recommendation |
|--------|----------------|----------------|
| **Link only** | Button to open Uptime Kuma | ✅ Simplest |
| **Status badge** | Embed status image | ✅ Good |
| **API integration** | Fetch from Uptime Kuma API | ⚠️ Complex |
| **iframe embed** | Embed status page | ⚠️ Security concerns |

**Chosen approach**: Link + optional status badge

---

## Deployment

### Uptime Kuma in deploy.yml

```yaml
accessories:
  uptime-kuma:
    image: louislam/uptime-kuma:1
    host: your-server-ip
    port: 3001:3001
    volumes:
      - uptime-kuma-data:/app/data
    options:
      network: traefik-public
    labels:
      traefik.http.routers.status.rule: Host(`status.${SITE_DOMAIN}`)
      traefik.http.routers.status.tls: true
      traefik.http.routers.status.tls.certresolver: letsencrypt
```

### Environment Variables

The monitoring URL is constructed from `SITE_DOMAIN` + `UPTIME_KUMA_SUBDOMAIN` (see `.env.example`):

```bash
# Domain (set once, used everywhere)
SITE_DOMAIN=example.com

# Monitoring - subdomain for Uptime Kuma status page
# Full URL: https://${UPTIME_KUMA_SUBDOMAIN}.${SITE_DOMAIN}
UPTIME_KUMA_SUBDOMAIN=status
```

---

## Uptime Kuma Setup

After deployment, configure these monitors:

| Monitor | Type | URL | Interval |
|---------|------|-----|----------|
| App Health | HTTP(s) | `https://${SITE_DOMAIN}/api/health` | 60s |
| Homepage | HTTP(s) | `https://${SITE_DOMAIN}` | 60s |
| SSL Cert | HTTP(s) | `https://${SITE_DOMAIN}` | 86400s (daily) |

### Notifications

Configure Telegram notification:
1. Create bot via @BotFather
2. Get chat ID via @userinfobot
3. Add notification in Uptime Kuma settings

---

## RAM Budget

| Component | RAM Usage |
|-----------|-----------|
| OS + Docker | 400MB |
| PuppetMaster | 300MB |
| Uptime Kuma | 50MB |
| Log Buffer | ~1MB |
| **Total** | **~750MB** |
| **Free (2GB VPS)** | **~1.2GB** |

---

## When to Reconsider

| Trigger | Solution |
|---------|----------|
| Need detailed metrics | Add Prometheus + Grafana |
| Need log search | Add Loki |
| Multiple apps to monitor | Uptime Kuma handles this well |
| Need APM (traces) | Consider SigNoz |
| Compliance requirements | Consider Zabbix |

---

## Quick Reference

| Action | Command/URL |
|--------|-------------|
| View app logs | `kamal logs -f` |
| View Uptime Kuma | `https://status.${SITE_DOMAIN}` |
| Check health API | `curl https://${SITE_DOMAIN}/api/health` |
| View audit logs | Admin → Health → Audit Trail |
