# Scripts Reference

Complete reference for all Puppet Master scripts and commands.

---

## NPM Scripts

These scripts are available via `npm run <script>`:

### Development

| Command | Description |
|---------|-------------|
| `npm run dev` | Start development server |
| `npm run build` | Build for production |
| `npm run preview` | Preview production build |
| `npm run generate` | Generate static site |
| `npm run postinstall` | Post-install setup (auto-runs) |

### Code Quality

| Command | Description |
|---------|-------------|
| `npm run lint` | Run ESLint |
| `npm run lint:fix` | Fix ESLint errors |
| `npm run format` | Format code with Prettier |
| `npm run format:check` | Check code formatting |
| `npm run lint:css-tokens` | Lint CSS token usage |
| `npm run lint:css-tokens:enforce` | Lint CSS tokens (fail on errors) |

### Testing

| Command | Description |
|---------|-------------|
| `npm run test` | Run staged tests (unit + API) |
| `npm run test:run` | Run all tests with Vitest |
| `npm run test:unit` | Run unit tests only |
| `npm run test:api` | Run API tests only |
| `npm run test:api:single` | Run API tests (single worker) |
| `npm run test:e2e` | Run e2e tests |
| `npm run test:e2e:playwright` | Run Playwright visual tests |
| `npm run test:e2e:playwright:ui` | Run Playwright with UI |

### Database

| Command | Description |
|---------|-------------|
| `npm run db:generate` | Generate Drizzle migrations |
| `npm run db:migrate` | Run database migrations |
| `npm run db:push` | Push schema to database |
| `npm run db:studio` | Open Drizzle Studio |
| `npm run db:seed` | Seed database with sample data |
| `npm run db:reset` | Reset database (delete + migrate + seed) |

### Assets

| Command | Description |
|---------|-------------|
| `npm run assets:optimize` | Optimize images (AVIF/WebP/PNG) |
| `npm run assets:optimize:full` | Full optimization (includes PNG) |
| `npm run generate:favicons` | Generate favicons from SVG |

### Workflow (AI)

| Command | Description |
|---------|-------------|
| `npm run workflow:switch <name>` | Switch AI workflow (claude/qwen/codex) |
| `npm run workflow:info` | Show current workflow status |

### Deployment

| Command | Description |
|---------|-------------|
| `npm run deploy` | Deploy with Kamal |
| `npm run deploy:setup` | Setup Kamal on server |
| `npm run deploy:rollback` | Rollback deployment |
| `npm run deploy:logs` | View deployment logs |
| `npm run deploy:shell` | SSH into deployment |
| `npm run docker:build` | Build Docker image |
| `npm run docker:run` | Run Docker container |
| `npm run server:provision` | Provision server with Ansible |
| `npm run server:provision:check` | Check Ansible playbook |

### Audit & Validation

| Command | Description |
|---------|-------------|
| `npm run audit:api-envelope` | Audit API response envelopes |
| `npm run audit:api-envelope:enforce` | Audit API envelopes (fail on errors) |
| `npm run check-doc-entrypoints` | Check documented scripts exist |
| `npm run check-doc-entrypoints:enforce` | Check scripts (fail on mismatch) |

### Initialization

| Command | Description |
|---------|-------------|
| `npm run init` | Initialize project (opens wizard) |
| `npm run init -- --headless` | Headless initialization |

---

## Shell Scripts

Located in `scripts/`:

### Database Maintenance

| Script | Description | Usage |
|--------|-------------|-------|
| `backup-db.sh` | Backup SQLite database | `./scripts/backup-db.sh [--remote] [--verify]` |
| `restore-db.sh` | Restore from backup | `./scripts/restore-db.sh [--latest \| <file>]` |

**Environment Variables:**
- `DATABASE_URL` — Database path (default: `/app/data/sqlite.db`)
- `BACKUP_DIR` — Backup directory (default: `/app/data/backups`)
- `RETENTION_DAYS` — Local retention (default: 7)
- `BACKUP_ENCRYPTION_KEY` — Encryption key (optional)

---

## Node/TypeScript Scripts

Located in `scripts/`:

### Core Scripts

| Script | NPM Command | Description |
|--------|-------------|-------------|
| `init-cli.ts` | `npm run init` | Project initialization wizard |
| `switch-workflow.ts` | `npm run workflow:switch` | Switch AI workflow |
| `workflow-info.ts` | `npm run workflow:info` | Show workflow status |

### Audit Scripts

| Script | NPM Command | Description |
|--------|-------------|-------------|
| `audit-api-envelope.mjs` | `npm run audit:api-envelope` | Validate API response envelopes |
| `check-doc-entrypoints.mjs` | `npm run check-doc-entrypoints` | Verify documented scripts exist |
| `lint-css-tokens.js` | `npm run lint:css-tokens` | Lint CSS token usage |

### Asset Scripts

| Script | NPM Command | Description |
|--------|-------------|-------------|
| `optimize-assets.mjs` | `npm run assets:optimize` | Image optimization pipeline |

### Maintenance Scripts

| Script | Description |
|--------|-------------|
| `maintenance/migrate-portfolio.ts` | One-time portfolio schema migration |

---

## Script Libraries

Located in `scripts/lib/`:

| Module | Description |
|--------|-------------|
| `config-reader.ts` | Read puppet-master.config.ts |
| `config-writer.ts` | Write configuration changes |
| `index.ts` | Library exports |
| `modules.ts` | Module utilities |

---

## Usage Examples

### Initialize New Project

```bash
npm run init
```

### Optimize Images

```bash
# Automatic (runs on build)
npm run build

# Manual
npm run assets:optimize
```

### Audit API Envelopes

```bash
# Check only
npm run audit:api-envelope

# Fail on errors (CI)
npm run audit:api-envelope:enforce
```

### Backup Database

```bash
# Local backup
./scripts/backup-db.sh

# With S3 upload
./scripts/backup-db.sh --remote

# Verify integrity
./scripts/backup-db.sh --verify
```

### Switch AI Workflow

```bash
# Switch to Qwen
npm run workflow:switch -- qwen

# Check current
npm run workflow:info
```

---

## Related Documentation

- [Getting Started](../guides/getting-started.md) — Setup guide
- [AI Workflows](../architecture/ai-workflows.md) — Workflow selection
- [Configuration](../reference/configuration.md) — Full configuration reference
