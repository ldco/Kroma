# /pm-ch — PM China Team Review

Runs a review using all Chinese PM experts across specialties.

## Usage

```
/pm-ch                        # Review uncommitted changes with Chinese team
/pm-ch {file or feature}      # Review specific code with Chinese team
/pm-ch --deep                # Deep analysis mode
```

## Algorithm

### 1. Load Chinese Team Members (7 experts)

Load Chinese expert roles from `.qwen/roles/pm/`:

- `ux-ch.md` → Li Wei (UX/UI Designer)
- `fullstack-ch.md` → Zhang Chen (Fullstack Developer)
- `frontend-ch.md` → Wang Mei (Frontend Developer)
- `backend-ch.md` → Liu Yang (Backend Developer)
- `fastapi-ch.md` → Chen Ming (FastAPI Developer)
- `devops-ch.md` → Zhao Feng (DevOps/SRE)
- `security-ch.md` → Huang Lin (Security Engineer)

### 2. Apply Each Chinese Perspective

Review the code from each Chinese expert's perspective:
- Apply their specialty focus
- Consider Chinese market context (mobile-first, scale, regulatory compliance)
- Use their review checklist
- Search in Chinese as appropriate

### 3. Generate Chinese Team Report

## Output Format

```
🇨🇳 PM China Team Review: {target}

Chinese Team (7 experts):
- 🎯 Li Wei (UX/UI Designer)
- 🧙 Zhang Chen (Fullstack Developer)
- 🎨 Wang Mei (Frontend Developer)
- ⚙️ Liu Yang (Backend Developer)
- 🐍 Chen Ming (FastAPI Developer)
- 🚀 Zhao Feng (DevOps/SRE)
- 🔒 Huang Lin (Security Engineer)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎯 Li Wei (UX - China)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... repeat for each Chinese specialist ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Specialty | Verdict | Issues |
|--------|-----------|---------|--------|
| Li Wei | UX | ✅ | 0 |
| Zhang Chen | Fullstack | ⚠️ | 2 |
| ... | ... | ... | ... |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Chinese Market Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Chinese Market Context

- Mobile-first: Mobile-optimized experiences
- Scale focus: Designed for large-scale applications
- Regulatory compliance: Adherence to local regulations
- Local integrations: Support for Chinese services
- Performance: Optimized for local infrastructure

## Notes

- Provides Chinese market perspective
- All specialties covered with Chinese context
- Mobile-first and scale focus
- Regulatory compliance and local integration considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-ch --deep
```

This applies deep analysis mode:
- More thorough analysis
- Deeper reasoning about Chinese market trade-offs
- Extended deliberation on scale and regulatory issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-ch` | 7 Chinese | Chinese market focus |
| `/pm-team-all` | All 42 | Major releases |