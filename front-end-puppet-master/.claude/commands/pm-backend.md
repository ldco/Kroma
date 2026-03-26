# /pm-backend — Backend Specialist Review (6 Experts)

Runs a focused backend review using all 6 backend specialists from different countries.

## Usage

```
/pm-backend                      # Review uncommitted changes
/pm-backend {file or feature}    # Review specific code
/pm-backend --deep               # Deep thinking mode
```

## Backend Team (6 Experts)

| Country | Expert | Focus Area |
|---------|--------|------------|
| IL | Avi Goldstein (אבי גולדשטיין) | Nitro/H3, real-time systems |
| RU | Viktor Kozlov (Виктор Козлов) | High-load optimization, caching |
| US | Michael Rodriguez | REST design, cloud architecture |
| FR | Jean-Luc Moreau | GDPR compliance, data privacy |
| JP | Takeshi Nakamura (中村武) | Microservices, efficiency |
| CH | Liu Yang (刘洋) | Distributed systems, scaling |

## Algorithm

### 1. Load All Backend Roles

```
for country in [il, ru, us, fr, jp, ch]:
  load .claude/roles/pm/backend-{country}.md
```

### 2. Review from Each Perspective

Each expert reviews focusing on:
- API design and consistency
- Database queries and optimization
- Error handling and logging
- Security practices
- Regional compliance (GDPR, data residency)

### 3. Generate Backend Report

## Output Format

```
⚙️ PM BACKEND TEAM REVIEW: {target}

6 Backend Experts • 6 Countries • 1 Specialty

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🇮🇱 Avi Goldstein (Israel)

**Focus:** Nitro/H3, real-time systems

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Repeat for all 6 experts]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Backend Consensus

### API Issues
- {issue}

### Performance Issues
- {issue}

### Security Issues
- {issue}

### Compliance Issues
| Region | Regulation | Status |
|--------|------------|--------|
| FR/EU | GDPR | ✅/⚠️/❌ |
| US | CCPA | ✅/⚠️/❌ |
| CH | PIPL | ✅/⚠️/❌ |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical
1. {fix}

### Important
1. {fix}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## OVERALL BACKEND VERDICT: {✅/⚠️/❌}
```

## Related Commands

| Command | Experts | Focus |
|---------|---------|-------|
| `/pm-backend` | 6 Backend | API/DB |
| `/pm-fastapi` | 6 FastAPI | External integrations |
| `/pm-security` | 6 Security | Security audit |
