<script setup lang="ts">
/**
 * Admin Testimonials Page
 *
 * Manage testimonials with unified multi-language support.
 * All languages are equal - editable in the same modal with tabs.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox, .input-row, .form-divider
 * - ui/content/tabs.css: .tabs, .tabs--underline, .tab, .is-active, .tab__indicator
 * - ui/content/cards.css: .card, .card-body, .card-actions
 * - ui/admin/pages.css: .testimonials-admin-grid, .testimonials-admin-item__*
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import IconUpload from '~icons/tabler/upload'
import IconStar from '~icons/tabler/star'
import IconStarFilled from '~icons/tabler/star-filled'
import config from '~/puppet-master.config'
import AppImage from '~/components/atoms/AppImage.vue'

interface TestimonialTranslation {
  quote: string | null
  authorTitle: string | null
}

interface Testimonial {
  id: number
  authorName: string
  authorPhotoUrl: string | null
  rating: number
  order: number
  published: boolean
  translations: Record<string, TestimonialTranslation>
}

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

// SSR-safe config access with defensive defaults
const localesConfig = computed(() => config?.locales ?? [])

// Available locales from config
const locales = localesConfig.value.map(l => l.code)

useHead({
  title: () => `${t('admin.testimonials')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// Fetch testimonials
const headers = useRequestHeaders(['cookie'])
const {
  data: testimonials,
  pending,
  refresh
} = await useFetch<Testimonial[]>('/api/admin/testimonials', { headers })

// Modal state
const showModal = ref(false)
const editingTestimonial = ref<Testimonial | null>(null)
const saving = ref(false)
const activeLocale = ref(locales[0] || 'en')

// Create empty translations structure for all locales
function createEmptyTranslations(): Record<string, { quote: string; authorTitle: string }> {
  const translations: Record<string, { quote: string; authorTitle: string }> = {}
  for (const locale of locales) {
    translations[locale] = { quote: '', authorTitle: '' }
  }
  return translations
}

const form = reactive({
  authorName: '',
  authorPhoto: '',
  rating: 5,
  order: 0,
  published: true,
  translations: createEmptyTranslations()
})

function resetForm() {
  form.authorName = ''
  form.authorPhoto = ''
  form.rating = 5
  form.order = 0
  form.published = true
  form.translations = createEmptyTranslations()
  activeLocale.value = locales[0] || 'en'
}

function openCreate() {
  editingTestimonial.value = null
  resetForm()
  showModal.value = true
}

function openEdit(testimonial: Testimonial) {
  editingTestimonial.value = testimonial
  form.authorName = testimonial.authorName
  form.authorPhoto = testimonial.authorPhotoUrl || ''
  form.rating = testimonial.rating
  form.order = testimonial.order
  form.published = testimonial.published

  // Load translations for all locales
  form.translations = createEmptyTranslations()
  for (const locale of locales) {
    const trans = testimonial.translations?.[locale]
    if (trans) {
      form.translations[locale] = {
        quote: trans.quote || '',
        authorTitle: trans.authorTitle || ''
      }
    }
  }

  // Find first locale with content
  const localeWithContent = locales.find(locale => {
    const trans = testimonial.translations?.[locale]
    return trans?.quote
  })
  activeLocale.value = localeWithContent || locales[0] || 'en'

  showModal.value = true
}

async function saveTestimonial() {
  saving.value = true

  try {
    const payload = {
      authorName: form.authorName,
      authorPhoto: form.authorPhoto || null,
      rating: form.rating,
      order: form.order,
      published: form.published,
      translations: form.translations
    }

    if (editingTestimonial.value) {
      await $fetch(`/api/admin/testimonials/${editingTestimonial.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch('/api/admin/testimonials', {
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

async function deleteTestimonial(testimonial: Testimonial) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/testimonials/${testimonial.id}`, { method: 'DELETE' })
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

// Get first available quote for display in list
function getTestimonialQuote(testimonial: Testimonial): string {
  for (const locale of locales) {
    const quote = testimonial.translations?.[locale]?.quote
    if (quote) return quote
  }
  return ''
}

// Get first available authorTitle for display in list
function getTestimonialTitle(testimonial: Testimonial): string {
  for (const locale of locales) {
    const title = testimonial.translations?.[locale]?.authorTitle
    if (title) return title
  }
  return ''
}

// Check if locale has translation
function hasTranslation(testimonial: Testimonial, locale: string): boolean {
  return !!(testimonial.translations?.[locale]?.quote)
}

// Check if all locales have translations
function hasAllTranslations(testimonial: Testimonial): boolean {
  return locales.every(locale => hasTranslation(testimonial, locale))
}

// Get list of missing translations
function getMissingTranslations(testimonial: Testimonial): string[] {
  return locales.filter(locale => !hasTranslation(testimonial, locale))
}

// Check if current form locale has content
function formLocaleHasContent(locale: string): boolean {
  const trans = form.translations[locale]
  return !!(trans?.quote)
}

// Photo upload
const uploadingPhoto = ref(false)
async function uploadPhoto(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  uploadingPhoto.value = true
  try {
    const formData = new FormData()
    formData.append('file', file)

    const result = await $fetch<{ url: string }>('/api/upload/image', {
      method: 'POST',
      body: formData
    })

    form.authorPhoto = result.url
    toast.success(t('common.uploaded'))
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    uploadingPhoto.value = false
    input.value = ''
  }
}
</script>

<template>
  <div class="admin-testimonials">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.testimonials') }}</h1>
      <button class="btn btn-primary" @click="openCreate">
        <IconPlus />
        {{ t('admin.addItem') }}
      </button>
    </div>

    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <div v-else-if="!testimonials?.length" class="empty-state">
      <p>{{ t('admin.noItems') }}</p>
    </div>

    <div v-else class="testimonials-admin-grid">
      <div v-for="testimonial in testimonials" :key="testimonial.id" class="card">
        <div class="card-body">
          <div class="testimonials-admin-item">
            <p class="testimonials-admin-item__quote">"{{ getTestimonialQuote(testimonial) }}"</p>
            <div class="testimonials-admin-item__author">
              <div class="testimonials-admin-item__photo">
                <AppImage
                  :src="testimonial.authorPhotoUrl"
                  :alt="testimonial.authorName"
                  fallback="initials"
                  :initials="testimonial.authorName.charAt(0)"
                />
              </div>
              <div class="testimonials-admin-item__info">
                <span class="testimonials-admin-item__name">{{ testimonial.authorName }}</span>
                <span v-if="getTestimonialTitle(testimonial)" class="testimonials-admin-item__title">
                  {{ getTestimonialTitle(testimonial) }}
                </span>
              </div>
            </div>
            <div class="testimonials-admin-item__meta">
              <div class="testimonials-admin-item__rating">
                <template v-for="i in 5" :key="i">
                  <IconStarFilled v-if="i <= testimonial.rating" class="icon-sm text-warning" />
                  <IconStar v-else class="icon-sm text-secondary" />
                </template>
              </div>
              <span
                class="badge"
                :class="testimonial.published ? 'badge-success' : 'badge-warning'"
              >
                {{ testimonial.published ? t('admin.published') : t('common.draft') }}
              </span>
            </div>
          </div>

          <!-- Translation status -->
          <div v-if="!hasAllTranslations(testimonial)" class="mt-2">
            <span class="text-warning text-sm">
              {{ t('admin.missingTranslations') }}: {{ getMissingTranslations(testimonial).map(l => l.toUpperCase()).join(', ') }}
            </span>
          </div>
        </div>

        <div class="card-actions">
          <button class="btn btn-sm btn-secondary" @click="openEdit(testimonial)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button class="btn btn-sm btn-ghost text-error" @click="deleteTestimonial(testimonial)">
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
            <h2>{{ editingTestimonial ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveTestimonial">
            <!-- Non-translatable fields -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label" for="testimonial-author">{{ t('admin.authorName') }} *</label>
                <input
                  id="testimonial-author"
                  v-model="form.authorName"
                  type="text"
                  class="input"
                  required
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="testimonial-rating">{{ t('admin.rating') }}</label>
                <select id="testimonial-rating" v-model.number="form.rating" class="input">
                  <option :value="5">5 Stars</option>
                  <option :value="4">4 Stars</option>
                  <option :value="3">3 Stars</option>
                  <option :value="2">2 Stars</option>
                  <option :value="1">1 Star</option>
                </select>
              </div>

              <div class="form-group">
                <label class="form-label" for="testimonial-order">{{ t('admin.order') }}</label>
                <input
                  id="testimonial-order"
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

            <div class="form-group">
              <label class="form-label">{{ t('admin.photo') }}</label>
              <div class="input-row">
                <input
                  v-model="form.authorPhoto"
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
                    :disabled="uploadingPhoto"
                    @change="uploadPhoto"
                  />
                </label>
              </div>
              <div v-if="form.authorPhoto" class="mt-2">
                <AppImage :src="form.authorPhoto" alt="Preview" class="testimonials-admin-photo-preview" />
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
                <label class="form-label">{{ t('admin.quote') }}</label>
                <textarea
                  v-model="form.translations[locale].quote"
                  class="input"
                  rows="4"
                ></textarea>
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.authorTitle') }}</label>
                <input
                  v-model="form.translations[locale].authorTitle"
                  type="text"
                  class="input"
                  placeholder="CEO, Company Name"
                />
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
              @click="saveTestimonial"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>
