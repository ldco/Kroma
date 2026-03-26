<script setup lang="ts">
/**
 * Admin Settings Page
 *
 * Config-driven form to edit site settings.
 * Settings schema is defined in puppet-master.config.ts
 */
import config from '~/puppet-master.config'

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()
const { toast } = useToast()

useHead({
  title: () => `${t('admin.navSettings')} | Admin`
})

// SSR-safe config access with defensive defaults
const settingsConfig = computed(() => config?.settings ?? [])

// Fetch settings
const {
  data: settings,
  pending,
  refresh
} = await useFetch<{ success: boolean; data: Record<string, Record<string, string | null>> }>(
  '/api/admin/settings'
)

// Form state - dynamically built from config
const form = reactive<Record<string, string>>(
  Object.fromEntries(settingsConfig.value.map(s => [s.key, '']))
)

// Populate form when settings load
watchEffect(() => {
  if (settings.value) {
    const settingsPayload = settings.value.data || {}
    for (const setting of settingsConfig.value) {
      const [group, key] = setting.key.split('.')
      const groupData = group ? (settingsPayload as Record<string, Record<string, string>>)[group] : undefined
      form[setting.key] = (key && groupData?.[key]) || ''
    }
  }
})

// Group settings by their group
const settingsByGroup = computed(() => {
  const grouped: Record<string, (typeof settingsConfig.value)[number][]> = {}
  for (const setting of settingsConfig.value) {
    const group = (grouped[setting.group] ??= [])
    group.push(setting)
  }
  return grouped
})

// Get input type for setting
function getInputType(type: string): string {
  switch (type) {
    case 'email':
      return 'email'
    case 'url':
      return 'url'
    case 'tel':
      return 'tel'
    case 'text':
      return 'textarea'
    case 'password':
      return 'password'
    default:
      return 'text'
  }
}

const saving = ref(false)

async function saveSettings() {
  saving.value = true

  try {
    await $fetch('/api/admin/settings', {
      method: 'PUT',
      body: form
    })
    toast.success(t('admin.settingsSaved'))
    await refresh()
  } catch (e: any) {
    toast.error(e.data?.message || 'Failed to save settings')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="admin-settings">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.settings') }}</h1>
    </div>

    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <form v-else @submit.prevent="saveSettings" class="settings-form">
      <!-- Dynamic settings groups from config -->
      <section v-for="group in config.settingGroups" :key="group.key" class="card">
        <div class="card-header">
          <h2 class="section-title">{{ t(group.label) }}</h2>
        </div>
        <div class="card-body">
          <div class="form-grid">
            <div
              v-for="setting in settingsByGroup[group.key]"
              :key="setting.key"
              class="form-group"
              :class="{ 'form-group--full': setting.type === 'text' }"
            >
              <label class="form-label">{{ setting.label }}</label>
              <textarea
                v-if="setting.type === 'text'"
                v-model="form[setting.key]"
                class="input"
                rows="3"
              ></textarea>
              <input
                v-else
                v-model="form[setting.key]"
                :type="getInputType(setting.type)"
                class="input"
              />
            </div>
          </div>
        </div>
      </section>

      <!-- Save Button -->
      <div class="form-actions">
        <button type="submit" class="btn btn-primary" :disabled="saving">
          {{ saving ? t('common.saving') : t('common.save') }}
        </button>
      </div>
    </form>
  </div>
</template>

<!--
  Uses global CSS classes from admin/index.css:
  - .admin-settings, .section-title, .settings-form, .form-actions
-->
