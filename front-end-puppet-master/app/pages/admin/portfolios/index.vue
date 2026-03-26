<script setup lang="ts">
/**
 * Admin Portfolios List Page
 *
 * Displays all portfolios with create/edit/delete functionality.
 * Supports multiple locales for name/description.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox, .input-row
 * - ui/content/cards.css: .card-actions
 * - ui/content/badges.css: .badge, .badge-success, .badge-warning
 * - ui/content/portfolio-grid.css: .portfolio-grid, .portfolio-item, .portfolio-item-*
 * - ui/content/tabs.css: .tabs, .tabs--underline, .tab, .is-active, .tab__indicator
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import IconUpload from '~icons/tabler/upload'
import IconPhoto from '~icons/tabler/photo'
import IconPhotoOff from '~icons/tabler/photo-off'
import IconPresentation from '~icons/tabler/presentation'
import type { Portfolio, PortfolioWithItems, PortfolioItem } from '~/types'
import config from '~/puppet-master.config'

interface PortfolioTranslation {
  name: string | null
  description: string | null
}

interface PortfolioWithTranslations extends PortfolioWithItems {
  translations?: Record<string, PortfolioTranslation>
}

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('admin.navPortfolio')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// SSR-safe config access with defensive defaults
const localesConfig = computed(() => config?.locales ?? [])
const defaultLocaleConfig = computed(() => config?.defaultLocale || 'en')

// Available locales from config
const locales = localesConfig.value.map(l => l.code)

// Fetch portfolios with items for preview
const headers = useRequestHeaders(['cookie'])
const {
  data: portfolios,
  pending,
  refresh
} = await useFetch<PortfolioWithTranslations[]>('/api/portfolios', {
  query: { all: 'true', includeItems: 'true' },
  headers
})

// Modal state
const showModal = ref(false)
const editingPortfolio = ref<PortfolioWithTranslations | null>(null)
const saving = ref(false)
const activeLocale = ref(locales[0] || 'en')

// Create empty translations structure for all locales
function createEmptyTranslations(): Record<string, { name: string; description: string }> {
  const translations: Record<string, { name: string; description: string }> = {}
  for (const locale of locales) {
    translations[locale] = { name: '', description: '' }
  }
  return translations
}

const form = reactive({
  slug: '',
  name: '',
  description: '',
  type: 'gallery' as 'gallery' | 'case_study',
  published: false,
  order: 0,
  coverImageUrl: '',
  coverThumbnailUrl: '',
  translations: createEmptyTranslations()
})

// Image upload state
const uploadingCover = ref(false)

function resetForm() {
  form.slug = ''
  form.name = ''
  form.description = ''
  form.type = 'gallery'
  form.published = false
  form.order = 0
  form.coverImageUrl = ''
  form.coverThumbnailUrl = ''
  form.translations = createEmptyTranslations()
  activeLocale.value = locales[0] || 'en'
}

function openCreate() {
  editingPortfolio.value = null
  resetForm()
  showModal.value = true
}

function openEdit(portfolio: PortfolioWithTranslations) {
  editingPortfolio.value = portfolio
  form.slug = portfolio.slug
  form.name = portfolio.name
  form.description = portfolio.description || ''
  form.type = portfolio.type
  form.published = portfolio.published ?? false
  form.order = portfolio.order ?? 0
  form.coverImageUrl = portfolio.coverImageUrl || ''
  form.coverThumbnailUrl = portfolio.coverThumbnailUrl || ''

  // Load translations for all locales
  form.translations = createEmptyTranslations()
  for (const locale of locales) {
    const trans = portfolio.translations?.[locale]
    if (trans) {
      form.translations[locale] = {
        name: trans.name || '',
        description: trans.description || ''
      }
    }
  }

  // Find first locale with content
  const localeWithContent = locales.find(locale => {
    const trans = portfolio.translations?.[locale]
    return trans?.name
  })
  activeLocale.value = localeWithContent || locales[0] || 'en'

  showModal.value = true
}

async function uploadCoverImage(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  uploadingCover.value = true
  try {
    const formData = new FormData()
    formData.append('image', file)

    const result = await $fetch<{ url: string; thumbnailUrl: string }>('/api/upload/image', {
      method: 'POST',
      body: formData
    })

    form.coverImageUrl = result.url
    form.coverThumbnailUrl = result.thumbnailUrl
    toast.success(t('common.uploaded'))
  } catch (e: unknown) {
    const err = e as { data?: { message?: string } }
    toast.error(err.data?.message || t('common.error'))
  } finally {
    uploadingCover.value = false
    input.value = ''
  }
}

// Auto-generate slug from name (using first non-empty translation)
function generateSlug() {
  if (!editingPortfolio.value && !form.slug) {
    // Try form.name first, then check translations
    const name = form.name || locales.map(l => form.translations[l]?.name).find(n => n)
    if (name) {
      form.slug = name
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-|-$/g, '')
    }
  }
}

async function savePortfolio() {
  saving.value = true

  try {
    const payload = {
      slug: form.slug,
      name: form.name,
      description: form.description || null,
      type: form.type,
      published: form.published,
      order: form.order,
      coverImageUrl: form.coverImageUrl || null,
      coverThumbnailUrl: form.coverThumbnailUrl || null,
      translations: form.translations
    }

    if (editingPortfolio.value) {
      await $fetch(`/api/portfolios/${editingPortfolio.value.id}`, { method: 'PUT', body: payload })
    } else {
      await $fetch('/api/portfolios', { method: 'POST', body: payload })
    }

    showModal.value = false
    toast.success(t('common.saved'))
    await refresh()
  } catch (e: any) {
    console.error('Portfolio save error:', e)
    toast.error(e.data?.statusMessage || e.data?.message || e.message || 'Failed to save')
  } finally {
    saving.value = false
  }
}

const deleting = ref<number | null>(null)

async function deletePortfolio(portfolio: Portfolio) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })
  if (!confirmed) return

  deleting.value = portfolio.id
  try {
    await $fetch(`/api/portfolios/${portfolio.id}`, { method: 'DELETE' })
    await refresh()
  } catch (e: any) {
    toast.error(e.data?.message || 'Failed to delete')
  } finally {
    deleting.value = null
  }
}

function getTypeLabel(type: string) {
  return type === 'gallery' ? t('admin.portfolioGallery') : t('admin.portfolioCaseStudy')
}

// Translation helpers
function getLocaleName(code: string): string {
  const locale = localesConfig.value.find(l => l.code === code)
  return locale?.name || code.toUpperCase()
}

function hasTranslation(portfolio: PortfolioWithTranslations, locale: string): boolean {
  // Default locale content is stored in base fields (name, description)
  const defaultLocale = defaultLocaleConfig.value || 'en'
  if (locale === defaultLocale) {
    return !!(portfolio.name || portfolio.translations?.[locale]?.name)
  }
  return !!(portfolio.translations?.[locale]?.name)
}

function hasAllTranslations(portfolio: PortfolioWithTranslations): boolean {
  return locales.every(locale => hasTranslation(portfolio, locale))
}

function getMissingTranslations(portfolio: PortfolioWithTranslations): string[] {
  return locales.filter(locale => !hasTranslation(portfolio, locale))
}

function formLocaleHasContent(locale: string): boolean {
  const trans = form.translations[locale]
  return !!(trans?.name)
}

// Track failed images
const failedImages = ref(new Set<string>())

function onImageError(key: string) {
  failedImages.value.add(key)
}

// Get items with valid thumbnails for preview grid
function getPreviewItems(items: PortfolioItem[] | undefined) {
  if (!items) return []
  return items
    .filter(item => item.thumbnailUrl && !failedImages.value.has(`item-${item.id}`))
    .slice(0, 4)
}

// Check if cover image is valid
function hasCoverImage(portfolio: PortfolioWithItems) {
  return portfolio.coverThumbnailUrl && !failedImages.value.has(`cover-${portfolio.id}`)
}
</script>

<template>
  <div class="admin-portfolios">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.portfolios') }}</h1>
      <button class="btn btn-primary" @click="openCreate">
        <IconPlus />
        {{ t('admin.addPortfolio') }}
      </button>
    </div>

    <!-- Loading -->
    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <!-- Empty state -->
    <div v-else-if="!portfolios?.length" class="empty-state">
      <p>{{ t('admin.noPortfolios') }}</p>
    </div>

    <!-- Portfolio grid -->
    <div v-else class="portfolio-grid">
      <NuxtLink
        v-for="portfolio in portfolios"
        :key="portfolio.id"
        :to="`/admin/portfolios/${portfolio.id}`"
        class="portfolio-item card"
      >
        <div class="portfolio-item-image">
          <!-- Show cover image if set and not failed -->
          <img
            v-if="hasCoverImage(portfolio)"
            :src="portfolio.coverThumbnailUrl!"
            :alt="portfolio.name"
            @error="onImageError(`cover-${portfolio.id}`)"
          />
          <!-- Show item previews grid if no cover but has valid items -->
          <div
            v-else-if="getPreviewItems(portfolio.items).length"
            class="portfolio-preview-grid"
          >
            <img
              v-for="item in getPreviewItems(portfolio.items)"
              :key="item.id"
              :src="item.thumbnailUrl!"
              :alt="portfolio.name"
              class="portfolio-preview-thumb"
              @error="onImageError(`item-${item.id}`)"
            />
          </div>
          <!-- Empty placeholder when no valid images -->
          <div v-else class="portfolio-item-placeholder">
            <IconPhotoOff v-if="failedImages.size > 0" />
            <IconPhoto v-else-if="portfolio.type === 'gallery'" />
            <IconPresentation v-else />
          </div>
          <span v-if="!portfolio.published" class="badge badge-warning status-badge">Draft</span>
          <span class="badge badge-secondary type-badge">{{ getTypeLabel(portfolio.type) }}</span>
        </div>
        <div class="portfolio-item-info">
          <h3 class="portfolio-item-title">{{ portfolio.name }}</h3>
          <p v-if="portfolio.description" class="portfolio-item-description">
            {{ portfolio.description }}
          </p>
          <!-- Translation status -->
          <div v-if="!hasAllTranslations(portfolio)" class="mt-1">
            <span class="text-warning text-xs">
              {{ t('admin.missingTranslations') }}: {{ getMissingTranslations(portfolio).map(l => l.toUpperCase()).join(', ') }}
            </span>
          </div>
        </div>
        <div class="card-actions" @click.prevent>
          <button class="btn btn-sm btn-secondary" @click="openEdit(portfolio)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button
            class="btn btn-sm btn-ghost text-error"
            @click="deletePortfolio(portfolio)"
            :disabled="deleting === portfolio.id"
          >
            <IconTrash />
          </button>
        </div>
      </NuxtLink>
    </div>

    <!-- Modal -->
    <Teleport to="body">
      <div v-if="showModal" class="modal-backdrop" @click.self="showModal = false">
        <div class="modal modal--lg">
          <header class="modal-header">
            <h2>{{ editingPortfolio ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="savePortfolio">
            <!-- Cover image upload -->
            <div class="form-group">
              <label class="form-label">{{ t('admin.coverImage') }}</label>
              <div class="input-row">
                <input
                  v-model="form.coverImageUrl"
                  type="text"
                  class="input"
                  placeholder="https://..."
                />
                <label class="btn btn-secondary">
                  <IconUpload />
                  <input
                    type="file"
                    accept="image/*"
                    class="sr-only"
                    :disabled="uploadingCover"
                    @change="uploadCoverImage"
                  />
                </label>
              </div>
              <div v-if="form.coverImageUrl" class="mt-2">
                <img :src="form.coverImageUrl" alt="Preview" class="image-preview" />
              </div>
            </div>

            <!-- Non-translatable fields -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label">{{ t('admin.portfolioName') }} *</label>
                <input
                  v-model="form.name"
                  type="text"
                  class="input"
                  required
                  @blur="generateSlug"
                />
              </div>
              <div class="form-group">
                <label class="form-label">{{ t('admin.slug') }} *</label>
                <input
                  v-model="form.slug"
                  type="text"
                  class="input"
                  required
                  pattern="[a-z0-9-]+"
                />
              </div>
              <div class="form-group">
                <label class="form-label">{{ t('admin.portfolioType') }} *</label>
                <select v-model="form.type" class="input" :disabled="!!editingPortfolio">
                  <option value="gallery">{{ t('admin.portfolioGallery') }}</option>
                  <option value="case_study">{{ t('admin.portfolioCaseStudy') }}</option>
                </select>
              </div>
              <div class="form-group">
                <label class="form-label">{{ t('admin.order') }}</label>
                <input v-model.number="form.order" type="number" class="input" min="0" />
              </div>
            </div>

            <div class="form-group">
              <label class="form-label">{{ t('admin.description') }}</label>
              <textarea v-model="form.description" class="input" rows="3"></textarea>
            </div>

            <div class="form-group">
              <label class="form-checkbox">
                <input v-model="form.published" type="checkbox" />
                <span>{{ t('admin.published') }}</span>
              </label>
            </div>

            <!-- Language tabs for translatable content -->
            <div class="form-divider">
              <span>{{ t('admin.translations') }}</span>
            </div>

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
                <span
                  v-if="!formLocaleHasContent(locale)"
                  class="tab__indicator tab__indicator--warning"
                  :title="t('admin.missingTranslations')"
                ></span>
              </button>
            </div>

            <!-- Translation fields for active locale -->
            <div v-for="locale in locales" v-show="activeLocale === locale" :key="locale">
              <div class="form-group">
                <label class="form-label">{{ t('admin.portfolioName') }} ({{ getLocaleName(locale) }})</label>
                <input
                  v-model="form.translations[locale].name"
                  type="text"
                  class="input"
                  @blur="generateSlug"
                />
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.description') }} ({{ getLocaleName(locale) }})</label>
                <textarea
                  v-model="form.translations[locale].description"
                  class="input"
                  rows="3"
                ></textarea>
              </div>
            </div>
          </form>

          <footer class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="showModal = false">
              {{ t('common.cancel') }}
            </button>
            <button
              type="submit"
              class="btn btn-primary"
              :disabled="saving"
              @click="savePortfolio"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<!--
  Uses global CSS classes:
  - admin/index.css: .portfolio-grid, .portfolio-item, .portfolio-item-*
  - ui/overlays/index.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
-->
