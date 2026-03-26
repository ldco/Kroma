# /pm-security — Security Specialist Review (6 Experts)

Runs a focused security review using all 6 security specialists from different countries.

## Usage

```
/pm-security                      # Review uncommitted changes
/pm-security {file or feature}    # Review specific code
/pm-security --deep               # Deep thinking mode (recommended for audits)
```

## Security Team (6 Experts)

| Country | Expert | Focus Area |
|---------|--------|------------|
| IL | Maya Katz (מאיה כץ) | Offensive security, penetration testing |
| RU | Yulia Novikova (Юлия Новикова) | Cryptography, secure protocols |
| US | Jessica Williams | OWASP, compliance (SOC2, HIPAA) |
| FR | Camille Rousseau | GDPR security, data protection |
| JP | Akiko Kobayashi (小林明子) | Secure coding, audit trails |
| CH | Huang Lin (黄林) | Government compliance, data sovereignty |

## Algorithm

### 1. Load All Security Roles

```
for country in [il, ru, us, fr, jp, ch]:
  load .claude/roles/pm/security-{country}.md
```

### 2. Review from Each Perspective

Each expert reviews focusing on:
- OWASP Top 10 vulnerabilities
- Authentication & authorization
- Input validation & sanitization
- Cryptographic practices
- Regional compliance requirements
- Data protection & privacy

### 3. Generate Security Report

## Output Format

```
🔒 PM SECURITY TEAM REVIEW: {target}

6 Security Experts • 6 Countries • 1 Specialty

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🇮🇱 Maya Katz (Israel)

**Focus:** Offensive security, penetration testing

**Verdict:** ✅/⚠️/❌

**Vulnerabilities Found:**
1. {vulnerability}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Repeat for all 6 experts]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Security Consensus

### OWASP Top 10 Check

| Vulnerability | Status | Notes |
|---------------|--------|-------|
| A01 Broken Access Control | ✅/⚠️/❌ | {notes} |
| A02 Cryptographic Failures | ✅/⚠️/❌ | {notes} |
| A03 Injection | ✅/⚠️/❌ | {notes} |
| A04 Insecure Design | ✅/⚠️/❌ | {notes} |
| A05 Security Misconfiguration | ✅/⚠️/❌ | {notes} |
| A06 Vulnerable Components | ✅/⚠️/❌ | {notes} |
| A07 Auth Failures | ✅/⚠️/❌ | {notes} |
| A08 Data Integrity Failures | ✅/⚠️/❌ | {notes} |
| A09 Logging Failures | ✅/⚠️/❌ | {notes} |
| A10 SSRF | ✅/⚠️/❌ | {notes} |

### Compliance Status

| Framework | Status | Notes |
|-----------|--------|-------|
| GDPR (EU) | ✅/⚠️/❌ | {notes} |
| SOC2 | ✅/⚠️/❌ | {notes} |
| PIPL (China) | ✅/⚠️/❌ | {notes} |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (Security vulnerabilities)
1. {fix}

### High (Compliance gaps)
1. {fix}

### Medium (Best practices)
1. {fix}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## OVERALL SECURITY VERDICT: {✅/⚠️/❌}

**Safe for production:** Yes/No/With fixes
```

## When to Use

| Scenario | Command |
|----------|---------|
| Security-focused review | `/pm-security` |
| Pre-release audit | `/pm-security --deep` |
| Full team + security | `/pm-team-all` |

## Related Commands

| Command | Experts | Focus |
|---------|---------|-------|
| `/pm-security` | 6 Security | Vulnerabilities |
| `/pm-backend` | 6 Backend | API security |
| `/pm-devops` | 6 DevOps | Infrastructure security |
