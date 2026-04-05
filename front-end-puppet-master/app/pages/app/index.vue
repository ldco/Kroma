<script setup lang="ts">
/**
 * Kroma Dashboard Page
 *
 * Main dashboard showing:
 * - Recent projects
 * - Recent runs
 * - Quick stats
 * - Quick actions
 */
import { useProjectsStore } from '~/stores/projects'

definePageMeta({
  layout: 'kroma'
})

const { t } = useI18n()
const localePath = useLocalePath()
const projectsStore = useProjectsStore()
const { user } = useAuth()

// Fetch projects on mount
onMounted(async () => {
  await projectsStore.fetchProjects()
})

// Get recent projects (last 5)
const recentProjects = computed(() => {
  return projectsStore.projects.slice(0, 5)
})

// Stats
const stats = computed(() => ({
  totalProjects: projectsStore.projects.length,
  totalRuns: projectsStore.projects.reduce((sum, p) => sum + (p as any).run_count || 0, 0),
  totalAssets: projectsStore.projects.reduce((sum, p) => sum + (p as any).asset_count || 0, 0)
}))

// Loading state
const isLoading = computed(() => projectsStore.isLoading)
</script>

<template>
  <div class="kroma-dashboard">
    <!-- Page Header -->
    <header class="page-header">
      <div class="page-header__content">
        <h1 class="page-header__title">
          {{ t('app.nav.dashboard') }}
        </h1>
        <p class="page-header__subtitle">
          {{ t('app.dashboard.subtitle', { name: user?.name || 'User' }) }}
        </p>
      </div>
      <div class="page-header__actions">
        <NuxtLink :to="localePath('/app/projects/new')" class="btn btn-primary">
          <Icon name="lucide:plus" class="icon-sm" />
          {{ t('app.projects.create') }}
        </NuxtLink>
      </div>
    </header>

    <!-- Stats Grid -->
    <section class="dashboard-stats">
      <div class="stat-card">
        <div class="stat-card__icon stat-card__icon--projects">
          <Icon name="lucide:folder" class="icon-lg" />
        </div>
        <div class="stat-card__content">
          <span class="stat-card__value">{{ stats.totalProjects }}</span>
          <span class="stat-card__label">{{ t('app.projects.title') }}</span>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-card__icon stat-card__icon--runs">
          <Icon name="lucide:zap" class="icon-lg" />
        </div>
        <div class="stat-card__content">
          <span class="stat-card__value">{{ stats.totalRuns }}</span>
          <span class="stat-card__label">{{ t('app.runs.title') }}</span>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-card__icon stat-card__icon--assets">
          <Icon name="lucide:image" class="icon-lg" />
        </div>
        <div class="stat-card__content">
          <span class="stat-card__value">{{ stats.totalAssets }}</span>
          <span class="stat-card__label">{{ t('app.assets.title') }}</span>
        </div>
      </div>
    </section>

    <!-- Recent Projects -->
    <section class="dashboard-section">
      <div class="section-header">
        <h2 class="section-header__title">
          {{ t('app.projects.recent') }}
        </h2>
        <NuxtLink :to="localePath('/app/projects')" class="section-header__link">
          {{ t('common.viewAll') }}
          <Icon name="lucide:arrow-right" class="icon-sm" />
        </NuxtLink>
      </div>

      <div v-if="isLoading" class="projects-loading">
        <LoadingText />
      </div>

      <div v-else-if="recentProjects.length === 0" class="empty-state">
        <Icon name="lucide:folder-open" class="empty-state__icon" />
        <h3 class="empty-state__title">{{ t('app.projects.empty.title') }}</h3>
        <p class="empty-state__description">
          {{ t('app.projects.empty.description') }}
        </p>
        <NuxtLink :to="localePath('/app/projects/new')" class="btn btn-primary">
          {{ t('app.projects.create') }}
        </NuxtLink>
      </div>

      <div v-else class="projects-grid">
        <AtomsProjectCard
          v-for="project in recentProjects"
          :key="project.slug"
          :project="project"
          @select="navigateToProject"
          @edit="editProject"
          @delete="deleteProject"
        />
      </div>
    </section>

    <!-- Quick Actions -->
    <section class="dashboard-section">
      <div class="section-header">
        <h2 class="section-header__title">
          {{ t('app.quickActions') }}
        </h2>
      </div>

      <div class="quick-actions-grid">
        <NuxtLink :to="localePath('/app/quick-tools')" class="quick-action-card">
          <div class="quick-action-card__icon">
            <Icon name="lucide:wand" class="icon-lg" />
          </div>
          <div class="quick-action-card__content">
            <h3 class="quick-action-card__title">
              {{ t('app.nav.quickTools') }}
            </h3>
            <p class="quick-action-card__description">
              {{ t('app.quickTools.description') }}
            </p>
          </div>
          <Icon name="lucide:arrow-right" class="quick-action-card__arrow" />
        </NuxtLink>

        <NuxtLink :to="localePath('/app/projects')" class="quick-action-card">
          <div class="quick-action-card__icon">
            <Icon name="lucide:key" class="icon-lg" />
          </div>
          <div class="quick-action-card__content">
            <h3 class="quick-action-card__title">
              {{ t('app.nav.projects') }}
            </h3>
            <p class="quick-action-card__description">
              {{ t('app.providers.description') }}
            </p>
          </div>
          <Icon name="lucide:arrow-right" class="quick-action-card__arrow" />
        </NuxtLink>
      </div>
    </section>
  </div>
</template>

<script lang="ts">
export default {
  methods: {
    navigateToProject(slug: string) {
      navigateTo(`/app/projects/${slug}`)
    },
    editProject(slug: string) {
      navigateTo(`/app/projects/${slug}/edit`)
    },
    async deleteProject(slug: string) {
      const confirmed = await confirm({
        title: 'Delete Project',
        message: 'Are you sure you want to delete this project? This action cannot be undone.',
        confirmText: 'Delete',
        cancelText: 'Cancel',
        confirmVariant: 'danger'
      })

      if (confirmed) {
        // TODO: Call delete API
        toast.success('Project deleted successfully')
      }
    }
  }
}
</script>

<!--
  Uses global CSS classes:
  - ui/projects/project-card.css: .project-card
  - ui/content/empty-states.css: .empty-state
  - common/text.css: .page-header__title
  - ui/forms/buttons.css: .btn, .btn-primary
-->
