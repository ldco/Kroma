# /pm-devops — PM DevOps/SRE Review

Runs a review using all DevOps/SRE PM experts across countries.

## Usage

```
/pm-devops                   # Review uncommitted changes with DevOps team
/pm-devops {file or feature} # Review specific code with DevOps team
/pm-devops --deep           # Deep analysis mode
```

## Algorithm

### 1. Load DevOps/SRE Team Members (6 experts)

Load DevOps expert roles from `.qwen/roles/pm/`:

- `devops-il.md` → Oren Levi (Israel)
- `devops-ru.md` → Dmitri Volkov (Russia)
- `devops-us.md` → Chris Anderson (USA)
- `devops-fr.md` → Nicolas Petit (France)
- `devops-jp.md` → Ryo Sato (Japan)
- `devops-ch.md` → Zhao Feng (China)

### 2. Apply Each DevOps Perspective

Review the code from each DevOps expert's perspective:
- Apply their DevOps/SRE specialty focus
- Consider their cultural/market context
- Use their DevOps review checklist
- Focus on deployment, CI/CD, observability, and infra consistency

### 3. Generate DevOps Team Report

## Output Format

```
🚀 PM DevOps/SRE Review: {target}

DevOps Team (6 experts):
- 🇮🇱 Oren Levi (Israel)
- 🇷🇺 Dmitri Volkov (Russia)
- 🇺🇸 Chris Anderson (USA)
- 🇫🇷 Nicolas Petit (France)
- 🇯🇵 Ryo Sato (Japan)
- 🇨🇳 Zhao Feng (China)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Oren Levi (DevOps - Israel)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Israeli DevOps Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Dmitri Volkov (DevOps - Russia)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Russian DevOps Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Chris Anderson (DevOps - USA)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**US DevOps Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Nicolas Petit (DevOps - France)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**French DevOps Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Ryo Sato (DevOps - Japan)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Japanese DevOps Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### Zhao Feng (DevOps - China)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

**Chinese DevOps Focus:**
- {market-specific consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Country | Verdict | Issues |
|--------|---------|---------|--------|
| Oren Levi | IL | ✅ | 0 |
| Dmitri Volkov | RU | ⚠️ | 2 |
| Chris Anderson | US | ✅ | 1 |
| Nicolas Petit | FR | ⚠️ | 3 |
| Ryo Sato | JP | ✅ | 0 |
| Zhao Feng | CH | ⚠️ | 1 |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### DevOps Best Practices
- {best practice}

### Cross-Cultural DevOps Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## DevOps/SRE Focus Areas

- Docker and container builds
- Kamal deploy workflows
- Monitoring, logging, tracing
- Performance and caching
- Environment and secrets management
- Build and deploy scripts alignment
- Deploy.yml and env vars consistency
- Health checks and monitoring
- Log format structuring
- DB migrations in deploy process

## Cultural Context Considerations

- **Israel**: Startup efficiency, security-conscious, mobile-first
- **Russia**: Performance, scalability, systematic approach
- **USA**: Best practices, reliability, modern tooling
- **France**: GDPR compliance, refined architecture, formal business
- **Japan**: Quality, polish, clear guidance, careful operations
- **China**: Mobile-first, scale, regulatory compliance

## Notes

- Provides comprehensive DevOps perspective across cultures
- All DevOps specialties covered with cultural context
- Deployment and observability focus
- Cross-cultural DevOps considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-devops --deep
```

This applies deep analysis mode:
- More thorough DevOps analysis
- Deeper reasoning about cross-cultural DevOps trade-offs
- Extended deliberation on reliability and scalability issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-devops` | 6 DevOps experts | DevOps-focused reviews |
| `/pm-team-all` | All 42 | Major releases |