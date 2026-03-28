/**
 * Runs Pinia Store
 *
 * Manages run state for Kroma app.
 * Uses useKromaApi for direct backend communication with Bearer auth.
 */
import { defineStore } from 'pinia'
import type { Run, RunConfig, Candidate } from '~/types/kroma'

export interface RunsState {
  runs: Run[]
  activeRun: Run | null
  isLoading: boolean
  error: string | null
}

export const useRunsStore = defineStore('runs', {
  state: (): RunsState => ({
    runs: [],
    activeRun: null,
    isLoading: false,
    error: null
  }),

  getters: {
    /**
     * Get run by ID
     */
    getRunById: (state) => (id: string) => {
      return state.runs.find(r => r.id === id)
    },

    /**
     * Get runs by status
     */
    getRunsByStatus: (state) => (status: string) => {
      return state.runs.filter(r => r.status === status)
    },

    /**
     * Get pending runs
     */
    pendingRuns: (state) => {
      return state.runs.filter(r => r.status === 'pending')
    },

    /**
     * Get running runs
     */
    runningRuns: (state) => {
      return state.runs.filter(r => r.status === 'running')
    },

    /**
     * Get completed runs
     */
    completedRuns: (state) => {
      return state.runs.filter(r => r.status === 'completed')
    },

    /**
     * Get total run count
     */
    runCount: (state) => state.runs.length,

    /**
     * Check if store has any runs
     */
    hasRuns: (state) => state.runs.length > 0
  },

  actions: {
    /**
     * Fetch all runs for a project
     */
    async fetchRuns(projectSlug: string) {
      this.isLoading = true
      this.error = null

      try {
        const api = useKromaApi()
        const response = await api.getRuns(projectSlug)
        this.runs = response
        return response
      } catch (error) {
        this.error = 'Failed to load runs'
        console.error('Error fetching runs:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Fetch single run by ID
     */
    async fetchRun(projectSlug: string, runId: string) {
      this.isLoading = true
      this.error = null

      try {
        const api = useKromaApi()
        const response = await api.getRun(projectSlug, runId)
        this.activeRun = response
        return response
      } catch (error) {
        this.error = 'Failed to load run'
        console.error('Error fetching run:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Trigger new run
     */
    async triggerRun(projectSlug: string, config: RunConfig) {
      this.isLoading = true
      this.error = null

      try {
        const api = useKromaApi()
        const response = await api.triggerRun(projectSlug, config)

        // Add to local state
        this.runs.unshift(response)

        return response
      } catch (error) {
        this.error = 'Failed to trigger run'
        console.error('Error triggering run:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Clear active run
     */
    clearActiveRun() {
      this.activeRun = null
    },

    /**
     * Clear error
     */
    clearError() {
      this.error = null
    }
  }
})
