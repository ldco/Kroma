# /pm-jp — PM Japan Team Review

Runs a review using all Japanese PM experts across specialties.

## Usage

```
/pm-jp                        # Review uncommitted changes with Japanese team
/pm-jp {file or feature}      # Review specific code with Japanese team
/pm-jp --deep                # Deep analysis mode
```

## Algorithm

### 1. Load Japanese Team Members (7 experts)

Load Japanese expert roles from `.qwen/roles/pm/`:

- `ux-jp.md` → Yuki Tanaka (UX/UI Designer)
- `fullstack-jp.md` → Kenji Yamamoto (Fullstack Developer)
- `frontend-jp.md` → Sakura Tanaka (Frontend Developer)
- `backend-jp.md` → Takeshi Nakamura (Backend Developer)
- `fastapi-jp.md` → Hiroshi Tanaka (FastAPI Developer)
- `devops-jp.md` → Ryo Sato (DevOps/SRE)
- `security-jp.md` → Akiko Tanaka (Security Engineer)

### 2. Apply Each Japanese Perspective

Review the code from each Japanese expert's perspective:
- Apply their specialty focus
- Consider Japanese market context (quality, polish, clear guidance)
- Use their review checklist
- Search in Japanese as appropriate

### 3. Generate Japanese Team Report

## Output Format

```
🇯🇵 PM Japan Team Review: {target}

Japanese Team (7 experts):
- 🎯 Yuki Tanaka (UX/UI Designer)
- 🧙 Kenji Yamamoto (Fullstack Developer)
- 🎨 Sakura Tanaka (Frontend Developer)
- ⚙️ Takeshi Nakamura (Backend Developer)
- 🐍 Hiroshi Tanaka (FastAPI Developer)
- 🚀 Ryo Sato (DevOps/SRE)
- 🔒 Akiko Tanaka (Security Engineer)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎯 Yuki Tanaka (UX - Japan)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... repeat for each Japanese specialist ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Specialty | Verdict | Issues |
|--------|-----------|---------|--------|
| Yuki Tanaka | UX | ✅ | 0 |
| Kenji Yamamoto | Fullstack | ⚠️ | 2 |
| ... | ... | ... | ... |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Japanese Market Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Japanese Market Context

- Quality focus: High expectations for quality and polish
- Clear guidance: Well-designed onboarding and guidance
- Careful spacing: Attention to spacing and alignment
- Business formality: Professional and respectful interfaces
- Polished experience: Attention to detail and refinement

## Notes

- Provides Japanese market perspective
- All specialties covered with Japanese context
- Quality and polish focus
- Clear guidance and careful spacing considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-jp --deep
```

This applies deep analysis mode:
- More thorough analysis
- Deeper reasoning about Japanese market trade-offs
- Extended deliberation on quality and polish issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-jp` | 7 Japanese | Japanese market focus |
| `/pm-team-all` | All 42 | Major releases |