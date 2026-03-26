# /pm-fr — PM France Team Review

Runs a review using all French PM experts across specialties.

## Usage

```
/pm-fr                        # Review uncommitted changes with French team
/pm-fr {file or feature}      # Review specific code with French team
/pm-fr --deep                # Deep analysis mode
```

## Algorithm

### 1. Load French Team Members (7 experts)

Load French expert roles from `.qwen/roles/pm/`:

- `ux-fr.md` → Marie Dubois (UX/UI Designer)
- `fullstack-fr.md` → Pierre Martin (Fullstack Developer)
- `frontend-fr.md` → Sophie Bernard (Frontend Developer)
- `backend-fr.md` → Jean-Luc Dubois (Backend Developer)
- `fastapi-fr.md` → Antoine Moreau (FastAPI Developer)
- `devops-fr.md` → Nicolas Petit (DevOps/SRE)
- `security-fr.md` → Camille Dubois (Security Engineer)

### 2. Apply Each French Perspective

Review the code from each French expert's perspective:
- Apply their specialty focus
- Consider French market context (GDPR, privacy, refined typography)
- Use their review checklist
- Search in French as appropriate

### 3. Generate French Team Report

## Output Format

```
🇫🇷 PM France Team Review: {target}

French Team (7 experts):
- 🎯 Marie Dubois (UX/UI Designer)
- 🧙 Pierre Martin (Fullstack Developer)
- 🎨 Sophie Bernard (Frontend Developer)
- ⚙️ Jean-Luc Dubois (Backend Developer)
- 🐍 Antoine Moreau (FastAPI Developer)
- 🚀 Nicolas Petit (DevOps/SRE)
- 🔒 Camille Dubois (Security Engineer)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎯 Marie Dubois (UX - France)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... repeat for each French specialist ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Specialty | Verdict | Issues |
|--------|-----------|---------|--------|
| Marie Dubois | UX | ✅ | 0 |
| Pierre Martin | Fullstack | ⚠️ | 2 |
| ... | ... | ... | ... |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### French Market Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## French Market Context

- GDPR compliance: Privacy and data protection
- Refined typography: Attention to typography and spacing
- Formal business: Professional and formal interfaces
- European standards: EU regulation compliance
- Privacy focus: Data protection and user privacy

## Notes

- Provides French market perspective
- All specialties covered with French context
- GDPR and privacy compliance focus
- Refined typography and spacing considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-fr --deep
```

This applies deep analysis mode:
- More thorough analysis
- Deeper reasoning about French market trade-offs
- Extended deliberation on GDPR and privacy issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-fr` | 7 French | French market focus |
| `/pm-team-all` | All 42 | Major releases |