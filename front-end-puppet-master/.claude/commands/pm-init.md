# /pm-init — Initialize Puppet Master Project

**ACTION REQUIRED: Execute all steps below.**

## Usage

```
/pm-init              # Initialize project
/pm-init --reset      # Reset to unconfigured
```

---

## EXECUTE These Steps

### Step 1: Install Dependencies

Check if node_modules exists:
```bash
ls node_modules/.bin/nuxt 2>/dev/null
```

If NOT found, run:
```bash
npm install
```

---

### Step 2: Check Current Configuration

Read `puppet-master.config.ts` and check `pmMode`:
- `'unconfigured'` → Step 3
- `'build'` or `'develop'` → Step 4

---

### Step 3: If Unconfigured — Ask ONE Question

**Use AskUserQuestion:**

Question: "What are you building?"
Header: "Mode"
Options:
- **BUILD** — Client project (website or app)
- **DEVELOP** — Framework development

#### If DEVELOP selected:

1. Update config: `pmMode: 'develop'`
2. Run `npm run db:push && npm run db:seed`
3. Start dev server: `npm run dev &`
4. Display:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         DEVELOP MODE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  PM Showcase: http://localhost:3000

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
**Done.**

#### If BUILD selected:

1. Update config: `pmMode: 'build'`
2. Run `npm run db:push` (empty tables, no seed data)
3. Start dev server: `npm run dev &`
4. Display:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         BUILD MODE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Complete setup: http://localhost:3000/init

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
**User completes wizard in browser.**

---

### Step 4: If Already Configured — Show Status

Use AskUserQuestion:

Question: "What would you like to do?"
Header: "Action"
Options:
- **Start dev server** (Recommended)
- **Reconfigure** — Reset and start over

Handle accordingly.

---

## The --reset Flag

Set `pmMode: 'unconfigured'` and re-run Step 2.
