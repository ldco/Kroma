<script setup lang="ts">
/**
 * Admin Features Page
 *
 * Manage features with unified multi-language support.
 * All languages are equal - editable in the same modal with tabs.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox, .input-row, .form-divider
 * - ui/content/tabs.css: .tabs, .tabs--underline, .tab, .is-active, .tab__indicator
 * - ui/content/cards.css: .card, .card-body, .card-actions
 * - ui/admin/pages.css: .features-admin-list, .features-admin-item__*
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import IconUpload from '~icons/tabler/upload'
import config from '~/puppet-master.config'
import AppImage from '~/components/atoms/AppImage.vue'

interface FeatureTranslation {
  title: string | null
  description: string | null
}

interface Feature {
  id: number
  slug: string
  imageUrl: string | null
  hoverImageUrl: string | null
  order: number
  published: boolean
  translations: Record<string, FeatureTranslation>
}

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('admin.features')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// SSR-safe config access with defensive defaults
const localesConfig = computed(() => config?.locales ?? [])

// Available locales from config
const locales = localesConfig.value.map(l => l.code)

// Fetch features
const headers = useRequestHeaders(['cookie'])
const {
  data: features,
  pending,
  refresh
} = await useFetch<Feature[]>('/api/admin/features', { headers })

// Modal state
const showModal = ref(false)
const editingFeature = ref<Feature | null>(null)
const saving = ref(false)
const activeLocale = ref(locales[0] || 'en')

// Create empty translations structure for all locales
function createEmptyTranslations(): Record<string, { title: string; description: string }> {
  const translations: Record<string, { title: string; description: string }> = {}
  for (const locale of locales) {
    translations[locale] = { title: '', description: '' }
  }
  return translations
}

const form = reactive({
  slug: '',
  imageUrl: '',
  hoverImageUrl: '',
  order: 0,
  published: true,
  translations: createEmptyTranslations()
})

function resetForm() {
  form.slug = ''
  form.imageUrl = ''
  form.hoverImageUrl = ''
  form.order = 0
  form.published = true
  form.translations = createEmptyTranslations()
  activeLocale.value = locales[0] || 'en'
}

function openCreate() {
  editingFeature.value = null
  resetForm()
  showModal.value = true
}

function openEdit(feature: Feature) {
  editingFeature.value = feature
  form.slug = feature.slug
  form.imageUrl = feature.imageUrl || ''
  form.hoverImageUrl = feature.hoverImageUrl || ''
  form.order = feature.order
  form.published = feature.published

  // Load translations for all locales
  form.translations = createEmptyTranslations()
  for (const locale of locales) {
    const trans = feature.translations?.[locale]
    if (trans) {
      form.translations[locale] = {
        title: trans.title || '',
        description: trans.description || ''
      }
    }
  }

  // Find first locale with content
  const localeWithContent = locales.find(locale => {
    const trans = feature.translations?.[locale]
    return trans?.title
  })
  activeLocale.value = localeWithContent || locales[0] || 'en'

  showModal.value = true
}

function generateSlug() {
  if (!editingFeature.value && !form.slug) {
    // Generate slug from first non-empty title
    for (const locale of locales) {
      const title = form.translations[locale]?.title
      if (title) {
        form.slug = title
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, '-')
          .replace(/^-|-$/g, '')
        break
      }
    }
  }
}

async function saveFeature() {
  saving.value = true

  try {
    const payload = {
      slug: form.slug,
      imageUrl: form.imageUrl || null,
      hoverImageUrl: form.hoverImageUrl || null,
      order: form.order,
      published: form.published,
      translations: form.translations
    }

    if (editingFeature.value) {
      await $fetch(`/api/admin/features/${editingFeature.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch('/api/admin/features', {
        method: 'POST',
        body: payload
      })
    }
    toast.success(t('common.saved'))
    showModal.value = false
    refresh()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    saving.value = false
  }
}

async function deleteFeature(feature: Feature) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/features/${feature.id}`, { method: 'DELETE' })
    toast.success(t('common.deleted'))
    refresh()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  }
}

function getLocaleName(code: string): string {
  const locale = localesConfig.value.find(l => l.code === code)
  return locale?.name || code.toUpperCase()
}

// Get first available title for display in list
function getFeatureTitle(feature: Feature): string {
  for (const locale of locales) {
    const title = feature.translations?.[locale]?.title
    if (title) return title
  }
  return feature.slug
}

// Get first available description for display in list
function getFeatureDescription(feature: Feature): string {
  for (const locale of locales) {
    const desc = feature.translations?.[locale]?.description
    if (desc) return desc
  }
  return ''
}

// Check if locale has translation
function hasTranslation(feature: Feature, locale: string): boolean {
  return !!(feature.translations?.[locale]?.title)
}

// Check if all locales have translations
function hasAllTranslations(feature: Feature): boolean {
  return locales.every(locale => hasTranslation(feature, locale))
}

// Get list of missing translations
function getMissingTranslations(feature: Feature): string[] {
  return locales.filter(locale => !hasTranslation(feature, locale))
}

// Check if current form locale has content
function formLocaleHasContent(locale: string): boolean {
  const trans = form.translations[locale]
  return !!(trans?.title)
}

// Image upload
const uploadingImage = ref(false)
const uploadingHoverImage = ref(false)

async function uploadImage(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  uploadingImage.value = true
  try {
    const formData = new FormData()
    formData.append('file', file)

    const result = await $fetch<{ url: string }>('/api/upload/image', {
      method: 'POST',
      body: formData
    })

    form.imageUrl = result.url
    toast.success(t('common.uploaded'))
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    uploadingImage.value = false
    input.value = ''
  }
}

async function uploadHoverImage(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  uploadingHoverImage.value = true
  try {
    const formData = new FormData()
    formData.append('file', file)

    const result = await $fetch<{ url: string }>('/api/upload/image', {
      method: 'POST',
      body: formData
    })

    form.hoverImageUrl = result.url
    toast.success(t('common.uploaded'))
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    uploadingHoverImage.value = false
    input.value = ''
  }
}
</script>

<template>
  <div class="admin-features">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.features') }}</h1>
      <button class="btn btn-primary" @click="openCreate">
        <IconPlus />
        {{ t('admin.addItem') }}
      </button>
    </div>

    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <div v-else-if="!features?.length" class="empty-state">
      <p>{{ t('admin.noItems') }}</p>
    </div>

    <div v-else class="features-admin-grid">
      <div v-for="feature in features" :key="feature.id" class="card">
        <div class="card-body">
          <h3 class="features-admin-item__title">{{ getFeatureTitle(feature) }}</h3>
          <p v-if="getFeatureDescription(feature)" class="features-admin-item__desc">
            {{ getFeatureDescription(feature) }}
          </p>
          <div class="features-admin-item__meta">
            <span
              class="badge"
              :class="feature.published ? 'badge-success' : 'badge-warning'"
            >
              {{ feature.published ? t('admin.published') : t('common.draft') }}
            </span>
            <span v-if="!feature.imageUrl" class="badge badge-warning">{{ t('admin.noImage') }}</span>
          </div>

          <!-- Translation status -->
          <div v-if="!hasAllTranslations(feature)" class="mt-2">
            <span class="text-warning text-sm">
              {{ t('admin.missingTranslations') }}: {{ getMissingTranslations(feature).map(l => l.toUpperCase()).join(', ') }}
            </span>
          </div>
        </div>

        <div class="card-actions">
          <button class="btn btn-sm btn-secondary" @click="openEdit(feature)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button class="btn btn-sm btn-ghost text-error" @click="deleteFeature(feature)">
            <IconTrash />
          </button>
        </div>
      </div>
    </div>

    <!-- Edit Modal with Language Tabs -->
    <Teleport to="body">
      <div v-if="showModal" class="modal-backdrop" @click.self="showModal = false">
        <div class="modal modal--lg">
          <header class="modal-header">
            <h2>{{ editingFeature ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveFeature">
            <!-- Non-translatable fields -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label" for="feature-slug">{{ t('admin.slug') }} *</label>
                <input
                  id="feature-slug"
                  v-model="form.slug"
                  type="text"
                  class="input"
                  required
                  pattern="[a-z0-9-]+"
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="feature-order">{{ t('admin.order') }}</label>
                <input
                  id="feature-order"
                  v-model.number="form.order"
                  type="number"
                  class="input"
                  min="0"
                />
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.status') }}</label>
                <label class="form-checkbox">
                  <input v-model="form.published" type="checkbox" />
                  <span>{{ t('admin.published') }}</span>
                </label>
              </div>
            </div>

            <!-- Feature Images -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label">Feature Image</label>
                <div class="input-row">
                  <input
                    v-model="form.imageUrl"
                    type="text"
                    class="input"
                    placeholder="Image URL"
                  />
                  <label class="btn btn-secondary">
                    <IconUpload />
                    <input
                      type="file"
                      accept="image/*"
                      class="sr-only"
                      :disabled="uploadingImage"
                      @change="uploadImage"
                    />
                  </label>
                </div>
                <div v-if="form.imageUrl" class="mt-2">
                  <AppImage :src="form.imageUrl" alt="Preview" class="image-preview" />
                </div>
              </div>

              <div class="form-group">
                <label class="form-label">Hover Image</label>
                <div class="input-row">
                  <input
                    v-model="form.hoverImageUrl"
                    type="text"
                    class="input"
                    placeholder="Hover image URL"
                  />
                  <label class="btn btn-secondary">
                    <IconUpload />
                    <input
                      type="file"
                      accept="image/*"
                      class="sr-only"
                      :disabled="uploadingHoverImage"
                      @change="uploadHoverImage"
                    />
                  </label>
                </div>
                <div v-if="form.hoverImageUrl" class="mt-2">
                  <AppImage :src="form.hoverImageUrl" alt="Hover Preview" class="image-preview" />
                </div>
              </div>
            </div>

            <!-- Language tabs for translatable content -->
            <div class="form-divider">
              <span>{{ t('admin.content') }}</span>
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
                <label class="form-label">{{ t('admin.title') }}</label>
                <input
                  v-model="form.translations[locale].title"
                  type="text"
                  class="input"
                  @blur="generateSlug"
                />
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.description') }}</label>
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
              @click="saveFeature"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>
