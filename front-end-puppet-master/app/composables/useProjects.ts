/**
 * Projects Composable
 *
 * Project management for Kroma app.
 * Provides project CRUD operations and state management.
 */
import { useProjectsStore } from '~/stores/projects'
import type { ProjectSummary, ProjectDetail, CreateProjectInput } from '~/types/kroma'

export interface UseProjectsReturn {
  // State
  projects: ComputedRef<ProjectSummary[]>
  activeProject: ComputedRef<ProjectDetail | null>
  isLoading: ComputedRef<boolean>
  error: ComputedRef<string | null>

  // Getters
  getProjectBySlug: (slug: string) => ProjectSummary | undefined
  projectCount: ComputedRef<number>
  hasProjects: ComputedRef<boolean>

  // Actions
  fetchProjects: () => Promise<ProjectSummary[]>
  fetchProject: (slug: string) => Promise<ProjectDetail>
  createProject: (input: CreateProjectInput) => Promise<any>
  updateProject: (slug: string, input: Partial<CreateProjectInput>) => Promise<any>
  deleteProject: (slug: string) => Promise<void>
  clearActiveProject: () => void
  clearError: () => void
}

export function useProjects(): UseProjectsReturn {
  const store = useProjectsStore()
  const toast = useToast()

  // State
  const projects = computed(() => store.projects)
  const activeProject = computed(() => store.activeProject)
  const isLoading = computed(() => store.isLoading)
  const error = computed(() => store.error)

  // Getters
  const getProjectBySlug = (slug: string) => store.getProjectBySlug(slug)
  const projectCount = computed(() => store.projectCount)
  const hasProjects = computed(() => store.hasProjects)

  // Actions with error handling
  async function fetchProjects(): Promise<ProjectSummary[]> {
    try {
      return await store.fetchProjects()
    } catch (error) {
      toast.error('Failed to load projects')
      throw error
    }
  }

  async function fetchProject(slug: string): Promise<ProjectDetail> {
    try {
      return await store.fetchProject(slug)
    } catch (error) {
      toast.error('Failed to load project')
      throw error
    }
  }

  async function createProject(input: CreateProjectInput): Promise<any> {
    try {
      const result = await store.createProject(input)
      toast.success('Project created successfully')
      return result
    } catch (error) {
      toast.error('Failed to create project')
      throw error
    }
  }

  async function updateProject(
    slug: string,
    input: Partial<CreateProjectInput>
  ): Promise<any> {
    try {
      const result = await store.updateProject(slug, input)
      toast.success('Project updated successfully')
      return result
    } catch (error) {
      toast.error('Failed to update project')
      throw error
    }
  }

  async function deleteProject(slug: string): Promise<void> {
    try {
      await store.deleteProject(slug)
      toast.success('Project deleted successfully')
    } catch (error) {
      toast.error('Failed to delete project')
      throw error
    }
  }

  function clearActiveProject() {
    store.clearActiveProject()
  }

  function clearError() {
    store.clearError()
  }

  return {
    // State
    projects,
    activeProject,
    isLoading,
    error,

    // Getters
    getProjectBySlug,
    projectCount,
    hasProjects,

    // Actions
    fetchProjects,
    fetchProject,
    createProject,
    updateProject,
    deleteProject,
    clearActiveProject,
    clearError
  }
}
