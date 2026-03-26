<script setup lang="ts">
/**
 * Project Runs Page
 *
 * List of all runs for a project.
 * Allows triggering new runs and viewing run history.
 */
import { useRunsStore } from '~/stores/runs'
import type { RunConfig } from '~/types/kroma'

definePageMeta({
  layout: 'kroma'
})

const route = useRoute()
const { t } = useI18n()
const localePath = useLocalePath()
const projectSlug = computed(() => route.params.slug as string)

const runsStore = useRunsStore()
const { fetchRuns, triggerRun } = useRuns()

// Modal state
const showTriggerModal = ref(false)

// Fetch runs on mount
onMounted(async () => {
  await fetchRuns(projectSlug.value)
})

// Get runs
const runs = computed(() => runsStore.runs)
const isLoading = computed(() => runsStore.isLoading)
const hasRuns = computed(() => runsStore.hasRuns)

// Navigate to run detail
function navigateToRun(runId: string) {
  navigateTo(`/app/projects/${projectSlug.value}/runs/${runId}`)
}

// Open trigger modal
function openTriggerModal() {
  showTriggerModal.value = true
}

// Close trigger modal
function closeTriggerModal() {
  showTriggerModal.value = false
}

// Handle trigger run
async function handleTrigger(config: RunConfig) {
  try {
    await triggerRun(projectSlug.value, config)
    closeTriggerModal()
  } catch (error) {
    // Error handled by composable
  }
}

// Format date
function formatDate(dateString: string) {
  return new Date(dateString).toLocaleString()
}
</script>

<template>
  <div class="runs-page">
    <!-- Page Header -->
    <header class="page-header">
      <div class="page-header__content">
        <NuxtLink :to="localePath(`/app/projects/${projectSlug}`)" class="back-link">
          <Icon name="lucide:arrow-left" class="icon-sm" />
          Back to Project
        </NuxtLink>
        <h1 class="page-header__title">
          {{ t('app.runs.title') }}
        </h1>
        <p class="page-header__subtitle">
          {{ t('app.runs.subtitle') }}
        </p>
      </div>
      <div class="page-header__actions">
        <button class="btn btn-primary" @click="openTriggerModal">
          <Icon name="lucide:zap" class="icon-sm" />
          {{ t('app.runs.create') }}
        </button>
      </div>
    </header>

    <!-- Loading State -->
    <div v-if="isLoading" class="runs-loading">
      <LoadingTable />
    </div>

    <!-- Empty State -->
    <div v-else-if="!hasRuns" class="empty-state">
      <Icon name="lucide:zap" class="empty-state__icon" />
      <h3 class="empty-state__title">{{ t('app.runs.empty.title') }}</h3>
      <p class="empty-state__description">
        {{ t('app.runs.empty.description') }}
      </p>
      <button class="btn btn-primary" @click="openTriggerModal">
        {{ t('app.runs.create') }}
      </button>
    </div>

    <!-- Runs Table -->
    <div v-else class="runs-table-wrapper">
      <table class="runs-table">
        <thead>
          <tr>
            <th>Run ID</th>
            <th>Status</th>
            <th>Prompt</th>
            <th>Candidates</th>
            <th>Created</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="run in runs"
            :key="run.id"
            class="runs-table__row"
            @click="navigateToRun(run.id)"
          >
            <td class="runs-table__cell">
              <span class="run-id">{{ run.id.slice(0, 8) }}...</span>
            </td>
            <td class="runs-table__cell">
              <AtomsRunStatusBadge :status="run.status" />
            </td>
            <td class="runs-table__cell runs-table__cell--prompt">
              <span class="prompt-text">{{ run.prompt.slice(0, 100) }}{{ run.prompt.length > 100 ? '...' : '' }}</span>
            </td>
            <td class="runs-table__cell">
              {{ run.candidates?.length || 0 }}
            </td>
            <td class="runs-table__cell">
              <span class="date-text">{{ formatDate(run.created_at) }}</span>
            </td>
            <td class="runs-table__cell runs-table__cell--actions">
              <button
                class="btn btn-ghost btn-sm"
                @click.stop="navigateToRun(run.id)"
              >
                View
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Trigger Run Modal -->
    <OrganismsConfirmDialog
      v-model="showTriggerModal"
      :title="t('app.runs.create')"
      :show-confirm="false"
      :show-cancel="false"
    >
      <div class="modal-container">
        <MoleculesRunTriggerForm
          :project-slug="projectSlug"
          @submit="handleTrigger"
          @cancel="closeTriggerModal"
        />
      </div>
    </OrganismsConfirmDialog>
  </div>
</template>

<!--
  Uses global CSS classes:
  - ui/runs/runs-table.css: .runs-table
  - ui/runs/run-status.css: .run-status-badge
  - ui/content/empty-states.css: .empty-state
-->
