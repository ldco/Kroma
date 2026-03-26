# /pm-fullstack — Fullstack Specialist Review (6 Experts)

Runs a focused fullstack architecture review using all 6 fullstack specialists from different countries.

## Usage

```
/pm-fullstack                      # Review uncommitted changes
/pm-fullstack {file or feature}    # Review specific code
/pm-fullstack --deep               # Deep thinking mode
```

## Fullstack Team (6 Experts)

| Country | Expert | Focus Area |
|---------|--------|------------|
| IL | Yonatan Cohen (יונתן כהן) | Nuxt architecture, full-stack patterns |
| RU | Alexei Volkov (Алексей Волков) | Performance optimization, SSR |
| US | Jake Thompson | Modern web architecture, DX |
| FR | Pierre Martin | Clean architecture, European standards |
| JP | Kenji Yamamoto (山本健二) | Efficiency, minimal footprint |
| CH | Zhang Chen (张晨) | Scalability, multi-tenant |

## Algorithm

### 1. Load All Fullstack Roles

```
for country in [il, ru, us, fr, jp, ch]:
  load .claude/roles/pm/fullstack-{country}.md
```

### 2. Review from Each Perspective

Each expert reviews focusing on:
- Overall architecture coherence
- Frontend-backend integration
- Data flow patterns
- Code organization
- Performance considerations
- Regional best practices

### 3. Generate Fullstack Report

## Output Format

```
🧙 PM FULLSTACK TEAM REVIEW: {target}

6 Fullstack Experts • 6 Countries • 1 Specialty

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🇮🇱 Yonatan Cohen (Israel)

**Focus:** Nuxt architecture, full-stack patterns

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Repeat for all 6 experts]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Fullstack Consensus

### Architecture Issues
- {issue}

### Integration Issues
- {issue}

### Performance Issues
- {issue}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical
1. {fix}

### Important
1. {fix}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## OVERALL FULLSTACK VERDICT: {✅/⚠️/❌}
```

## Related Commands

| Command | Experts | Focus |
|---------|---------|-------|
| `/pm-fullstack` | 6 Fullstack | Architecture |
| `/pm-frontend` | 6 Frontend | UI components |
| `/pm-backend` | 6 Backend | API/DB |
