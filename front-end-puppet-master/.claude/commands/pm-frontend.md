# /pm-frontend — Frontend Specialist Review (6 Experts)

Runs a focused frontend review using all 6 frontend specialists from different countries.

## Usage

```
/pm-frontend                      # Review uncommitted changes
/pm-frontend {file or feature}    # Review specific code
/pm-frontend --deep               # Deep thinking mode
```

## Frontend Team (6 Experts)

| Country | Expert | Focus Area |
|---------|--------|------------|
| IL | Shira Ben-David (שירה בן-דוד) | Vue/Nuxt, RTL components |
| RU | Marina Sokolova (Марина Соколова) | Performance, Cyrillic rendering |
| US | Emily Chen | React patterns, accessibility |
| FR | Sophie Bernard | Component architecture, EU compliance |
| JP | Sakura Sato (佐藤さくら) | CJK optimization, micro-interactions |
| CH | Wang Mei (王梅) | WeChat mini-programs, mobile-first |

## Algorithm

### 1. Load All Frontend Roles

```
for country in [il, ru, us, fr, jp, ch]:
  load .claude/roles/pm/frontend-{country}.md
```

### 2. Review from Each Perspective

Each expert reviews focusing on:
- Component architecture
- State management patterns
- Performance optimization
- Internationalization implementation
- Their regional requirements

### 3. Generate Frontend Report

## Output Format

```
🎨 PM FRONTEND TEAM REVIEW: {target}

6 Frontend Experts • 6 Countries • 1 Specialty

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🇮🇱 Shira Ben-David (Israel)

**Focus:** Vue/Nuxt, RTL components

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Repeat for all 6 experts]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Frontend Consensus

### Architecture Issues
- {issue}

### Performance Issues
- {issue}

### i18n Issues
| Region | Issue | Priority |
|--------|-------|----------|
| IL | {RTL issue} | High |
| JP/CH | {CJK issue} | Medium |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical
1. {fix}

### Important
1. {fix}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## OVERALL FRONTEND VERDICT: {✅/⚠️/❌}
```

## Related Commands

| Command | Experts | Focus |
|---------|---------|-------|
| `/pm-frontend` | 6 Frontend | Components |
| `/pm-ux` | 6 UX | User experience |
| `/pm-backend` | 6 Backend | API integration |
