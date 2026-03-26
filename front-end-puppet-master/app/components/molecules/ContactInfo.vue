<script setup lang="ts">
/**
 * ContactInfo Molecule
 *
 * Config-driven contact information display.
 * - Reads from useSiteSettings()
 * - Only renders items that have values
 * - Icons from Tabler
 * - Location special handling:
 *   - Empty: hidden
 *   - Address text: show with icon
 *   - Coordinates (lat,lng): show Yandex map widget
 */

import IconMail from '~icons/tabler/mail'
import IconPhone from '~icons/tabler/phone'
import IconMapPin from '~icons/tabler/map-pin'

defineProps<{
  /** Show in vertical list (default) or horizontal */
  horizontal?: boolean
  /** Show map if location has coordinates */
  showMap?: boolean
}>()

const { settings } = useSiteSettings()

// Check if location is coordinates (format: "lat,lng" or "lat, lng")
const isCoordinates = computed(() => {
  const loc = settings.value?.contact.location
  if (!loc) return false
  // Match pattern: number,number (with optional spaces and decimals)
  return /^-?\d+\.?\d*\s*,\s*-?\d+\.?\d*$/.test(loc.trim())
})

// Parse coordinates for map
const coordinates = computed(() => {
  if (!isCoordinates.value) return null
  const loc = settings.value?.contact.location
  if (!loc) return null
  const [lat, lng] = loc.split(',').map((s: string) => parseFloat(s.trim()))
  return { lat, lng }
})

// Yandex map embed URL
const mapUrl = computed(() => {
  if (!coordinates.value) return null
  const { lat, lng } = coordinates.value
  // Yandex Maps static embed
  return `https://yandex.com/map-widget/v1/?ll=${lng},${lat}&z=15&pt=${lng},${lat},pm2rdm`
})

// Contact items to display (only non-empty)
const contactItems = computed(() => {
  const items: Array<{ key: string; value: string; icon: Component; href?: string }> = []
  const contact = settings.value?.contact

  if (contact?.email) {
    items.push({
      key: 'email',
      value: contact.email,
      icon: IconMail,
      href: `mailto:${contact.email}`
    })
  }

  if (contact?.phone) {
    items.push({
      key: 'phone',
      value: contact.phone,
      icon: IconPhone,
      href: `tel:${contact.phone.replace(/\s/g, '')}`
    })
  }

  // Location as text (not coordinates)
  if (contact?.location && !isCoordinates.value) {
    items.push({
      key: 'location',
      value: contact.location,
      icon: IconMapPin
    })
  }

  return items
})
</script>

<template>
  <div
    v-if="contactItems.length > 0 || (showMap && coordinates)"
    class="contact-info"
    :class="{ 'contact-info--horizontal': horizontal }"
  >
    <!-- Contact items list -->
    <div v-if="contactItems.length > 0" class="contact-info-list">
      <div v-for="item in contactItems" :key="item.key" class="contact-info-item">
        <component :is="item.icon" class="contact-info-icon" />
        <component :is="item.href ? 'a' : 'span'" :href="item.href" class="contact-info-value">
          {{ item.value }}
        </component>
      </div>
    </div>

    <!-- Yandex Map (if coordinates and showMap) -->
    <div v-if="showMap && coordinates && mapUrl" class="contact-info-map">
      <iframe
        :src="mapUrl"
        width="100%"
        height="300"
        frameborder="0"
        allowfullscreen
        loading="lazy"
        title="Location map"
      />
    </div>
  </div>
</template>
