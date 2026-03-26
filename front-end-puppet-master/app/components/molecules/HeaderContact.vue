<script setup lang="ts">
/**
 * HeaderContact Molecule
 *
 * Quick contact buttons for header - max 2 buttons (phone + messenger).
 * Best practice: Direct action links for instant contact.
 *
 * Uses contact settings from database:
 * - Phone: tel: link
 * - Messenger: telegram/whatsapp/viber URL
 *
 * Config-driven: Shows buttons based on headerContact.items config.
 */
import config from '~/puppet-master.config'

// Tabler icons for contact
import IconPhone from '~icons/tabler/phone'
import IconBrandTelegram from '~icons/tabler/brand-telegram'
import IconBrandWhatsapp from '~icons/tabler/brand-whatsapp'
import IconMail from '~icons/tabler/mail'

// Icon map
const iconMap: Record<string, Component> = {
  phone: IconPhone,
  'brand-telegram': IconBrandTelegram,
  'brand-whatsapp': IconBrandWhatsapp,
  mail: IconMail
}

// Get contact values from database
const { settings } = useSiteSettings()

// SSR-safe config access with defensive defaults
const headerContactConfig = computed(() => config?.headerContact ?? { enabled: false, items: [] })
const settingsConfig = computed(() => config?.settings ?? [])

// Helper to get nested value from settings using dot notation key
function getSettingValue(key: string): string | null {
  const parts = key.split('.')
  if (parts.length !== 2) return null
  const [group, field] = parts
  if (!group || !field) return null
  const groupData = settings.value?.[group as keyof typeof settings.value]
  if (!groupData || typeof groupData !== 'object') return null
  return (groupData as Record<string, string | null>)[field] || null
}

// Build contact items from config (max 2)
const contactItems = computed(() => {
  const items: Array<{ key: string; url: string; icon: Component; label: string }> = []

  // Get configured header contact items (max 2)
  const defaultItems: string[] = ['contact.phone', 'social.telegram']
  const configItems = headerContactConfig.value.items ?? defaultItems

  for (const configKey of configItems.slice(0, 2)) {
    const key: string = configKey
    const value = getSettingValue(key)
    if (!value) continue

    // Find setting definition for icon
    const settingDef = settingsConfig.value.find(s => s.key === key)
    if (!settingDef || !('icon' in settingDef) || !settingDef.icon) continue

    const iconComponent = iconMap[settingDef.icon as string]
    if (!iconComponent) continue

    // Build URL based on type
    let url = value
    if (key === 'contact.phone') {
      url = `tel:${value.replace(/\s/g, '')}`
    } else if (key === 'contact.email') {
      url = `mailto:${value}`
    }
    // Social URLs are already full URLs

    items.push({
      key,
      url,
      icon: iconComponent,
      label: settingDef.label
    })
  }

  return items
})
</script>

<template>
  <div v-if="contactItems.length > 0" class="header-contact">
    <a
      v-for="item in contactItems"
      :key="item.key"
      :href="item.url"
      class="header-contact-btn"
      :aria-label="item.label"
      target="_blank"
      rel="noopener noreferrer"
    >
      <component :is="item.icon" class="header-contact-icon" />
    </a>
  </div>
</template>
