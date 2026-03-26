# AI Workflows in Puppet Master

Puppet Master supports multiple AI agent workflows to assist with development. You can select the workflow that best fits your needs during project setup.

---

## Quick Start

**Puppet Master is designed to work with AI assistants like Claude Code and Qwen Code.**

```bash
# 1. Clone the repo
git clone puppet-master my-project
cd my-project/app

# 2. Install dependencies
npm install

# 3. Initialize project (opens wizard)
npm run init

# 4. Start dev server
npm run dev
```

Open `http://localhost:3000` in your browser.

---

## Available Workflows

### Claude Code
- **Description**: Basic PM commands and workflows
- **Commands**: Core commands like `/pm-init`, `/pm-dev`, `/pm-status`, `/pm-migrate`
- **Personas**: Limited expert personas
- **Best For**: Simple projects or users familiar with Claude Code

### Qwen Code
- **Description**: Advanced workflow with comprehensive expert system
- **Commands**: Full command set including country and specialty-specific commands
- **Personas**: 42 expert personas (7 specialties × 6 countries)
- **Specialties**: UX, Fullstack, Frontend, Backend, FastAPI, DevOps, Security
- **Countries**: IL, RU, US, FR, JP, CH
- **Best For**: Complex projects requiring specialized expertise

### Codex
- **Description**: Experimental workflow (coming soon)
- **Status**: Planned for future releases
- **Best For**: Early adopters and experimental features

---

## Feature Comparison

| Feature | Claude Code | Qwen Code | Codex |
|---------|-------------|-----------|-------|
| Core Commands | ✅ | ✅ | ❌ |
| Expert Personas | Limited | 42 (7×6) | TBD |
| Country-Specific | ❌ | ✅ | TBD |
| Specialty-Specific | ❌ | ✅ | TBD |
| Migration Tools | ✅ | ✅ | ❌ |
| Setup Wizard | ✅ | ✅ | ❌ |

---

## PM Commands Reference

### Core Commands

| Command | Purpose |
|---------|---------|
| `/pm-init` | **Start here** — Initialize or reconfigure project |
| `/pm-dev` | Start/restart development server |
| `/pm-status` | Show current configuration state |
| `/pm-migrate` | AI-powered migration for brownfield projects |
| `/pm-plan` | Create development plan from technical brief |
| `/pm-contribute` | Export fix/feature as contribution doc |
| `/pm-apply` | Apply contribution doc to PM framework |

### Team Review Commands (Qwen)

**By Country** (7 specialists per country):
`/pm-il` • `/pm-ru` • `/pm-us` • `/pm-fr` • `/pm-jp` • `/pm-ch`

**By Specialty** (6 country experts per specialty):
| Command | Focus |
|---------|-------|
| `/pm-ux` | UX/UI design, accessibility |
| `/pm-frontend` | Vue/Nuxt, CSS, performance |
| `/pm-backend` | API, database, security |
| `/pm-security` | OWASP, compliance, pentest |
| `/pm-devops` | CI/CD, deployment, monitoring |
| `/pm-fastapi` | Python/FastAPI backends |

---

## Switching Between Workflows

You can change your AI workflow selection in `app/puppet-master.config.ts` by modifying the `aiWorkflow` field:

```typescript
aiWorkflow: 'claude' // Change to 'qwen' or 'codex'
```

Or use the workflow switch command:

```bash
npm run workflow:switch -- qwen
npm run workflow:info  # Check current workflow
```

After changing the workflow, restart your development server to ensure all paths are updated correctly.

---

## Migration Between Workflows

When switching between workflows, your project configuration remains the same, but the AI agent commands and expert personas available to you will change based on the selected workflow.

> **Note**: Project data and configuration are preserved across workflow changes. Only the AI agent interface and available commands change.

---

## Related Documentation

- [Getting Started Guide](../guides/getting-started.md) — Installation and setup
- [Configuration Reference](../reference/configuration.md) — Full configuration options
- [Entrypoints](../entrypoints/README.md) — Choose your starting point
