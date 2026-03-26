# /pm-fullstack — PM Fullstack Developer Review

Runs a review using all Fullstack Developer PM experts across countries.

## Usage

```
/pm-fullstack                # Review uncommitted changes with Fullstack team
/pm-fullstack {file or feature} # Review specific code with Fullstack team
/pm-fullstack --deep         # Deep analysis mode
```

## Algorithm

### 1. Load Fullstack Developer Team Members (6 experts)

Load Fullstack expert roles from `.qwen/roles/pm/`:

- `fullstack-il.md` → Yonatan Cohen (Israel)
- `fullstack-ru.md` → Alexei Volkov (Russia)
- `fullstack-us.md` → Jake Thompson (USA)
- `fullstack-fr.md` → Pierre Martin (France)
- `fullstack-jp.md` → Kenji Yamamoto (Japan)
- `fullstack-ch.md` → Zhang Chen (China)

### 2. Apply Each Fullstack Perspective

Review the code from each Fullstack expert's perspective:
- Apply their Nuxt3 Fullstack specialty focus
- Consider their cultural/market context
- Use their Fullstack review checklist
- Focus on end-to-end flows, integration, and architecture

### 3. Generate Fullstack Team Report

## Output Format

```
🧙 PM Fullstack Developer Review: {target}

Fullstack Team (6 experts):
- 🇮🇱 Yonatan Cohen (Israel)
- 🇷🇺 Alexei Volkov (Russia)
- 🇺🇸 Jake Thompson (USA)
- 🇫🇷 Pierre Martin (France)
- 🇯🇵 Kenji Yamamoto (Japan)
- 🇨🇳 Zhang Chen (China)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Yonatan Cohen (Fullstack - Israel)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Israeli Fullstack Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Alexei Volkov (Fullstack - Russia)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Russian Fullstack Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Jake Thompson (Fullstack - USA)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**US Fullstack Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Pierre Martin (Fullstack - France)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**French Fullstack Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Kenji Yamamoto (Fullstack - Japan)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Japanese Fullstack Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Zhang Chen (Fullstack - China)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Chinese Fullstack Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Country | Verdict | Issues |
|--------|---------|---------|--------|
| Yonatan Cohen | IL | ✅ | 0 |
| Alexei Volkov | RU | ⚠️ | 2 |
| Jake Thompson | US | ✅ | 1 |
| Pierre Martin | FR | ⚠️ | 3 |
| Kenji Yamamoto | JP | ✅ | 0 |
| Zhang Chen | CH | ⚠️ | 1 |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Fullstack Best Practices
- {best practice}

### Cross-Cultural Fullstack Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Fullstack Developer Focus Areas

- Vue 3.5 Composition API
- Nuxt 4 architecture and features
- Nitro/H3 server logic
- TypeScript and type safety
- Component architecture
- API-to-UI integration
- End-to-end feature implementation
- Cross-layer architecture
- Shared types and DTOs
- Module-level configuration
- Performance optimization across layers
- Data loading and caching strategies

## Cultural Context Considerations

- **Israel**: Startup efficiency, security-conscious, mobile-first
- **Russia**: Performance, scalability, dense information
- **USA**: Best practices, accessibility, modern tooling
- **France**: GDPR compliance, refined architecture, formal business
- **Japan**: Quality, polish, clear guidance, careful architecture
- **China**: Mobile-first, scale, regulatory compliance

## Notes

- Provides comprehensive Fullstack perspective across cultures
- All Fullstack specialties covered with cultural context
- End-to-end and integration focus
- Cross-cultural fullstack considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-fullstack --deep
```

This applies deep analysis mode:
- More thorough Fullstack analysis
- Deeper reasoning about cross-cultural Fullstack trade-offs
- Extended deliberation on architecture and integration issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-fullstack` | 6 Fullstack experts | Fullstack-focused reviews |
| `/pm-team-all` | All 42 | Major releases |