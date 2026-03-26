# /pm-team-all — Complete PM Team Review (42 Experts)

Runs an exhaustive review using ALL 42 PM specialists from all countries and specialties.

## Usage

```
/pm-team-all                      # Review uncommitted changes
/pm-team-all {file or feature}    # Review specific code
/pm-team-all --deep               # Deep thinking mode (combine with /ultra)
```

## ⚠️ Warning

This command invokes **42 expert perspectives**. Use for:
- Major architecture decisions
- Pre-release audits
- Critical security reviews
- International product launches

For regular reviews, use `/pm-team` (7 random experts) or country-specific commands.

## Complete Team Roster

### 🇮🇱 Israeli Team
| Specialty | Expert |
|-----------|--------|
| UX | Noa Levy (נועה לוי) |
| Fullstack | Yonatan Cohen (יונתן כהן) |
| Frontend | Shira Ben-David (שירה בן-דוד) |
| Backend | Avi Goldstein (אבי גולדשטיין) |
| FastAPI | Eyal Shapiro (אייל שפירא) |
| DevOps | Oren Mizrahi (אורן מזרחי) |
| Security | Maya Katz (מאיה כץ) |

### 🇷🇺 Russian Team
| Specialty | Expert |
|-----------|--------|
| UX | Olga Petrova (Ольга Петрова) |
| Fullstack | Alexei Volkov (Алексей Волков) |
| Frontend | Marina Sokolova (Марина Соколова) |
| Backend | Viktor Kozlov (Виктор Козлов) |
| FastAPI | Ivan Smirnov (Иван Смирнов) |
| DevOps | Dmitri Orlov (Дмитрий Орлов) |
| Security | Yulia Novikova (Юлия Новикова) |

### 🇺🇸 American Team
| Specialty | Expert |
|-----------|--------|
| UX | Sarah Mitchell |
| Fullstack | Jake Thompson |
| Frontend | Emily Chen |
| Backend | Michael Rodriguez |
| FastAPI | David Kim |
| DevOps | Chris Anderson |
| Security | Jessica Williams |

### 🇫🇷 French Team
| Specialty | Expert |
|-----------|--------|
| UX | Marie Dubois |
| Fullstack | Pierre Martin |
| Frontend | Sophie Bernard |
| Backend | Jean-Luc Moreau |
| FastAPI | Antoine Lefèvre |
| DevOps | Nicolas Dupont |
| Security | Camille Rousseau |

### 🇯🇵 Japanese Team
| Specialty | Expert |
|-----------|--------|
| UX | Yuki Tanaka (田中ゆき) |
| Fullstack | Kenji Yamamoto (山本健二) |
| Frontend | Sakura Sato (佐藤さくら) |
| Backend | Takeshi Nakamura (中村武) |
| FastAPI | Hiroshi Suzuki (鈴木博) |
| DevOps | Ryo Watanabe (渡辺亮) |
| Security | Akiko Kobayashi (小林明子) |

### 🇨🇳 Chinese Team
| Specialty | Expert |
|-----------|--------|
| UX | Li Wei (李伟) |
| Fullstack | Zhang Chen (张晨) |
| Frontend | Wang Mei (王梅) |
| Backend | Liu Yang (刘洋) |
| FastAPI | Chen Ming (陈明) |
| DevOps | Zhao Feng (赵峰) |
| Security | Huang Lin (黄林) |

## Algorithm

### 1. Load All 42 Roles

```
for country in [il, ru, us, fr, jp, ch]:
  for specialty in [ux, fullstack, frontend, backend, fastapi, devops, security]:
    load .claude/roles/pm/{specialty}-{country}.md
```

### 2. Review by Specialty Groups

Group findings by specialty across all countries:

#### UX Review (6 experts)
- Noa (IL) - RTL focus
- Olga (RU) - Cyrillic focus
- Sarah (US) - ADA focus
- Marie (FR) - RGAA focus
- Yuki (JP) - CJK focus
- Li Wei (CH) - WeChat focus

#### Fullstack Review (6 experts)
[All 6 fullstack experts review architecture]

#### Frontend Review (6 experts)
[All 6 frontend experts review components]

#### Backend Review (6 experts)
[All 6 backend experts review APIs]

#### FastAPI Review (6 experts)
[All 6 FastAPI experts review integrations]

#### DevOps Review (6 experts)
[All 6 DevOps experts review infrastructure]

#### Security Review (6 experts)
[All 6 security experts review security]

### 3. Cross-Cultural Analysis

Identify issues that affect specific regions:
- RTL issues (IL)
- Cyrillic issues (RU)
- CJK issues (JP, CH)
- GDPR issues (FR/EU)
- Compliance issues (all regions)

### 4. Generate Ultra Report

## Output Format

```
👥 PM ULTRA TEAM REVIEW: {target}

42 Experts • 7 Specialties • 6 Countries

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 🎯 UX Consensus (6 experts)

**Cross-region findings:**
- {finding affecting multiple regions}

**Region-specific:**
- 🇮🇱 IL: {RTL issue}
- 🇯🇵 JP: {CJK issue}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 🧙 Fullstack Consensus (6 experts)

[Similar format...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Repeat for all 7 specialties]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 🌍 Regional Requirements

### Must support:
- RTL layouts (IL market)
- Cyrillic text (RU market)
- CJK text (JP/CH market)
- GDPR compliance (FR/EU market)
- Various timezones

### Infrastructure:
- AWS (US/JP)
- Alibaba/Tencent Cloud (CH)
- Yandex Cloud (RU)
- OVH/Scaleway (FR)
- Multi-CDN strategy

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Summary Matrix

| Specialty | IL | RU | US | FR | JP | CH | Consensus |
|-----------|:--:|:--:|:--:|:--:|:--:|:--:|:---------:|
| UX | ✅ | ✅ | ⚠️ | ✅ | ✅ | ✅ | ✅ |
| Fullstack | ✅ | ⚠️ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Frontend | ⚠️ | ✅ | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| Backend | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ✅ |
| FastAPI | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| DevOps | ✅ | ⚠️ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| Security | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⚠️ | ⚠️ |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Priority Fixes (Global Impact)

### Critical (blocks all markets)
1. {issue}

### Important (blocks some markets)
1. 🇮🇱🇷🇺 {issue affecting IL and RU}
2. 🇫🇷 {GDPR issue}

### Nice to Have
1. {optimization}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## OVERALL VERDICT: {✅/⚠️/❌}

Ready for global launch: {Yes/No/With fixes}
```

## When to Use

| Scenario | Command |
|----------|---------|
| Quick review | `/pm-team` (7 random) |
| Regional focus | `/pm-il`, `/pm-ru`, etc. |
| **Major release** | `/pm-team-all` (all 42) |
| **Architecture review** | `/pm-team-all` |
| **Security audit** | `/pm-team-all` |
| **International launch** | `/pm-team-all` |

## Deep Thinking Mode

Add `--deep` flag to enable extended analysis:

```
/pm-team-all --deep
```

This applies `/ultra` deep thinking mode to the review, providing:
- More thorough analysis
- Deeper reasoning about trade-offs
- Extended deliberation on complex issues
