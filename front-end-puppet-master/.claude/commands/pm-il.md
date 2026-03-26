# /pm-il — Israeli PM Team Review

Runs a review using all 7 Israeli PM specialists.

## Usage

```
/pm-il                      # Review uncommitted changes
/pm-il {file or feature}    # Review specific code
/pm-il --deep               # Deep thinking mode (like /ultra)
```

## Team Members

| Role | Name | Expertise |
|------|------|-----------|
| 🎯 UX | Noa Levy (נועה לוי) | RTL/bidi, accessibility |
| 🧙 Fullstack | Yonatan Cohen (יונתן כהן) | Nuxt3, RTL-first |
| 🎨 Frontend | Shira Ben-David (שירה בן-דוד) | Vue3, RTL CSS |
| ⚙️ Backend | Avi Goldstein (אבי גולדשטיין) | Nitro, security |
| 🐍 FastAPI | Eyal Shapiro (אייל שפירא) | Python, integrations |
| 🚀 DevOps | Oren Mizrahi (אורן מזרחי) | Infrastructure, security |
| 🔒 Security | Maya Katz (מאיה כץ) | Unit 8200 standards |

## Special Focus

- **RTL Expertise**: All members understand right-to-left layouts
- **Hebrew Language**: Native Hebrew speakers
- **Search Engine**: Google.il, Israeli tech resources
- **Security Mindset**: Unit 8200 / IDF tech influence
- **Startup Culture**: Fast iteration, ship early

## Algorithm

### 1. Load All Israeli Roles

```
.claude/roles/pm/ux-il.md
.claude/roles/pm/fullstack-il.md
.claude/roles/pm/frontend-il.md
.claude/roles/pm/backend-il.md
.claude/roles/pm/fastapi-il.md
.claude/roles/pm/devops-il.md
.claude/roles/pm/security-il.md
```

### 2. Review from Each Perspective

Apply each expert's:
- Specialty focus (UX, backend, etc.)
- RTL/bidi considerations
- Israeli market context
- Security awareness

### 3. Generate Team Report

Standard PM team report format with Israeli context.

## Best For

- RTL language support review
- Hebrew localization
- Security-critical features
- Startup-style rapid review

## Deep Thinking Mode

Add `--deep` for extended analysis with `/ultra` thinking mode.
