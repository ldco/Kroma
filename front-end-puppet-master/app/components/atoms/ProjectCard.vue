<script setup lang="ts">
/**
 * Project Card Atom
 *
 * Displays project summary information.
 * Used in project list and dashboard.
 */
import type { ProjectSummary } from '~/types/kroma'

const props = defineProps<{
  project: ProjectSummary
}>()

const emit = defineEmits<{
  select: [slug: string]
  edit: [slug: string]
  delete: [slug: string]
}>()

const formattedDate = computed(() => {
  return new Date(props.project.created_at).toLocaleDateString()
})

const runCount = computed(() => {
  return (props.project as any).run_count || 0
})

const assetCount = computed(() => {
  return (props.project as any).asset_count || 0
})
</script>

<template>
  <!-- Uses global classes from ui/projects/project-card.css -->
  <div class="project-card" @click="emit('select', project.slug)">
    <div class="project-card__header">
      <h3 class="project-card__title">{{ project.name }}</h3>
      <div class="project-card__actions">
        <button
          type="button"
          class="project-card__action"
          @click.stop="emit('edit', project.slug)"
          aria-label="Edit project"
        >
          <Icon name="lucide:edit" class="icon-sm" />
        </button>
        <button
          type="button"
          class="project-card__action project-card__action--danger"
          @click.stop="emit('delete', project.slug)"
          aria-label="Delete project"
        >
          <Icon name="lucide:trash-2" class="icon-sm" />
        </button>
      </div>
    </div>

    <p class="project-card__description">
      {{ project.description || 'No description' }}
    </p>

    <div class="project-card__meta">
      <div class="project-card__stat">
        <Icon name="lucide:zap" class="icon-xs" />
        <span>{{ runCount }} runs</span>
      </div>
      <div class="project-card__stat">
        <Icon name="lucide:image" class="icon-xs" />
        <span>{{ assetCount }} assets</span>
      </div>
      <span class="project-card__date">Created: {{ formattedDate }}</span>
    </div>
  </div>
</template>

<!-- No scoped styles - uses ui/projects/project-card.css -->
