<script setup lang="ts">
/**
 * Admin Clients Page
 *
 * Manage client logos and partnerships.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox, .input-row
 * - ui/content/cards.css: .card, .card-body, .card-actions
 * - ui/admin/pages.css: .clients-admin-grid, .clients-admin-card__*
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import IconUpload from '~icons/tabler/upload'
import IconExternalLink from '~icons/tabler/external-link'
import AppImage from '~/components/atoms/AppImage.vue'

interface Client {
  id: number
  slug: string
  name: string
  logoUrl: string | null
  websiteUrl: string | null
  category: string | null
  featured: boolean
  order: number
}

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('admin.clients')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// Fetch clients
const headers = useRequestHeaders(['cookie'])
const {
  data: clients,
  pending,
  refresh
} = await useAsyncData(
  'admin-clients',
  () => apiFetch<Client[]>('/api/admin/clients', { headers }),
  { default: () => [] }
)

// Modal state
const showModal = ref(false)
const editingClient = ref<Client | null>(null)
const saving = ref(false)

const form = reactive({
  slug: '',
  name: '',
  logoUrl: '',
  websiteUrl: '',
  category: '',
  featured: false,
  order: 0
})

function resetForm() {
  form.slug = ''
  form.name = ''
  form.logoUrl = ''
  form.websiteUrl = ''
  form.category = ''
  form.featured = false
  form.order = 0
}

function openCreate() {
  editingClient.value = null
  resetForm()
  showModal.value = true
}

function openEdit(client: Client) {
  editingClient.value = client
  form.slug = client.slug
  form.name = client.name
  form.logoUrl = client.logoUrl || ''
  form.websiteUrl = client.websiteUrl || ''
  form.category = client.category || ''
  form.featured = client.featured
  form.order = client.order
  showModal.value = true
}

function generateSlug() {
  if (!editingClient.value && form.name && !form.slug) {
    form.slug = form.name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '')
  }
}

async function saveClient() {
  saving.value = true

  try {
    const payload = {
      ...form,
      logoUrl: form.logoUrl || null,
      websiteUrl: form.websiteUrl || null,
      category: form.category || null
    }

    if (editingClient.value) {
      await $fetch(`/api/admin/clients/${editingClient.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch('/api/admin/clients', {
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

async function deleteClient(client: Client) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/clients/${client.id}`, { method: 'DELETE' })
    toast.success(t('common.deleted'))
    refresh()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  }
}

// Logo upload
const uploadingLogo = ref(false)
async function uploadLogo(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  uploadingLogo.value = true
  try {
    const formData = new FormData()
    formData.append('file', file)

    const result = await $fetch<{ url: string }>('/api/upload/image', {
      method: 'POST',
      body: formData
    })

    form.logoUrl = result.url
    toast.success(t('common.uploaded'))
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    uploadingLogo.value = false
    input.value = ''
  }
}

</script>

<template>
  <div class="admin-clients">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.clients') }}</h1>
      <button class="btn btn-primary" @click="openCreate">
        <IconPlus />
        {{ t('admin.addItem') }}
      </button>
    </div>

    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <div v-else-if="!clients?.length" class="empty-state">
      <p>{{ t('admin.noItems') }}</p>
    </div>

    <div v-else class="clients-admin-grid">
      <div v-for="client in clients" :key="client.id" class="card">
        <div class="card-body">
          <div class="clients-admin-card__header">
            <div class="clients-admin-card__logo">
              <AppImage
                :src="client.logoUrl"
                :alt="client.name"
                fallback="initials"
                :initials="client.name.charAt(0)"
              />
            </div>
            <div class="clients-admin-card__info">
              <h3 class="clients-admin-card__name">{{ client.name }}</h3>
              <p v-if="client.category" class="text-secondary text-sm">{{ client.category }}</p>
              <div class="clients-admin-card__badges">
                <span v-if="client.featured" class="badge badge-success">Featured</span>
                <a
                  v-if="client.websiteUrl"
                  :href="client.websiteUrl"
                  target="_blank"
                  class="text-sm text-brand"
                >
                  <IconExternalLink class="icon-sm" />
                </a>
              </div>
            </div>
          </div>
        </div>

        <div class="card-actions">
          <button class="btn btn-sm btn-secondary" @click="openEdit(client)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button class="btn btn-sm btn-ghost text-error" @click="deleteClient(client)">
            <IconTrash />
          </button>
        </div>
      </div>
    </div>

    <!-- Modal -->
    <Teleport to="body">
      <div v-if="showModal" class="modal-backdrop" @click.self="showModal = false">
        <div class="modal">
          <header class="modal-header">
            <h2>{{ editingClient ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showModal = false">
              <IconX />
            </button>
          </header>

          <form class="modal-body" @submit.prevent="saveClient">
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label" for="client-name">{{ t('admin.name') }} *</label>
                <input
                  id="client-name"
                  v-model="form.name"
                  type="text"
                  class="input"
                  required
                  @blur="generateSlug"
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="client-slug">{{ t('admin.slug') }} *</label>
                <input
                  id="client-slug"
                  v-model="form.slug"
                  type="text"
                  class="input"
                  required
                  pattern="[a-z0-9-]+"
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="client-category">{{ t('admin.category') }}</label>
                <input
                  id="client-category"
                  v-model="form.category"
                  type="text"
                  class="input"
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="client-order">{{ t('admin.order') }}</label>
                <input
                  id="client-order"
                  v-model.number="form.order"
                  type="number"
                  class="input"
                  min="0"
                />
              </div>
            </div>

            <div class="form-group">
              <label class="form-label" for="client-website">{{ t('admin.websiteUrl') }}</label>
              <input
                id="client-website"
                v-model="form.websiteUrl"
                type="url"
                class="input"
                placeholder="https://..."
              />
            </div>

            <div class="form-group">
              <label class="form-label">{{ t('admin.logo') }}</label>
              <div class="input-row">
                <input
                  v-model="form.logoUrl"
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
                    :disabled="uploadingLogo"
                    @change="uploadLogo"
                  />
                </label>
              </div>
              <div v-if="form.logoUrl" class="mt-2">
                <AppImage :src="form.logoUrl" alt="Preview" class="clients-admin-logo-preview" />
              </div>
            </div>

            <div class="form-row">
              <label class="form-checkbox">
                <input v-model="form.featured" type="checkbox" />
                <span>{{ t('admin.featured') }}</span>
              </label>
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
              @click="saveClient"
            >
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>
