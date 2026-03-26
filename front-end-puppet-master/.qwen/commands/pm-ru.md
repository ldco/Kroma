# /pm-ru — PM Russia Team Review

Runs a review using all Russian PM experts across specialties.

## Usage

```
/pm-ru                        # Review uncommitted changes with Russian team
/pm-ru {file or feature}      # Review specific code with Russian team
/pm-ru --deep                # Deep analysis mode
```

## Algorithm

### 1. Load Russian Team Members (7 experts)

Load Russian expert roles from `.qwen/roles/pm/`:

- `ux-ru.md` → Olga Petrova (UX/UI Designer)
- `fullstack-ru.md` → Alexei Volkov (Fullstack Developer)
- `frontend-ru.md` → Marina Volkova (Frontend Developer)
- `backend-ru.md` → Viktor Petrov (Backend Developer)
- `fastapi-ru.md` → Ivan Smirnov (FastAPI Developer)
- `devops-ru.md` → Dmitri Volkov (DevOps/SRE)
- `security-ru.md` → Yulia Sokolova (Security Engineer)

### 2. Apply Each Russian Perspective

Review the code from each Russian expert's perspective:
- Apply their specialty focus
- Consider Russian market context (performance, scalability, dense info)
- Use their review checklist
- Search in Russian as appropriate

### 3. Generate Russian Team Report

## Output Format

```
🇷🇺 PM Russia Team Review: {target}

Russian Team (7 experts):
- 🎯 Olga Petrova (UX/UI Designer)
- 🧙 Alexei Volkov (Fullstack Developer)
- 🎨 Marina Volkova (Frontend Developer)
- ⚙️ Viktor Petrov (Backend Developer)
- 🐍 Ivan Smirnov (FastAPI Developer)
- 🚀 Dmitri Volkov (DevOps/SRE)
- 🔒 Yulia Sokolova (Security Engineer)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### 🎯 Olga Petrova (UX - Russia)

**Verdict:** ✅/⚠️/❌

**Findings:**
1. {finding}
2. {finding}

**Recommendations:**
- {recommendation}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... repeat for each Russian specialist ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary

| Expert | Specialty | Verdict | Issues |
|--------|-----------|---------|--------|
| Olga Petrova | UX | ✅ | 0 |
| Alexei Volkov | Fullstack | ⚠️ | 2 |
| ... | ... | ... | ... |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes

### Critical (must fix)
- {fix}

### Important (should fix)
- {fix}

### Russian Market Considerations
- {consideration}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Overall Verdict: {✅/⚠️/❌}
```

## Russian Market Context

- Performance focus: High performance and scalability
- Dense information: Rich information architecture
- Clear hierarchy: Well-defined structures
- Systematic approach: Methodical development
- Scalability: Designed for large-scale applications

## Notes

- Provides Russian market perspective
- All specialties covered with Russian context
- Performance and scalability focus
- Dense information architecture considerations

## Deep Analysis Mode

Add `--deep` flag for extended analysis:

```
/pm-ru --deep
```

This applies deep analysis mode:
- More thorough analysis
- Deeper reasoning about Russian market trade-offs
- Extended deliberation on performance and scalability issues

## Related Commands

| Command | Experts | Best For |
|---------|---------|----------|
| `/pm-team` | 7 random | Quick reviews |
| `/pm-ru` | 7 Russian | Russian market focus |
| `/pm-team-all` | All 42 | Major releases |