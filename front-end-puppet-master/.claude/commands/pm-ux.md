# /pm-ux — UX/UI Specialist Review (6 Experts)

Runs a focused UX/UI review using all 6 UX specialists from different countries.

## Usage

```
/pm-ux                      # Review uncommitted changes
/pm-ux {file or feature}    # Review specific code
/pm-ux --deep               # Deep thinking mode
```

## UX Team (6 Experts)

| Country | Expert | Focus Area |
|---------|--------|------------|
| IL | Noa Levy (נועה לוי) | RTL layouts, Hebrew UX |
| RU | Olga Petrova (Ольга Петрова) | Cyrillic typography, Eastern European UX |
| US | Sarah Mitchell | ADA accessibility, Western UX patterns |
| FR | Marie Dubois | RGAA compliance, European design standards |
| JP | Yuki Tanaka (田中ゆき) | CJK layouts, Japanese UX conventions |
| CH | Li Wei (李伟) | WeChat/Alipay integration, Chinese UX |

## Algorithm

### 1. Load All UX Roles

```
for country in [il, ru, us, fr, jp, ch]:
  load .claude/roles/pm/ux-{country}.md
```

### 2. Review from Each Perspective

Each expert reviews focusing on:
- Their regional UX conventions
- Their accessibility standards (ADA, RGAA, etc.)
- Their language/script requirements (RTL, CJK, Cyrillic)
- Their market expectations

### 3. Generate UX Report

## Output Format

```
🎯 PM UX TEAM REVIEW: {target}

6 UX Experts • 6 Countries • 1 Specialty

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🇮🇱 Noa Levy (Israel)

**Focus:** RTL layouts, Hebrew UX

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🇷🇺 Olga Petrova (Russia)

**Focus:** Cyrillic typography

[... similar format ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Repeat for all 6 experts]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## UX Consensus

### Cross-Region Issues
- {issue affecting multiple regions}

### Region-Specific Issues
| Region | Issue | Priority |
|--------|-------|----------|
| IL | {RTL issue} | High |
| JP/CH | {CJK issue} | Medium |
| FR | {RGAA issue} | Medium |

### Accessibility Summary
| Standard | Status | Notes |
|----------|--------|-------|
| WCAG 2.1 AA | ✅/⚠️/❌ | {notes} |
| ADA (US) | ✅/⚠️/❌ | {notes} |
| RGAA (FR) | ✅/⚠️/❌ | {notes} |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (blocks markets)
1. {fix}

### Important (degrades UX)
1. {fix}

### Nice to Have
1. {enhancement}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## OVERALL UX VERDICT: {✅/⚠️/❌}
```

## When to Use

| Scenario | Command |
|----------|---------|
| UX-focused review | `/pm-ux` |
| Quick multi-specialty | `/pm-team` |
| Full team review | `/pm-team-all` |

## Related Commands

| Command | Experts | Focus |
|---------|---------|-------|
| `/pm-ux` | 6 UX | User experience |
| `/pm-frontend` | 6 Frontend | Component implementation |
| `/pm-a11y` | Cross-specialty | Accessibility audit |
