# /pm-backend — PM Backend Developer Review

Runs a review using all Backend Developer PM experts across countries.

## Usage

```
/pm-backend                  # Review uncommitted changes with Backend team
/pm-backend {file or feature} # Review specific code with Backend team
/pm-backend --deep           # Deep analysis mode
```

## Algorithm

### 1. Load Backend Developer Team Members (6 experts)

Load Backend expert roles from `.qwen/roles/pm/`:

- `backend-il.md` → Avi Goldstein (Israel)
- `backend-ru.md` → Viktor Petrov (Russia)
- `backend-us.md` → Michael Rodriguez (USA)
- `backend-fr.md` → Jean-Luc Dubois (France)
- `backend-jp.md` → Takeshi Nakamura (Japan)
- `backend-ch.md` → Liu Yang (China)

### 2. Apply Each Backend Perspective

Review the code from each Backend expert's perspective:
- Apply their Nitro/H3 specialty focus
- Consider their cultural/market context
- Use their Backend review checklist
- Focus on APIs, databases, auth, and validation

### 3. Generate Backend Team Report

## Output Format

```
⚙️ PM Backend Developer Review: {target}

Backend Team (6 experts):
- 🇮🇱 Avi Goldstein (Israel)
- 🇷🇺 Viktor Petrov (Russia)
- 🇺🇸 Michael Rodriguez (USA)
- 🇫🇷 Jean-Luc Dubois (France)
- 🇯🇵 Takeshi Nakamura (Japan)
- 🇨🇳 Liu Yang (China)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Avi Goldstein (Backend - Israel)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Israeli Backend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Viktor Petrov (Backend - Russia)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Russian Backend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Michael Rodriguez (Backend - USA)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**US Backend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Jean-Luc Dubois (Backend - France)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**French Backend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Takeshi Nakamura (Backend - Japan)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Japanese Backend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Liu Yang (Backend - China)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Chinese Backend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Country | Verdict | Issues |
|--------|---------|---------|--------|
| Avi Goldstein | IL | ✅ | 0 |
| Viktor Petrov | RU | ⚠️ | 2 |
| Michael Rodriguez | US | ✅ | 1 |
| Jean-Luc Dubois | FR | ⚠️ | 3 |
| Takeshi Nakamura | JP | ✅ | 0 |
| Liu Yang | CH | ⚠️ | 1 |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Backend Best Practices
- {best practice}

### Cross-Cultural Backend Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Backend Developer Focus Areas

- H3 event handlers and middleware
- Zod validation patterns
- Drizzle ORM query patterns
- Auth flows, sessions, and audit logs
- API route conventions and error handling
- Nitro/H3 API routes
- Database access patterns
- Auth and RBAC implementation
- Validation and error handling
- API performance optimization
- Security best practices

## Cultural Context Considerations

- **Israel**: Startup efficiency, security-conscious, mobile-first
- **Russia**: Performance, scalability, dense information
- **USA**: Best practices, accessibility, modern tooling
- **France**: GDPR compliance, refined architecture, formal business
- **Japan**: Quality, polish, clear guidance, careful architecture
- **China**: Mobile-first, scale, regulatory compliance

## Notes

- Provides comprehensive Backend perspective across cultures
- All Backend specialties covered with cultural context
- API and database focus
- Cross-cultural backend considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-backend --deep
```

This applies deep analysis mode:
- More thorough Backend analysis
- Deeper reasoning about cross-cultural Backend trade-offs
- Extended deliberation on security and performance issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-backend` | 6 Backend experts | Backend-focused reviews |
| `/pm-team-all` | All 42 | Major releases |