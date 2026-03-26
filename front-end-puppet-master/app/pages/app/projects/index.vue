<script setup lang="ts">
/**
 * Projects List Page
 *
 * Displays all projects with create, edit, delete functionality.
 */
import { useProjectsStore } from '~/stores/projects'
import type { CreateProjectInput } from '~/types/kroma'

definePageMeta({
  layout: 'kroma'
})

const { t } = useI18n()
const localePath = useLocalePath()
const projectsStore = useProjectsStore()
const { fetchProjects, createProject, updateProject, deleteProject } = useProjects()

// Modal state
const showCreateModal = ref(false)
const showEditModal = ref(false)
const editingProjectSlug = ref<string | null>(null)

// Form state
const formName = ref('')
const formDescription = ref('')

// Fetch projects on mount
onMounted(async () => {
  await fetchProjects()
})

// Get all projects
const projects = computed(() => projectsStore.projects)
const isLoading = computed(() => projectsStore.isLoading)

// Open create modal
function openCreateModal() {
  formName.value = ''
  formDescription.value = ''
  showCreateModal.value = true
}

// Open edit modal
function openEditModal(slug: string) {
  const project = projects.value.find(p => p.slug === slug)
  if (project) {
    formName.value = project.name
    formDescription.value = project.description || ''
    editingProjectSlug.value = slug
    showEditModal.value = true
  }
}

// Close modals
function closeCreateModal() {
  showCreateModal.value = false
}

function closeEditModal() {
  showEditModal.value = false
  editingProjectSlug.value = null
}

// Handle create
async function handleCreate() {
  if (!formName.value.trim()) {
    toast.error('Project name is required')
    return
  }

  const input: CreateProjectInput = {
    name: formName.value.trim(),
    description: formDescription.value.trim() || null
  }

  try {
    await createProject(input)
    closeCreateModal()
  } catch (error) {
    // Error handled by composable
  }
}

// Handle edit
async function handleEdit() {
  if (!formName.value.trim() || !editingProjectSlug.value) {
    toast.error('Project name is required')
    return
  }

  const input: Partial<CreateProjectInput> = {
    name: formName.value.trim(),
    description: formDescription.value.trim() || null
  }

  try {
    await updateProject(editingProjectSlug.value, input)
    closeEditModal()
  } catch (error) {
    // Error handled by composable
  }
}

// Handle delete
// NOTE: Project deletion is not yet supported by the backend.
async function handleDelete(slug: string) {
  toast.error('Project deletion is not yet supported by the backend.')
  // Backend contract does not include DELETE /api/projects/{slug}
  // This is a known gap tracked for future implementation
}

// Navigate to project detail
function navigateToProject(slug: string) {
  navigateTo(`/app/projects/${slug}`)
}
</script>

<template>
  <div class="projects-page">
    <!-- Page Header -->
    <header class="page-header">
      <div class="page-header__content">
        <h1 class="page-header__title">
          {{ t('app.projects.title') }}
        </h1>
        <p class="page-header__subtitle">
          {{ t('app.projects.subtitle') }}
        </p>
      </div>
      <div class="page-header__actions">
        <button class="btn btn-primary" @click="openCreateModal">
          <Icon name="lucide:plus" class="icon-sm" />
          {{ t('app.projects.create') }}
        </button>
      </div>
    </header>

    <!-- Loading State -->
    <div v-if="isLoading" class="projects-loading">
      <LoadingTable />
    </div>

    <!-- Empty State -->
    <div v-else-if="projects.length === 0" class="empty-state">
      <Icon name="lucide:folder-open" class="empty-state__icon" />
      <h3 class="empty-state__title">{{ t('app.projects.empty.title') }}</h3>
      <p class="empty-state__description">
        {{ t('app.projects.empty.description') }}
      </p>
      <button class="btn btn-primary" @click="openCreateModal">
        {{ t('app.projects.create') }}
      </button>
    </div>

    <!-- Projects Grid -->
    <div v-else class="projects-grid">
      <AtomsProjectCard
        v-for="project in projects"
        :key="project.slug"
        :project="project"
        @select="navigateToProject"
        @edit="openEditModal"
        @delete="handleDelete"
      />
    </div>

    <!-- Create Project Modal -->
    <OrganismsConfirmDialog
      v-model="showCreateModal"
      :title="t('app.projects.create')"
      :show-confirm="false"
      :show-cancel="false"
    >
      <div class="modal-form">
        <div class="form-group">
          <label for="project-name" class="form-label">
            {{ t('app.projects.name') }}
          </label>
          <input
            id="project-name"
            v-model="formName"
            type="text"
            class="form-input"
            :placeholder="t('app.projects.namePlaceholder')"
            @keyup.enter="handleCreate"
          />
        </div>

        <div class="form-group">
          <label for="project-description" class="form-label">
            {{ t('app.projects.description') }}
          </label>
          <textarea
            id="project-description"
            v-model="formDescription"
            class="form-input form-input--textarea"
            :placeholder="t('app.projects.descriptionPlaceholder')"
            rows="3"
          />
        </div>

        <div class="modal-actions">
          <button class="btn btn-ghost" @click="closeCreateModal">
            {{ t('common.cancel') }}
          </button>
          <button class="btn btn-primary" @click="handleCreate">
            {{ t('app.projects.create') }}
          </button>
        </div>
      </div>
    </OrganismsConfirmDialog>

    <!-- Edit Project Modal -->
    <OrganismsConfirmDialog
      v-model="showEditModal"
      :title="t('app.projects.edit')"
      :show-confirm="false"
      :show-cancel="false"
    >
      <div class="modal-form">
        <div class="form-group">
          <label for="edit-project-name" class="form-label">
            {{ t('app.projects.name') }}
          </label>
          <input
            id="edit-project-name"
            v-model="formName"
            type="text"
            class="form-input"
            :placeholder="t('app.projects.namePlaceholder')"
            @keyup.enter="handleEdit"
          />
        </div>

        <div class="form-group">
          <label for="edit-project-description" class="form-label">
            {{ t('app.projects.description') }}
          </label>
          <textarea
            id="edit-project-description"
            v-model="formDescription"
            class="form-input form-input--textarea"
            :placeholder="t('app.projects.descriptionPlaceholder')"
            rows="3"
          />
        </div>

        <div class="modal-actions">
          <button class="btn btn-ghost" @click="closeEditModal">
            {{ t('common.cancel') }}
          </button>
          <button class="btn btn-primary" @click="handleEdit">
            {{ t('common.save') }}
          </button>
        </div>
      </div>
    </OrganismsConfirmDialog>
  </div>
</template>

<!--
  Uses global CSS classes:
  - ui/projects/project-card.css: .project-card
  - ui/projects/project-grid.css: .projects-grid
  - ui/forms/index.css: .form-group, .form-label, .form-input
  - ui/content/empty-states.css: .empty-state
  - ui/overlays/modal.css: .modal-form, .modal-actions
-->
