# Puppet Master

## Vision
Config-driven studio toolkit for building client websites - fast, secure, production-ready.

## Stack
- Frontend: Nuxt3 (Vue 3.5, TypeScript) | Entry: `app/`, `app/pages/`
- Backend: Nitro (H3) | Entry: `server/api/`
- Database: SQLite + Drizzle ORM | Schema: `server/database/schema.ts`
- Styling: Pure CSS (OKLCH, CSS Layers) | Entry: `app/assets/css/main.css`

## Critical Rules
- **Imports**: Use `~/` for app directory, `~~/` for root, `~icons/` for Tabler icons
- **Components**: Atomic Design pattern (atoms → molecules → organisms → sections)
- **API Routes**: Use `defineEventHandler`, validate with Zod, parse with `readBody`
- **Styles**: Global CSS classes only, NO scoped styles in components
- **Types**: Database entities & API responses → SHARED in `~/types`. Local types ONLY for composable/plugin internals
- **Config**: All build-time config in `app/puppet-master.config.ts`
- **Git**: Ask before commit/push

## Puppet Master Architect Role

When working on PM projects, Qwen acts as **Puppet Master Architect** — a specialized AI co-pilot that:
- Understands the entire PM framework deeply
- Guides users through Greenfield (new) and Brownfield (import) workflows
- Uses `/pm` commands for PM-specific operations
- Collaborates with expert personas via `/as` command

**Note**: Qwen Code is one of multiple available AI workflows in Puppet Master. See `docs/architecture/ai-workflows.md` for information about workflow selection.

See: `docs/architecture/pm-qwen-system.md` for full documentation.

## PM Commands (Qwen)

Project-specific commands in `.qwen/commands/`:

| Command | Purpose |
|---------|---------|
| `/pm-init` | Main entry point — starts wizard if unconfigured, shows options if configured |
| `/pm-plan` | Create development plan from technical brief (run after init) |
| `/pm-dev` | Start/restart dev server (kills existing first) |
| `/pm-status` | Show current config — pmMode, modules, features, database state |
| `/pm-migrate` | AI-powered migration — analyzes ./import/ folder and creates migration plan |
| `/pm-contribute` | Export fix/feature as contribution doc (for client projects) |
| `/pm-apply` | Apply contribution doc to PM framework |

**Deprecated:** `/pm-start` (replaced by `/pm-dev`)

### Workflows

**Greenfield (New Project):**
```
git clone puppet-master my-project
cd my-project/app
npm install
/pm-init                 # Starts wizard at /init
# Complete wizard in browser (add technical brief!)
/pm-plan                 # Creates development plan from brief
# Pick a task and start building
```

**Brownfield (Import Existing):**
```
git clone puppet-master my-project
cd my-project/app
npm install
/pm-init                 # Starts wizard
# In wizard: upload your project as zip
# Complete wizard configuration
/pm-migrate              # AI analyzes code and creates migration plan
# Follow the plan to migrate each piece
```

**Quick Start:**
```
/pm-init                 # Opens wizard → configure → done
```

**Daily Development:**
```
/pm-dev                  # Start dev server
/pm-status               # Check configuration
/pm-init --reset         # Reset to reconfigure
```

**Contributing Back to PM** (from client project):
```
# In client project - after fixing/adding something to PM base
/pm-contribute           # Generates .pm-contribution.md
cp .pm-contribution.md ~/puppet-master/

# In PM framework repo
/pm-apply                # Reads and implements the contribution
```

### pmMode Configuration

The `pmMode` field in `puppet-master.config.ts` controls project state:

| Value | Description |
|-------|-------------|
| `'unconfigured'` | Fresh clone, needs setup → wizard shown |
| `'build'` | Client project (website or app) |
| `'develop'` | Framework development (showcase) |

## Patterns

### Vue Component
```vue
<script setup lang="ts">
import IconExample from '~icons/tabler/example'
const { t } = useI18n()
const colorMode = useColorMode()
</script>

<template>
  <!-- Use global CSS classes -->
  <div class="card">
    <IconExample class="icon" />
  </div>
</template>
<!-- NO scoped styles -->
```

### API Route
```typescript
import { useDatabase, schema } from '../../database/client'
import { mySchema } from '../../utils/validation'

export default defineEventHandler(async event => {
  const body = await readBody(event)
  const result = mySchema.safeParse(body)

  if (!result.success) {
    throw createError({ statusCode: 400, data: result.error.flatten() })
  }

  const db = useDatabase()
  // ... business logic
  return { success: true, data: result }
})
```

## NPM Commands
```bash
npm run dev        # Start dev server
npm run build      # Build for production
npm run test       # Run tests (Vitest)
npm run lint       # ESLint
npm run db:push    # Apply schema
npm run db:seed    # Seed data
npm run db:studio  # Drizzle Studio
```

## Key Files
- `app/puppet-master.config.ts` - All feature toggles, colors, locales
- `server/database/schema.ts` - Database schema (Drizzle)
- `server/utils/validation.ts` - Zod schemas
- `app/types/` - Shared TypeScript types (import from `~/types`)
- `app/assets/css/` - CSS system (50+ files)
- `docs/` - Full documentation
- `.qwen/commands/` - PM-specific Qwen commands

## Context
- Session context: `.qwen-data/context.md`
- Project config: `.qwen/config.json`
- PM Qwen docs: `docs/PM-QWEN-SYSTEM.md`