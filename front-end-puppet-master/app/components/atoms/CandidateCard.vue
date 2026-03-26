<script setup lang="ts">
/**
 * Candidate Card Atom
 *
 * Displays run candidate with image and metadata.
 * Used in run review workflow.
 */
import type { Candidate } from '~/types/kroma'

const props = defineProps<{
  candidate: Candidate
  showActions?: boolean
  selected?: boolean
}>()

const emit = defineEmits<{
  select: [candidateId: string]
  approve: [candidateId: string]
}>()

const showActions = computed(() => props.showActions ?? true)
const isSelected = computed(() => props.selected ?? false)
const isWinner = computed(() => props.candidate.is_winner)

const seed = computed(() => props.candidate.metadata?.seed || 0)
const steps = computed(() => props.candidate.metadata?.steps || 0)
const guidanceScale = computed(() => props.candidate.metadata?.guidance_scale || 7)
const width = computed(() => props.candidate.metadata?.width || 512)
const height = computed(() => props.candidate.metadata?.height || 512)
</script>

<template>
  <!-- Uses global classes from ui/runs/candidate-card.css -->
  <div
    class="candidate-card"
    :class="{
      'candidate-card--selected': isSelected,
      'candidate-card--winner': isWinner
    }"
    @click="emit('select', candidate.id)"
  >
    <div class="candidate-card__image-wrapper">
      <img
        :src="candidate.image_url"
        :alt="`Candidate ${candidate.id}`"
        class="candidate-card__image"
        loading="lazy"
      />

      <!-- Winner badge -->
      <div v-if="isWinner" class="candidate-card__winner-badge">
        <Icon name="lucide:trophy" class="icon-sm" />
        <span>Winner</span>
      </div>

      <!-- Selected overlay -->
      <div v-if="isSelected && !isWinner" class="candidate-card__selected-overlay">
        <Icon name="lucide:check" class="icon-lg" />
      </div>
    </div>

    <div class="candidate-card__content">
      <div class="candidate-card__header">
        <h4 class="candidate-card__title">Candidate {{ candidate.id.slice(0, 8) }}...</h4>
        <span v-if="candidate.score" class="candidate-card__score">
          Score: {{ candidate.score.toFixed(2) }}
        </span>
      </div>

      <div class="candidate-card__meta">
        <span class="candidate-card__meta-item">
          <Icon name="lucide:hash" class="icon-xs" />
          {{ seed }}
        </span>
        <span class="candidate-card__meta-item">
          <Icon name="lucide:layers" class="icon-xs" />
          {{ steps }} steps
        </span>
        <span class="candidate-card__meta-item">
          <Icon name="lucide:sliders" class="icon-xs" />
          {{ guidanceScale }}
        </span>
        <span class="candidate-card__meta-item">
          <Icon name="lucide:maximize" class="icon-xs" />
          {{ width }}×{{ height }}
        </span>
      </div>

      <div v-if="showActions && !isWinner" class="candidate-card__actions">
        <button
          type="button"
          class="candidate-card__action candidate-card__action--select"
          :class="{ 'candidate-card__action--active': isSelected }"
          @click.stop="emit('select', candidate.id)"
        >
          <Icon :name="isSelected ? 'lucide:check' : 'lucide:circle'" class="icon-sm" />
          <span>{{ isSelected ? 'Selected' : 'Select' }}</span>
        </button>
        <button
          type="button"
          class="candidate-card__action candidate-card__action--approve"
          :disabled="!isSelected"
          @click.stop="emit('approve', candidate.id)"
        >
          <Icon name="lucide:trophy" class="icon-sm" />
          <span>Approve</span>
        </button>
      </div>
    </div>
  </div>
</template>

<!-- No scoped styles - uses ui/runs/candidate-card.css -->
