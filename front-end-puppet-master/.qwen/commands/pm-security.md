# /pm-security — PM Security Engineer Review

Runs a review using all Security Engineer PM experts across countries.

## Usage

```
/pm-security                 # Review uncommitted changes with Security team
/pm-security {file or feature} # Review specific code with Security team
/pm-security --deep          # Deep analysis mode
```

## Algorithm

### 1. Load Security Engineer Team Members (6 experts)

Load Security expert roles from `.qwen/roles/pm/`:

- `security-il.md` → Maya Rosen (Israel)
- `security-ru.md` → Yulia Sokolova (Russia)
- `security-us.md` → Jessica Williams (USA)
- `security-fr.md` → Camille Dubois (France)
- `security-jp.md` → Akiko Tanaka (Japan)
- `security-ch.md` → Huang Lin (China)

### 2. Apply Each Security Perspective

Review the code from each Security expert's perspective:
- Apply their Security Engineering specialty focus
- Consider their cultural/market context
- Use their Security review checklist
- Focus on auth, RBAC, secrets, OWASP, and audit logging

### 3. Generate Security Team Report

## Output Format

```
🔒 PM Security Engineer Review: {target}

Security Team (6 experts):
- 🇮🇱 Maya Rosen (Israel)
- 🇷🇺 Yulia Sokolova (Russia)
- 🇺🇸 Jessica Williams (USA)
- 🇫🇷 Camille Dubois (France)
- 🇯🇵 Akiko Tanaka (Japan)
- 🇨🇳 Huang Lin (China)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Maya Rosen (Security - Israel)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Israeli Security Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Yulia Sokolova (Security - Russia)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Russian Security Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Jessica Williams (Security - USA)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**US Security Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Camille Dubois (Security - France)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**French Security Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Akiko Tanaka (Security - Japan)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Japanese Security Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Huang Lin (Security - China)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Chinese Security Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Country | Verdict | Issues |
|--------|---------|---------|--------|
| Maya Rosen | IL | ✅ | 0 |
| Yulia Sokolova | RU | ⚠️ | 2 |
| Jessica Williams | US | ✅ | 1 |
| Camille Dubois | FR | ⚠️ | 3 |
| Akiko Tanaka | JP | ✅ | 0 |
| Huang Lin | CH | ⚠️ | 1 |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Security Best Practices
- {best practice}

### Cross-Cultural Security Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Security Engineer Focus Areas

- OWASP Top 10
- Session management and auth hardening
- Input validation and output encoding
- Security headers and rate limiting
- Secure file uploads and storage
- Auth required for sensitive endpoints
- RBAC checks for admin features
- Input validation and sanitization
- Audit logs for privileged actions
- Error messages not leaking sensitive info
- Safe file upload validation

## Cultural Context Considerations

- **Israel**: Security-conscious, startup efficiency, mobile-first
- **Russia**: Performance, scalability, systematic approach
- **USA**: Best practices, compliance, modern tooling
- **France**: GDPR compliance, refined architecture, formal business
- **Japan**: Quality, polish, clear guidance, careful security
- **China**: Mobile-first, scale, regulatory compliance

## Notes

- Provides comprehensive Security perspective across cultures
- All Security specialties covered with cultural context
- Auth and vulnerability focus
- Cross-cultural security considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-security --deep
```

This applies deep analysis mode:
- More thorough Security analysis
- Deeper reasoning about cross-cultural Security trade-offs
- Extended deliberation on vulnerabilities and compliance issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-security` | 6 Security experts | Security-focused reviews |
| `/pm-team-all` | All 42 | Major releases |