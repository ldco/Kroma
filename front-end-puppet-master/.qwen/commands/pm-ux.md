# /pm-ux — PM UX/UI Designer Review

Runs a review using all UX/UI Designer PM experts across countries.

## Usage

```
/pm-ux                        # Review uncommitted changes with UX team
/pm-ux {file or feature}      # Review specific code with UX team
/pm-ux --deep                 # Deep analysis mode
```

## Algorithm

### 1. Load UX/UI Designer Team Members (6 experts)

Load UX expert roles from `.qwen/roles/pm/`:

- `ux-il.md` → Noa Levi (Israel)
- `ux-ru.md` → Olga Petrova (Russia)
- `ux-us.md` → Sarah Johnson (USA)
- `ux-fr.md` → Marie Dubois (France)
- `ux-jp.md` → Yuki Tanaka (Japan)
- `ux-ch.md` → Li Wei (China)

### 2. Apply Each UX Perspective

Review the code from each UX expert's perspective:
- Apply their UX/UI specialty focus
- Consider their cultural/market context
- Use their UX review checklist
- Focus on user experience, accessibility, and design systems

### 3. Generate UX Team Report

## Output Format

```
🎯 PM UX/UI Designer Review: {target}

UX Team (6 experts):
- 🇮🇱 Noa Levi (Israel)
- 🇷🇺 Olga Petrova (Russia)
- 🇺🇸 Sarah Johnson (USA)
- 🇫🇷 Marie Dubois (France)
- 🇯🇵 Yuki Tanaka (Japan)
- 🇨🇳 Li Wei (China)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Noa Levi (UX - Israel)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Israeli UX Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Olga Petrova (UX - Russia)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Russian UX Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Sarah Johnson (UX - USA)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**US UX Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Marie Dubois (UX - France)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**French UX Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Yuki Tanaka (UX - Japan)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Japanese UX Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Li Wei (UX - China)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Chinese UX Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Country | Verdict | Issues |
|--------|---------|---------|--------|
| Noa Levi | IL | ✅ | 0 |
| Olga Petrova | RU | ⚠️ | 2 |
| Sarah Johnson | US | ✅ | 1 |
| Marie Dubois | FR | ⚠️ | 3 |
| Yuki Tanaka | JP | ✅ | 0 |
| Li Wei | CH | ⚠️ | 1 |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### UX Best Practices
- {best practice}

### Cross-Cultural UX Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## UX/UI Designer Focus Areas

- User experience research and design
- Interface design and prototyping
- Accessibility and WCAG compliance
- Mobile-first responsive design
- User journey mapping
- Interaction design
- Design systems and component libraries
- Atomic design methodology
- WCAG 2.1 accessibility standards
- RTL and bidirectional text considerations

## Cultural Context Considerations

- **Israel**: Startup efficiency, security-conscious, mobile-first
- **Russia**: Performance, scalability, dense information
- **USA**: Best practices, accessibility, modern design
- **France**: GDPR compliance, refined typography, formal business
- **Japan**: Quality, polish, clear guidance, careful spacing
- **China**: Mobile-first, scale, regulatory compliance

## Notes

- Provides comprehensive UX perspective across cultures
- All UX specialties covered with cultural context
- Accessibility and responsive design focus
- Cross-cultural design considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-ux --deep
```

This applies deep analysis mode:
- More thorough UX analysis
- Deeper reasoning about cross-cultural UX trade-offs
- Extended deliberation on accessibility and internationalization issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-ux` | 6 UX experts | UX-focused reviews |
| `/pm-team-all` | All 42 | Major releases |