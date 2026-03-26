<script setup lang="ts">
/**
 * Admin FAQ Page
 *
 * Manage FAQ items with unified multi-language support.
 * All languages are equal - editable in the same modal with tabs.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox, .form-divider
 * - ui/content/tabs.css: .tabs, .tabs--underline, .tab, .is-active, .tab__indicator
 * - ui/content/cards.css: .card, .card-body, .card-actions
 * - ui/admin/pages.css: .faq-admin-list, .faq-admin-item__*
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import config from '~/puppet-master.config'

interface FaqTranslation {
  question: string | null
  answer: string | null
}

interface FaqItem {
  id: number
  category: string | null
  order: number
  published: boolean
  translations: Record<string, FaqTranslation>
}

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('admin.faq')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// SSR-safe config access with defensive defaults
const localesConfig = computed(() => config?.locales ?? [])

// Available locales from config
const locales = localesConfig.value.map(l => l.code)

// Fetch FAQ items
const headers = useRequestHeaders(['cookie'])
const {
  data: faqItems,
  pending,
  refresh
} = await useAsyncData(
  'admin-faq-items',
  () => apiFetch<FaqItem[]>('/api/admin/faq', { headers }),
  { default: () => [] }
)

// Modal state
const showModal = ref(false)
const editingItem = ref<FaqItem | null>(null)
const saving = ref(false)
const activeLocale = ref(locales[0] || 'en')

// Create empty translations structure for all locales
function createEmptyTranslations(): Record<string, { question: string; answer: string }> {
  const translations: Record<string, { question: string; answer: string }> = {}
  for (const locale of locales) {
    translations[locale] = { question: '', answer: '' }
  }
  return translations
}

const form = reactive({
  category: '',
  order: 0,
  published: true,
  translations: createEmptyTranslations()
})

function resetForm() {
  form.category = ''
  form.order = 0
  form.published = true
  form.translations = createEmptyTranslations()
  activeLocale.value = locales[0] || 'en'
}

function openCreate() {
  editingItem.value = null
  resetForm()
  showModal.value = true
}

function openEdit(item: FaqItem) {
  editingItem.value = item
  form.category = item.category || ''
  form.order = item.order
  form.published = item.published

  // Load translations for all locales
  form.translations = createEmptyTranslations()
  for (const locale of locales) {
    const trans = item.translations?.[locale]
    if (trans) {
      form.translations[locale] = {
        question: trans.question || '',
        answer: trans.answer || ''
      }
    }
  }

  // Find first locale with content
  const localeWithContent = locales.find(locale => {
    const trans = item.translations?.[locale]
    return trans?.question
  })
  activeLocale.value = localeWithContent || locales[0] || 'en'

  showModal.value = true
}

async function saveItem() {
  saving.value = true

  try {
    const payload = {
      category: form.category || null,
      order: form.order,
      published: form.published,
      translations: form.translations
    }

    if (editingItem.value) {
      await $fetch(`/api/admin/faq/${editingItem.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch('/api/admin/faq', {
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

async function deleteItem(item: FaqItem) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/faq/${item.id}`, { method: 'DELETE' })
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

// Get first available question for display in list
function getFaqQuestion(item: FaqItem): string {
  for (const locale of locales) {
    const question = item.translations?.[locale]?.question
    if (question) return question
  }
  return ''
}

// Get first available answer for display in list
function getFaqAnswer(item: FaqItem): string {
  for (const locale of locales) {
    const answer = item.translations?.[locale]?.answer
    if (answer) return answer
  }
  return ''
}

// Check if locale has translation
function hasTranslation(item: FaqItem, locale: string): boolean {
  return !!(item.translations?.[locale]?.question)
}

// Check if all locales have translations
function hasAllTranslations(item: FaqItem): boolean {
  return locales.every(locale => hasTranslation(item, locale))
}

// Get list of missing translations
function getMissingTranslations(item: FaqItem): string[] {
  return locales.filter(locale => !hasTranslation(item, locale))
}

// Check if current form locale has content
function formLocaleHasContent(locale: string): boolean {
  const trans = form.translations[locale]
  return !!(trans?.question)
}
</script>

<template>
  <div class="admin-faq">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.faq') }}</h1>
      <button class="btn btn-primary" @click="openCreate">
        <IconPlus />
        {{ t('admin.addItem') }}
      </button>
    </div>

    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <div v-else-if="!faqItems?.length" class="empty-state">
      <p>{{ t('admin.noItems') }}</p>
    </div>

    <div v-else class="faq-admin-list">
      <div v-for="item in faqItems" :key="item.id" class="card">
        <div class="card-body">
          <h3 class="faq-admin-item__question">{{ getFaqQuestion(item) }}</h3>
          <p class="faq-admin-item__answer">{{ getFaqAnswer(item) }}</p>
          <div class="faq-admin-item__meta">
            <span v-if="item.category" class="badge badge-secondary">{{ item.category }}</span>
            <span
              class="badge"
              :class="item.published ? 'badge-success' : 'badge-warning'"
            >
              {{ item.published ? t('admin.published') : t('common.draft') }}
            </span>
          </div>

          <!-- Translation status -->
          <div v-if="!hasAllTranslations(item)" class="mt-2">
            <span class="text-warning text-sm">
              {{ t('admin.missingTranslations') }}: {{ getMissingTranslations(item).map(l => l.toUpperCase()).join(', ') }}
            </span>
          </div>
        </div>

        <div class="card-actions">
          <button class="btn btn-sm btn-secondary" @click="openEdit(item)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button class="btn btn-sm btn-ghost text-error" @click="deleteItem(item)">
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
            <h2>{{ editingItem ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveItem">
            <!-- Non-translatable fields -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label" for="faq-category">{{ t('admin.category') }}</label>
                <input
                  id="faq-category"
                  v-model="form.category"
                  type="text"
                  class="input"
                  placeholder="General, Pricing, Support..."
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="faq-order">{{ t('admin.order') }}</label>
                <input
                  id="faq-order"
                  v-model.number="form.order"
                  type="number"
                  class="input"
                  min="0"
                />
              </div>
            </div>

            <div class="form-row">
              <label class="form-checkbox">
                <input v-model="form.published" type="checkbox" />
                <span>{{ t('admin.published') }}</span>
              </label>
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
                <label class="form-label">{{ t('admin.question') }}</label>
                <input
                  v-model="form.translations[locale].question"
                  type="text"
                  class="input"
                />
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.answer') }}</label>
                <textarea
                  v-model="form.translations[locale].answer"
                  class="input"
                  rows="5"
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
              @click="saveItem"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>
