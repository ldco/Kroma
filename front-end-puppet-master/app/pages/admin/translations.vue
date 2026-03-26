<script setup lang="ts">
/**
 * Admin Translations Page
 *
 * Manage all text content across languages.
 * - Content translations: editable by admin+
 * - System translations: editable by master only (separate section)
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label
 * - ui/forms/search.css: .search-input
 * - ui/content/data-table.css: .data-table
 * - ui/admin/pages.css: .section-toggle, .section-toggle-btn
 */
import IconPlus from '~icons/tabler/plus'
import IconTrash from '~icons/tabler/trash'
import IconSearch from '~icons/tabler/search'
import IconX from '~icons/tabler/x'
import IconSettings from '~icons/tabler/settings'
import IconFileText from '~icons/tabler/file-text'
import type { Translation, TranslationsData } from '~/types'

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('admin.navTranslations')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// Fetch translations - pass cookies for SSR auth
const headers = useRequestHeaders(['cookie'])
const {
  data,
  pending,
  refresh
} = await useFetch<TranslationsData>('/api/admin/translations', {
  headers
})

const locales = computed(() => data.value?.locales || ['en', 'ru', 'he'])
const localeNames = computed(() => data.value?.localeNames || { en: 'English', ru: 'Русский', he: 'עברית' })
const contentTranslations = computed(() => data.value?.content || {})
const systemTranslations = computed(() => data.value?.system || {})
const canEditSystem = computed(() => data.value?.canEditSystem || false)

// Section toggle: 'content' or 'system'
const activeSection = ref<'content' | 'system'>('content')

// Current locale tab
const activeLocale = ref('en')

// Search/filter
const searchQuery = ref('')

// Get translations for current section and locale
const currentTranslations = computed(() => {
  const source = activeSection.value === 'system' ? systemTranslations.value : contentTranslations.value
  return source[activeLocale.value] || []
})

const filteredTranslations = computed(() => {
  const items = currentTranslations.value
  if (!searchQuery.value) return items
  const query = searchQuery.value.toLowerCase()
  return items.filter(
    t => t.key.toLowerCase().includes(query) || t.value.toLowerCase().includes(query)
  )
})

// Count for tabs
const contentCount = computed(() => {
  return Object.values(contentTranslations.value).flat().length
})
const systemCount = computed(() => {
  return Object.values(systemTranslations.value).flat().length
})

// Add/Edit modal
const showModal = ref(false)
const editingId = ref<number | null>(null)
const editingLocale = ref<string | null>(null)
const form = reactive({
  key: '',
  values: {} as Record<string, string>  // All locale values
})

function openAdd() {
  editingId.value = null
  editingLocale.value = null
  form.key = ''
  // Initialize empty values for all locales
  form.values = {}
  for (const loc of locales.value) {
    form.values[loc] = ''
  }
  showModal.value = true
}

function openEdit(item: Translation) {
  editingId.value = item.id
  editingLocale.value = activeLocale.value
  form.key = item.key
  // For editing, only show the current locale's value
  form.values = { [activeLocale.value]: item.value }
  showModal.value = true
}

const saving = ref(false)

async function saveTranslation() {
  saving.value = true
  try {
    if (editingId.value && editingLocale.value) {
      // Editing single locale
      await $fetch('/api/admin/translations', {
        method: 'POST',
        body: {
          locale: editingLocale.value,
          key: form.key,
          value: form.values[editingLocale.value]
        }
      })
    } else {
      // Adding new - save all locales that have values
      for (const loc of locales.value) {
        if (form.values[loc]?.trim()) {
          await $fetch('/api/admin/translations', {
            method: 'POST',
            body: {
              locale: loc,
              key: form.key,
              value: form.values[loc]
            }
          })
        }
      }
    }
    showModal.value = false
    await refresh()
    toast.success(t('common.success'))
  } catch (e: any) {
    toast.error(e.data?.message || 'Failed to save')
  } finally {
    saving.value = false
  }
}

const deleting = ref<number | null>(null)

async function deleteTranslation(item: Translation) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })
  if (!confirmed) return

  deleting.value = item.id
  try {
    await $fetch(`/api/admin/translations/${item.id}`, { method: 'DELETE' })
    await refresh()
    toast.success(t('common.success'))
  } catch (e: any) {
    toast.error(e.data?.message || 'Failed to delete')
  } finally {
    deleting.value = null
  }
}

</script>

<template>
  <div class="admin-translations">
    <div class="page-header">
      <div>
        <h1 class="page-title">{{ t('admin.translations') }}</h1>
        <p class="text-secondary">{{ t('admin.translationsSubtitle') }}</p>
      </div>
      <button class="btn btn-primary" @click="openAdd">
        <IconPlus />
        {{ t('admin.addTranslation') }}
      </button>
    </div>

    <!-- Section toggle (Content / System) - only show if user can edit system -->
    <div v-if="canEditSystem" class="section-toggle">
      <button
        class="section-toggle-btn"
        :class="{ 'is-active': activeSection === 'content' }"
        @click="activeSection = 'content'"
      >
        <IconFileText />
        {{ t('admin.translationsContent') }}
        <span class="section-toggle-count">{{ contentCount }}</span>
      </button>
      <button
        class="section-toggle-btn"
        :class="{ 'is-active': activeSection === 'system' }"
        @click="activeSection = 'system'"
      >
        <IconSettings />
        {{ t('admin.translationsSystem') }}
        <span class="section-toggle-count">{{ systemCount }}</span>
      </button>
    </div>

    <!-- System translations notice -->
    <div v-if="activeSection === 'system'" class="alert alert--warning">
      <strong>{{ t('admin.translationsSystemWarning') }}</strong> - {{ t('admin.translationsSystemWarningDesc') }}
    </div>

    <!-- Locale tabs -->
    <div class="tabs tabs--underline mb-6">
      <button
        v-for="loc in locales"
        :key="loc"
        class="tab"
        :class="{ 'is-active': activeLocale === loc }"
        @click="activeLocale = loc"
      >
        {{ localeNames[loc] || loc.toUpperCase() }}
        <span class="tab-badge">
          {{ (activeSection === 'system' ? systemTranslations : contentTranslations)[loc]?.length || 0 }}
        </span>
      </button>
    </div>

    <!-- Search -->
    <div class="search-bar">
      <IconSearch class="search-icon" />
      <input
        v-model="searchQuery"
        type="text"
        class="input"
        placeholder="Search by key or value..."
      />
    </div>

    <!-- Loading -->
    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <!-- Empty state -->
    <div v-else-if="!filteredTranslations.length" class="empty-state">
      <p>{{ searchQuery ? 'No matches found' : t('admin.noTranslations') }}</p>
    </div>

    <!-- Translations table -->
    <div v-else class="card">
      <table class="data-table">
        <thead>
          <tr>
            <th>{{ t('admin.translationKey') }}</th>
            <th>{{ t('admin.translationValue') }}</th>
            <th class="actions-col"></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in filteredTranslations" :key="item.id">
            <td class="cell--key" :data-label="t('admin.translationKey')">
              <code>{{ item.key }}</code>
            </td>
            <td
              class="cell--value cell--editable"
              :data-label="t('admin.translationValue')"
              dir="auto"
              @click="openEdit(item)"
            >
              {{ item.value }}
            </td>
            <td class="actions-col">
              <button
                class="btn btn-icon btn-ghost btn-danger"
                @click="deleteTranslation(item)"
                :disabled="deleting === item.id"
              >
                <IconTrash />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Modal -->
    <Teleport to="body">
      <div v-if="showModal" class="modal-backdrop" @click.self="showModal = false">
        <div class="modal">
          <header class="modal-header">
            <h2>
              {{ editingId ? t('common.edit') : t('admin.addItem') }}
              <span v-if="!editingId && canEditSystem" class="modal-section-badge" :class="activeSection">
                {{ activeSection === 'system' ? t('admin.translationsSystem') : t('admin.translationsContent') }}
              </span>
            </h2>
            <button class="btn btn-ghost btn-sm" @click="showModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveTranslation">
            <!-- Section info for new translations -->
            <div v-if="!editingId && activeSection === 'system'" class="form-info form-info--system">
              <strong>{{ t('admin.translationsAddingToSystem') }}</strong>
              {{ t('admin.translationsSystemPrefixHint') }} <code>common.</code>, <code>admin.</code>, <code>nav.</code>, <code>auth.</code>
            </div>
            <div v-else-if="!editingId && canEditSystem" class="form-info">
              {{ t('admin.translationsAddingToContent') }} <code>hero.</code>, <code>about.</code>, <code>cta.</code>
            </div>

            <div class="form-group">
              <label class="form-label">{{ t('admin.translationKey') }}</label>
              <input
                v-model="form.key"
                type="text"
                class="input"
                required
                :placeholder="activeSection === 'system' ? 'e.g., common.newKey or admin.label' : 'e.g., hero.title or about.description'"
                :disabled="!!editingId"
              />
              <small class="form-hint">Use dot notation: section.key</small>
            </div>

            <!-- When editing: show single locale -->
            <div v-if="editingId && editingLocale" class="form-group">
              <label class="form-label">{{ localeNames[editingLocale] || editingLocale }}</label>
              <textarea v-model="form.values[editingLocale]" class="input" rows="3" required dir="auto"></textarea>
            </div>

            <!-- When adding: show ALL locales -->
            <template v-else>
              <div v-for="loc in locales" :key="loc" class="form-group">
                <label class="form-label">{{ localeNames[loc] || loc }}</label>
                <textarea
                  v-model="form.values[loc]"
                  class="input"
                  rows="2"
                  :dir="loc === 'he' ? 'rtl' : 'ltr'"
                  :placeholder="`${localeNames[loc]} value`"
                ></textarea>
              </div>
            </template>
          </form>

          <footer class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="showModal = false">
              {{ t('common.cancel') }}
            </button>
            <button
              type="submit"
              class="btn btn-primary"
              :disabled="saving"
              @click="saveTranslation"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>
