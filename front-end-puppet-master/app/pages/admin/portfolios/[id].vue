<script setup lang="ts">
/**
 * Admin Portfolio Detail Page
 *
 * Manages items within a portfolio.
 * Different UI for gallery (media grid) vs case_study (project cards).
 * Supports multiple locales for case study translatable fields.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal--lg, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox
 * - ui/content/cards.css: .card, .card-body, .card-actions
 * - ui/content/portfolio-grid.css: .media-grid, .media-item, .media-item-*, .upload-area, .type-selector
 * - ui/content/tabs.css: .tabs, .tabs--underline, .tab, .is-active, .tab__indicator
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import IconArrowLeft from '~icons/tabler/arrow-left'
import IconPhoto from '~icons/tabler/photo'
import IconPhotoOff from '~icons/tabler/photo-off'
import IconVideo from '~icons/tabler/video'
import IconLink from '~icons/tabler/link'
import IconUpload from '~icons/tabler/upload'
import type { PortfolioWithItems, PortfolioItem } from '~/types'
import config from '~/puppet-master.config'

interface ItemTranslation {
  title: string | null
  description: string | null
  content: string | null
  category: string | null
}

interface PortfolioItemWithTranslations extends PortfolioItem {
  translations?: Record<string, ItemTranslation>
}

interface PortfolioWithItemTranslations extends PortfolioWithItems {
  items?: PortfolioItemWithTranslations[]
}

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

// SSR-safe config access with defensive defaults
const localesConfig = computed(() => config?.locales ?? [])
const defaultLocaleConfig = computed(() => config?.defaultLocale || 'en')

// Available locales from config
const locales = localesConfig.value.map(l => l.code)

useHead({
  title: () => `${t('admin.navPortfolio')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()
const route = useRoute()

// Use computed for reactive route params
const portfolioId = computed(() => route.params.id as string)

// Fetch portfolio with items - use computed URL for reactivity on route change
const headers = useRequestHeaders(['cookie'])
const {
  data: portfolio,
  pending,
  refresh
} = await useFetch<PortfolioWithItemTranslations>(() => `/api/portfolios/${portfolioId.value}`, {
  query: { includeItems: 'true' },
  headers,
  watch: [portfolioId]
})

// Modal state
const showItemModal = ref(false)
const editingItem = ref<PortfolioItemWithTranslations | null>(null)
const activeLocale = ref(locales[0] || 'en')

// Create empty translations structure for all locales
function createEmptyItemTranslations(): Record<string, { title: string; description: string; content: string; category: string }> {
  const translations: Record<string, { title: string; description: string; content: string; category: string }> = {}
  for (const locale of locales) {
    translations[locale] = { title: '', description: '', content: '', category: '' }
  }
  return translations
}

// Form for gallery items
const galleryForm = reactive({
  itemType: 'image' as 'image' | 'video' | 'link',
  mediaUrl: '',
  caption: '',
  published: true,
  order: 0
})

// Form for case study items
const caseStudyForm = reactive({
  slug: '',
  title: '',
  description: '',
  content: '',
  category: '',
  tags: '',
  coverUrl: '',
  published: true,
  order: 0,
  translations: createEmptyItemTranslations()
})

// Image/video upload state
const mediaFile = ref<File | null>(null)
const mediaPreview = ref('')
const uploading = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

function triggerFileInput() {
  fileInput.value?.click()
}

function resetForms() {
  galleryForm.itemType = 'image'
  galleryForm.mediaUrl = ''
  galleryForm.caption = ''
  galleryForm.published = true
  galleryForm.order = 0

  caseStudyForm.slug = ''
  caseStudyForm.title = ''
  caseStudyForm.description = ''
  caseStudyForm.content = ''
  caseStudyForm.category = ''
  caseStudyForm.tags = ''
  caseStudyForm.coverUrl = ''
  caseStudyForm.published = true
  caseStudyForm.order = 0
  caseStudyForm.translations = createEmptyItemTranslations()
  activeLocale.value = locales[0] || 'en'

  mediaFile.value = null
  mediaPreview.value = ''
  // Clear file input so the same file can be selected again
  if (fileInput.value) {
    fileInput.value.value = ''
  }
}

function openAddItem() {
  editingItem.value = null
  resetForms()
  showItemModal.value = true
}

function openEditItem(item: PortfolioItemWithTranslations) {
  editingItem.value = item
  resetForms()

  if (item.itemType === 'case_study') {
    caseStudyForm.slug = item.slug || ''
    caseStudyForm.title = item.title || ''
    caseStudyForm.description = item.description || ''
    caseStudyForm.content = item.content || ''
    caseStudyForm.category = item.category || ''
    // tags is stored as JSON string in DB
    caseStudyForm.tags = item.tags || ''
    caseStudyForm.coverUrl = item.thumbnailUrl || item.mediaUrl || ''
    caseStudyForm.published = item.published ?? true
    caseStudyForm.order = item.order ?? 0

    // Load translations for all locales
    caseStudyForm.translations = createEmptyItemTranslations()
    for (const locale of locales) {
      const trans = item.translations?.[locale]
      if (trans) {
        caseStudyForm.translations[locale] = {
          title: trans.title || '',
          description: trans.description || '',
          content: trans.content || '',
          category: trans.category || ''
        }
      }
    }

    // Find first locale with content
    const localeWithContent = locales.find(locale => {
      const trans = item.translations?.[locale]
      return trans?.title
    })
    activeLocale.value = localeWithContent || locales[0] || 'en'
  } else {
    galleryForm.itemType = item.itemType as 'image' | 'video' | 'link'
    galleryForm.mediaUrl = item.mediaUrl || ''
    galleryForm.caption = item.caption || ''
    galleryForm.published = item.published ?? true
    galleryForm.order = item.order ?? 0
    mediaPreview.value = item.thumbnailUrl || item.mediaUrl || ''
  }

  showItemModal.value = true
}

function handleMediaSelect(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files?.[0]) {
    mediaFile.value = input.files[0]
    mediaPreview.value = URL.createObjectURL(input.files[0])
    // Auto-detect type
    if (input.files[0].type.startsWith('video/')) {
      galleryForm.itemType = 'video'
    } else {
      galleryForm.itemType = 'image'
    }
  }
}

// Upload cover image for case study
const uploadingCover = ref(false)
async function handleCoverUpload(e: Event) {
  const input = e.target as HTMLInputElement
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

    caseStudyForm.coverUrl = result.thumbnailUrl || result.url
    toast.success(t('common.uploaded'))
  } catch (err: any) {
    toast.error(err.data?.message || t('common.error'))
  } finally {
    uploadingCover.value = false
    input.value = ''
  }
}

// Auto-generate slug from title
function generateSlug() {
  if (!editingItem.value && caseStudyForm.title && !caseStudyForm.slug) {
    caseStudyForm.slug = caseStudyForm.title
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '')
  }
}

const saving = ref(false)

async function saveItem() {
  saving.value = true

  try {
    const isGallery = portfolio.value?.type === 'gallery'
    let payload: Record<string, unknown>

    if (isGallery) {
      // Validate: for image/video, need either a file or existing URL
      // For link, need a URL
      if (galleryForm.itemType === 'link') {
        if (!galleryForm.mediaUrl) {
          toast.error(t('validation.required'))
          saving.value = false
          return
        }
      } else {
        // image or video: need file for new items, or existing URL for edits
        if (!mediaFile.value && !galleryForm.mediaUrl && !editingItem.value?.mediaUrl) {
          toast.error(t('validation.required'))
          saving.value = false
          return
        }
      }

      let mediaUrl = galleryForm.mediaUrl || editingItem.value?.mediaUrl || ''
      let thumbnailUrl: string | null = editingItem.value?.thumbnailUrl || null

      // Upload media if file selected
      if (mediaFile.value) {
        uploading.value = true
        const formData = new FormData()
        const isVideo = mediaFile.value.type.startsWith('video/')
        formData.append(isVideo ? 'video' : 'image', mediaFile.value)

        const uploadResult = await $fetch<{ url: string; thumbnailUrl?: string }>(
          `/api/upload/${isVideo ? 'video' : 'image'}`,
          {
            method: 'POST',
            body: formData
          }
        )
        mediaUrl = uploadResult.url
        thumbnailUrl = uploadResult.thumbnailUrl || null
        uploading.value = false
      }

      payload = {
        itemType: galleryForm.itemType,
        mediaUrl,
        thumbnailUrl: thumbnailUrl || undefined,
        caption: galleryForm.caption || undefined,
        published: galleryForm.published,
        order: galleryForm.order
      }
    } else {
      // Case study - coverUrl already uploaded via handleCoverUpload
      payload = {
        itemType: 'case_study',
        slug: caseStudyForm.slug,
        title: caseStudyForm.title,
        description: caseStudyForm.description || undefined,
        content: caseStudyForm.content || undefined,
        category: caseStudyForm.category || undefined,
        tags: caseStudyForm.tags
          ? caseStudyForm.tags
              .split(',')
              .map(t => t.trim())
              .filter(Boolean)
          : [],
        mediaUrl: caseStudyForm.coverUrl || undefined,
        thumbnailUrl: caseStudyForm.coverUrl || undefined,
        published: caseStudyForm.published,
        order: caseStudyForm.order,
        translations: caseStudyForm.translations
      }
    }

    if (editingItem.value) {
      await $fetch(`/api/portfolios/${portfolioId.value}/items/${editingItem.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch(`/api/portfolios/${portfolioId.value}/items`, {
        method: 'POST',
        body: payload
      })
    }

    showItemModal.value = false
    toast.success(t('common.saved'))
    await refresh()
  } catch (e: any) {
    console.error('Item save error:', e)
    toast.error(e.data?.statusMessage || e.data?.message || e.message || 'Failed to save')
  } finally {
    saving.value = false
    uploading.value = false
  }
}

const deleting = ref<number | null>(null)

async function deleteItem(item: PortfolioItem) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })
  if (!confirmed) return

  deleting.value = item.id
  try {
    await $fetch(`/api/portfolios/${portfolioId.value}/items/${item.id}`, { method: 'DELETE' })
    await refresh()
  } catch (e: any) {
    toast.error(e.data?.message || 'Failed to delete')
  } finally {
    deleting.value = null
  }
}

function getItemTypeIcon(type: string) {
  switch (type) {
    case 'video':
      return IconVideo
    case 'link':
      return IconLink
    default:
      return IconPhoto
  }
}

// Track failed images
const failedImages = ref(new Set<number>())

function onImageError(itemId: number) {
  failedImages.value.add(itemId)
}

function hasValidImage(item: PortfolioItem) {
  const url = item.thumbnailUrl || item.mediaUrl
  return url && !failedImages.value.has(item.id)
}

// Translation helpers
function getLocaleName(code: string): string {
  const locale = localesConfig.value.find(l => l.code === code)
  return locale?.name || code.toUpperCase()
}

function hasItemTranslation(item: PortfolioItemWithTranslations, locale: string): boolean {
  // Default locale content is in the base fields (title, description, etc.)
  const defaultLocale = defaultLocaleConfig.value || 'en'
  if (locale === defaultLocale) {
    return !!(item.title || item.translations?.[locale]?.title)
  }
  return !!(item.translations?.[locale]?.title)
}

function hasAllItemTranslations(item: PortfolioItemWithTranslations): boolean {
  return locales.every(locale => hasItemTranslation(item, locale))
}

function getMissingItemTranslations(item: PortfolioItemWithTranslations): string[] {
  return locales.filter(locale => !hasItemTranslation(item, locale))
}

function formLocaleHasContent(locale: string): boolean {
  const trans = caseStudyForm.translations[locale]
  return !!(trans?.title)
}
</script>

<template>
  <div class="admin-portfolio-detail">
    <!-- Back button and header -->
    <div class="page-header">
      <div class="page-header-left">
        <NuxtLink to="/admin/portfolios" class="btn btn-ghost btn-icon">
          <IconArrowLeft />
        </NuxtLink>
        <div v-if="portfolio">
          <h1 class="page-title">{{ portfolio.name }}</h1>
          <p v-if="portfolio.description" class="page-subtitle">{{ portfolio.description }}</p>
        </div>
      </div>
      <button v-if="portfolio" class="btn btn-primary" @click="openAddItem">
        <IconPlus />
        {{ portfolio.type === 'gallery' ? t('admin.addMedia') : t('admin.addCaseStudy') }}
      </button>
    </div>

    <!-- Loading -->
    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <!-- Portfolio not found -->
    <div v-else-if="!portfolio" class="empty-state">
      <p>{{ t('admin.portfolioNotFound') }}</p>
    </div>

    <!-- Empty state -->
    <div v-else-if="!portfolio.items?.length" class="empty-state">
      <p>{{ t('admin.noItems') }}</p>
      <button class="btn btn-primary" @click="openAddItem">
        <IconPlus />
        {{ portfolio.type === 'gallery' ? t('admin.addMedia') : t('admin.addCaseStudy') }}
      </button>
    </div>

    <!-- Gallery type: Media grid -->
    <div v-else-if="portfolio.type === 'gallery'" class="media-grid">
      <div v-for="item in portfolio.items" :key="item.id" class="media-item card">
        <div class="media-item-preview">
          <img
            v-if="hasValidImage(item)"
            :src="(item.thumbnailUrl || item.mediaUrl)!"
            :alt="item.caption || ''"
            @error="onImageError(item.id)"
          />
          <div v-else class="media-item-placeholder">
            <IconPhotoOff v-if="failedImages.has(item.id)" />
            <component v-else :is="getItemTypeIcon(item.itemType)" />
          </div>
          <span v-if="!item.published" class="badge badge-warning status-badge">Draft</span>
          <span class="badge badge-secondary type-badge">{{ item.itemType }}</span>
        </div>
        <div v-if="item.caption" class="media-item-caption">{{ item.caption }}</div>
        <div class="card-actions">
          <button class="btn btn-sm btn-secondary" @click="openEditItem(item)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button
            class="btn btn-sm btn-ghost text-error"
            @click="deleteItem(item)"
            :disabled="deleting === item.id"
          >
            <IconTrash />
          </button>
        </div>
      </div>
    </div>

    <!-- Case study type: Project cards -->
    <div v-else class="portfolio-grid">
      <div v-for="item in portfolio.items" :key="item.id" class="portfolio-item card">
        <div class="portfolio-item-image">
          <img
            v-if="hasValidImage(item)"
            :src="item.thumbnailUrl!"
            :alt="item.title || ''"
            @error="onImageError(item.id)"
          />
          <div v-else class="portfolio-item-placeholder">
            <IconPhotoOff v-if="failedImages.has(item.id)" />
            <IconPhoto v-else />
          </div>
          <span v-if="!item.published" class="badge badge-warning status-badge">Draft</span>
        </div>
        <div class="portfolio-item-info">
          <h3 class="portfolio-item-title">{{ item.title }}</h3>
          <p v-if="item.category" class="portfolio-item-category">{{ item.category }}</p>
          <!-- Translation status -->
          <div v-if="!hasAllItemTranslations(item)" class="mt-1">
            <span class="text-warning text-xs">
              {{ t('admin.missingTranslations') }}: {{ getMissingItemTranslations(item).map(l => l.toUpperCase()).join(', ') }}
            </span>
          </div>
        </div>
        <div class="card-actions">
          <button class="btn btn-sm btn-secondary" @click="openEditItem(item)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button
            class="btn btn-sm btn-ghost text-error"
            @click="deleteItem(item)"
            :disabled="deleting === item.id"
          >
            <IconTrash />
          </button>
        </div>
      </div>
    </div>

    <!-- Item Modal -->
    <Teleport to="body">
      <div v-if="showItemModal" class="modal-backdrop" @click.self="showItemModal = false">
        <div class="modal modal--lg">
          <header class="modal-header">
            <h2>{{ editingItem ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showItemModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveItem">
            <!-- Gallery item form -->
            <template v-if="portfolio?.type === 'gallery'">
              <!-- Type selector (for new items) -->
              <div v-if="!editingItem" class="form-group">
                <label class="form-label">{{ t('admin.mediaType') }}</label>
                <div class="type-selector">
                  <label class="type-option" :class="{ active: galleryForm.itemType === 'image' }">
                    <input
                      type="radio"
                      v-model="galleryForm.itemType"
                      value="image"
                      class="sr-only"
                    />
                    <IconPhoto />
                    <span>{{ t('admin.image') }}</span>
                  </label>
                  <label class="type-option" :class="{ active: galleryForm.itemType === 'video' }">
                    <input
                      type="radio"
                      v-model="galleryForm.itemType"
                      value="video"
                      class="sr-only"
                    />
                    <IconVideo />
                    <span>{{ t('admin.video') }}</span>
                  </label>
                  <label class="type-option" :class="{ active: galleryForm.itemType === 'link' }">
                    <input
                      type="radio"
                      v-model="galleryForm.itemType"
                      value="link"
                      class="sr-only"
                    />
                    <IconLink />
                    <span>{{ t('admin.externalLink') }}</span>
                  </label>
                </div>
              </div>

              <!-- Upload for image/video -->
              <div v-if="galleryForm.itemType !== 'link'" class="form-group">
                <label class="form-label">{{
                  galleryForm.itemType === 'video' ? t('admin.video') : t('admin.image')
                }}</label>
                <div class="upload-area" @click="triggerFileInput" role="button" tabindex="0" @keypress.enter="triggerFileInput">
                  <img v-if="mediaPreview" :src="mediaPreview" class="upload-preview" />
                  <div v-else class="upload-placeholder">
                    <IconUpload />
                    <span>{{ t('admin.clickToUpload') }}</span>
                  </div>
                  <input
                    ref="fileInput"
                    type="file"
                    :accept="galleryForm.itemType === 'video' ? 'video/*' : 'image/*'"
                    class="sr-only"
                    @change="handleMediaSelect"
                  />
                </div>
              </div>

              <!-- URL for link type -->
              <div v-if="galleryForm.itemType === 'link'" class="form-group">
                <label class="form-label">{{ t('admin.externalUrl') }} *</label>
                <input
                  v-model="galleryForm.mediaUrl"
                  type="url"
                  class="input"
                  required
                  placeholder="https://..."
                />
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.caption') }}</label>
                <input v-model="galleryForm.caption" type="text" class="input" />
              </div>

              <div class="form-row form-row--2col">
                <div class="form-group">
                  <label class="form-label">{{ t('admin.order') }}</label>
                  <input v-model.number="galleryForm.order" type="number" class="input" min="0" />
                </div>
                <div class="form-group">
                  <label class="form-label">{{ t('admin.status') }}</label>
                  <label class="form-checkbox">
                    <input v-model="galleryForm.published" type="checkbox" />
                    <span>{{ t('admin.published') }}</span>
                  </label>
                </div>
              </div>
            </template>

            <!-- Case study form -->
            <template v-else>
              <!-- Cover image -->
              <div class="form-group">
                <label class="form-label">{{ t('admin.coverImage') }}</label>
                <div class="input-row">
                  <input
                    v-model="caseStudyForm.coverUrl"
                    type="text"
                    class="input"
                    placeholder="https://..."
                  />
                  <label class="btn btn-secondary">
                    <IconUpload />
                    <input
                      ref="fileInput"
                      type="file"
                      accept="image/*"
                      class="sr-only"
                      @change="handleCoverUpload"
                    />
                  </label>
                </div>
                <div v-if="caseStudyForm.coverUrl" class="mt-2">
                  <img :src="caseStudyForm.coverUrl" alt="Preview" class="image-preview" />
                </div>
              </div>

              <!-- Non-translatable fields -->
              <div class="form-row form-row--2col">
                <div class="form-group">
                  <label class="form-label">{{ t('admin.itemTitle') }} *</label>
                  <input
                    v-model="caseStudyForm.title"
                    type="text"
                    class="input"
                    required
                    @blur="generateSlug"
                  />
                </div>
                <div class="form-group">
                  <label class="form-label">{{ t('admin.slug') }} *</label>
                  <input
                    v-model="caseStudyForm.slug"
                    type="text"
                    class="input"
                    required
                    pattern="[a-z0-9-]+"
                  />
                </div>
                <div class="form-group">
                  <label class="form-label">{{ t('admin.category') }}</label>
                  <input v-model="caseStudyForm.category" type="text" class="input" />
                </div>
                <div class="form-group">
                  <label class="form-label">{{ t('admin.order') }}</label>
                  <input v-model.number="caseStudyForm.order" type="number" class="input" min="0" />
                </div>
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.description') }}</label>
                <textarea v-model="caseStudyForm.description" class="input" rows="2"></textarea>
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.content') }}</label>
                <textarea v-model="caseStudyForm.content" class="input" rows="6"></textarea>
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.tags') }}</label>
                <input
                  v-model="caseStudyForm.tags"
                  type="text"
                  class="input"
                  :placeholder="t('admin.tagsHint')"
                />
              </div>

              <div class="form-group">
                <label class="form-checkbox">
                  <input v-model="caseStudyForm.published" type="checkbox" />
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
                  <label class="form-label">{{ t('admin.itemTitle') }} ({{ getLocaleName(locale) }})</label>
                  <input
                    v-model="caseStudyForm.translations[locale].title"
                    type="text"
                    class="input"
                    @blur="generateSlug"
                  />
                </div>

                <div class="form-group">
                  <label class="form-label">{{ t('admin.description') }} ({{ getLocaleName(locale) }})</label>
                  <textarea
                    v-model="caseStudyForm.translations[locale].description"
                    class="input"
                    rows="2"
                  ></textarea>
                </div>

                <div class="form-group">
                  <label class="form-label">{{ t('admin.content') }} ({{ getLocaleName(locale) }})</label>
                  <textarea
                    v-model="caseStudyForm.translations[locale].content"
                    class="input"
                    rows="4"
                  ></textarea>
                </div>

                <div class="form-group">
                  <label class="form-label">{{ t('admin.category') }} ({{ getLocaleName(locale) }})</label>
                  <input
                    v-model="caseStudyForm.translations[locale].category"
                    type="text"
                    class="input"
                  />
                </div>
              </div>
            </template>
          </form>

          <footer class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="showItemModal = false">
              {{ t('common.cancel') }}
            </button>
            <button
              type="submit"
              class="btn btn-primary"
              :disabled="saving || uploading || uploadingCover"
              @click="saveItem"
            >
              {{ (uploading || uploadingCover) ? t('common.uploading') : saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<!--
  Uses global CSS classes from admin/index.css and ui/ folders
  New classes needed: .media-grid, .media-item, .upload-area, .type-selector
-->
