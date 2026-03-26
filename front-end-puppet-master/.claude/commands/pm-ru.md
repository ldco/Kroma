# /pm-ru — Russian PM Team Review

Runs a review using all 7 Russian PM specialists.

## Usage

```
/pm-ru                      # Review uncommitted changes
/pm-ru {file or feature}    # Review specific code
/pm-ru --deep               # Deep thinking mode (like /ultra)
```

## Team Members

| Role | Name | Expertise |
|------|------|-----------|
| 🎯 UX | Olga Petrova (Ольга Петрова) | GOST standards, Cyrillic |
| 🧙 Fullstack | Alexei Volkov (Алексей Волков) | Nuxt3, Russian integrations |
| 🎨 Frontend | Marina Sokolova (Марина Соколова) | Vue3, Cyrillic typography |
| ⚙️ Backend | Viktor Kozlov (Виктор Козлов) | Nitro, timezones |
| 🐍 FastAPI | Ivan Smirnov (Иван Смирнов) | Python, Yandex services |
| 🚀 DevOps | Dmitri Orlov (Дмитрий Орлов) | Yandex Cloud, 152-FZ |
| 🔒 Security | Yulia Novikova (Юлия Новикова) | GOST crypto, compliance |

## Special Focus

- **Cyrillic Expertise**: Native Russian speakers
- **Russian Services**: Yandex, VK, 1C integration
- **Search Engine**: Yandex, Habr, Russian forums
- **Compliance**: 152-FZ data localization
- **Timezones**: 11 Russian timezones

## Algorithm

### 1. Load All Russian Roles

```
.claude/roles/pm/ux-ru.md
.claude/roles/pm/fullstack-ru.md
.claude/roles/pm/frontend-ru.md
.claude/roles/pm/backend-ru.md
.claude/roles/pm/fastapi-ru.md
.claude/roles/pm/devops-ru.md
.claude/roles/pm/security-ru.md
```

### 2. Review from Each Perspective

Apply each expert's:
- Specialty focus
- Cyrillic text handling
- Russian market context
- Data localization requirements

### 3. Generate Team Report

Standard PM team report format with Russian context.

## Best For

- Russian localization
- Cyrillic typography review
- Yandex/Russian service integration
- Data residency compliance
- Multi-timezone handling
