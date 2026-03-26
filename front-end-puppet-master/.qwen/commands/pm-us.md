# /pm-us — PM USA Team Review

Runs a review using all US PM experts across specialties.

## Usage

```
/pm-us                        # Review uncommitted changes with US team
/pm-us {file or feature}      # Review specific code with US team
/pm-us --deep                # Deep analysis mode
```

## Algorithm

### 1. Load US Team Members (7 experts)

Load US expert roles from `.qwen/roles/pm/`:

- `ux-us.md` → Sarah Johnson (UX/UI Designer)
- `fullstack-us.md` → Jake Thompson (Fullstack Developer)
- `frontend-us.md` → Emily Chen (Frontend Developer)
- `backend-us.md` → Michael Rodriguez (Backend Developer)
- `fastapi-us.md` → David Kim (FastAPI Developer)
- `devops-us.md` → Chris Anderson (DevOps/SRE)
- `security-us.md` → Jessica Williams (Security Engineer)

### 2. Apply Each US Perspective

Review the code from each US expert's perspective:
- Apply their specialty focus
- Consider US market context (best practices, accessibility, SaaS focus)
- Use their review checklist
- Search in English as appropriate

### 3. Generate US Team Report

## Output Format

```
🇺🇸 PM USA Team Review: {target}

US Team (7 experts):
- 🎯 Sarah Johnson (UX/UI Designer)
- 🧙 Jake Thompson (Fullstack Developer)
- 🎨 Emily Chen (Frontend Developer)
- ⚙️ Michael Rodriguez (Backend Developer)
- 🐍 David Kim (FastAPI Developer)
- 🚀 Chris Anderson (DevOps/SRE)
- 🔒 Jessica Williams (Security Engineer)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎯 Sarah Johnson (UX - USA)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... repeat for each US specialist ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Specialty | Verdict | Issues |
|--------|-----------|---------|--------|
| Sarah Johnson | UX | ✅ | 0 |
| Jake Thompson | Fullstack | ⚠️ | 2 |
| ... | ... | ... | ... |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### US Market Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## US Market Context

- Best practices: Industry standard approaches
- Accessibility: WCAG compliance and accessibility
- SaaS focus: Software-as-a-Service patterns
- Performance: Optimized for speed and efficiency
- Modern tooling: Latest technology adoption

## Notes

- Provides US market perspective
- All specialties covered with US context
- Best practices and accessibility focus
- Modern frontend and backend tooling considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-us --deep
```

This applies deep analysis mode:
- More thorough analysis
- Deeper reasoning about US market trade-offs
- Extended deliberation on accessibility and best practices issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-us` | 7 US | US market focus |
| `/pm-team-all` | All 42 | Major releases |