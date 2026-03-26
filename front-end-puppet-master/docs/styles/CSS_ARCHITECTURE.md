# 🎨 Puppet Master - CSS Architecture

**Version:** 1.0
**Last Updated:** 2024-12-19

---

## Table of Contents

1. [Overview](#1-overview)
2. [Application Modes & Visual Modes](#2-application-modes--visual-modes)
3. [Layer System](#3-layer-system)
4. [File Structure](#4-file-structure)
5. [Design Tokens](#5-design-tokens)
6. [Responsive System](#6-responsive-system)
7. [Core Rules](#7-core-rules)
8. [Finding the Right File](#8-finding-the-right-file)

---

## 1. Overview

Puppet Master uses a **pure CSS architecture** with no frameworks (no Tailwind, no Bootstrap).

### Key Principles

| Principle                    | Description                                          |
| ---------------------------- | ---------------------------------------------------- |
| **One file per component**   | Each visual component has its own CSS file           |
| **No scoped styles**         | All styles in global CSS files, not `<style scoped>` |
| **CSS variables everywhere** | No magic numbers, all values are design tokens       |
| **Responsive in same file**  | Base styles + media queries together                 |
| **Logical properties**       | RTL support via CSS logical properties               |

### Entry Point

```css
/* assets/css/main.css */
@layer reset, primitives, semantic, components, utilities;

@import './reset.css' layer(reset);
@import './colors/index.css' layer(primitives);
@import './typography/index.css' layer(primitives);
@import './layout/index.css' layer(components);
@import './animations/index.css' layer(components);
@import './skeleton/index.css' layer(components);
@import './ui/index.css' layer(components);
@import './common/index.css' layer(utilities);
@import './layout/responsive.css' layer(utilities);
```

---

## 2. Application Modes & Visual Modes

### Application Modes (4 modes)

| Mode            | Website Portion        | App Portion                        |
| --------------- | ---------------------- | ---------------------------------- |
| `app-only`      | ❌ None                | ✅ App mode (sidebar + bottom nav) |
| `website-app`   | ✅ Website (hamburger) | ✅ App mode (visible login button) |
| `website-admin` | ✅ Website (hamburger) | ✅ App mode (hidden at `/admin`)   |
| `website-only`  | ✅ Website (hamburger) | ❌ None                            |

### Visual Modes (2 modes)

| Visual Mode | Desktop           | Mobile                            |
| ----------- | ----------------- | --------------------------------- |
| **Website** | Horizontal header | Hamburger menu (slide-out drawer) |
| **App**     | Vertical sidebar  | Bottom navigation bar             |

**Key distinction:**

- **Website mode** = Hamburger menu on mobile (ALWAYS)
- **App mode** = Bottom nav bar on mobile (ALWAYS)
- **Admin panel** = App mode (it IS app mode, not a separate visual mode)

### Website Sub-Modes

| Mode         | Navigation                                  | URL Structure                 |
| ------------ | ------------------------------------------- | ----------------------------- |
| **Onepager** | Scroll-based anchors (`#about`, `#contact`) | Single page with sections     |
| **SPA**      | Route-based (`/about`, `/contact`)          | Multiple pages via Vue Router |

Controlled by `features.onepager` in `puppet-master.config.ts`.

---

## 3. Layer System

### Cascade Priority (Low → High)

```css
@layer reset, primitives, semantic, components, utilities;
```

| Layer        | Purpose                    | Example                   |
| ------------ | -------------------------- | ------------------------- |
| `reset`      | Normalize browser defaults | CSS reset                 |
| `primitives` | Raw design tokens          | `--c-brand: #aa0000`      |
| `semantic`   | Calculated values          | `--l-bg: light-dark(...)` |
| `components` | UI element styling         | `.admin-sidebar { }`      |
| `utilities`  | Override helpers           | `.flex`, `.hidden`        |

### How Layers Work

```css
/* Lower layer - can be overridden */
@layer components {
  .button {
    background: var(--c-brand);
  }
}

/* Higher layer - overrides components without !important */
@layer utilities {
  .bg-transparent {
    background: transparent;
  }
}
```

---

## 4. File Structure

```
assets/css/
├── main.css                    # Entry point
├── reset.css                   # Browser reset
│
├── colors/                     # Color system
│   ├── index.css
│   ├── primitives.css          # Base colors
│   └── auto.css                # Auto-calculated colors
│
├── typography/                 # 4-Layer Font System
│   ├── index.css               # Entry point
│   ├── variables.css           # Layers 1-4: fallbacks, brand, semantic, sizes
│   ├── font-faces.css          # @font-face for self-hosted fonts
│   ├── lang-overrides.css      # Per-language font overrides (:lang)
│   ├── base.css                # Apply fonts to HTML elements
│   └── fonts/                  # Self-hosted font files (.woff2)
│
├── layout/                     # Page structure
│   ├── index.css
│   ├── page.css                # Page wrapper, breakpoint vars
│   ├── breakpoints.css         # Custom media queries
│   ├── responsive.css          # Responsive utilities
│   ├── sections.css            # Full-height sections
│   ├── containers.css          # Max-width containers
│   ├── grid.css                # Grid system
│   ├── admin-sidebar.css       # Admin sidebar
│   ├── admin-content.css       # Admin content area
│   └── admin-header.css        # Admin mobile header
│
├── skeleton/                   # Site STRUCTURAL CSS (header/footer/nav)
│   ├── index.css               # ⚠️ "Skeleton" = structural, NOT loading placeholders!
│   ├── header.css              # Site header structure
│   ├── footer.css              # Site footer structure
│   ├── nav.css                 # Desktop navigation
│   ├── mobile-nav.css          # Mobile drawer navigation
│   ├── bottom-nav.css          # App bottom navigation (Material Design)
│   └── social-nav.css          # Social icons navigation
│
├── common/                     # Utility classes
│   ├── index.css
│   ├── utilities.css           # Display, visibility
│   ├── spacing.css             # Spacing tokens
│   ├── flexbox.css             # Flex utilities
│   ├── grid.css                # Grid utilities
│   ├── sizing.css              # Width/height
│   ├── icons.css               # Icon sizing
│   ├── text.css                # Text utilities
│   ├── accessibility.css       # A11y helpers
│   ├── effects.css             # Shadows, transitions
│   ├── scrollbars.css          # Scrollbar styles
│   └── edge-cases.css          # Browser fixes
│
├── ui/                         # UI components
│   ├── index.css
│   ├── hamburger.css
│   ├── forms/                  # Form elements
│   │   ├── inputs.css
│   │   ├── buttons.css
│   │   └── search.css
│   ├── content/                # Content elements
│   │   ├── cards.css
│   │   ├── tabs.css
│   │   ├── badges.css
│   │   ├── avatars.css
│   │   ├── state-indicators.css
│   │   ├── inbox.css
│   │   └── portfolio-grid.css
│   └── overlays/               # Overlays
│       ├── modal.css
│       ├── lightbox.css
│       ├── confirm.css
│       └── toast.css
│
└── animations/                 # Animations
    ├── index.css
    ├── keyframes.css
    └── transitions.css
```

---

## 5. Design Tokens

### Spacing Scale

| Token        | Value   | Pixels |
| ------------ | ------- | ------ |
| `--space-1`  | 0.25rem | 4px    |
| `--space-2`  | 0.5rem  | 8px    |
| `--space-3`  | 0.75rem | 12px   |
| `--space-4`  | 1rem    | 16px   |
| `--space-6`  | 1.5rem  | 24px   |
| `--space-8`  | 2rem    | 32px   |
| `--space-12` | 3rem    | 48px   |
| `--space-16` | 4rem    | 64px   |
| `--space-24` | 6rem    | 96px   |
| `--space-32` | 8rem    | 128px  |

### Color Tokens

| Token            | Purpose                  |
| ---------------- | ------------------------ |
| `--c-black`      | Dark text/backgrounds    |
| `--c-white`      | Light text/backgrounds   |
| `--c-brand`      | Primary brand color      |
| `--c-accent`     | Secondary accent color   |
| `--l-bg`         | Current theme background |
| `--l-text`       | Current theme text       |
| `--l-surface`    | Elevated surfaces        |
| `--l-border`     | Border color             |
| `--l-text-muted` | Muted/secondary text     |

### Typography System (4-Layer Architecture)

The typography system uses a 4-layer architecture for maximum flexibility:

**Layer 1: Fallback Stacks** (never change per-client)
| Token | Use |
|-------|-----|
| `--fallback-sans` | Sans-serif system fonts |
| `--fallback-serif` | Serif system fonts |
| `--fallback-slab` | Slab-serif system fonts |
| `--fallback-mono` | Monospace system fonts |

**Layer 2: Brand Fonts** (change per-client)
| Token | Use |
|-------|-----|
| `--font-brand-primary` | Main brand font |
| `--font-brand-accent` | Headings/display font |
| `--font-brand-mono` | Code font |

**Layer 3: Semantic Aliases** (use these in CSS)
| Token | Use |
|-------|-----|
| `--font-body` | Body text, paragraphs |
| `--font-heading` | Headings, titles |
| `--font-ui` | Buttons, labels, nav |
| `--font-code` | Code blocks, monospace |

**Layer 4: Sizes**
| Token | Value | Use |
|-------|-------|-----|
| `--text-xs` | 0.75rem | Small labels |
| `--text-sm` | 0.875rem | Secondary text |
| `--text-base` | 1rem | Body text |
| `--text-lg` | 1.125rem | Large body |
| `--text-xl` | 1.25rem | Subheadings |
| `--text-2xl` | 1.5rem | Section titles |
| `--heading-1` to `--heading-4` | clamp() | Fluid headings |

### Avatar Tokens

| Token         | Use                    |
| ------------- | ---------------------- |
| `--avatar-xs` | 24px - Tiny avatars    |
| `--avatar-sm` | 32px - Small avatars   |
| `--avatar-md` | 40px - Default avatars |
| `--avatar-lg` | 48px - Large avatars   |
| `--avatar-xl` | 64px - Extra large     |

### Icon Tokens

| Token        | Use                     |
| ------------ | ----------------------- |
| `--icon-xs`  | 0.75rem - Tiny icons    |
| `--icon-sm`  | 1rem - Small icons      |
| `--icon-md`  | 1.25rem - Default icons |
| `--icon-lg`  | 1.5rem - Large icons    |
| `--icon-xl`  | 2rem - Extra large      |
| `--icon-2xl` | 2.5rem - Hero icons     |

---

## 6. Responsive System

### Breakpoints (Tailwind-aligned, industry standard)

| Name    | Query                    | Target                 |
| ------- | ------------------------ | ---------------------- |
| Phone   | `max-width: 639px`       | Portrait phones        |
| Tablet  | `min-width: 640px`       | Landscape phones, tablets, small laptops |
| Desktop | `min-width: 1024px`      | Laptops, desktops      |
| Large   | `min-width: 1280px`      | Large desktops, external monitors |

### Custom Media Queries

```css
/* Defined in layout/breakpoints.css */
@custom-media --phone (max-width: 639px);
@custom-media --tablet (min-width: 640px);
@custom-media --desktop (min-width: 1024px);
@custom-media --large (min-width: 1280px);
@custom-media --tablet-only (min-width: 640px) and (max-width: 1023px);
@custom-media --below-desktop (max-width: 1023px);
```

### Usage in Components

```css
/* admin-sidebar.css - Example of responsive in same file */
.admin-sidebar {
  display: flex; /* Base - always defined */
  width: var(--admin-sidebar-width);
}

@media (--phone) {
  .admin-sidebar {
    display: none; /* Hide on phones */
  }
}

@media (--tablet) {
  .admin-sidebar {
    width: var(--admin-sidebar-collapsed-width);
  }
}
```

---

## 7. Core Rules

### ❌ NEVER Do

```vue
<!-- WRONG: Scoped styles in components -->
<style scoped>
.my-component {
  color: red;
}
</style>

<!-- WRONG: Magic numbers -->
<style>
.sidebar {
  width: 280px;
  padding: 16px;
}
</style>
```

### ✅ ALWAYS Do

```css
/* CORRECT: Use CSS variables */
.sidebar {
  width: var(--admin-sidebar-width);
  padding: var(--space-4);
}

/* CORRECT: Use logical properties for RTL */
.sidebar {
  margin-inline-start: var(--space-4);
  border-block-end: 1px solid var(--l-border);
}
```

### Exception

Scoped styles are ONLY allowed for **custom client sections with heavy custom graphics**.

---

## 8. Finding the Right File

### Quick Reference

| I want to change...     | Edit this file                       |
| ----------------------- | ------------------------------------ |
| Brand colors            | `colors/primitives.css`              |
| Brand fonts             | `typography/variables.css` (Layer 2) |
| Font sizes              | `typography/variables.css` (Layer 4) |
| Self-hosted fonts       | `typography/font-faces.css`          |
| Language-specific fonts | `typography/lang-overrides.css`      |
| Google Fonts            | `nuxt.config.ts` (app.head.link)     |
| Site header             | `skeleton/header.css`                |
| Site footer             | `skeleton/footer.css`                |
| Admin sidebar           | `layout/admin-sidebar.css`           |
| Admin mobile header     | `layout/admin-header.css`            |
| Button styles           | `ui/forms/buttons.css`               |
| Input fields            | `ui/forms/inputs.css`                |
| Card components         | `ui/content/cards.css`               |
| Tab components          | `ui/content/tabs.css`                |
| Modal dialogs           | `ui/overlays/modal.css`              |
| Icon sizes              | `common/icons.css`                   |
| Text utilities          | `common/text.css`                    |
| Flexbox utilities       | `common/flexbox.css`                 |
| Spacing                 | `common/spacing.css`                 |
| Breakpoints             | `layout/breakpoints.css`             |

---

_This document should be updated when CSS architecture changes._
