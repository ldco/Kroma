# /pm-devops — DevOps Specialist Review (6 Experts)

Runs a focused DevOps/infrastructure review using all 6 DevOps specialists from different countries.

## Usage

```
/pm-devops                      # Review uncommitted changes
/pm-devops {file or feature}    # Review specific code
/pm-devops --deep               # Deep thinking mode
```

## DevOps Team (6 Experts)

| Country | Expert | Focus Area |
|---------|--------|------------|
| IL | Oren Mizrahi (אורן מזרחי) | Kubernetes, GitOps |
| RU | Dmitri Orlov (Дмитрий Орлов) | High availability, Yandex Cloud |
| US | Chris Anderson | AWS, CI/CD pipelines |
| FR | Nicolas Dupont | OVH/Scaleway, EU data centers |
| JP | Ryo Watanabe (渡辺亮) | Efficiency, minimal resource usage |
| CH | Zhao Feng (赵峰) | Alibaba Cloud, multi-region |

## Algorithm

### 1. Load All DevOps Roles

```
for country in [il, ru, us, fr, jp, ch]:
  load .claude/roles/pm/devops-{country}.md
```

### 2. Review from Each Perspective

Each expert reviews focusing on:
- Docker/container configuration
- CI/CD pipeline efficiency
- Deployment strategies
- Monitoring and logging
- Regional infrastructure requirements
- Cost optimization

### 3. Generate DevOps Report

## Output Format

```
🚀 PM DEVOPS TEAM REVIEW: {target}

6 DevOps Experts • 6 Countries • 1 Specialty

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🇮🇱 Oren Mizrahi (Israel)

**Focus:** Kubernetes, GitOps

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Repeat for all 6 experts]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## DevOps Consensus

### Infrastructure Issues
- {issue}

### CI/CD Issues
- {issue}

### Regional Deployment

| Region | Cloud | Status | Notes |
|--------|-------|--------|-------|
| US | AWS | ✅/⚠️/❌ | {notes} |
| EU | OVH/Scaleway | ✅/⚠️/❌ | {notes} |
| RU | Yandex | ✅/⚠️/❌ | {notes} |
| JP | AWS Tokyo | ✅/⚠️/❌ | {notes} |
| CH | Alibaba | ✅/⚠️/❌ | {notes} |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical
1. {fix}

### Important
1. {fix}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## OVERALL DEVOPS VERDICT: {✅/⚠️/❌}
```

## Related Commands

| Command | Experts | Focus |
|---------|---------|-------|
| `/pm-devops` | 6 DevOps | Infrastructure |
| `/pm-security` | 6 Security | Infra security |
| `/pm-backend` | 6 Backend | App architecture |
