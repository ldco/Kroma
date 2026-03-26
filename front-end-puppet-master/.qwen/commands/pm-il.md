# /pm-il — PM Israel Team Review

Runs a review using all Israeli PM experts across specialties.

## Usage

```
/pm-il                        # Review uncommitted changes with Israeli team
/pm-il {file or feature}      # Review specific code with Israeli team
/pm-il --deep                # Deep analysis mode
```

## Algorithm

### 1. Load Israeli Team Members (7 experts)

Load Israeli expert roles from `.qwen/roles/pm/`:

- `ux-il.md` → Noa Levi (UX/UI Designer)
- `fullstack-il.md` → Yonatan Cohen (Fullstack Developer)
- `frontend-il.md` → Shira Goldstein (Frontend Developer)
- `backend-il.md` → Avi Goldstein (Backend Developer)
- `fastapi-il.md` → Eyal Ben-David (FastAPI Developer)
- `devops-il.md` → Oren Levi (DevOps/SRE)
- `security-il.md` → Maya Rosen (Security Engineer)

### 2. Apply Each Israeli Perspective

Review the code from each Israeli expert's perspective:
- Apply their specialty focus
- Consider Israeli market context (startup culture, security focus, mobile-first)
- Use their review checklist
- Search in Hebrew/English as appropriate

### 3. Generate Israeli Team Report

## Output Format

```
🇮🇱 PM Israel Team Review: {target}

Israeli Team (7 experts):
- 🎯 Noa Levi (UX/UI Designer)
- 🧙 Yonatan Cohen (Fullstack Developer)
- 🎨 Shira Goldstein (Frontend Developer)
- ⚙️ Avi Goldstein (Backend Developer)
- 🐍 Eyal Ben-David (FastAPI Developer)
- 🚀 Oren Levi (DevOps/SRE)
- 🔒 Maya Rosen (Security Engineer)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎯 Noa Levi (UX - Israel)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... repeat for each Israeli specialist ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Specialty | Verdict | Issues |
|--------|-----------|---------|--------|
| Noa Levi | UX | ✅ | 0 |
| Yonatan Cohen | Fullstack | ⚠️ | 2 |
| ... | ... | ... | ... |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Israeli Market Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Israeli Market Context

- Startup culture: Rapid iteration and MVP mindset
- Security-conscious: High security requirements
- Mobile-first: Mobile market focus
- Direct communication: Clear, actionable feedback
- Efficiency: Optimized for performance

## Notes

- Provides Israeli market perspective
- All specialties covered with Israeli context
- Security and performance focus
- Mobile-first considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-il --deep
```

This applies deep analysis mode:
- More thorough analysis
- Deeper reasoning about Israeli market trade-offs
- Extended deliberation on security and performance issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-il` | 7 Israeli | Israeli market focus |
| `/pm-team-all` | All 42 | Major releases |