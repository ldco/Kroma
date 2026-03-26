<script setup lang="ts">
/**
 * Providers Page (Project-Scoped)
 *
 * AI provider account management for a specific project.
 * Configure OpenAI, Stability, Anthropic, and other AI providers.
 */
import type { CreateProviderInput } from '~/types/kroma'

definePageMeta({
  layout: 'kroma'
})

const route = useRoute()
const { t } = useI18n()
const projectSlug = computed(() => route.params.slug as string)

const { fetchProviders, createProvider, updateProvider, deleteProvider } = useProviders()

// Modal state
const showCreateModal = ref(false)
const showEditModal = ref(false)
const editingProviderCode = ref<string | null>(null)
const editingProvider = ref<any>(null)

// Fetch providers on mount
onMounted(async () => {
  await fetchProviders(projectSlug.value)
})

// Get all providers
const { providers, isLoading, hasProviders } = useProvidersStore()

// Open create modal
function openCreateModal() {
  editingProvider.value = null
  showCreateModal.value = true
}

// Open edit modal
function openEditModal(providerCode: string) {
  const provider = providers.value.find(p => p.provider_code === providerCode)
  if (provider) {
    editingProvider.value = provider
    editingProviderCode.value = providerCode
    showEditModal.value = true
  }
}

// Close modals
function closeCreateModal() {
  showCreateModal.value = false
  editingProvider.value = null
}

function closeEditModal() {
  showEditModal.value = false
  editingProviderCode.value = null
  editingProvider.value = null
}

// Handle create
async function handleCreate(input: CreateProviderInput) {
  try {
    await createProvider(projectSlug.value, input)
    closeCreateModal()
  } catch (error) {
    // Error handled by composable
  }
}

// Handle edit (update name only)
async function handleEdit(input: CreateProviderInput) {
  if (!editingProviderCode.value) return

  try {
    await updateProvider(projectSlug.value, editingProviderCode.value, {
      name: input.name
    })
    closeEditModal()
  } catch (error) {
    // Error handled by composable
  }
}

// Handle delete
async function handleDelete(providerCode: string) {
  const confirmed = await confirm({
    title: 'Delete Provider',
    message:
      'Are you sure you want to delete this provider? This will remove the API key configuration.',
    confirmText: 'Delete',
    cancelText: 'Cancel',
    confirmVariant: 'danger'
  })

  if (confirmed) {
    try {
      await deleteProvider(projectSlug.value, providerCode)
    } catch (error) {
      // Error handled by composable
    }
  }
}
</script>

<template>
  <div class="providers-page">
    <!-- Page Header -->
    <header class="page-header">
      <div class="page-header__content">
        <h1 class="page-header__title">
          {{ t('app.providers') }}
        </h1>
        <p class="page-header__subtitle">
          {{ t('app.providers.subtitle') }}
        </p>
      </div>
      <div class="page-header__actions">
        <button class="btn btn-primary" @click="openCreateModal">
          <Icon name="lucide:plus" class="icon-sm" />
          {{ t('app.providers.add') }}
        </button>
      </div>
    </header>

    <!-- Loading State -->
    <div v-if="isLoading" class="providers-loading">
      <LoadingTable />
    </div>

    <!-- Empty State -->
    <div v-else-if="!hasProviders" class="empty-state">
      <Icon name="lucide:key" class="empty-state__icon" />
      <h3 class="empty-state__title">{{ t('app.providers.empty.title') }}</h3>
      <p class="empty-state__description">
        {{ t('app.providers.empty.description') }}
      </p>
      <button class="btn btn-primary" @click="openCreateModal">
        {{ t('app.providers.add') }}
      </button>
    </div>

    <!-- Providers Grid -->
    <div v-else class="providers-grid">
      <AtomsProviderCard
        v-for="provider in providers"
        :key="provider.provider_code"
        :provider="provider"
        @edit="openEditModal"
        @delete="handleDelete"
      />
    </div>

    <!-- Create Provider Modal -->
    <OrganismsConfirmDialog
      v-model="showCreateModal"
      :title="t('app.providers.add')"
      :show-confirm="false"
      :show-cancel="false"
    >
      <div class="modal-container">
        <MoleculesProviderAccountForm
          mode="create"
          @submit="handleCreate"
          @cancel="closeCreateModal"
        />
      </div>
    </OrganismsConfirmDialog>

    <!-- Edit Provider Modal -->
    <OrganismsConfirmDialog
      v-model="showEditModal"
      :title="t('app.providers.edit')"
      :show-confirm="false"
      :show-cancel="false"
    >
      <div class="modal-container">
        <MoleculesProviderAccountForm
          v-if="editingProvider"
          :provider="editingProvider"
          mode="edit"
          @submit="handleEdit"
          @cancel="closeEditModal"
        />
      </div>
    </OrganismsConfirmDialog>
  </div>
</template>

<!--
  Uses global CSS classes:
  - ui/providers/provider-card.css: .provider-card
  - ui/providers/provider-grid.css: .providers-grid
  - ui/forms/index.css: form styles
  - ui/content/empty-states.css: .empty-state
-->
