# 🚀 CSS Quick Reference

Quick lookup for PuppetMaster CSS classes and variables.

---

## Spacing Classes

```html
<!-- Padding -->
<div class="p-1">...</div>
<!-- 0.25rem (4px) -->
<div class="p-2">...</div>
<!-- 0.5rem (8px) -->
<div class="p-4">...</div>
<!-- 1rem (16px) -->
<div class="px-4">...</div>
<!-- padding-inline: 1rem -->
<div class="py-4">...</div>
<!-- padding-block: 1rem -->

<!-- Margin -->
<div class="m-4">...</div>
<!-- 1rem all sides -->
<div class="mx-auto">...</div>
<!-- margin-inline: auto (center) -->
<div class="mt-4">...</div>
<!-- margin-block-start: 1rem -->

<!-- Gap (for flex/grid) -->
<div class="gap-2">...</div>
<!-- gap: 0.5rem -->
<div class="gap-4">...</div>
<!-- gap: 1rem -->
```

---

## Flexbox Classes

```html
<div class="flex">...</div>
<!-- display: flex -->
<div class="flex-col">...</div>
<!-- flex-direction: column -->
<div class="items-center">...</div>
<!-- align-items: center -->
<div class="justify-center">...</div>
<!-- justify-content: center -->
<div class="justify-between">...</div>
<!-- justify-content: space-between -->
<div class="flex-wrap">...</div>
<!-- flex-wrap: wrap -->
<div class="flex-1">...</div>
<!-- flex: 1 -->
<div class="shrink-0">...</div>
<!-- flex-shrink: 0 -->
```

---

## Grid Classes

```html
<div class="grid">...</div>
<!-- display: grid -->
<div class="grid-cols-2">...</div>
<!-- 2 columns -->
<div class="grid-cols-3">...</div>
<!-- 3 columns -->
<div class="grid-cols-4">...</div>
<!-- 4 columns -->
<div class="grid-auto-fit">...</div>
<!-- auto-fit responsive -->
```

---

## Text Classes

```html
<!-- Sizes -->
<span class="text-xs">...</span>
<!-- 0.75rem -->
<span class="text-sm">...</span>
<!-- 0.875rem -->
<span class="text-base">...</span>
<!-- 1rem -->
<span class="text-lg">...</span>
<!-- 1.125rem -->
<span class="text-xl">...</span>
<!-- 1.25rem -->
<span class="text-2xl">...</span>
<!-- 1.5rem -->

<!-- Alignment -->
<p class="text-center">...</p>
<p class="text-start">...</p>
<p class="text-end">...</p>

<!-- Utilities -->
<span class="truncate">...</span>
<!-- Ellipsis overflow -->
<span class="line-clamp-2">...</span>
<!-- Max 2 lines -->
<span class="font-bold">...</span>
<span class="text-muted">...</span>
<!-- Muted color -->
```

---

## Icon Classes

```html
<IconSettings class="icon-xs" />
<!-- 0.75rem -->
<IconSettings class="icon-sm" />
<!-- 1rem -->
<IconSettings class="icon-md" />
<!-- 1.25rem -->
<IconSettings class="icon-lg" />
<!-- 1.5rem -->
<IconSettings class="icon-xl" />
<!-- 2rem -->
<IconSettings class="icon-2xl" />
<!-- 2.5rem -->
```

---

## Layout Classes

```html
<!-- Containers -->
<div class="container">...</div>
<!-- Max-width centered -->
<div class="container-narrow">...</div>
<!-- Narrower width -->
<div class="container-wide">...</div>
<!-- Wider width -->

<!-- Sections -->
<section class="section">...</section>
<!-- Full-height section -->
<section class="section-hero">...</section>

<!-- Display -->
<div class="hidden">...</div>
<!-- display: none -->
<div class="block">...</div>
<!-- display: block -->
```

---

## Form Classes

```html
<!-- Inputs -->
<input class="input" />
<textarea class="input"></textarea>
<select class="input">
  ...
</select>

<!-- Buttons -->
<button class="btn">Default</button>
<button class="btn btn-primary">Primary</button>
<button class="btn btn-secondary">Secondary</button>
<button class="btn btn-ghost">Ghost</button>
<button class="btn btn-danger">Danger</button>
<button class="btn btn-sm">Small</button>
<button class="btn btn-lg">Large</button>

<!-- Form Groups -->
<div class="form-group">
  <label class="form-label">Label</label>
  <input class="input" />
  <span class="form-hint">Helper text</span>
</div>
```

---

## Content Classes

```html
<!-- Cards -->
<div class="card">...</div>
<div class="card-header">...</div>
<div class="card-body">...</div>

<!-- Tabs -->
<div class="tabs">
  <button class="tab tab--active">Tab 1</button>
  <button class="tab">Tab 2</button>
</div>
<div class="tabs tabs--underline">...</div>

<!-- Badges -->
<span class="badge">Default</span>
<span class="badge badge--success">Success</span>
<span class="badge badge--warning">Warning</span>
<span class="badge badge--danger">Danger</span>

<!-- Avatars -->
<img class="avatar avatar-sm" />
<img class="avatar avatar-md" />
<img class="avatar avatar-lg" />
```

---

## State Classes

```html
<div class="loading-state">Loading...</div>
<div class="empty-state">No items found</div>
```

---

## Accessibility Classes

```html
<span class="sr-only">Screen reader only</span>
<span class="visually-hidden">Hidden but accessible</span>
```

---

## CSS Variables Quick Reference

```css
/* Colors */
var(--c-brand)      /* Brand color */
var(--l-bg)         /* Background */
var(--l-text)       /* Text color */
var(--l-border)     /* Border color */
var(--l-text-muted) /* Muted text */

/* Spacing */
var(--space-1)   /* 0.25rem */
var(--space-2)   /* 0.5rem */
var(--space-4)   /* 1rem */
var(--space-8)   /* 2rem */

/* Typography - Semantic (USE THESE) */
var(--font-body)    /* Body text, paragraphs */
var(--font-heading) /* Headings, titles */
var(--font-ui)      /* Buttons, labels, nav */
var(--font-code)    /* Code blocks, monospace */

/* Typography - Brand (change per-client) */
var(--font-brand-primary) /* Main brand font */
var(--font-brand-accent)  /* Headings/display font */
var(--font-brand-mono)    /* Code font */

/* Typography - Fallbacks (never change) */
var(--fallback-sans)  /* Sans-serif system fonts */
var(--fallback-serif) /* Serif system fonts */
var(--fallback-slab)  /* Slab-serif system fonts */
var(--fallback-mono)  /* Monospace system fonts */

/* Typography - Font Size */
var(--text-sm)   /* 0.875rem */
var(--text-base) /* 1rem */
var(--text-lg)   /* 1.125rem */
var(--heading-1) /* clamp(2.25rem, 5vw, 3.75rem) */
var(--heading-2) /* clamp(1.875rem, 4vw, 3rem) */

/* Sizing */
var(--avatar-sm) /* 32px */
var(--avatar-md) /* 40px */
var(--icon-md)   /* 1.25rem */
```
