# /pm-frontend — PM Frontend Developer Review

Runs a review using all Frontend Developer PM experts across countries.

## Usage

```
/pm-frontend                  # Review uncommitted changes with Frontend team
/pm-frontend {file or feature} # Review specific code with Frontend team
/pm-frontend --deep           # Deep analysis mode
```

## Algorithm

### 1. Load Frontend Developer Team Members (6 experts)

Load Frontend expert roles from `.qwen/roles/pm/`:

- `frontend-il.md` → Shira Goldstein (Israel)
- `frontend-ru.md` → Marina Volkova (Russia)
- `frontend-us.md` → Emily Chen (USA)
- `frontend-fr.md` → Sophie Bernard (France)
- `frontend-jp.md` → Sakura Tanaka (Japan)
- `frontend-ch.md` → Wang Mei (China)

### 2. Apply Each Frontend Perspective

Review the code from each Frontend expert's perspective:
- Apply their Vue3/Nuxt3 specialty focus
- Consider their cultural/market context
- Use their Frontend review checklist
- Focus on components, styling, and user interfaces

### 3. Generate Frontend Team Report

## Output Format

```
🎨 PM Frontend Developer Review: {target}

Frontend Team (6 experts):
- 🇮🇱 Shira Goldstein (Israel)
- 🇷🇺 Marina Volkova (Russia)
- 🇺🇸 Emily Chen (USA)
- 🇫🇷 Sophie Bernard (France)
- 🇯🇵 Sakura Tanaka (Japan)
- 🇨🇳 Wang Mei (China)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Shira Goldstein (Frontend - Israel)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Israeli Frontend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Marina Volkova (Frontend - Russia)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Russian Frontend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Emily Chen (Frontend - USA)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**US Frontend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Sophie Bernard (Frontend - France)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**French Frontend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Sakura Tanaka (Frontend - Japan)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Japanese Frontend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Wang Mei (Frontend - China)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Chinese Frontend Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Country | Verdict | Issues |
|--------|---------|---------|--------|
| Shira Goldstein | IL | ✅ | 0 |
| Marina Volkova | RU | ⚠️ | 2 |
| Emily Chen | US | ✅ | 1 |
| Sophie Bernard | FR | ⚠️ | 3 |
| Sakura Tanaka | JP | ✅ | 0 |
| Wang Mei | CH | ⚠️ | 1 |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Frontend Best Practices
- {best practice}

### Cross-Cultural Frontend Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Frontend Developer Focus Areas

- Vue 3.5 Composition API
- Script setup syntax
- Reactive patterns (ref, reactive, computed)
- Component communication (props, emits, provide/inject)
- Lifecycle hooks and watchers
- Template syntax and directives
- Transitions and animations
- Atomic Design (atoms → molecules → organisms → sections)
- Global CSS classes (no scoped styles!)
- Icon imports from `~icons/tabler/`
- Composables in `app/composables/`
- useI18n() for translations
- useColorMode() for theming
- Page layouts and middleware
- Pure CSS (no Tailwind)
- OKLCH color system
- CSS Layers architecture
- CSS custom properties
- light-dark() function
- Responsive patterns
- Modern CSS features

## Cultural Context Considerations

- **Israel**: Startup efficiency, security-conscious, mobile-first
- **Russia**: Performance, scalability, dense information
- **USA**: Best practices, accessibility, modern tooling
- **France**: GDPR compliance, refined typography, formal business
- **Japan**: Quality, polish, clear guidance, careful spacing
- **China**: Mobile-first, scale, regulatory compliance

## Notes

- Provides comprehensive Frontend perspective across cultures
- All Frontend specialties covered with cultural context
- Component and styling focus
- Cross-cultural frontend considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-frontend --deep
```

This applies deep analysis mode:
- More thorough Frontend analysis
- Deeper reasoning about cross-cultural Frontend trade-offs
- Extended deliberation on performance and accessibility issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-frontend` | 6 Frontend experts | Frontend-focused reviews |
| `/pm-team-all` | All 42 | Major releases |