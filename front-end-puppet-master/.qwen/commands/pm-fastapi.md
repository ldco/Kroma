# /pm-fastapi — PM FastAPI Developer Review

Runs a review using all FastAPI Developer PM experts across countries.

## Usage

```
/pm-fastapi                  # Review uncommitted changes with FastAPI team
/pm-fastapi {file or feature} # Review specific code with FastAPI team
/pm-fastapi --deep          # Deep analysis mode
```

## Algorithm

### 1. Load FastAPI Developer Team Members (6 experts)

Load FastAPI expert roles from `.qwen/roles/pm/`:

- `fastapi-il.md` → Eyal Ben-David (Israel)
- `fastapi-ru.md` → Ivan Smirnov (Russia)
- `fastapi-us.md` → David Kim (USA)
- `fastapi-fr.md` → Antoine Moreau (France)
- `fastapi-jp.md` → Hiroshi Tanaka (Japan)
- `fastapi-ch.md` → Chen Ming (China)

### 2. Apply Each FastAPI Perspective

Review the code from each FastAPI expert's perspective:
- Apply their Python FastAPI specialty focus
- Consider their cultural/market context
- Use their FastAPI review checklist
- Focus on Python services, integrations, and data processing

### 3. Generate FastAPI Team Report

## Output Format

```
🐍 PM FastAPI Developer Review: {target}

FastAPI Team (6 experts):
- 🇮🇱 Eyal Ben-David (Israel)
- 🇷🇺 Ivan Smirnov (Russia)
- 🇺🇸 David Kim (USA)
- 🇫🇷 Antoine Moreau (France)
- 🇯🇵 Hiroshi Tanaka (Japan)
- 🇨🇳 Chen Ming (China)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Eyal Ben-David (FastAPI - Israel)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Israeli FastAPI Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Ivan Smirnov (FastAPI - Russia)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Russian FastAPI Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### David Kim (FastAPI - USA)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**US FastAPI Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Antoine Moreau (FastAPI - France)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**French FastAPI Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Hiroshi Tanaka (FastAPI - Japan)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Japanese FastAPI Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Chen Ming (FastAPI - China)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Chinese FastAPI Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Country | Verdict | Issues |
|--------|---------|---------|--------|
| Eyal Ben-David | IL | ✅ | 0 |
| Ivan Smirnov | RU | ⚠️ | 2 |
| David Kim | US | ✅ | 1 |
| Antoine Moreau | FR | ⚠️ | 3 |
| Hiroshi Tanaka | JP | ✅ | 0 |
| Chen Ming | CH | ⚠️ | 1 |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### FastAPI Best Practices
- {best practice}

### Cross-Cultural FastAPI Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## FastAPI Developer Focus Areas

- FastAPI routers and dependency injection
- Pydantic models and validation
- Background tasks and workers
- External APIs and data pipelines
- Inter-service authentication
- API schemas and contracts
- Data validation on inbound/outbound requests
- Auth and signing for service calls
- Integration points with Nuxt
- Error handling and timeouts

## Cultural Context Considerations

- **Israel**: Startup efficiency, security-conscious, mobile-first
- **Russia**: Performance, scalability, systematic approach
- **USA**: Best practices, integration, modern tooling
- **France**: GDPR compliance, refined architecture, formal business
- **Japan**: Quality, polish, clear guidance, careful integration
- **China**: Mobile-first, scale, regulatory compliance

## Notes

- Provides comprehensive FastAPI perspective across cultures
- All FastAPI specialties covered with cultural context
- Integration and service focus
- Cross-cultural FastAPI considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-fastapi --deep
```

This applies deep analysis mode:
- More thorough FastAPI analysis
- Deeper reasoning about cross-cultural FastAPI trade-offs
- Extended deliberation on integration and performance issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-fastapi` | 6 FastAPI experts | FastAPI-focused reviews |
| `/pm-team-all` | All 42 | Major releases |