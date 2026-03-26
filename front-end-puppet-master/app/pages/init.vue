<script setup lang="ts">
/**
 * Build Configuration Wizard
 *
 * Configures project settings for BUILD mode.
 * User reaches this page after running `npm run init` and selecting BUILD.
 *
 * Styles: app/assets/css/ui/content/init.css
 */
import IconWorld from '~icons/tabler/world'
import IconApps from '~icons/tabler/apps'
import IconCheck from '~icons/tabler/check'
import IconLoader from '~icons/tabler/loader-2'
import IconUpload from '~icons/tabler/upload'
import IconFileZip from '~icons/tabler/file-zip'
import IconX from '~icons/tabler/x'
import IconFile from '~icons/tabler/file-text'
import Logo from '~/components/atoms/Logo.vue'
import { getModulesForWizard, getLocalesForWizard } from '~~/shared/modules'

definePageMeta({
  layout: false // Using <NuxtLayout> explicitly for named slots
})

useHead({
  title: 'Configure Project | Puppet Master'
})

// Phase state - complete-greenfield for new projects, complete-brownfield for imports
const phase = ref<'configure' | 'complete-greenfield' | 'complete-brownfield'>('configure')
const loading = ref(true)
const saving = ref(false)
const error = ref('')
const validationErrors = ref<string[]>([])

// Configuration
const config = reactive({
  // Project info - gathered for Claude context
  projectName: '',
  projectDescription: '',
  targetAudience: '',
  technicalBrief: '', // Technical brief for Claude planning
  // AI Workflow selection
  aiWorkflow: 'claude' as 'claude' | 'qwen' | 'codex',
  // Project type
  projectType: 'website' as 'website' | 'app',
  adminEnabled: true,
  // Modules
  modules: ['contact'] as string[],
  customModules: '', // Free text for other modules needed
  // Features
  features: {
    multiLangs: false,
    doubleTheme: true,
    onepager: false,
    pwa: false,
    twoFactorAuth: false
  },
  // Locales
  locales: [{ code: 'en', iso: 'en-US', name: 'English' }] as Array<{
    code: string
    iso: string
    name: string
  }>,
  defaultLocale: 'en',
  customLocale: { code: '', name: '' } // For "Other" language
})

// Import zip upload
const importZipFile = ref<File | null>(null)
const uploadingZip = ref(false)
const importFolderFiles = ref<string[]>([])
const hasBrownfieldContent = ref(false)

// Current config from server
const currentConfig = ref<any>(null)

// Fetch current config on mount
onMounted(async () => {
  try {
    const data = await $fetch('/api/setup/config')
    currentConfig.value = data

    // Capture brownfield info
    if (data.hasBrownfieldContent) {
      hasBrownfieldContent.value = true
      importFolderFiles.value = data.importFolderFiles || []
    }

    // Load existing config values if available
    if (data.pmMode !== 'unconfigured') {
      config.aiWorkflow = data.aiWorkflow || 'claude'
      config.projectType = data.projectType || 'website'
      config.adminEnabled = data.adminEnabled
      config.modules = data.enabledModules || ['contact']
      config.features = data.features || config.features
      config.locales = data.locales || config.locales
      config.defaultLocale = data.defaultLocale || 'en'
      // Load project info if previously saved
      if (data.projectName) config.projectName = data.projectName
      if (data.projectDescription) config.projectDescription = data.projectDescription
      if (data.targetAudience) config.targetAudience = data.targetAudience
      if (data.technicalBrief) config.technicalBrief = data.technicalBrief
      if (data.customModules) config.customModules = data.customModules
    }
  } catch (e: any) {
    error.value = e.data?.message || 'Failed to load configuration'
  } finally {
    loading.value = false
  }
})

// Available modules and locales from shared registry (single source of truth)
const availableModules = getModulesForWizard()
const availableLocales = getLocalesForWizard()

// Retry loading config
async function retryLoad() {
  error.value = ''
  loading.value = true
  try {
    const data = await $fetch('/api/setup/config')
    currentConfig.value = data
    if (data.hasBrownfieldContent) {
      hasBrownfieldContent.value = true
      importFolderFiles.value = data.importFolderFiles || []
    }
  } catch (e: any) {
    error.value = e.data?.message || 'Failed to load configuration'
  } finally {
    loading.value = false
  }
}

// Module toggle
function toggleModule(moduleId: string) {
  const index = config.modules.indexOf(moduleId)
  if (index === -1) {
    config.modules.push(moduleId)
  } else {
    config.modules.splice(index, 1)
  }
}

// Locale toggle
function toggleLocale(locale: (typeof availableLocales)[0]) {
  const index = config.locales.findIndex(l => l.code === locale.code)
  if (index === -1) {
    config.locales.push(locale)
    if (config.locales.length === 1) {
      config.defaultLocale = locale.code
    }
  } else if (config.locales.length > 1) {
    config.locales.splice(index, 1)
    if (config.defaultLocale === locale.code) {
      config.defaultLocale = config.locales[0].code
    }
  }
}

// Add custom locale
function addCustomLocale() {
  if (!config.customLocale.code || !config.customLocale.name) return

  // Check if already exists
  if (config.locales.some(l => l.code === config.customLocale.code)) {
    error.value = 'This language code already exists'
    return
  }

  config.locales.push({
    code: config.customLocale.code.toLowerCase(),
    iso: `${config.customLocale.code.toLowerCase()}-${config.customLocale.code.toUpperCase()}`,
    name: config.customLocale.name
  })

  if (config.locales.length === 1) {
    config.defaultLocale = config.customLocale.code.toLowerCase()
  }

  // Clear the input
  config.customLocale = { code: '', name: '' }
}

// Zip file upload handler
async function handleZipUpload(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  if (!file.name.endsWith('.zip')) {
    error.value = 'Please upload a .zip file'
    return
  }

  uploadingZip.value = true
  error.value = ''

  try {
    const formData = new FormData()
    formData.append('file', file)

    const result = await $fetch('/api/setup/import-zip', {
      method: 'POST',
      body: formData
    }) as { files: string[] }

    importZipFile.value = file
    importFolderFiles.value = result.files
    hasBrownfieldContent.value = result.files.length > 0
  } catch (e: any) {
    error.value = e.data?.message || 'Failed to upload zip file'
    importZipFile.value = null
  } finally {
    uploadingZip.value = false
  }
}

// Remove uploaded zip
async function removeImportZip() {
  try {
    await $fetch('/api/setup/import-zip', { method: 'DELETE' })
    importZipFile.value = null
    importFolderFiles.value = []
    hasBrownfieldContent.value = false
  } catch (e: any) {
    error.value = e.data?.message || 'Failed to remove import files'
  }
}

// Technical brief file upload handler
function handleBriefUpload(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  // Validate file extension
  const validExtensions = ['.md', '.txt', '.markdown']
  const hasValidExt = validExtensions.some(ext => file.name.toLowerCase().endsWith(ext))
  if (!hasValidExt) {
    error.value = 'Please upload a .md or .txt file'
    input.value = ''
    return
  }

  // Validate file size (max 500KB)
  if (file.size > 500 * 1024) {
    error.value = 'File too large. Maximum size is 500KB.'
    input.value = ''
    return
  }

  const reader = new FileReader()
  reader.onload = (e) => {
    const content = e.target?.result as string
    if (content) {
      config.technicalBrief = content
    }
  }
  reader.onerror = () => {
    error.value = 'Failed to read file'
  }
  reader.readAsText(file)

  // Reset input so same file can be re-uploaded
  input.value = ''
}

// Validation
function validate(): boolean {
  validationErrors.value = []

  if (config.locales.length === 0) {
    validationErrors.value.push('At least one language is required')
  }

  return validationErrors.value.length === 0
}

// Apply configuration
async function applyConfig() {
  if (!validate()) {
    return
  }

  saving.value = true
  error.value = ''

  try {
    await $fetch('/api/setup/config', {
      method: 'POST',
      body: {
        pmMode: 'build',
        aiWorkflow: config.aiWorkflow,
        // Project info (for Claude context)
        projectName: config.projectName,
        projectDescription: config.projectDescription,
        targetAudience: config.targetAudience,
        technicalBrief: config.technicalBrief,
        // Type and modules
        projectType: config.projectType,
        adminEnabled: config.adminEnabled,
        modules: config.modules,
        customModules: config.customModules,
        // Features and locales
        features: config.features,
        locales: config.locales,
        defaultLocale: config.defaultLocale
      }
    })

    // Always show completion screen - developer needs to start coding
    saving.value = false
    if (hasBrownfieldContent.value) {
      phase.value = 'complete-brownfield'
    } else {
      phase.value = 'complete-greenfield'
    }
  } catch (e: any) {
    error.value = e.data?.message || 'Failed to save configuration'
    saving.value = false
  }
}

</script>

<template>
  <NuxtLayout name="blank">
    <!-- Header in named slot - OUTSIDE .main so position:fixed works -->
    <template #header>
      <header class="header">
        <div class="header-inner">
          <div class="header-logo">
            <Logo to="/" alt="Puppet Master" />
          </div>
          <span class="text-sm text-secondary">Configure Project</span>
        </div>
      </header>
    </template>

    <!-- Default slot content goes inside .main -->
    <div class="setup-page">
      <!-- Loading -->
    <div v-if="loading" class="setup-loading">
      <IconLoader class="spinner" />
      <p>Loading configuration...</p>
    </div>

    <!-- Configuration Form -->
    <main v-else-if="phase === 'configure'" class="setup-content setup-form">
      <!-- Error Banner -->
      <div v-if="error" class="error-banner">
        {{ error }}
      </div>

      <!-- Validation Errors -->
      <div v-if="validationErrors.length > 0" class="error-banner">
        <ul class="validation-errors">
          <li v-for="err in validationErrors" :key="err">{{ err }}</li>
        </ul>
      </div>

      <!-- Project Info -->
      <section class="setup-section">
        <h2 class="section-title">Project Info</h2>
        <p class="section-desc">Tell us about your project. This information helps Claude understand your needs.</p>

        <div class="form-group">
          <label class="form-label">Project Name</label>
          <input
            v-model="config.projectName"
            type="text"
            class="input"
            placeholder="My Awesome Project"
          />
        </div>

        <div class="form-group">
          <label class="form-label">Project Description</label>
          <textarea
            v-model="config.projectDescription"
            class="input"
            rows="3"
            placeholder="What is this project about? What problem does it solve? What are its main features?"
          ></textarea>
        </div>

        <div class="form-group">
          <label class="form-label">Target Audience</label>
          <input
            v-model="config.targetAudience"
            type="text"
            class="input"
            placeholder="Who will use this? (e.g., small businesses, developers, consumers)"
          />
        </div>

        <div class="form-group">
          <label class="form-label">Technical Brief <span class="optional-badge">Optional</span></label>
          <div class="brief-input-wrapper">
            <textarea
              v-model="config.technicalBrief"
              class="input"
              rows="6"
              placeholder="Paste or upload a technical brief, PRD, or feature spec. Claude will use this to understand requirements and generate better plans."
            ></textarea>
            <div class="brief-upload">
              <input
                type="file"
                accept=".md,.txt,.markdown"
                id="brief-upload"
                class="upload-input"
                @change="handleBriefUpload"
              />
              <label for="brief-upload" class="btn btn-secondary btn-sm">
                <IconFile class="icon-sm" />
                Upload .md/.txt
              </label>
            </div>
          </div>
          <p class="form-hint">Include PRDs, technical specs, or feature lists. This helps Claude generate accurate implementation plans.</p>
        </div>
      </section>

      <!-- AI Workflow Selection -->
      <section class="setup-section">
        <h2 class="section-title">AI Agent Workflow</h2>
        <p class="section-desc">Choose which AI agent workflow to use for development assistance.</p>

        <div class="workflow-cards">
          <button
            class="workflow-card"
            :class="{ active: config.aiWorkflow === 'claude' }"
            @click="config.aiWorkflow = 'claude'"
          >
            <div class="workflow-info">
              <h3>Claude Code</h3>
              <p>Basic PM commands and workflows</p>
            </div>
            <IconCheck v-if="config.aiWorkflow === 'claude'" class="check-icon" />
          </button>

          <button
            class="workflow-card"
            :class="{ active: config.aiWorkflow === 'qwen' }"
            @click="config.aiWorkflow = 'qwen'"
          >
            <div class="workflow-info">
              <h3>Qwen Code</h3>
              <p>Advanced with 42 expert personas (7 specialties × 6 countries)</p>
            </div>
            <IconCheck v-if="config.aiWorkflow === 'qwen'" class="check-icon" />
          </button>

          <button
            class="workflow-card"
            :class="{ active: config.aiWorkflow === 'codex' }"
            @click="config.aiWorkflow = 'codex'"
          >
            <div class="workflow-info">
              <h3>Codex</h3>
              <p>Experimental workflow (coming soon)</p>
            </div>
            <IconCheck v-if="config.aiWorkflow === 'codex'" class="check-icon" />
          </button>
        </div>
      </section>

      <!-- Project Type -->
      <section class="setup-section">
        <h2 class="section-title">Project Type</h2>
        <p class="section-desc">What are you building?</p>

        <div class="type-cards">
          <button
            class="type-card"
            :class="{ active: config.projectType === 'website' }"
            @click="config.projectType = 'website'"
          >
            <IconWorld class="type-icon" />
            <div class="type-info">
              <h3>Website</h3>
              <p>Marketing site, landing pages, SEO content</p>
            </div>
            <IconCheck v-if="config.projectType === 'website'" class="check-icon" />
          </button>

          <button
            class="type-card"
            :class="{ active: config.projectType === 'app' }"
            @click="config.projectType = 'app'"
          >
            <IconApps class="type-icon" />
            <div class="type-info">
              <h3>App</h3>
              <p>Dashboard, user features, login required</p>
            </div>
            <IconCheck v-if="config.projectType === 'app'" class="check-icon" />
          </button>
        </div>

        <label class="checkbox-label">
          <input v-model="config.adminEnabled" type="checkbox" />
          <span>Enable Admin Panel</span>
        </label>
      </section>

      <!-- Import Existing Code (Optional) -->
      <section class="setup-section">
        <h2 class="section-title">Import Existing Code <span class="optional-badge">Optional</span></h2>
        <p class="section-desc">Have an existing project to migrate? Upload a zip file.</p>

        <div v-if="!importZipFile" class="upload-zone">
          <input
            type="file"
            accept=".zip"
            id="zip-upload"
            class="upload-input"
            :disabled="uploadingZip"
            @change="handleZipUpload"
          />
          <label for="zip-upload" class="upload-label">
            <IconUpload v-if="!uploadingZip" class="upload-icon" />
            <IconLoader v-else class="upload-icon spinner" />
            <span>{{ uploadingZip ? 'Uploading...' : 'Drop zip file or click to upload' }}</span>
            <span class="upload-hint">Your existing code will be analyzed after setup</span>
          </label>
        </div>

        <div v-else class="import-detected">
          <div class="import-header">
            <IconFileZip class="zip-icon" />
            <div class="import-info">
              <strong>{{ importZipFile.name }}</strong>
              <span>{{ importFolderFiles.length }} files extracted</span>
            </div>
            <button class="btn-icon" @click="removeImportZip" title="Remove">
              <IconX />
            </button>
          </div>
          <ul v-if="importFolderFiles.length > 0" class="import-files">
            <li v-for="file in importFolderFiles.slice(0, 5)" :key="file">{{ file }}</li>
            <li v-if="importFolderFiles.length > 5" class="more-files">
              +{{ importFolderFiles.length - 5 }} more
            </li>
          </ul>
          <p class="import-note">Run <code>/pm-migrate</code> after setup to analyze and import.</p>
        </div>
      </section>

      <!-- Modules -->
      <section class="setup-section">
        <h2 class="section-title">Modules</h2>
        <p class="section-desc">
          <template v-if="config.projectType === 'website'">
            Select pre-built modules for your website. Each includes admin CRUD, database tables, and components.
          </template>
          <template v-else>
            App-specific modules are coming soon. For now, describe what you need below and build custom modules.
          </template>
        </p>

        <!-- Pre-built modules (website only) -->
        <div v-if="config.projectType === 'website'" class="modules-grid">
          <button
            v-for="mod in availableModules"
            :key="mod.id"
            class="module-card"
            :class="{ active: config.modules.includes(mod.id) }"
            @click="toggleModule(mod.id)"
          >
            <IconCheck v-if="config.modules.includes(mod.id)" class="check-icon" />
            <h3>{{ mod.name }}</h3>
            <p>{{ mod.desc }}</p>
          </button>
        </div>

        <!-- App modules notice -->
        <div v-else class="info-box">
          <p>
            Pre-built app modules (Dashboard, User Profiles, Notifications, etc.) are planned for future releases.
            Describe your requirements below and Claude will help you build custom modules.
          </p>
        </div>

        <!-- Custom modules (always show) -->
        <div class="form-group mt-4">
          <label class="form-label">Other Modules Needed <span class="optional-badge">Optional</span></label>
          <textarea
            v-model="config.customModules"
            class="input"
            rows="3"
            :placeholder="config.projectType === 'website'
              ? 'Describe any custom modules you need that aren\'t listed above (e.g., Events calendar, Job board, E-commerce...)'
              : 'Describe the modules/features your app needs (e.g., Dashboard with analytics, User profiles, Real-time notifications, Activity feed...)'"
          ></textarea>
          <p class="form-hint">Claude will use this to help you build custom modules during development.</p>
        </div>
      </section>

      <!-- Languages -->
      <section class="setup-section">
        <h2 class="section-title">Languages <span class="required-badge">Required</span></h2>
        <p class="section-desc">Select at least one language. Add custom languages if yours isn't listed.</p>

        <div class="locales-grid">
          <button
            v-for="locale in availableLocales"
            :key="locale.code"
            class="locale-chip"
            :class="{
              active: config.locales.some(l => l.code === locale.code),
              default: config.defaultLocale === locale.code
            }"
            @click="toggleLocale(locale)"
          >
            {{ locale.name }}
            <span v-if="config.defaultLocale === locale.code" class="default-badge">default</span>
          </button>
        </div>

        <!-- Custom Language Input -->
        <div class="custom-locale-input">
          <div class="custom-locale-fields">
            <input
              v-model="config.customLocale.code"
              type="text"
              class="input"
              placeholder="Code (e.g., pt, ar, ko)"
              maxlength="5"
            />
            <input
              v-model="config.customLocale.name"
              type="text"
              class="input"
              placeholder="Language name (e.g., Portuguese)"
            />
            <button
              type="button"
              class="btn btn-secondary"
              :disabled="!config.customLocale.code || !config.customLocale.name"
              @click="addCustomLocale"
            >
              Add
            </button>
          </div>
          <p class="form-hint">Need a language not in the list? Add it here.</p>
        </div>

        <!-- Selected languages (if any custom added) -->
        <div v-if="config.locales.some(l => !availableLocales.find(a => a.code === l.code))" class="selected-locales">
          <p class="form-label">Custom languages added:</p>
          <div class="locales-grid">
            <button
              v-for="locale in config.locales.filter(l => !availableLocales.find(a => a.code === l.code))"
              :key="locale.code"
              class="locale-chip active"
              @click="toggleLocale(locale)"
            >
              {{ locale.name }} ({{ locale.code }})
              <span v-if="config.defaultLocale === locale.code" class="default-badge">default</span>
            </button>
          </div>
        </div>

        <div v-if="config.locales.length > 1" class="form-group">
          <label class="form-label">Default Language</label>
          <select v-model="config.defaultLocale" class="input">
            <option v-for="l in config.locales" :key="l.code" :value="l.code">
              {{ l.name }}
            </option>
          </select>
        </div>
      </section>

      <!-- Features -->
      <section class="setup-section">
        <h2 class="section-title">Features</h2>
        <p class="section-desc">Additional functionality</p>

        <div class="features-list">
          <label class="checkbox-label">
            <input v-model="config.features.doubleTheme" type="checkbox" />
            <span>Dark/Light Mode Toggle</span>
          </label>
          <label class="checkbox-label">
            <input v-model="config.features.onepager" type="checkbox" />
            <span>One-page Layout (scroll navigation)</span>
          </label>
          <label class="checkbox-label">
            <input v-model="config.features.pwa" type="checkbox" />
            <span>Progressive Web App (PWA)</span>
          </label>
          <label v-if="config.adminEnabled || config.projectType === 'app'" class="checkbox-label">
            <input v-model="config.features.twoFactorAuth" type="checkbox" />
            <span>Two-Factor Authentication (TOTP)</span>
          </label>
        </div>
      </section>

      <!-- Database Warning -->
      <div v-if="currentConfig?.databaseExists" class="warning-box">
        <strong>Note:</strong> A database already exists. Your data will be preserved.
      </div>

      <!-- Actions -->
      <div class="form-actions form-actions-center">
        <button
          class="btn btn-primary btn-lg"
          @click="applyConfig"
          :disabled="saving"
        >
          <IconLoader v-if="saving" class="spinner" />
          <IconCheck v-else />
          {{ saving ? 'Applying...' : 'Apply & Start Building' }}
        </button>
      </div>
    </main>

    <!-- Complete Phase: Greenfield (new project) -->
    <main v-else-if="phase === 'complete-greenfield'" class="setup-content">
      <div class="complete-phase">
        <div class="complete-icon">
          <IconCheck />
        </div>
        <h1 class="complete-title">Ready to Code</h1>
        <p class="complete-desc">
          Your project is configured. Dev server is running.<br />
          Start building your {{ config.projectType }} with Claude.
        </p>

        <div v-if="config.technicalBrief" class="complete-steps">
          <div class="step">
            <span class="step-num">1</span>
            <div class="step-content">
              <strong>Generate Development Plan</strong>
              <p>Claude will analyze your technical brief and create a roadmap:</p>
              <code>/pm-plan</code>
            </div>
          </div>
        </div>

        <div class="complete-info">
          <div class="info-item">
            <strong>Dev Server</strong>
            <a href="http://localhost:3000" target="_blank" class="dev-link">http://localhost:3000</a>
          </div>
          <div class="info-item">
            <strong>Configuration</strong>
            <code>app/puppet-master.config.ts</code>
          </div>
        </div>

        <div class="complete-dirs">
          <strong>Key Directories</strong>
          <div class="dir-grid">
            <div class="dir-item">
              <code>app/pages/</code>
              <span>Route pages</span>
            </div>
            <div class="dir-item">
              <code>app/components/</code>
              <span>Vue components</span>
            </div>
            <div class="dir-item">
              <code>server/api/</code>
              <span>API endpoints</span>
            </div>
            <div class="dir-item">
              <code>app/assets/css/</code>
              <span>Styles</span>
            </div>
          </div>
        </div>

        <div v-if="config.adminEnabled" class="complete-hint">
          Admin panel available at <a href="/admin">/admin</a>
        </div>
      </div>
    </main>

    <!-- Complete Phase: Brownfield (imported code) -->
    <main v-else-if="phase === 'complete-brownfield'" class="setup-content">
      <div class="complete-phase">
        <div class="complete-icon">
          <IconCheck />
        </div>
        <h1 class="complete-title">Configuration Complete</h1>
        <p class="complete-desc">
          Your project is configured. Since you uploaded existing code,
          the next step is to migrate it to Puppet Master.
        </p>

        <div class="complete-steps">
          <div class="step">
            <span class="step-num">1</span>
            <div class="step-content">
              <strong>With Claude Code (Recommended)</strong>
              <p>Run AI-powered migration analysis:</p>
              <code>/pm-migrate</code>
              <p class="step-note">Claude analyzes your code and creates a migration plan, mapping each piece to PM equivalents.</p>
            </div>
          </div>
          <div class="step step-alt">
            <span class="step-num">or</span>
            <div class="step-content">
              <strong>Without Claude Code</strong>
              <p>Your files are in <code>./import/</code></p>
              <p class="step-note">Reference them manually as you build your PM project. Check package.json for dependencies, browse components and pages for logic to port.</p>
            </div>
          </div>
        </div>

        <div class="complete-files">
          <strong>Uploaded files:</strong>
          <div class="file-count">{{ importFolderFiles.length }} files in ./import/</div>
        </div>

        <div class="complete-info">
          <div class="info-item">
            <strong>Dev Server</strong>
            <a href="http://localhost:3000" target="_blank" class="dev-link">http://localhost:3000</a>
          </div>
        </div>
      </div>
    </main>
    </div>
  </NuxtLayout>
</template>

<!-- Styles in: app/assets/css/ui/content/init.css -->
