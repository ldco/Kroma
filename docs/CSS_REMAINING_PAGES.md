# CSS Architecture Fixes - Remaining Pages

**Date:** 2026-03-23  
**Status:** Pattern Established - Apply to Remaining Pages  
**Completed:** login.vue, default.vue, projects.vue (CSS created)

---

## Pattern for Converting Scoped Styles to Global CSS

### Step 1: Extract Scoped Styles
```vue
<!-- BEFORE -->
<style scoped>
.page-container {
  padding: 1.5rem;
  background: #f9fafb;
}
</style>
```

### Step 2: Create Global CSS File
```css
/* assets/css/ui/content/page-name.css */
.page-container {
  padding: var(--space-6);
  background: var(--l-bg);
}
```

### Step 3: Update Vue File
```vue
<!-- AFTER -->
<template>
  <div class="page-container">
    <!-- Uses global CSS -->
  </div>
</template>

<!-- Comment documenting CSS source -->
<!--
  Uses global CSS classes:
  - ui/content/page-name.css: .page-container
-->
```

---

## Token Replacement Map

| Old Token | New Token | File |
|-----------|-----------|------|
| `--color-gray-50` | `--l-bg` | auto.css |
| `--color-gray-100` | `--l-bg-sunken` | auto.css |
| `--color-gray-200` | `--l-border` | auto.css |
| `--color-gray-300` | `--l-border-strong` | auto.css |
| `--color-gray-400` | `--t-muted` | auto.css |
| `--color-gray-500` | `--t-secondary` | auto.css |
| `--color-gray-600` | `--t-secondary` | auto.css |
| `--color-gray-700` | `--t-primary` | auto.css |
| `--color-gray-800` | `--t-primary` | auto.css |
| `--color-gray-900` | `--t-primary` | auto.css |
| `--color-brand` | `--i-brand` | auto.css |
| `--color-brand-100` | `--i-brand-subtle` | auto.css |
| `--color-white` | `--l-bg-elevated` | auto.css |
| `1rem` | `var(--space-4)` | variables.css |
| `0.5rem` | `var(--space-2)` | variables.css |
| `0.75rem` | `var(--space-3)` | variables.css |
| `1.5rem` | `var(--space-6)` | variables.css |
| `#rgb` / `#rrggbb` | Use `var(--*)` | Remove hardcoded |

---

## Remaining Pages to Convert

### 1. projects.vue ✅ (CSS Created)
**File:** `frontend-nuxt/pages/projects.vue`  
**CSS Created:** `assets/css/ui/content/projects-page.css`  
**Action Needed:** Update template class names, remove `<style>` block

### 2. projects/[slug]/bootstrap.vue
**Estimated Size:** ~300 lines  
**Key Components:**
- Bootstrap form
- Import/export UI
- Progress indicators

**CSS File to Create:** `ui/content/bootstrap-page.css`

### 3. projects/[slug]/references.vue
**Estimated Size:** ~400 lines  
**Key Components:**
- Reference set grid
- Reference item cards
- Upload area

**CSS File to Create:** `ui/content/references-page.css`

### 4. projects/[slug]/run.vue
**Estimated Size:** ~500 lines  
**Key Components:**
- Run configuration form
- Settings panels
- Progress display

**CSS File to Create:** `ui/content/run-page.css`

### 5. projects/[slug]/post-process.vue
**Estimated Size:** ~400 lines  
**Key Components:**
- Post-process options
- Queue display
- Results gallery

**CSS File to Create:** `ui/content/post-process-page.css`

### 6. projects/[slug]/export.vue
**Estimated Size:** ~350 lines  
**Key Components:**
- Export configuration
- Package preview
- Download progress

**CSS File to Create:** `ui/content/export-page.css`

### 7. projects/[slug]/runs/[runId]/review.vue
**Estimated Size:** ~600 lines  
**Key Components:**
- Candidate grid
- Comparison view
- Approval workflow

**CSS File to Create:** `ui/content/review-page.css`

---

## Quick Fix Commands

### Find Scoped Styles
```bash
# Find all <style scoped> in pages
grep -r "<style scoped>" frontend-nuxt/pages/

# Find all --color-* tokens
grep -r "var(--color-" frontend-nuxt/pages/

# Find hardcoded colors
grep -rE "#[0-9a-fA-F]{3,8}" frontend-nuxt/pages/
```

### Run Lint
```bash
cd frontend-nuxt
npm run lint:css-tokens
```

---

## CSS File Naming Convention

```
assets/css/
├── ui/
│   └── content/
│       ├── login-page.css          ✅ Created
│       ├── projects-page.css       ✅ Created
│       ├── bootstrap-page.css      ⏳ TODO
│       ├── references-page.css     ⏳ TODO
│       ├── run-page.css            ⏳ TODO
│       ├── post-process-page.css   ⏳ TODO
│       ├── export-page.css         ⏳ TODO
│       └── review-page.css         ⏳ TODO
└── layout/
    └── kroma-layout.css            ✅ Created
```

---

## Class Naming Convention

```css
/* Page container */
.{page}-page { }

/* Page header */
.{page}-page__header { }
.{page}-page__title { }
.{page}-page__subtitle { }

/* Content sections */
.{page}-page__section { }

/* Empty states */
.{page}-page__empty { }
.{page}-page__empty-icon { }
.{page}-page__empty-title { }
.{page}-page__empty-description { }

/* Modals */
.{page}-modal__overlay { }
.{page}-modal__content { }
.{page}-modal__header { }
.{page}-modal__body { }
.{page}-modal__footer { }

/* Forms */
.{page}-form__group { }
.{page}-form__label { }
.{page}-form__input { }
.{page}-form__help { }
```

---

## Testing Checklist

For each converted page:

- [ ] Remove `<style scoped>` block
- [ ] Create global CSS file
- [ ] Replace `--color-*` with `--l-*`, `--t-*`, `--i-*`
- [ ] Replace hardcoded values with `--space-*`, `--text-*`, etc.
- [ ] Update template class names
- [ ] Add documentation comment
- [ ] Run `npm run lint:css-tokens`
- [ ] Test page renders correctly
- [ ] Test dark/light mode
- [ ] Test responsive

---

## Priority Order

1. **High Priority (Core Flow)**
   - ✅ projects.vue (CSS created, needs template update)
   - ⏳ run.vue (Core generation flow)
   - ⏳ review.vue (Candidate approval)

2. **Medium Priority (Setup)**
   - ⏳ bootstrap.vue (Project setup)
   - ⏳ references.vue (Continuity references)

3. **Lower Priority (Post-Process)**
   - ⏳ post-process.vue
   - ⏳ export.vue

---

## Estimated Time

| Page | Time | Lines |
|------|------|-------|
| projects.vue | 15 min | 665 |
| bootstrap.vue | 20 min | ~300 |
| references.vue | 25 min | ~400 |
| run.vue | 30 min | ~500 |
| post-process.vue | 25 min | ~400 |
| export.vue | 20 min | ~350 |
| review.vue | 35 min | ~600 |
| **Total** | **~3 hours** | **~3,215** |

---

## Next Steps

1. **Apply pattern to projects.vue** (update template, remove style block)
2. **Create CSS for bootstrap.vue**
3. **Create CSS for references.vue**
4. **Create CSS for run.vue**
5. **Create CSS for remaining pages**
6. **Run lint and fix any issues**
7. **Test all pages**

---

**Ready to continue with the remaining pages?**

The pattern is established. Each page follows the same process:
1. Extract styles to global CSS
2. Replace tokens with canonical ones
3. Remove hardcoded values
4. Update Vue template
5. Document CSS sources

---

*Last updated: 2026-03-23 22:30 (UTC)*
