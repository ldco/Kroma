# Getting Started

Quick setup guide for Puppet Master. For complete details, see the [main README](../../README.md).

---

## Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| Node.js | 20.x | LTS required |
| npm | 10+ | Package manager |
| Git | Latest | Version control |

---

## Installation

### 1. Clone and Install

```bash
git clone <repository-url>
cd app
npm install
```

### 2. Environment Setup

```bash
cp .env.example .env
```

Default local settings work out of the box. Database is created automatically at `./data/sqlite.db`.

### 3. Initialize and Run

```bash
# Initialize (opens setup wizard)
npm run init

# Start dev server
npm run dev
```

Open `http://localhost:3000`.

---

## Default Accounts (Develop Mode)

After initialization with `--mode=develop`:

| Email | Password | Role |
|-------|----------|------|
| master@example.com | master123 | Master |
| admin@example.com | admin123 | Admin |
| editor@example.com | editor123 | Editor |

---

## Next Steps

- [Admin Guide](./admin-guide.md) — Learn the admin panel
- [Configuration](../reference/configuration.md) — Customize your project
- [Entrypoints](../entrypoints/README.md) — Choose your workflow

---

## Troubleshooting

**Port 3000 in use?**
```bash
lsof -i :3000
pkill -f "nuxt"
npm run dev
```

**Setup issues?** Re-run init:
```bash
npm run init -- --headless --mode=develop
```

For more help, see [README.md](../../README.md#troubleshooting-faq).
