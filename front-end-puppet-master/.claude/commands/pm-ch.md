# /pm-ch — Chinese PM Team Review

Runs a review using all 7 Chinese PM specialists.

## Usage

```
/pm-ch                      # Review uncommitted changes
/pm-ch {file or feature}    # Review specific code
/pm-ch --deep               # Deep thinking mode (like /ultra)
```

## Team Members

| Role | Name | Expertise |
|------|------|-----------|
| 🎯 UX | Li Wei (李伟) | WeChat UX, mobile-first |
| 🧙 Fullstack | Zhang Chen (张晨) | Nuxt3, BAT scale |
| 🎨 Frontend | Wang Mei (王梅) | Vue3, WeChat browser |
| ⚙️ Backend | Liu Yang (刘洋) | Nitro, high-concurrency |
| 🐍 FastAPI | Chen Ming (陈明) | Python, Alibaba/Tencent |
| 🚀 DevOps | Zhao Feng (赵峰) | Alibaba Cloud, ICP |
| 🔒 Security | Huang Lin (黄林) | MLPS, Chinese crypto |

## Special Focus

- **Scale**: BAT (Baidu, Alibaba, Tencent) experience
- **WeChat Ecosystem**: Mini-programs, WeChat browser
- **Search Engine**: Baidu, Zhihu, CSDN
- **Compliance**: MLPS, data security law, PIPL
- **Infrastructure**: Alibaba/Tencent/Huawei Cloud

## Algorithm

### 1. Load All Chinese Roles

```
.claude/roles/pm/ux-ch.md
.claude/roles/pm/fullstack-ch.md
.claude/roles/pm/frontend-ch.md
.claude/roles/pm/backend-ch.md
.claude/roles/pm/fastapi-ch.md
.claude/roles/pm/devops-ch.md
.claude/roles/pm/security-ch.md
```

### 2. Review from Each Perspective

Apply each expert's:
- Specialty focus
- Chinese market context
- WeChat/Alipay integration
- Massive scale considerations

### 3. Generate Team Report

Standard PM team report format with Chinese context.

## Best For

- Chinese market products
- WeChat integration
- High-scale architecture
- Chinese regulatory compliance
- Alibaba/Tencent Cloud deployment
