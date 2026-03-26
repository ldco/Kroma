<script setup lang="ts">
/**
 * Run Detail Page
 *
 * View run details and review candidates.
 * Note: Candidate selection is a UI-only curation action that feeds into
 * the next run's scene_refs input. There is no backend concept of 'approving' a candidate.
 * Retry is simply triggering a new run with the same parameters.
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
const runId = computed(() => route.params.id as string)

const runsStore = useRunsStore()
const { fetchRun, triggerRun } = useRuns()

// Selected candidate (UI-only curation)
const selectedCandidateId = ref<string | null>(null)

// Fetch run on mount
onMounted(async () => {
  await fetchRun(projectSlug.value, runId.value)
})

// Get run
const run = computed(() => runsStore.activeRun)
const isLoading = computed(() => runsStore.isLoading)

// Get candidates
const candidates = computed(() => run.value?.candidates || [])
const hasWinner = computed(() => candidates.value.some(c => c.is_winner))
const winner = computed(() => candidates.value.find(c => c.is_winner))

// Select candidate (UI-only action)
function selectCandidate(candidateId: string) {
  selectedCandidateId.value = candidateId
}

// Retry run - triggers a new run with same parameters
async function handleRetry() {
  if (!run.value) return

  const confirmed = await confirm({
    title: 'Retry Run',
    message: 'This will create a new run with the same settings. Continue?',
    confirmText: 'Retry',
    cancelText: 'Cancel'
  })

  if (confirmed) {
    try {
      // Build run config from current run
      const config: RunConfig = {
        prompt: run.value.prompt,
        negative_prompt: run.value.negative_prompt || undefined
      }
      await triggerRun(projectSlug.value, config)
      navigateTo(`/app/projects/${projectSlug.value}/runs`)
    } catch (error) {
      // Error handled by composable
    }
  }
}

// Navigate back
function goBack() {
  navigateTo(`/app/projects/${projectSlug.value}/runs`)
}

// Format date
function formatDate(dateString: string) {
  return new Date(dateString).toLocaleString()
}
</script>

<template>
  <div class="run-detail-page">
    <!-- Page Header -->
    <header class="page-header">
      <div class="page-header__content">
        <button class="back-link" @click="goBack">
          <Icon name="lucide:arrow-left" class="icon-sm" />
          Back to Runs
        </button>
        <h1 class="page-header__title">
          Run {{ runId.slice(0, 8) }}...
        </h1>
        <div class="run-meta">
          <AtomsRunStatusBadge v-if="run" :status="run.status" />
          <span v-if="run" class="run-date">{{ formatDate(run.created_at) }}</span>
        </div>
      </div>
      <div class="page-header__actions">
        <button
          v-if="run?.status === 'failed'"
          class="btn btn-secondary"
          @click="handleRetry"
        >
          <Icon name="lucide:refresh-cw" class="icon-sm" />
          Retry Run
        </button>
      </div>
    </header>

    <!-- Loading State -->
    <div v-if="isLoading || !run" class="run-detail-loading">
      <LoadingText />
    </div>

    <template v-else>
      <!-- Run Info -->
      <section class="run-info-section">
        <div class="info-card">
          <h3 class="info-card__title">Run Details</h3>
          <div class="info-card__grid">
            <div class="info-item">
              <span class="info-item__label">Prompt:</span>
              <span class="info-item__value">{{ run.prompt }}</span>
            </div>
            <div v-if="run.negative_prompt" class="info-item">
              <span class="info-item__label">Negative Prompt:</span>
              <span class="info-item__value">{{ run.negative_prompt }}</span>
            </div>
            <div class="info-item">
              <span class="info-item__label">Candidates:</span>
              <span class="info-item__value">{{ run.candidates?.length || 0 }}</span>
            </div>
            <div class="info-item">
              <span class="info-item__label">Status:</span>
              <AtomsRunStatusBadge :status="run.status" />
            </div>
          </div>
        </div>
      </section>

      <!-- Winner Display -->
      <section v-if="winner" class="winner-section">
        <div class="winner-card">
          <div class="winner-card__badge">
            <Icon name="lucide:trophy" class="icon-lg" />
            <span>Winner</span>
          </div>
          <img :src="winner.image_url" alt="Winner" class="winner-card__image" />
          <div class="winner-card__meta">
            <span>Seed: {{ winner.metadata?.seed }}</span>
            <span>Steps: {{ winner.metadata?.steps }}</span>
            <span>Size: {{ winner.metadata?.width }}×{{ winner.metadata?.height }}</span>
          </div>
        </div>
      </section>

      <!-- Candidates Grid -->
      <section v-if="candidates.length > 0 && !hasWinner" class="candidates-section">
        <div class="section-header">
          <h2 class="section-header__title">Candidates</h2>
          <p class="section-header__subtitle">
            Select a candidate to review. Note: Candidate selection is a UI-only curation action.
          </p>
        </div>

        <div class="candidates-grid">
          <AtomsCandidateCard
            v-for="candidate in candidates"
            :key="candidate.id"
            :candidate="candidate"
            :selected="candidate.id === selectedCandidateId"
            @select="selectCandidate"
          />
        </div>

        <div v-if="selectedCandidateId" class="candidates-actions">
          <div class="info-banner">
            <Icon name="lucide:info" class="icon-md" />
            <p>Candidate selected. This is a UI-only action for curation purposes.</p>
          </div>
        </div>
      </section>

      <!-- No Candidates -->
      <section v-if="run.status === 'pending' || run.status === 'running'" class="waiting-section">
        <div class="waiting-state">
          <Icon name="lucide:loader" class="waiting-state__icon" />
          <h3 class="waiting-state__title">Run in Progress</h3>
          <p class="waiting-state__description">
            Generating candidates... This may take a few minutes
          </p>
        </div>
      </section>
    </template>
  </div>
</template>

<!--
  Uses global CSS classes:
  - ui/runs/run-detail.css: .run-detail-page
  - ui/runs/candidate-card.css: .candidates-grid
  - ui/runs/run-status.css: .run-status-badge
-->
