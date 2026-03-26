<script setup lang="ts">
/**
 * Admin Team Page
 *
 * Manage team members with unified multi-language support.
 * All languages are equal - editable in the same modal with tabs.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox, .input-row, .form-divider
 * - ui/content/tabs.css: .tabs, .tabs--underline, .tab, .is-active, .tab__indicator
 * - ui/content/cards.css: .card, .card-body, .card-actions
 * - ui/admin/pages.css: .team-admin-grid, .team-admin-card__*
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import IconUpload from '~icons/tabler/upload'
import config from '~/puppet-master.config'
import AppImage from '~/components/atoms/AppImage.vue'

interface TeamTranslation {
  position: string | null
  bio: string | null
}

interface TeamMember {
  id: number
  slug: string
  name: string
  photoUrl: string | null
  hoverPhotoUrl: string | null
  email: string | null
  phone: string | null
  department: string | null
  socialLinks: Record<string, string> | null
  published: boolean | null
  order: number | null
  translations: Record<string, TeamTranslation>
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
  title: () => `${t('admin.team')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// Fetch team members
const headers = useRequestHeaders(['cookie'])
const {
  data: members,
  pending,
  refresh
} = await useFetch<TeamMember[]>('/api/admin/team', { headers })

// Modal state
const showModal = ref(false)
const editingMember = ref<TeamMember | null>(null)
const saving = ref(false)
const activeLocale = ref(locales[0] || 'en')

// Create empty translations structure for all locales
function createEmptyTranslations(): Record<string, { position: string; bio: string }> {
  const translations: Record<string, { position: string; bio: string }> = {}
  for (const locale of locales) {
    translations[locale] = { position: '', bio: '' }
  }
  return translations
}

const form = reactive({
  slug: '',
  name: '',
  photoUrl: '',
  hoverPhotoUrl: '',
  email: '',
  phone: '',
  department: '',
  socialLinks: {} as Record<string, string>,
  order: 0,
  published: true,
  translations: createEmptyTranslations()
})

// Social link editing
const newSocialKey = ref('')
const newSocialUrl = ref('')

function resetForm() {
  form.slug = ''
  form.name = ''
  form.photoUrl = ''
  form.hoverPhotoUrl = ''
  form.email = ''
  form.phone = ''
  form.department = ''
  form.socialLinks = {}
  form.order = 0
  form.published = true
  form.translations = createEmptyTranslations()
  activeLocale.value = locales[0] || 'en'
  newSocialKey.value = ''
  newSocialUrl.value = ''
}

function openCreate() {
  editingMember.value = null
  resetForm()
  showModal.value = true
}

function openEdit(member: TeamMember) {
  editingMember.value = member
  form.slug = member.slug
  form.name = member.name
  form.photoUrl = member.photoUrl || ''
  form.hoverPhotoUrl = member.hoverPhotoUrl || ''
  form.email = member.email || ''
  form.phone = member.phone || ''
  form.department = member.department || ''
  form.socialLinks = { ...(member.socialLinks || {}) }
  form.order = member.order || 0
  form.published = member.published ?? true

  // Load translations for all locales
  form.translations = createEmptyTranslations()
  for (const locale of locales) {
    const trans = member.translations?.[locale]
    if (trans) {
      form.translations[locale] = {
        position: trans.position || '',
        bio: trans.bio || ''
      }
    }
  }

  // Find first locale with content
  const localeWithContent = locales.find(locale => {
    const trans = member.translations?.[locale]
    return trans?.position || trans?.bio
  })
  activeLocale.value = localeWithContent || locales[0] || 'en'

  showModal.value = true
}

function generateSlug() {
  if (!editingMember.value && form.name && !form.slug) {
    form.slug = form.name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '')
  }
}

function addSocialLink() {
  if (newSocialKey.value && newSocialUrl.value) {
    form.socialLinks[newSocialKey.value] = newSocialUrl.value
    newSocialKey.value = ''
    newSocialUrl.value = ''
  }
}

function removeSocialLink(key: string) {
  delete form.socialLinks[key]
}

async function saveMember() {
  saving.value = true

  try {
    const payload = {
      slug: form.slug,
      name: form.name,
      photoUrl: form.photoUrl || null,
      hoverPhotoUrl: form.hoverPhotoUrl || null,
      email: form.email || null,
      phone: form.phone || null,
      department: form.department || null,
      socialLinks: Object.keys(form.socialLinks).length > 0 ? form.socialLinks : null,
      order: form.order,
      published: form.published,
      translations: form.translations
    }

    if (editingMember.value) {
      await $fetch(`/api/admin/team/${editingMember.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch('/api/admin/team', {
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

async function deleteMember(member: TeamMember) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/team/${member.id}`, { method: 'DELETE' })
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

// Get first available position for display in list
function getMemberPosition(member: TeamMember): string {
  for (const locale of locales) {
    const position = member.translations?.[locale]?.position
    if (position) return position
  }
  return ''
}

// Check if locale has translation
function hasTranslation(member: TeamMember, locale: string): boolean {
  return !!(member.translations?.[locale]?.position || member.translations?.[locale]?.bio)
}

// Check if all locales have translations
function hasAllTranslations(member: TeamMember): boolean {
  return locales.every(locale => hasTranslation(member, locale))
}

// Get list of missing translations
function getMissingTranslations(member: TeamMember): string[] {
  return locales.filter(locale => !hasTranslation(member, locale))
}

// Check if current form locale has content
function formLocaleHasContent(locale: string): boolean {
  const trans = form.translations[locale]
  return !!(trans?.position || trans?.bio)
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

    form.photoUrl = result.url
    toast.success(t('common.uploaded'))
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    uploadingPhoto.value = false
    input.value = ''
  }
}

const uploadingHoverPhoto = ref(false)
async function uploadHoverPhoto(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  uploadingHoverPhoto.value = true
  try {
    const formData = new FormData()
    formData.append('file', file)

    const result = await $fetch<{ url: string }>('/api/upload/image', {
      method: 'POST',
      body: formData
    })

    form.hoverPhotoUrl = result.url
    toast.success(t('common.uploaded'))
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    uploadingHoverPhoto.value = false
    input.value = ''
  }
}
</script>

<template>
  <div class="admin-team">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.team') }}</h1>
      <button class="btn btn-primary" @click="openCreate">
        <IconPlus />
        {{ t('admin.addItem') }}
      </button>
    </div>

    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <div v-else-if="!members?.length" class="empty-state">
      <p>{{ t('admin.noItems') }}</p>
    </div>

    <div v-else class="team-admin-grid">
      <div v-for="member in members" :key="member.id" class="card">
        <div class="card-body">
          <div class="team-admin-card__header">
            <div class="team-admin-card__photo">
              <AppImage
                :src="member.photoUrl"
                :alt="member.name"
                fallback="initials"
                :initials="member.name.charAt(0)"
              />
            </div>
            <div class="team-admin-card__info">
              <h3 class="team-admin-card__name">{{ member.name }}</h3>
              <p v-if="getMemberPosition(member)" class="team-admin-card__position">
                {{ getMemberPosition(member) }}
              </p>
              <span class="badge" :class="member.published ? 'badge-success' : 'badge-warning'">
                {{ member.published ? t('admin.published') : t('common.draft') }}
              </span>
            </div>
          </div>

          <!-- Translation status -->
          <div v-if="!hasAllTranslations(member)" class="mt-2">
            <span class="text-warning text-sm">
              {{ t('admin.missingTranslations') }}: {{ getMissingTranslations(member).map(l => l.toUpperCase()).join(', ') }}
            </span>
          </div>
        </div>

        <div class="card-actions">
          <button class="btn btn-sm btn-secondary" @click="openEdit(member)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button class="btn btn-sm btn-ghost text-error" @click="deleteMember(member)">
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
            <h2>{{ editingMember ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveMember">
            <!-- Non-translatable fields -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label" for="member-name">{{ t('admin.name') }} *</label>
                <input id="member-name" v-model="form.name" type="text" class="input" required @blur="generateSlug" />
              </div>

              <div class="form-group">
                <label class="form-label" for="member-slug">{{ t('admin.slug') }} *</label>
                <input id="member-slug" v-model="form.slug" type="text" class="input" required pattern="[a-z0-9-]+" />
              </div>

              <div class="form-group">
                <label class="form-label" for="member-department">{{ t('admin.department') }}</label>
                <input id="member-department" v-model="form.department" type="text" class="input" />
              </div>

              <div class="form-group">
                <label class="form-label" for="member-email">{{ t('admin.email') }}</label>
                <input id="member-email" v-model="form.email" type="email" class="input" />
              </div>

              <div class="form-group">
                <label class="form-label" for="member-phone">{{ t('admin.phone') }}</label>
                <input id="member-phone" v-model="form.phone" type="tel" class="input" />
              </div>

              <div class="form-group">
                <label class="form-label" for="member-order">{{ t('admin.order') }}</label>
                <input id="member-order" v-model.number="form.order" type="number" class="input" min="0" />
              </div>
            </div>

            <!-- Photo uploads -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label">{{ t('admin.photo') }}</label>
                <div class="input-row">
                  <input v-model="form.photoUrl" type="text" class="input" :placeholder="t('admin.photoUrl')" />
                  <label class="btn btn-secondary">
                    <IconUpload />
                    <input type="file" accept="image/*" class="sr-only" :disabled="uploadingPhoto" @change="uploadPhoto" />
                  </label>
                </div>
                <div v-if="form.photoUrl" class="mt-2">
                  <AppImage :src="form.photoUrl" alt="Preview" class="team-admin-card__photo-preview" />
                </div>
              </div>

              <div class="form-group">
                <label class="form-label">Hover Photo</label>
                <div class="input-row">
                  <input v-model="form.hoverPhotoUrl" type="text" class="input" placeholder="Hover photo URL" />
                  <label class="btn btn-secondary">
                    <IconUpload />
                    <input type="file" accept="image/*" class="sr-only" :disabled="uploadingHoverPhoto" @change="uploadHoverPhoto" />
                  </label>
                </div>
                <div v-if="form.hoverPhotoUrl" class="mt-2">
                  <AppImage :src="form.hoverPhotoUrl" alt="Hover Preview" class="team-admin-card__photo-preview" />
                </div>
              </div>
            </div>

            <!-- Social Links -->
            <div class="form-group">
              <label class="form-label">{{ t('admin.socialLinks') }}</label>
              <div v-for="(url, key) in form.socialLinks" :key="key" class="input-row mb-2">
                <input :value="key" type="text" class="input" disabled />
                <input :value="url" type="text" class="input" disabled />
                <button type="button" class="btn btn-ghost btn-sm text-error" @click="removeSocialLink(key as string)">
                  <IconX />
                </button>
              </div>
              <div class="input-row">
                <input v-model="newSocialKey" type="text" class="input" placeholder="linkedin, twitter, github..." />
                <input v-model="newSocialUrl" type="url" class="input" placeholder="https://..." />
                <button type="button" class="btn btn-ghost btn-sm" @click="addSocialLink">
                  <IconPlus />
                </button>
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
                <label class="form-label">{{ t('admin.position') }}</label>
                <input v-model="form.translations[locale].position" type="text" class="input" />
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.bio') }}</label>
                <textarea v-model="form.translations[locale].bio" class="input" rows="3"></textarea>
              </div>
            </div>
          </form>

          <footer class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="showModal = false">
              {{ t('common.cancel') }}
            </button>
            <button type="submit" class="btn btn-primary" :disabled="saving" @click="saveMember">
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>
