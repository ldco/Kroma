# 📚 PuppetMaster CSS Documentation

This folder contains comprehensive documentation for the PuppetMaster CSS system.

---

## Documents

| Document                                           | Purpose                              | Audience        |
| -------------------------------------------------- | ------------------------------------ | --------------- |
| [CSS_ARCHITECTURE.md](./CSS_ARCHITECTURE.md)       | Complete CSS system architecture     | All developers  |
| [CSS_QUICK_REFERENCE.md](./CSS_QUICK_REFERENCE.md) | Quick lookup for classes & variables | Daily reference |
| [CSS_CUSTOMIZATION.md](./CSS_CUSTOMIZATION.md)     | How to customize for clients         | Project setup   |

---

## Quick Links

### I need to...

| Task                              | Document                                                                                         |
| --------------------------------- | ------------------------------------------------------------------------------------------------ |
| Understand the layer system       | [CSS_ARCHITECTURE.md#3-layer-system](./CSS_ARCHITECTURE.md#3-layer-system)                       |
| Find available CSS classes        | [CSS_QUICK_REFERENCE.md](./CSS_QUICK_REFERENCE.md)                                               |
| Customize colors for a client     | [CSS_CUSTOMIZATION.md#quick-start-brand-colors](./CSS_CUSTOMIZATION.md#quick-start-brand-colors) |
| Find the right file to edit       | [CSS_ARCHITECTURE.md#8-finding-the-right-file](./CSS_ARCHITECTURE.md#8-finding-the-right-file)   |
| Learn about design tokens         | [CSS_ARCHITECTURE.md#5-design-tokens](./CSS_ARCHITECTURE.md#5-design-tokens)                     |
| Understand responsive breakpoints | [CSS_ARCHITECTURE.md#6-responsive-system](./CSS_ARCHITECTURE.md#6-responsive-system)             |

---

## Core Principles

1. **One file per component** - Easy to find and customize
2. **No scoped styles** - All CSS in global files
3. **CSS variables everywhere** - No magic numbers
4. **Responsive in same file** - Base + media queries together
5. **Logical properties** - Automatic RTL support

---

## Key Files

| File                       | Purpose                           |
| -------------------------- | --------------------------------- |
| `assets/css/main.css`      | Entry point, layer declarations   |
| `colors/primitives.css`    | Brand colors (4 base colors)      |
| `layout/breakpoints.css`   | Responsive breakpoint definitions |
| `common/spacing.css`       | Spacing scale tokens              |
| `typography/variables.css` | Font size tokens                  |

---

## Last Updated

- **Architecture refactor:** 2024-12-19
- **Modular file structure:** 16 new CSS files created
- **Documentation created:** 2024-12-19
