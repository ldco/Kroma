# /pm-fastapi — FastAPI/Integration Specialist Review (6 Experts)

Runs a focused external API integration review using all 6 FastAPI specialists from different countries.

## Usage

```
/pm-fastapi                      # Review uncommitted changes
/pm-fastapi {file or feature}    # Review specific code
/pm-fastapi --deep               # Deep thinking mode
```

## FastAPI Team (6 Experts)

| Country | Expert | Focus Area |
|---------|--------|------------|
| IL | Eyal Shapiro (אייל שפירא) | External API design, webhooks |
| RU | Ivan Smirnov (Иван Смирнов) | High-performance integrations |
| US | David Kim | REST/GraphQL, API versioning |
| FR | Antoine Lefèvre | API contracts, documentation |
| JP | Hiroshi Suzuki (鈴木博) | Efficient integrations, rate limiting |
| CH | Chen Ming (陈明) | Payment gateways, WeChat/Alipay |

## Algorithm

### 1. Load All FastAPI Roles

```
for country in [il, ru, us, fr, jp, ch]:
  load .claude/roles/pm/fastapi-{country}.md
```

### 2. Review from Each Perspective

Each expert reviews focusing on:
- External API client implementation
- Error handling and retry logic
- Rate limiting and circuit breakers
- Authentication flows (OAuth, JWT)
- Regional payment integrations
- API documentation

### 3. Generate FastAPI Report

## Output Format

```
🐍 PM FASTAPI TEAM REVIEW: {target}

6 FastAPI Experts • 6 Countries • 1 Specialty

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🇮🇱 Eyal Shapiro (Israel)

**Focus:** External API design, webhooks

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Repeat for all 6 experts]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## FastAPI Consensus

### Integration Issues
- {issue}

### Error Handling Issues
- {issue}

### Regional Integrations

| Region | Service | Status | Notes |
|--------|---------|--------|-------|
| Global | Stripe | ✅/⚠️/❌ | {notes} |
| CH | WeChat Pay | ✅/⚠️/❌ | {notes} |
| CH | Alipay | ✅/⚠️/❌ | {notes} |
| RU | YooKassa | ✅/⚠️/❌ | {notes} |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical
1. {fix}

### Important
1. {fix}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## OVERALL FASTAPI VERDICT: {✅/⚠️/❌}
```

## Related Commands

| Command | Experts | Focus |
|---------|---------|-------|
| `/pm-fastapi` | 6 FastAPI | External APIs |
| `/pm-backend` | 6 Backend | Internal APIs |
| `/pm-security` | 6 Security | API security |
