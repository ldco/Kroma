/**
 * Projects Pinia Store
 *
 * Manages project state for Kroma app.
 */
import { defineStore } from 'pinia'
import type { Project, ProjectSummary, ProjectDetail, CreateProjectInput } from '~/types/kroma'

export interface ProjectsState {
  projects: ProjectSummary[]
  activeProject: ProjectDetail | null
  isLoading: boolean
  error: string | null
}

export const useProjectsStore = defineStore('projects', {
  state: (): ProjectsState => ({
    projects: [],
    activeProject: null,
    isLoading: false,
    error: null
  }),

  getters: {
    /**
     * Get project by slug
     */
    getProjectBySlug: (state) => (slug: string) => {
      return state.projects.find(p => p.slug === slug)
    },

    /**
     * Get total project count
     */
    projectCount: (state) => state.projects.length,

    /**
     * Check if store has any projects
     */
    hasProjects: (state) => state.projects.length > 0
  },

  actions: {
    /**
     * Fetch all projects
     */
    async fetchProjects() {
      this.isLoading = true
      this.error = null

      try {
        const response = await apiFetch<{ projects: ProjectSummary[] }>('/api/projects')
        this.projects = response.projects
        return response.projects
      } catch (error) {
        this.error = 'Failed to load projects'
        console.error('Error fetching projects:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Fetch single project by slug
     */
    async fetchProject(slug: string) {
      this.isLoading = true
      this.error = null

      try {
        const response = await apiFetch<ProjectDetail>(`/api/projects/${slug}`)
        this.activeProject = response
        return response
      } catch (error) {
        this.error = 'Failed to load project'
        console.error('Error fetching project:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Create new project
     */
    async createProject(input: CreateProjectInput) {
      this.isLoading = true
      this.error = null

      try {
        const response = await apiFetch<Project>('/api/projects', {
          method: 'POST',
          body: input
        })

        // Refresh projects list
        await this.fetchProjects()

        return response
      } catch (error) {
        this.error = 'Failed to create project'
        console.error('Error creating project:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Update project
     */
    async updateProject(slug: string, input: Partial<CreateProjectInput>) {
      this.isLoading = true
      this.error = null

      try {
        const response = await apiFetch<Project>(`/api/projects/${slug}`, {
          method: 'PUT',
          body: input
        })

        // Update in local state
        const index = this.projects.findIndex(p => p.slug === slug)
        if (index !== -1) {
          this.projects[index] = { ...this.projects[index], ...response }
        }

        // Update active project if it's the same
        if (this.activeProject?.project.slug === slug) {
          this.activeProject.project = { ...this.activeProject.project, ...response }
        }

        return response
      } catch (error) {
        this.error = 'Failed to update project'
        console.error('Error updating project:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    /**
     * Delete project
     * NOTE: This operation is not yet supported by the backend.
     * Backend contract does not include DELETE /api/projects/{slug}.
     */
    async deleteProject(slug: string) {
      throw new Error('Project deletion is not yet supported by the backend.')
    },

    /**
     * Clear active project
     */
    clearActiveProject() {
      this.activeProject = null
    },

    /**
     * Clear error
     */
    clearError() {
      this.error = null
    }
  }
})
