<script setup lang="ts">
/**
 * Contact Section
 *
 * Contact form with config-driven contact info sidebar.
 * Uses <ContactInfo /> component for email, phone, location.
 * Location supports: empty (hidden), address text, coordinates (Yandex map).
 */

const { t, te } = useI18n()

// Self-contained: fetch title from i18n if not provided
const sectionTitle = computed(() => {
  if (te('contact.title')) return t('contact.title')
  if (te('nav.contact')) return t('nav.contact')
  return 'Get in Touch'
})

const props = withDefaults(defineProps<{
  /** Section title */
  title?: string
  /** Show contact info sidebar (from settings) - defaults to true for standalone pages */
  showInfo?: boolean
  /** Show map widget if location has coordinates */
  showMap?: boolean
}>(), {
  showInfo: true // Default to showing info on standalone pages
})

// Form state
const form = reactive({
  name: '',
  email: '',
  message: ''
})

const isSubmitting = ref(false)
const submitStatus = ref<'idle' | 'success' | 'error'>('idle')
const errorMessage = ref('')

// Inline validation state
const touched = reactive({ name: false, email: false, message: false })
const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

const validation = computed(() => ({
  email: !form.email || emailRegex.test(form.email),
  message: !form.message || form.message.length >= 10
}))

// Reset status when user starts typing (clears success/error messages)
watch(() => [form.name, form.email, form.message], () => {
  if (submitStatus.value !== 'idle') {
    submitStatus.value = 'idle'
    errorMessage.value = ''
  }
})

async function handleSubmit() {
  isSubmitting.value = true
  submitStatus.value = 'idle'
  errorMessage.value = ''

  try {
    await $fetch('/api/contact/submit', {
      method: 'POST',
      body: {
        name: form.name,
        email: form.email,
        message: form.message
      }
    })
    submitStatus.value = 'success'
    form.name = ''
    form.email = ''
    form.message = ''
    // Success message persists until user starts typing again
  } catch (e: any) {
    submitStatus.value = 'error'
    // Extract field errors from validation response
    const fieldErrors = e.data?.data?.fieldErrors
    if (fieldErrors) {
      const errors = Object.entries(fieldErrors)
        .map(([field, msgs]) => `${field}: ${(msgs as string[]).join(', ')}`)
        .join('; ')
      errorMessage.value = errors
    } else {
      errorMessage.value = e.data?.message || t('contact.errorMessage')
    }
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <!--
    Uses global classes from:
    - layout/sections.css (.section)
    - typography/base.css (.section-title, .section-title--center)
    - ui/forms/inputs.css (.contact-grid, .contact-form, .contact-info, .form-*, .input)
  -->
  <section id="contact" class="section">
    <div class="container">
      <h2 v-reveal class="section-title section-title--center">
        <slot name="title">{{ title ?? sectionTitle }}</slot>
      </h2>

      <div class="section-body">
        <div class="contact-grid">
        <form v-reveal="{ delay: 100 }" class="contact-form" @submit.prevent="handleSubmit">
          <div class="form-group">
            <label class="form-label" for="contact-name">{{ t('contact.nameLabel') }}</label>
            <input
              id="contact-name"
              v-model="form.name"
              type="text"
              class="input"
              autocomplete="name"
              :placeholder="t('contact.namePlaceholder')"
              required
            />
          </div>

          <div class="form-group">
            <label class="form-label" for="contact-email">{{ t('contact.emailLabel') }}</label>
            <input
              id="contact-email"
              v-model="form.email"
              type="email"
              class="input"
              :class="{ 'input--error': touched.email && !validation.email }"
              autocomplete="email"
              :placeholder="t('contact.emailPlaceholder')"
              :aria-invalid="touched.email && !validation.email"
              required
              @blur="touched.email = true"
            />
            <span v-if="touched.email && !validation.email" class="form-hint form-hint--error">
              {{ t('contact.invalidEmail', 'Please enter a valid email address') }}
            </span>
          </div>

          <div class="form-group">
            <label class="form-label" for="contact-message">{{ t('contact.messageLabel') }}</label>
            <textarea
              id="contact-message"
              v-model="form.message"
              class="input"
              :class="{ 'input--error': touched.message && !validation.message }"
              rows="5"
              minlength="10"
              :placeholder="t('contact.messagePlaceholder')"
              :aria-invalid="touched.message && !validation.message"
              required
              @blur="touched.message = true"
            ></textarea>
            <span v-if="touched.message && !validation.message" class="form-hint form-hint--error">
              {{ t('contact.messageTooShort', 'Message must be at least 10 characters') }}
            </span>
          </div>

          <button type="submit" class="btn btn-primary btn-lg btn-full" :disabled="isSubmitting">
            {{ isSubmitting ? t('contact.sending') : t('contact.sendButton') }}
          </button>

          <!-- Status messages with aria-live for screen readers -->
          <div aria-live="polite" aria-atomic="true">
            <p v-if="submitStatus === 'success'" class="form-success" role="status">
              ✓ {{ t('contact.successMessage') }}
            </p>
            <p v-if="submitStatus === 'error'" class="form-error" role="alert">
              ✕ {{ errorMessage || t('contact.errorMessage') }}
            </p>
          </div>
        </form>

        <!-- Contact Info sidebar - config-driven via settings -->
        <aside v-if="props.showInfo" v-reveal="{ animation: 'fade-left', delay: 200 }" class="contact-sidebar">
          <MoleculesContactInfo :show-map="showMap" />
        </aside>
        </div>
      </div>
    </div>
  </section>
</template>

<!-- No scoped styles needed - all styles come from global CSS -->
