# /pm-dev — Run Development Server

**ACTION REQUIRED: Execute the steps below to start the dev server.**

Simple command to run the development server. Kills any existing server first.

## Usage

```
/pm-dev              # Start dev server (kills existing first)
/pm-dev --setup      # Start and open /setup route
/pm-dev --fresh      # Reset database + seed + start
```

---

## EXECUTE These Steps

### Step 1: Check for Running Server

```bash
lsof -i :3000 2>/dev/null | grep LISTEN
```

If server is already running, kill it:

```bash
pkill -f "nuxt" || true
sleep 1
```

Display:
```
⏹️  Stopped existing server
```

---

### Step 2: Check Node Modules

```bash
ls node_modules/.bin/nuxt 2>/dev/null
```

If not found:
```
📦 Installing dependencies...
```

Run:
```bash
npm install
```

---

### Step 3: Handle --fresh Flag

If `--fresh` flag provided:

```
🗑️  Resetting database...
```

```bash
rm -f data/sqlite.db
npm run db:push
npm run db:seed
```

---

### Step 4: Start Development Server

```
🚀 Starting development server...
```

```bash
npm run dev &
```

Wait for server to be ready:
```bash
sleep 5
curl -s -o /dev/null -w "%{http_code}" http://localhost:3000
```

---

### Step 5: Display Success

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         ✅ DEV SERVER RUNNING
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  🌐 Site:     http://localhost:3000
  🔐 Admin:    http://localhost:3000/admin
  ⚙️  Setup:    http://localhost:3000/setup

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Commands:
    /pm-status    Check configuration
    /restartdev   Restart server
    /closedev     Stop server

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

If `--setup` flag was provided, also display:
```
  Opening setup wizard...
  → http://localhost:3000/setup
```

---

## Flags

### --setup

Start server and indicate that user should navigate to `/setup`.

Useful when reconfiguring an existing project.

### --fresh

Complete database reset:
1. Delete existing database
2. Run migrations (`db:push`)
3. Seed with sample data (`db:seed`)
4. Start server

Use when you want a clean slate.

---

## Error Handling

### Port Already in Use

If port 3000 is in use by another process (not nuxt):

```
❌ Port 3000 is in use by another application.

Options:
  1. Kill the process: kill -9 $(lsof -ti :3000)
  2. Use different port: PORT=3001 npm run dev
```

### Build Errors

If `npm run dev` fails:

```
❌ Failed to start dev server.

Check the error above for details.

Common fixes:
  - npm install (missing dependencies)
  - Check nuxt.config.ts for syntax errors
  - Check puppet-master.config.ts for issues
```

---

## Notes

- This is a simple wrapper around `npm run dev`
- Always kills existing server first (no port conflicts)
- Does NOT run database migrations automatically (use --fresh for that)
- For first-time setup, use `/pm-init` instead
