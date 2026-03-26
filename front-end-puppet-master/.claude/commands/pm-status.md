# /pm-status — Show Puppet Master Configuration Status

**ACTION REQUIRED: Read configuration and display current state clearly.**

Quick overview of current Puppet Master configuration and project state.

## Usage

```
/pm-status              # Full status overview
/pm-status --config     # Show raw config values
/pm-status --modules    # Show module details only
/pm-status --db         # Show database status
```

---

## EXECUTE These Steps

### Step 1: Read Configuration

Read the main config file:

```
Read: app/puppet-master.config.ts
```

Parse and extract:
- `pmMode` — 'unconfigured' | 'build' | 'develop'
- `projectType` — 'website' | 'app' (if BUILD mode)
- `admin.enabled` — Admin panel status
- `features` — Enabled features
- `modules` — Enabled modules
- `locales` — Configured languages
- `dataSource.provider` — Data source type
- `design` — Color and font settings

---

### Step 2: Check Database Status

```bash
# Check if database exists
ls data/sqlite.db 2>/dev/null

# Get file size if exists
du -h data/sqlite.db 2>/dev/null
```

---

### Step 3: Check Dev Server

```bash
# Check if dev server is running
lsof -i :3000 2>/dev/null | grep LISTEN
```

---

### Step 4: Display Status

#### If Unconfigured

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         📊 PUPPET MASTER STATUS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Mode:           ⚠️  UNCONFIGURED
Dev Server:     {● Running | ○ Stopped}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

This project needs to be configured.

Run /pm-init to start the setup wizard.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### If Configured (BUILD or DEVELOP mode)

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         📊 PUPPET MASTER STATUS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Mode:           {🏗️ BUILD | 🔧 DEVELOP}
Type:           {Website | App | —}
Admin:          {✅ Enabled | ❌ Disabled}
Data Source:    {database | api | hybrid}
Dev Server:     {● Running on :3000 | ○ Stopped}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Features:
  {✅ | ❌} Multilingual     {count} locales ({list})
  {✅ | ❌} Dark Mode        {Enabled | Disabled}
  {✅ | ❌} PWA              {Enabled | Disabled}
  {✅ | ❌} Contact Notify   {Methods or Disabled}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Modules:
  {✅ | ❌} blog             Blog posts
  {✅ | ❌} portfolio        Projects/gallery
  {✅ | ❌} team             Team members
  {✅ | ❌} pricing          Pricing tiers
  {✅ | ❌} testimonials     Customer reviews
  {✅ | ❌} faq              FAQ section
  {✅ | ❌} clients          Logo showcase
  {✅ | ❌} features         Feature cards

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Design:
  Primary:      {color}
  Accent:       {color}
  Fonts:        {accent} / {text}
  Icons:        {library}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Database:
  {✅ SQLite exists | ❌ No database}   {path} ({size})
  {✅ Seeded | ⚠️ Empty}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Commands:
  /pm-init      Reconfigure project (opens wizard)
  /pm-dev       Start/restart dev server
  /closedev     Stop dev server

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Flags

### --config

Show raw configuration values in a table format.
Useful for debugging.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         📄 RAW CONFIGURATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

| Key                      | Value                              |
|--------------------------|-----------------------------------|
| pmMode                   | build                              |
| projectType              | website                            |
| admin.enabled            | true                               |
| features.multilingual    | true                               |
| features.darkMode        | true                               |
| ...                      | ...                                |

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### --modules

Show detailed module configuration:
- Enabled/disabled state
- Module-specific options
- Content counts (if database exists)

### --db

Show database details:
- File path and size
- Table list
- Row counts per table
- Last seeded date (if tracked)

---

## Notes

- Always read fresh from config file (don't cache)
- Show actionable next steps based on state
- Indicate if something needs attention (missing db, server not running)
- Keep output concise but informative
