# /pm-team — PM Multi-Specialty Review

Runs a comprehensive review using one randomly selected expert from each PM specialty.

## Usage

```
/pm-team                      # Review uncommitted changes
/pm-team {file or feature}    # Review specific code
/pm-team --deep               # Deep analysis mode
```

## Algorithm

### 1. Select Random Team (7 experts, one per specialty)

For each specialty, randomly select one country variant:

| Specialty | Countries | Example Selection |
|-----------|-----------|-------------------|
| UX | il, ru, us, fr, jp, ch | ux-jp → Yuki Tanaka |
| Fullstack | il, ru, us, fr, jp, ch | fullstack-us → Jake Thompson |
| Frontend | il, ru, us, fr, jp, ch | frontend-fr → Sophie Bernard |
| Backend | il, ru, us, fr, jp, ch | backend-il → Avi Goldstein |
| FastAPI | il, ru, us, fr, jp, ch | fastapi-ru → Ivan Smirnov |
| DevOps | il, ru, us, fr, jp, ch | devops-ch → Zhao Feng |
| Security | il, ru, us, fr, jp, ch | security-us → Jessica Williams |

### 2. Load Selected Roles

For each selected expert:
```
Read .qwen/roles/pm/{specialty}-{country}.md
```

### 3. Apply Each Perspective

Review the code from each expert's perspective:
- Apply their specialty focus
- Consider their cultural/market context
- Use their review checklist
- Search in their preferred language if researching

### 4. Generate Team Report

## Output Format

```
👥 PM Team Review: {target}

Selected team (random from each specialty):
- 🎯 {ux_name} ({ux_country})
- 🧙 {fullstack_name} ({fullstack_country})
- 🎨 {frontend_name} ({frontend_country})
- ⚙️ {backend_name} ({backend_country})
- 🐍 {fastapi_name} ({fastapi_country})
- 🚀 {devops_name} ({devops_country})
- 🔒 {security_name} ({security_country})

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎯 {ux_name} (UX - {country})

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... repeat for each specialty ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Specialty | Verdict | Issues |
|--------|-----------|---------|--------|
| {name} | UX | ✅ | 0 |
| {name} | Fullstack | ⚠️ | 2 |
| ... | ... | ... | ... |

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

## PM Team Members (42 total)

### By Specialty

| Specialty | IL | RU | US | FR | JP | CH |
|-----------|----|----|----|----|----|----|
| UX | Noa | Olga | Sarah | Marie | Yuki | Li Wei |
| Fullstack | Yonatan | Alexei | Jake | Pierre | Kenji | Zhang Chen |
| Frontend | Shira | Marina | Emily | Sophie | Sakura | Wang Mei |
| Backend | Avi | Viktor | Michael | Jean-Luc | Takeshi | Liu Yang |
| FastAPI | Eyal | Ivan | David | Antoine | Hiroshi | Chen Ming |
| DevOps | Oren | Dmitri | Chris | Nicolas | Ryo | Zhao Feng |
| Security | Maya | Yulia | Jessica | Camille | Akiko | Huang Lin |

## Notes

- Each run selects a different random combination
- Provides diverse cultural perspectives
- All experts have deep PM framework knowledge
- Experts search in their native language when researching

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-team --deep
```

This applies deep analysis mode:
- More thorough analysis
- Deeper reasoning about trade-offs
- Extended deliberation on complex issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-il`, `/pm-ru`, etc. | 7 per country | Regional focus |
| `/pm-team-all` | All 42 | Major releases |