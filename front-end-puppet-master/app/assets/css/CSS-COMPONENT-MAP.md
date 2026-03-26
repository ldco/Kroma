# CSS Component Mapping

This document binds Vue components to their CSS files. **Always use existing CSS classes** - never invent new styles.

---

## Quick Reference

| Pattern | CSS File | Classes |
|---------|----------|---------|
| Modal | `ui/overlays/modal.css` | `.modal-backdrop`, `.modal`, `.modal-header`, `.modal-body`, `.modal-footer` |
| Form | `ui/forms/inputs.css` | `.form-group`, `.form-label`, `.input`, `.form-checkbox`, `.form-row--2col` |
| Button | `ui/forms/buttons.css` | `.btn`, `.btn-primary`, `.btn-secondary`, `.btn-ghost`, `.btn-sm` |
| Card | `ui/content/cards.css` | `.card`, `.card-header`, `.card-body`, `.card-actions` |
| Tabs | `ui/content/tabs.css` | `.tabs`, `.tab`, `.is-active`, `.tabs--underline` |
| Badge | `ui/content/badges.css` | `.badge`, `.badge-success`, `.badge-warning`, `.badge-error` |
| Upload | `ui/forms/inputs.css` | `.input-row` (URL + button) |
| Upload Area | `ui/content/portfolio-grid.css` | `.upload-area`, `.upload-preview`, `.upload-placeholder` |

---

## Admin Pages

All admin pages use these shared patterns:

### Page Layout
**CSS:** `layout/admin-content.css`
```html
<div class="admin-{module}">
  <div class="page-header">
    <h1 class="page-title">Title</h1>
    <button class="btn btn-primary">Add</button>
  </div>
  <!-- content -->
</div>
```

### Modal Structure
**CSS:** `ui/overlays/modal.css`, `ui/forms/inputs.css`
```html
<Teleport to="body">
  <div class="modal-backdrop">
    <div class="modal modal--lg">
      <header class="modal-header">
        <h2>Title</h2>
        <button class="btn btn-ghost btn-sm"><IconX /></button>
      </header>

      <form class="modal-body">
        <!-- Form groups -->
      </form>

      <footer class="modal-footer">
        <button class="btn btn-secondary">Cancel</button>
        <button class="btn btn-primary">Save</button>
      </footer>
    </div>
  </div>
</Teleport>
```

**Modal sizes:**
- Default: 500px
- `.modal--lg`: 700px
- `.modal--xl`: 900px
- `.modal--sm`: 400px

### Form Structure
**CSS:** `ui/forms/inputs.css`
```html
<form class="modal-body">
  <!-- Error message -->
  <div v-if="error" class="form-error">{{ error }}</div>

  <!-- Two column layout -->
  <div class="form-row form-row--2col">
    <div class="form-group">
      <label class="form-label">Name *</label>
      <input v-model="form.name" class="input" required />
    </div>
    <div class="form-group">
      <label class="form-label">Slug</label>
      <input v-model="form.slug" class="input" />
    </div>
  </div>

  <!-- Single field -->
  <div class="form-group">
    <label class="form-label">Description</label>
    <textarea v-model="form.description" class="input" rows="3"></textarea>
  </div>

  <!-- Checkbox -->
  <div class="form-group">
    <label class="form-checkbox">
      <input v-model="form.published" type="checkbox" />
      <span>Published</span>
    </label>
  </div>

  <!-- Section divider (before language tabs) -->
  <div class="form-divider"><span>Content</span></div>
</form>
```

### Language Tabs (Multilingual Content)
**CSS:** `ui/content/tabs.css`
```html
<div class="tabs tabs--underline mb-4">
  <button
    v-for="locale in locales"
    :key="locale"
    type="button"
    class="tab"
    :class="{ 'is-active': activeLocale === locale }"
    @click="activeLocale = locale"
  >
    {{ getLocaleName(locale) }}
    <span v-if="!hasContent(locale)" class="tab__indicator tab__indicator--warning"></span>
  </button>
</div>
```

### Card Actions
**CSS:** `ui/content/cards.css`
```html
<div class="card-actions" @click.prevent>
  <button class="btn btn-sm btn-secondary" @click="openEdit(item)">
    <IconEdit /> {{ t('common.edit') }}
  </button>
  <button class="btn btn-sm btn-ghost text-error" @click="deleteItem(item)">
    <IconTrash />
  </button>
</div>
```

### File Upload - URL Input Pattern
**CSS:** `ui/forms/inputs.css`
```html
<div class="form-group">
  <label class="form-label">Image</label>
  <div class="input-row">
    <input v-model="form.imageUrl" type="text" class="input" placeholder="https://..." />
    <label class="btn btn-secondary">
      <IconUpload />
      <input type="file" accept="image/*" class="sr-only" @change="uploadImage" />
    </label>
  </div>
  <div v-if="form.imageUrl" class="mt-2">
    <img :src="form.imageUrl" alt="Preview" class="image-preview" />
  </div>
</div>
```

### File Upload - Click Area Pattern
**CSS:** `ui/content/portfolio-grid.css`
```html
<label class="upload-area">
  <input type="file" accept="image/*" class="sr-only" @change="uploadImage" />
  <img v-if="form.imageUrl" :src="form.imageUrl" class="upload-preview" />
  <div v-else class="upload-placeholder">
    <IconUpload />
    <span>Click to upload</span>
  </div>
</label>
```

---

## Admin Module-Specific Styles

Each admin module has specific grid/list styles in `ui/admin/pages.css`:

| Module | Grid Class | Item Class |
|--------|------------|------------|
| Clients | `.clients-admin-grid` | `.clients-admin-card__*` |
| Team | `.team-admin-grid` | `.team-admin-card__*` |
| Features | `.features-admin-list` | `.features-admin-item__*` |
| Testimonials | `.testimonials-admin-grid` | `.testimonials-admin-item__*` |
| FAQ | `.faq-admin-list` | `.faq-admin-item__*` |
| Blog | `.blog-admin-list` | `.blog-admin-item__*` |

---

## Section Components (Public Site)

Each section has a dedicated CSS file in `ui/content/`:

| Component | CSS File |
|-----------|----------|
| `SectionAbout.vue` | `ui/content/about.css` |
| `SectionBlog.vue` | `ui/content/blog.css` |
| `SectionClients.vue` | `ui/content/clients.css` |
| `SectionContact.vue` | `ui/content/contact.css` |
| `SectionFaq.vue` | `ui/content/faq.css` |
| `SectionFeatures.vue` | `ui/content/features.css` |
| `SectionPricing.vue` | `ui/content/pricing.css` |
| `SectionTeam.vue` | `ui/content/team.css` |
| `SectionTestimonials.vue` | `ui/content/testimonials.css` |

---

## Button Patterns

**CSS:** `ui/forms/buttons.css`

| Pattern | Classes | Use Case |
|---------|---------|----------|
| Primary action | `.btn .btn-primary` | Submit, Create |
| Secondary action | `.btn .btn-secondary` | Cancel, Edit |
| Danger action | `.btn .btn-ghost .text-error` | Delete |
| Small button | `.btn .btn-sm .btn-*` | In cards, tables |
| Icon only | `.btn .btn-ghost .btn-sm` | Close modal |

---

## How to Use This System

1. **Before writing HTML**, check this map for the correct CSS classes
2. **Never invent new classes** - if a pattern doesn't exist, add it to the appropriate CSS file
3. **Add reference comments** to Vue components:
```vue
<!--
  Uses CSS from:
  - ui/overlays/modal.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
  - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col
  - ui/admin/pages.css: .{module}-admin-*
-->
```

4. **Follow the structure** - modal header/body/footer, form layout, button patterns
5. **Consistency** - all admin pages must look identical in structure
