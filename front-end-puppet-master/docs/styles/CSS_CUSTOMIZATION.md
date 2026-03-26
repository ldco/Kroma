# 🎨 CSS Customization Guide

How to customize PuppetMaster styles for client projects.

---

## Quick Start: Brand Colors

The fastest way to customize for a new client is to update the 4 base colors:

```css
/* assets/css/colors/primitives.css */
:root {
  --c-black: oklch(from #2f2f2f l c h); /* Dark color */
  --c-white: oklch(from #f0f0f0 l c h); /* Light color */
  --c-brand: oklch(from #aa0000 l c h); /* Primary brand */
  --c-accent: oklch(from #6366f1 l c h); /* Secondary accent */
}
```

All other colors are auto-calculated from these 4 values using `color-mix()` and `light-dark()`.

---

## Common Customizations

### 1. Header Height

```css
/* assets/css/skeleton/header.css */
:root {
  --header-height: 80px;
  --header-scrolled-height: 60px;
}
```

### 2. Sidebar Width

```css
/* assets/css/layout/admin-sidebar.css */
:root {
  --admin-sidebar-width: 240px;
  --admin-sidebar-collapsed-width: 64px;
}
```

### 3. Container Width

```css
/* assets/css/layout/containers.css */
:root {
  --content-default: min(1280px, 100%);
  --content-narrow: min(768px, 100%);
  --content-wide: min(1536px, 100%);
}
```

### 4. Typography (4-Layer System)

The typography system uses a 4-layer architecture. For client projects, you only need to change **Layer 2 (Brand Fonts)**.

```css
/* assets/css/typography/variables.css - Layer 2 */
:root {
  /* Primary brand font (body text, UI) */
  --font-brand-primary: 'ClientFont', var(--fallback-sans);

  /* Accent font for headings (optional - defaults to primary) */
  --font-brand-accent: 'ClientDisplay', var(--fallback-serif);
}
```

**Important:** Match the fallback stack to your font type:

- Sans-serif font → use `var(--fallback-sans)`
- Serif font → use `var(--fallback-serif)`
- Slab-serif font → use `var(--fallback-slab)`

**Default (PuppetMaster branding):** Montserrat for both primary and accent.

### Font Loading Options

**Option 1: Google Fonts** (recommended for common fonts)

Edit `nuxt.config.ts` to add the font link:

```typescript
// nuxt.config.ts - app.head.link
link: [
  { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
  { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
  {
    rel: 'stylesheet',
    href: 'https://fonts.googleapis.com/css2?family=Open+Sans:wght@400;500;700&family=Playfair+Display:wght@700&display=swap'
  }
]
```

**Option 2: Self-hosted** (for custom brand fonts)

```css
/* assets/css/typography/font-faces.css */
@font-face {
  font-family: 'ClientFont';
  src: url('./fonts/ClientFont-Regular.woff2') format('woff2');
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'ClientFont';
  src: url('./fonts/ClientFont-Bold.woff2') format('woff2');
  font-weight: 700;
  font-style: normal;
  font-display: swap;
}
```

### Multi-Language Font Support

For sites with multiple languages, use `typography/lang-overrides.css`:

```css
/* assets/css/typography/lang-overrides.css */
:lang(he) {
  --font-brand-primary: 'Heebo', 'Noto Sans Hebrew', var(--fallback-sans);
  direction: rtl;
}

:lang(ar) {
  --font-brand-primary: 'Cairo', 'Noto Sans Arabic', var(--fallback-sans);
  direction: rtl;
}
```

The browser automatically applies the correct font based on `<html lang="XX">`.

### 5. Border Radius

```css
/* assets/css/common/effects.css */
:root {
  --radius-sm: 0.375rem; /* 6px */
  --radius-md: 0.5rem; /* 8px */
  --radius-lg: 0.75rem; /* 12px */
  --radius-full: 9999px; /* Pills, avatars */
}
```

---

## File-by-File Reference

| I need to customize...  | Edit this file                       |
| ----------------------- | ------------------------------------ |
| Brand colors            | `colors/primitives.css`              |
| Brand fonts             | `typography/variables.css` (Layer 2) |
| Font sizes              | `typography/variables.css` (Layer 4) |
| Self-hosted fonts       | `typography/font-faces.css`          |
| Language-specific fonts | `typography/lang-overrides.css`      |
| Google Fonts            | `nuxt.config.ts` (app.head.link)     |
| Header                  | `skeleton/header.css`                |
| Footer                  | `skeleton/footer.css`                |
| App sidebar             | `layout/admin-sidebar.css`           |
| App bottom nav          | `skeleton/bottom-nav.css`            |
| Buttons                 | `ui/forms/buttons.css`               |
| Input fields            | `ui/forms/inputs.css`                |
| Cards                   | `ui/content/cards.css`               |
| Tabs                    | `ui/content/tabs.css`                |
| Modals                  | `ui/overlays/modal.css`              |
| Spacing scale           | `common/spacing.css`                 |

---

## Best Practices

### ✅ DO

1. **Use CSS variables** - Never hardcode values

   ```css
   /* Good */
   padding: var(--space-4);
   ```

2. **Use logical properties** - For RTL support

   ```css
   /* Good */
   margin-inline-start: var(--space-4);
   border-block-end: 1px solid var(--l-border);
   ```

3. **Keep responsive rules in same file** - Base + media queries together

   ```css
   /* Good - in admin-sidebar.css */
   .admin-sidebar {
     width: 240px;
   }
   @media (--phone) {
     .admin-sidebar {
       display: none;
     }
   }
   ```

4. **Use rem units** - For scalability
   ```css
   /* Good */
   max-width: 62.5rem; /* 1000px */
   ```

### ❌ DON'T

1. **Don't use scoped styles in Vue components**

   ```vue
   <!-- Bad -->
   <style scoped>
   .custom {
     color: red;
   }
   </style>
   ```

2. **Don't use magic numbers**

   ```css
   /* Bad */
   width: 280px;
   padding: 16px;
   ```

3. **Don't use physical properties** (for RTL)
   ```css
   /* Bad */
   margin-left: 1rem;
   border-top: 1px solid;
   ```

---

## Adding New Components

When adding a new CSS component:

1. **Create a new file** in the appropriate folder:
   - UI element → `ui/content/my-component.css`
   - Form element → `ui/forms/my-component.css`
   - Overlay → `ui/overlays/my-component.css`

2. **Import in index.css**:

   ```css
   /* ui/content/index.css */
   @import './my-component.css';
   ```

3. **Use design tokens**:

   ```css
   .my-component {
     padding: var(--space-4);
     background: var(--l-surface);
     border-radius: var(--radius-md);
     color: var(--l-text);
   }
   ```

4. **Add responsive rules in same file**:
   ```css
   @media (--phone) {
     .my-component {
       padding: var(--space-2);
     }
   }
   ```

---

## Client-Specific Overrides

For heavy custom styling on client projects:

1. Create `assets/css/client/` folder
2. Add client-specific CSS files
3. Import AFTER main.css in nuxt.config.ts

```typescript
// nuxt.config.ts
css: [
  '~/assets/css/main.css',
  '~/assets/css/client/overrides.css' // Client-specific
]
```

This keeps the core framework separate from client customizations.
