<script setup lang="ts">
/**
 * Admin Pricing Page
 *
 * Manage pricing tiers and their features with unified multi-language support.
 * All languages are equal - editable in the same modal with tabs.
 *
 * CSS Dependencies (see CSS-COMPONENT-MAP.md):
 * - ui/overlays/modal.css: .modal-backdrop, .modal, .modal--lg, .modal-header, .modal-body, .modal-footer
 * - ui/forms/inputs.css: .input, .form-group, .form-label, .form-row--2col, .form-checkbox, .form-divider
 * - ui/content/tabs.css: .tabs, .tabs--underline, .tab, .is-active, .tab__indicator
 * - ui/content/cards.css: .card, .card-body, .card-actions
 * - ui/content/pricing.css: .pricing-card, .pricing-card--featured
 */
import IconPlus from '~icons/tabler/plus'
import IconEdit from '~icons/tabler/pencil'
import IconTrash from '~icons/tabler/trash'
import IconX from '~icons/tabler/x'
import IconGripVertical from '~icons/tabler/grip-vertical'
import IconCheck from '~icons/tabler/check'
import config from '~/puppet-master.config'

interface PricingTranslation {
  name: string | null
  description: string | null
  ctaText: string | null
}

interface PricingTier {
  id: number
  slug: string
  price: number | null
  currency: string | null
  period: 'month' | 'year' | 'one-time' | null
  featured: boolean | null
  ctaUrl: string | null
  order: number | null
  published: boolean | null
  translations: Record<string, PricingTranslation>
  features: Array<{
    id: number
    included: boolean | null
    order: number | null
    translations: Record<string, string | null>
  }>
}

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

// SSR-safe config access with defensive defaults
const localesConfig = computed(() => config?.locales ?? [])
const defaultLocaleConfig = computed(() => config?.defaultLocale || 'en')

// Available locales from config
const locales = localesConfig.value.map(l => l.code)

useHead({
  title: () => `${t('admin.pricing')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// Fetch tiers
const headers = useRequestHeaders(['cookie'])
const {
  data: tiers,
  pending,
  refresh
} = await useFetch<PricingTier[]>('/api/admin/pricing', { headers })

// Modal state
const showModal = ref(false)
const editingTier = ref<PricingTier | null>(null)
const saving = ref(false)
const activeLocale = ref(locales[0] || 'en')

interface TierTranslationForm {
  name: string
  description: string
  ctaText: string
}

interface FeatureForm {
  id?: number
  included: boolean
  translations: Record<string, string>
}

function createEmptyTierTranslations(): Record<string, TierTranslationForm> {
  const translations: Record<string, TierTranslationForm> = {}
  for (const locale of locales) {
    translations[locale] = { name: '', description: '', ctaText: '' }
  }
  return translations
}

function createEmptyFeatureTranslations(): Record<string, string> {
  const translations: Record<string, string> = {}
  for (const locale of locales) {
    translations[locale] = ''
  }
  return translations
}

const form = reactive({
  slug: '',
  price: null as number | null,
  currency: 'USD',
  period: 'month' as 'month' | 'year' | 'one-time',
  featured: false,
  ctaUrl: '/contact',
  order: 0,
  published: true,
  translations: createEmptyTierTranslations(),
  features: [] as FeatureForm[]
})

function resetForm() {
  form.slug = ''
  form.price = null
  form.currency = 'USD'
  form.period = 'month'
  form.featured = false
  form.ctaUrl = '/contact'
  form.order = 0
  form.published = true
  form.translations = createEmptyTierTranslations()
  form.features = []
  activeLocale.value = locales[0] || 'en'
}

function openCreate() {
  editingTier.value = null
  resetForm()
  showModal.value = true
}

function openEdit(tier: PricingTier) {
  editingTier.value = tier
  form.slug = tier.slug
  form.price = tier.price !== null ? tier.price / 100 : null // Convert cents to dollars
  form.currency = tier.currency || 'USD'
  form.period = tier.period || 'month'
  form.featured = tier.featured || false
  form.ctaUrl = tier.ctaUrl || '/contact'
  form.order = tier.order || 0
  form.published = tier.published ?? true

  // Load tier translations
  form.translations = createEmptyTierTranslations()
  for (const locale of locales) {
    const trans = tier.translations?.[locale]
    if (trans) {
      form.translations[locale] = {
        name: trans.name || '',
        description: trans.description || '',
        ctaText: trans.ctaText || ''
      }
    }
  }

  // Load features with translations
  form.features = (tier.features || []).map(f => ({
    id: f.id,
    included: f.included ?? true,
    translations: locales.reduce((acc, locale) => {
      acc[locale] = f.translations?.[locale] || ''
      return acc
    }, {} as Record<string, string>)
  }))

  // Find first locale with content
  const localeWithContent = locales.find(locale => tier.translations?.[locale]?.name)
  activeLocale.value = localeWithContent || locales[0] || 'en'

  showModal.value = true
}

function generateSlug() {
  if (!editingTier.value && !form.slug) {
    for (const locale of locales) {
      const name = form.translations[locale]?.name
      if (name) {
        form.slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
        break
      }
    }
  }
}

function addFeature() {
  form.features.push({
    included: true,
    translations: createEmptyFeatureTranslations()
  })
}

function removeFeature(index: number) {
  form.features.splice(index, 1)
}

async function saveTier() {
  saving.value = true

  try {
    const payload = {
      slug: form.slug,
      price: form.price,
      currency: form.currency,
      period: form.period,
      featured: form.featured,
      ctaUrl: form.ctaUrl,
      order: form.order,
      published: form.published,
      translations: form.translations,
      features: form.features.filter(f => {
        // Keep feature if it has text in any locale
        return Object.values(f.translations).some(t => t.trim())
      }).map((f, i) => ({
        id: f.id,
        included: f.included,
        order: i,
        translations: f.translations
      }))
    }

    if (editingTier.value) {
      await $fetch(`/api/admin/pricing/${editingTier.value.id}`, {
        method: 'PUT',
        body: payload
      })
    } else {
      await $fetch('/api/admin/pricing', {
        method: 'POST',
        body: payload
      })
    }
    toast.success(t('common.saved'))
    showModal.value = false
    refresh()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  } finally {
    saving.value = false
  }
}

async function deleteTier(tier: PricingTier) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })

  if (!confirmed) return

  try {
    await $fetch(`/api/admin/pricing/${tier.id}`, { method: 'DELETE' })
    toast.success(t('common.deleted'))
    refresh()
  } catch (e: unknown) {
    const error = e as { data?: { message?: string } }
    toast.error(error.data?.message || t('common.error'))
  }
}

function formatPrice(tier: PricingTier): string {
  if (tier.price === null) return t('pricing.custom')
  if (tier.price === 0) return t('pricing.free')
  const symbols: Record<string, string> = { USD: '$', EUR: '€', GBP: '£', RUB: '₽', ILS: '₪' }
  const symbol = symbols[tier.currency || 'USD'] || tier.currency
  return `${symbol}${tier.price / 100}`
}

function getLocaleName(code: string): string {
  const locale = localesConfig.value.find(l => l.code === code)
  return locale?.name || code.toUpperCase()
}

function getTierName(tier: PricingTier): string {
  for (const locale of locales) {
    const name = tier.translations?.[locale]?.name
    if (name) return name
  }
  return tier.slug
}

function getTierDescription(tier: PricingTier): string {
  for (const locale of locales) {
    const desc = tier.translations?.[locale]?.description
    if (desc) return desc
  }
  return ''
}

function getFeatureText(feature: { translations: Record<string, string | null> }): string {
  for (const locale of locales) {
    const text = feature.translations?.[locale]
    if (text) return text
  }
  return ''
}

function hasTranslation(tier: PricingTier, locale: string): boolean {
  return !!(tier.translations?.[locale]?.name)
}

function hasAllTranslations(tier: PricingTier): boolean {
  return locales.every(locale => hasTranslation(tier, locale))
}

function getMissingTranslations(tier: PricingTier): string[] {
  return locales.filter(locale => !hasTranslation(tier, locale))
}

function formLocaleHasContent(locale: string): boolean {
  return !!(form.translations[locale]?.name)
}
</script>

<template>
  <div class="admin-pricing">
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.pricing') }}</h1>
      <button class="btn btn-primary" @click="openCreate">
        <IconPlus />
        {{ t('admin.addItem') }}
      </button>
    </div>

    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <div v-else-if="!tiers?.length" class="empty-state">
      <p>{{ t('admin.noItems') }}</p>
    </div>

    <div v-else class="pricing-tiers__grid">
      <div
        v-for="tier in tiers"
        :key="tier.id"
        class="pricing-card"
        :class="{ 'pricing-card--featured': tier.featured }"
      >
        <span v-if="tier.featured" class="pricing-card__badge">{{ t('admin.featured') }}</span>

        <span class="badge pricing-card__status" :class="tier.published ? 'badge-success' : 'badge-warning'">
          {{ tier.published ? t('admin.published') : t('common.draft') }}
        </span>

        <div class="pricing-card__header">
          <h3 class="pricing-card__name">{{ getTierName(tier) }}</h3>
          <p v-if="getTierDescription(tier)" class="pricing-card__description">{{ getTierDescription(tier) }}</p>
        </div>

        <div class="pricing-card__price">
          <span v-if="tier.price === null" class="pricing-card__price-custom">{{ t('pricing.custom') }}</span>
          <template v-else>
            <span class="pricing-card__price-value">{{ formatPrice(tier) }}</span>
            <span v-if="tier.period !== 'one-time'" class="pricing-card__price-period">
              {{ tier.period === 'year' ? t('pricing.perYear') : t('pricing.perMonth') }}
            </span>
          </template>
        </div>

        <ul class="pricing-card__features">
          <li v-for="feature in tier.features?.slice(0, 4)" :key="feature.id" class="pricing-card__feature"
              :class="{ 'pricing-card__feature--excluded': !feature.included }">
            <span class="pricing-card__feature-icon">
              <IconCheck v-if="feature.included" /><IconX v-else />
            </span>
            <span class="pricing-card__feature-text">{{ getFeatureText(feature) }}</span>
          </li>
          <li v-if="(tier.features?.length || 0) > 4" class="pricing-card__feature">
            <span class="pricing-card__feature-icon" />
            <span class="pricing-card__feature-text text-secondary">+{{ (tier.features?.length || 0) - 4 }} {{ t('common.more') }}</span>
          </li>
        </ul>

        <!-- Translation status -->
        <div v-if="!hasAllTranslations(tier)" class="mt-2">
          <span class="text-warning text-sm">
            {{ t('admin.missingTranslations') }}: {{ getMissingTranslations(tier).map(l => l.toUpperCase()).join(', ') }}
          </span>
        </div>

        <div class="pricing-card__actions">
          <button class="btn btn-sm btn-secondary" @click="openEdit(tier)">
            <IconEdit /> {{ t('common.edit') }}
          </button>
          <button class="btn btn-sm btn-ghost text-error" @click="deleteTier(tier)">
            <IconTrash />
          </button>
        </div>
      </div>
    </div>

    <!-- Edit Modal with Language Tabs -->
    <Teleport to="body">
      <div v-if="showModal" class="modal-backdrop" @click.self="showModal = false">
        <div class="modal modal--lg">
          <header class="modal-header">
            <h2>{{ editingTier ? t('common.edit') : t('admin.addItem') }}</h2>
            <button class="btn btn-ghost btn-sm" @click="showModal = false"><IconX /></button>
          </header>

          <form class="modal-body" @submit.prevent="saveTier">
            <!-- Non-translatable fields -->
            <div class="form-row form-row--2col">
              <div class="form-group">
                <label class="form-label" for="tier-slug">{{ t('admin.slug') }} *</label>
                <input id="tier-slug" v-model="form.slug" type="text" class="input" required pattern="[a-z0-9-]+" />
              </div>

              <div class="form-group">
                <label class="form-label" for="tier-price">{{ t('admin.price') }}</label>
                <input id="tier-price" v-model.number="form.price" type="number" class="input" min="0" step="0.01" />
              </div>

              <div class="form-group">
                <label class="form-label" for="tier-currency">{{ t('admin.currency') }}</label>
                <select id="tier-currency" v-model="form.currency" class="input">
                  <option value="USD">USD ($)</option>
                  <option value="EUR">EUR (€)</option>
                  <option value="GBP">GBP (£)</option>
                  <option value="RUB">RUB (₽)</option>
                  <option value="ILS">ILS (₪)</option>
                </select>
              </div>

              <div class="form-group">
                <label class="form-label" for="tier-period">{{ t('admin.period') }}</label>
                <select id="tier-period" v-model="form.period" class="input">
                  <option value="month">{{ t('pricing.monthly') }}</option>
                  <option value="year">{{ t('pricing.yearly') }}</option>
                  <option value="one-time">{{ t('pricing.oneTime') }}</option>
                </select>
              </div>

              <div class="form-group">
                <label class="form-label" for="tier-cta-url">{{ t('admin.ctaUrl') }}</label>
                <input id="tier-cta-url" v-model="form.ctaUrl" type="text" class="input" />
              </div>

              <div class="form-group">
                <label class="form-label" for="tier-order">{{ t('admin.order') }}</label>
                <input id="tier-order" v-model.number="form.order" type="number" class="input" min="0" />
              </div>
            </div>

            <div class="form-row">
              <label class="form-checkbox">
                <input v-model="form.featured" type="checkbox" /><span>{{ t('admin.featured') }}</span>
              </label>
              <label class="form-checkbox">
                <input v-model="form.published" type="checkbox" /><span>{{ t('admin.published') }}</span>
              </label>
            </div>

            <!-- Language tabs for translatable content -->
            <div class="form-divider"><span>{{ t('admin.content') }}</span></div>

            <div class="tabs tabs--underline mb-4">
              <button v-for="locale in locales" :key="locale" type="button" class="tab"
                      :class="{ 'is-active': activeLocale === locale }" @click="activeLocale = locale">
                {{ getLocaleName(locale) }}
                <span v-if="!formLocaleHasContent(locale)" class="tab__indicator tab__indicator--warning"
                      :title="t('admin.missingTranslations')"></span>
              </button>
            </div>

            <!-- Tier translation fields per locale -->
            <div v-for="locale in locales" v-show="activeLocale === locale" :key="locale">
              <div class="form-group">
                <label class="form-label">{{ t('admin.name') }}</label>
                <input v-model="form.translations[locale].name" type="text" class="input" @blur="generateSlug" />
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.description') }}</label>
                <textarea v-model="form.translations[locale].description" class="input" rows="2"></textarea>
              </div>

              <div class="form-group">
                <label class="form-label">{{ t('admin.ctaText') }}</label>
                <input v-model="form.translations[locale].ctaText" type="text" class="input" />
              </div>

              <!-- Features for this locale -->
              <div class="form-section">
                <div class="form-section-header">
                  <span class="form-section-title">{{ t('admin.features') }}</span>
                  <button type="button" class="btn-add-inline" @click="addFeature">
                    <IconPlus /> {{ t('admin.addItem') }}
                  </button>
                </div>

                <div v-if="!form.features.length" class="text-secondary text-sm">{{ t('admin.noItems') }}</div>

                <div v-for="(feature, index) in form.features" :key="index" class="input-row mb-2">
                  <button type="button" class="btn btn-ghost btn-sm"><IconGripVertical /></button>
                  <input v-model="feature.translations[locale]" type="text" class="input" />
                  <label class="form-checkbox">
                    <input v-model="feature.included" type="checkbox" /><span>{{ t('admin.included') }}</span>
                  </label>
                  <button type="button" class="btn btn-ghost btn-sm text-error" @click="removeFeature(index)">
                    <IconTrash />
                  </button>
                </div>
              </div>
            </div>
          </form>

          <footer class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="showModal = false">{{ t('common.cancel') }}</button>
            <button type="submit" class="btn btn-primary" :disabled="saving" @click="saveTier">
              {{ saving ? t('common.saving') : t('common.save') }}
            </button>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>
