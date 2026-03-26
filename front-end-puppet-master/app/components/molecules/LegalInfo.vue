<script setup lang="ts">
/**
 * LegalInfo Molecule
 *
 * Config-driven copyright and legal/juridical info.
 * All on one line: © 2025 Company. All rights reserved · ИНН: xxx · ОГРН: xxx · ...
 * Only shows items that have values in database.
 */

const { t } = useI18n()
const { settings } = useSiteSettings()
const currentYear = new Date().getFullYear()

// Company name for copyright (fallback to 'Puppet Master')
const companyName = computed(() => {
  return settings.value?.legal.companyName || settings.value?.seo.title || 'Puppet Master'
})

// Legal details (only those with values)
const legalDetails = computed(() => {
  const legal = settings.value?.legal
  if (!legal) return []

  const items: Array<{ label: string; value: string }> = []

  if (legal.inn) items.push({ label: 'ИНН', value: legal.inn })
  if (legal.ogrn) items.push({ label: 'ОГРН', value: legal.ogrn })
  if (legal.address) items.push({ label: 'Адрес', value: legal.address })
  if (legal.email) items.push({ label: 'Email', value: legal.email })

  return items
})
</script>

<template>
  <!-- All on ONE line: copyright + legal details -->
  <div class="legal-row">
    <span>© {{ currentYear }} {{ companyName }}. {{ t('footer.rights') }}</span>
    <template v-for="item in legalDetails" :key="item.label">
      <span class="legal-sep">·</span>
      <span>{{ item.label }}: {{ item.value }}</span>
    </template>
  </div>
</template>
