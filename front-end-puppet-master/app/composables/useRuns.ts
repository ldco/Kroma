/**
 * Runs Composable
 *
 * Run management for Kroma app.
 * Provides run CRUD operations and candidate management.
 * Note: approveCandidate, retryRun, and cancelRun have been removed as they
 * do not exist in the backend contract. Candidate approval is a UI-only curation
 * action. Retry is simply triggering a new run with the same parameters.
 */
import { useRunsStore } from '~/stores/runs'
import type { Run, RunConfig, Candidate } from '~/types/kroma'

export interface UseRunsReturn {
  // State
  runs: ComputedRef<Run[]>
  activeRun: ComputedRef<Run | null>
  isLoading: ComputedRef<boolean>
  error: ComputedRef<string | null>

  // Getters
  getRunById: (id: string) => Run | undefined
  getRunsByStatus: (status: string) => Run[]
  pendingRuns: ComputedRef<Run[]>
  runningRuns: ComputedRef<Run[]>
  completedRuns: ComputedRef<Run[]>
  runCount: ComputedRef<number>
  hasRuns: ComputedRef<boolean>

  // Actions
  fetchRuns: (projectSlug: string) => Promise<Run[]>
  fetchRun: (projectSlug: string, runId: string) => Promise<Run>
  triggerRun: (projectSlug: string, config: RunConfig) => Promise<Run>
  clearActiveRun: () => void
  clearError: () => void
}

export function useRuns(): UseRunsReturn {
  const store = useRunsStore()
  const toast = useToast()

  // State
  const runs = computed(() => store.runs)
  const activeRun = computed(() => store.activeRun)
  const isLoading = computed(() => store.isLoading)
  const error = computed(() => store.error)

  // Getters
  const getRunById = (id: string) => store.getRunById(id)
  const getRunsByStatus = (status: string) => store.getRunsByStatus(status)
  const pendingRuns = computed(() => store.pendingRuns)
  const runningRuns = computed(() => store.runningRuns)
  const completedRuns = computed(() => store.completedRuns)
  const runCount = computed(() => store.runCount)
  const hasRuns = computed(() => store.hasRuns)

  // Actions with error handling
  async function fetchRuns(projectSlug: string): Promise<Run[]> {
    try {
      return await store.fetchRuns(projectSlug)
    } catch (error) {
      toast.error('Failed to load runs')
      throw error
    }
  }

  async function fetchRun(projectSlug: string, runId: string): Promise<Run> {
    try {
      return await store.fetchRun(projectSlug, runId)
    } catch (error) {
      toast.error('Failed to load run')
      throw error
    }
  }

  async function triggerRun(projectSlug: string, config: RunConfig): Promise<Run> {
    try {
      const result = await store.triggerRun(projectSlug, config)
      toast.success('Run triggered successfully')
      return result
    } catch (error) {
      toast.error('Failed to trigger run')
      throw error
    }
  }

  function clearActiveRun() {
    store.clearActiveRun()
  }

  function clearError() {
    store.clearError()
  }

  return {
    // State
    runs,
    activeRun,
    isLoading,
    error,

    // Getters
    getRunById,
    getRunsByStatus,
    pendingRuns,
    runningRuns,
    completedRuns,
    runCount,
    hasRuns,

    // Actions
    fetchRuns,
    fetchRun,
    triggerRun,
    clearActiveRun,
    clearError
  }
}
