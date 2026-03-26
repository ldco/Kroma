<script setup lang="ts">
/**
 * Team Member Card Component
 *
 * Displays a single team member with photo, name, position, and social links.
 */
import IconMail from '~icons/tabler/mail'
import IconPhone from '~icons/tabler/phone'
import IconBrandLinkedin from '~icons/tabler/brand-linkedin'
import IconBrandTwitter from '~icons/tabler/brand-twitter'
import IconBrandGithub from '~icons/tabler/brand-github'
import IconWorld from '~icons/tabler/world'

interface TeamMemberData {
  id: number
  slug: string
  name: string
  position: string | null
  bio: string | null
  photoUrl: string | null
  hoverPhotoUrl: string | null
  email: string | null
  phone: string | null
  department: string | null
  socialLinks: Record<string, string> | null
}

const props = defineProps<{
  member: TeamMemberData
  showBio?: boolean
  showContact?: boolean
}>()

// Social link icons mapping
const socialIcons: Record<string, typeof IconBrandLinkedin> = {
  linkedin: IconBrandLinkedin,
  twitter: IconBrandTwitter,
  github: IconBrandGithub,
  website: IconWorld
}

// Get social links as array for iteration
const socialLinksArray = computed(() => {
  if (!props.member.socialLinks) return []
  return Object.entries(props.member.socialLinks).map(([key, url]) => ({
    key,
    url,
    icon: socialIcons[key] || IconWorld
  }))
})
</script>

<template>
  <div class="team-card">
    <!-- Photo with hover effect -->
    <div class="team-card__photo" :class="{ 'team-card__photo--has-hover': member.hoverPhotoUrl }">
      <img
        v-if="member.photoUrl"
        :src="member.photoUrl"
        :alt="member.name"
        class="team-card__img team-card__img--default"
        loading="lazy"
      />
      <img
        v-if="member.hoverPhotoUrl"
        :src="member.hoverPhotoUrl"
        :alt="member.name"
        class="team-card__img team-card__img--hover"
        loading="lazy"
      />
      <div v-if="!member.photoUrl" class="team-card__photo-placeholder">
        {{ member.name.charAt(0) }}
      </div>
    </div>

    <!-- Info -->
    <div class="team-card__info">
      <h3 class="team-card__name">{{ member.name }}</h3>
      <p v-if="member.position" class="team-card__position">
        {{ member.position }}
      </p>
      <p v-if="member.department" class="team-card__department">
        {{ member.department }}
      </p>

      <!-- Bio -->
      <p v-if="showBio && member.bio" class="team-card__bio">
        {{ member.bio }}
      </p>

      <!-- Contact & Social Links - Combined -->
      <div v-if="showContact && (member.email || member.phone || socialLinksArray.length > 0)" class="team-card__social">
        <!-- Email -->
        <a v-if="member.email" :href="`mailto:${member.email}`" class="team-card__contact-link" title="Email">
          <IconMail />
        </a>
        <!-- Phone -->
        <a v-if="member.phone" :href="`tel:${member.phone}`" class="team-card__contact-link" title="Phone">
          <IconPhone />
        </a>
        <!-- Social Links -->
        <a
          v-for="social in socialLinksArray"
          :key="social.key"
          :href="social.url"
          target="_blank"
          rel="noopener noreferrer"
          class="team-card__social-link"
          :title="social.key"
        >
          <component :is="social.icon" />
        </a>
      </div>
    </div>
  </div>
</template>
