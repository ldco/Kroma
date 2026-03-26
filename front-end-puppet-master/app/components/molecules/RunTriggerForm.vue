<script setup lang="ts">
/**
 * Run Trigger Form Molecule
 *
 * Form for triggering new generation runs.
 * Includes prompt, negative prompt, and advanced settings.
 */
import type { RunConfig } from '~/types/kroma'

const props = defineProps<{
  projectSlug: string
}>()

const emit = defineEmits<{
  submit: [config: RunConfig]
  cancel: []
}>()

// Form state
const formPrompt = ref('')
const formNegativePrompt = ref('')
const formModel = ref('')
const formWidth = ref(512)
const formHeight = ref(512)
const formSteps = ref(30)
const formGuidanceScale = ref(7)
const formSeed = ref<number | null>(null)
const formCandidateCount = ref(4)

// Advanced settings toggle
const showAdvanced = ref(false)

// Preset configurations
const presets = [
  { name: 'Quick', steps: 20, guidance: 7 },
  { name: 'Standard', steps: 30, guidance: 7.5 },
  { name: 'Quality', steps: 50, guidance: 8 }
]

function applyPreset(preset: typeof presets[0]) {
  formSteps.value = preset.steps
  formGuidanceScale.value = preset.guidance
}

// Validation
const errors = ref<Record<string, string>>({})

function validate() {
  errors.value = {}

  if (!formPrompt.value.trim()) {
    errors.value.prompt = 'Prompt is required'
  }

  if (formWidth.value < 64 || formWidth.value > 2048) {
    errors.value.width = 'Width must be between 64 and 2048'
  }

  if (formHeight.value < 64 || formHeight.value > 2048) {
    errors.value.height = 'Height must be between 64 and 2048'
  }

  if (formSteps.value < 1 || formSteps.value > 150) {
    errors.value.steps = 'Steps must be between 1 and 150'
  }

  return Object.keys(errors.value).length === 0
}

function handleSubmit() {
  if (!validate()) {
    return
  }

  const config: RunConfig = {
    prompt: formPrompt.value.trim(),
    negative_prompt: formNegativePrompt.value.trim() || null,
    model: formModel.value || undefined,
    width: formWidth.value,
    height: formHeight.value,
    steps: formSteps.value,
    guidance_scale: formGuidanceScale.value,
    seed: formSeed.value || undefined,
    candidate_count: formCandidateCount.value
  }

  emit('submit', config)
}

function handleCancel() {
  emit('cancel')
}
</script>

<template>
  <!-- Uses global classes from ui/forms/index.css -->
  <form class="run-trigger-form" @submit.prevent="handleSubmit">
    <div class="form-group">
      <label for="run-prompt" class="form-label">
        Prompt
        <span class="required">*</span>
      </label>
      <textarea
        id="run-prompt"
        v-model="formPrompt"
        class="form-input form-input--textarea"
        :class="{ 'form-input--error': errors.prompt }"
        placeholder="Describe the image you want to generate..."
        rows="4"
      />
      <p v-if="errors.prompt" class="form-error">{{ errors.prompt }}</p>
      <p class="form-hint">
        <Icon name="lucide:lightbulb" class="icon-xs" />
        Be descriptive about style, composition, lighting, and mood
      </p>
    </div>

    <div class="form-group">
      <label for="run-negative-prompt" class="form-label">
        Negative Prompt
      </label>
      <textarea
        id="run-negative-prompt"
        v-model="formNegativePrompt"
        class="form-input form-input--textarea"
        placeholder="What to avoid (e.g., blurry, low quality)..."
        rows="2"
      />
      <p class="form-hint">
        <Icon name="lucide:ban" class="icon-xs" />
        Elements you want to exclude from the image
      </p>
    </div>

    <!-- Presets -->
    <div class="form-group">
      <label class="form-label">Quality Preset</label>
      <div class="preset-buttons">
        <button
          v-for="preset in presets"
          :key="preset.name"
          type="button"
          class="preset-button"
          @click="applyPreset(preset)"
        >
          {{ preset.name }}
        </button>
      </div>
    </div>

    <!-- Advanced Settings Toggle -->
    <button
      type="button"
      class="advanced-toggle"
      @click="showAdvanced = !showAdvanced"
    >
      <Icon
        name="lucide:chevron-down"
        class="advanced-toggle__icon"
        :class="{ 'advanced-toggle__icon--open': showAdvanced }"
      />
      <span>Advanced Settings</span>
    </button>

    <!-- Advanced Settings -->
    <div v-if="showAdvanced" class="advanced-settings">
      <div class="form-row">
        <div class="form-group">
          <label for="run-width" class="form-label">Width</label>
          <input
            id="run-width"
            v-model.number="formWidth"
            type="number"
            class="form-input"
            :class="{ 'form-input--error': errors.width }"
            min="64"
            max="2048"
            step="64"
          />
          <p v-if="errors.width" class="form-error">{{ errors.width }}</p>
        </div>

        <div class="form-group">
          <label for="run-height" class="form-label">Height</label>
          <input
            id="run-height"
            v-model.number="formHeight"
            type="number"
            class="form-input"
            :class="{ 'form-input--error': errors.height }"
            min="64"
            max="2048"
            step="64"
          />
          <p v-if="errors.height" class="form-error">{{ errors.height }}</p>
        </div>
      </div>

      <div class="form-row">
        <div class="form-group">
          <label for="run-steps" class="form-label">Steps</label>
          <input
            id="run-steps"
            v-model.number="formSteps"
            type="number"
            class="form-input"
            :class="{ 'form-input--error': errors.steps }"
            min="1"
            max="150"
          />
          <p v-if="errors.steps" class="form-error">{{ errors.steps }}</p>
        </div>

        <div class="form-group">
          <label for="run-guidance" class="form-label">Guidance Scale</label>
          <input
            id="run-guidance"
            v-model.number="formGuidanceScale"
            type="number"
            class="form-input"
            min="1"
            max="20"
            step="0.5"
          />
        </div>
      </div>

      <div class="form-row">
        <div class="form-group">
          <label for="run-seed" class="form-label">Seed (optional)</label>
          <input
            id="run-seed"
            v-model.number="formSeed"
            type="number"
            class="form-input"
            placeholder="Random if empty"
          />
        </div>

        <div class="form-group">
          <label for="run-candidates" class="form-label">Candidates</label>
          <input
            id="run-candidates"
            v-model.number="formCandidateCount"
            type="number"
            class="form-input"
            min="1"
            max="10"
          />
        </div>
      </div>
    </div>

    <div class="form-actions">
      <button type="button" class="btn btn-ghost" @click="handleCancel">
        Cancel
      </button>
      <button type="submit" class="btn btn-primary">
        <Icon name="lucide:zap" class="icon-sm" />
        Trigger Run
      </button>
    </div>
  </form>
</template>

<!-- No scoped styles - uses ui/runs/run-trigger-form.css -->
