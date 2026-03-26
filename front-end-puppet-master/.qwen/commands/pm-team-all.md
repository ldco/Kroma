# /pm-team-all — PM Comprehensive Review (All 42 Experts)

Runs a comprehensive review using ALL PM experts across all specialties and countries.

## Usage

```
/pm-team-all                    # Review uncommitted changes with all 42 experts
/pm-team-all {file or feature}  # Review specific code with all 42 experts
/pm-team-all --deep             # Deep analysis mode
```

## Algorithm

### 1. Load All Team Members (42 experts total)

Load ALL expert roles from `.qwen/roles/pm/`:

**UX Designers (6):**
- `ux-il.md` → Noa Levi (Israel)
- `ux-ru.md` → Olga Petrova (Russia)
- `ux-us.md` → Sarah Johnson (USA)
- `ux-fr.md` → Marie Dubois (France)
- `ux-jp.md` → Yuki Tanaka (Japan)
- `ux-ch.md` → Li Wei (China)

**Fullstack Developers (6):**
- `fullstack-il.md` → Yonatan Cohen (Israel)
- `fullstack-ru.md` → Alexei Volkov (Russia)
- `fullstack-us.md` → Jake Thompson (USA)
- `fullstack-fr.md` → Pierre Martin (France)
- `fullstack-jp.md` → Kenji Yamamoto (Japan)
- `fullstack-ch.md` → Zhang Chen (China)

**Frontend Developers (6):**
- `frontend-il.md` → Shira Goldstein (Israel)
- `frontend-ru.md` → Marina Volkova (Russia)
- `frontend-us.md` → Emily Chen (USA)
- `frontend-fr.md` → Sophie Bernard (France)
- `frontend-jp.md` → Sakura Tanaka (Japan)
- `frontend-ch.md` → Wang Mei (China)

**Backend Developers (6):**
- `backend-il.md` → Avi Goldstein (Israel)
- `backend-ru.md` → Viktor Petrov (Russia)
- `backend-us.md` → Michael Rodriguez (USA)
- `backend-fr.md` → Jean-Luc Dubois (France)
- `backend-jp.md` → Takeshi Nakamura (Japan)
- `backend-ch.md` → Liu Yang (China)

**FastAPI Developers (6):**
- `fastapi-il.md` → Eyal Ben-David (Israel)
- `fastapi-ru.md` → Ivan Smirnov (Russia)
- `fastapi-us.md` → David Kim (USA)
- `fastapi-fr.md` → Antoine Moreau (France)
- `fastapi-jp.md` → Hiroshi Tanaka (Japan)
- `fastapi-ch.md` → Chen Ming (China)

**DevOps Engineers (6):**
- `devops-il.md` → Oren Levi (Israel)
- `devops-ru.md` → Dmitri Volkov (Russia)
- `devops-us.md` → Chris Anderson (USA)
- `devops-fr.md` → Nicolas Petit (France)
- `devops-jp.md` → Ryo Sato (Japan)
- `devops-ch.md` → Zhao Feng (China)

**Security Engineers (6):**
- `security-il.md` → Maya Rosen (Israel)
- `security-ru.md` → Yulia Sokolova (Russia)
- `security-us.md` → Jessica Williams (USA)
- `security-fr.md` → Camille Dubois (France)
- `security-jp.md` → Akiko Tanaka (Japan)
- `security-ch.md` → Huang Lin (China)

### 2. Apply Each Perspective

Review the code from each expert's perspective:
- Apply their specialty focus
- Consider their cultural/market context
- Use their review checklist
- Search in their preferred language if researching

### 3. Generate Comprehensive Team Report

## Output Format

```
👥 PM Comprehensive Team Review: {target}

Total Experts: 42
Specialties: UX, Fullstack, Frontend, Backend, FastAPI, DevOps, Security
Countries: IL, RU, US, FR, JP, CH

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎯 UX Experts (6)

#### Noa Levi (UX - Israel)
**Verdict:** ✅/⚠️/❌
**Findings:** {findings}
**Recommendations:** {recommendations}

#### Olga Petrova (UX - Russia)
**Verdict:** ✅/⚠️/❌
**Findings:** {findings}
**Recommendations:** {recommendations}

#### Sarah Johnson (UX - USA)
**Verdict:** ✅/⚠️/❌
**Findings:** {findings}
**Recommendations:** {recommendations}

#### Marie Dubois (UX - France)
**Verdict:** ✅/⚠️/❌
**Findings:** {findings}
**Recommendations:** {recommendations}

#### Yuki Tanaka (UX - Japan)
**Verdict:** ✅/⚠️/❌
**Findings:** {findings}
**Recommendations:** {recommendations}

#### Li Wei (UX - China)
**Verdict:** ✅/⚠️/❌
**Findings:** {findings}
**Recommendations:** {recommendations}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🧙 Fullstack Experts (6)

[Similar format for each fullstack expert...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎨 Frontend Experts (6)

[Similar format for each frontend expert...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### ⚙️ Backend Experts (6)

[Similar format for each backend expert...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🐍 FastAPI Experts (6)

[Similar format for each FastAPI expert...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🚀 DevOps Experts (6)

[Similar format for each DevOps expert...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🔒 Security Experts (6)

[Similar format for each Security expert...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Specialty | Country | Expert | Verdict | Issues |
|-----------|---------|--------|---------|--------|
| UX | IL | Noa Levi | ✅ | 0 |
| UX | RU | Olga Petrova | ⚠️ | 2 |
| ... | ... | ... | ... | ... |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Nice to Have
- {fix}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Notes

- Provides most comprehensive review possible
- All 42 experts apply their perspective
- All cultural and market contexts considered
- Most thorough analysis available

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-team-all --deep
```

This applies deep analysis mode:
- More thorough analysis
- Deeper reasoning about trade-offs
- Extended deliberation on complex issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-team-all` | All 42 | Major releases, critical reviews |
| `/pm-il`, `/pm-ru`, etc. | 7 per country | Regional focus |