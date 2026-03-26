<script setup lang="ts">
/**
 * Team Grid Component
 *
 * Displays team members in a grid layout.
 * Fetches members from database API.
 */

interface TeamMemberData {
  id: number
  slug: string
  name: string
  position: string | null
  bio: string | null
  photoUrl: string | null
  email: string | null
  phone: string | null
  department: string | null
  socialLinks: Record<string, string> | null
}

const { t, locale } = useI18n()

const props = withDefaults(defineProps<{
  /** Filter by department */
  department?: string
  /** Limit number of members */
  limit?: number
  /** Show member bio */
  showBio?: boolean
  /** Show contact info */
  showContact?: boolean
}>(), {
  showBio: false,
  showContact: true
})

// Check if module is enabled
const { isEnabled } = useModule('team')

// Fetch team members
const { data: members, pending } = await useFetch<TeamMemberData[]>('/api/team', {
  key: `team-members-${locale.value}-${props.department || 'all'}-${props.limit || 'all'}`,
  query: {
    locale: locale.value,
    department: props.department,
    limit: props.limit
  },
  watch: [locale]
})
</script>

<template>
  <div v-if="isEnabled" class="team-grid" v-reveal>
    <!-- Loading state -->
    <div v-if="pending" class="team-grid__loading">
      {{ t('common.loading') }}
    </div>

    <!-- Empty state -->
    <div v-else-if="!members?.length" class="team-grid__empty">
      {{ t('team.noMembers') }}
    </div>

    <!-- Team grid -->
    <div v-else class="team-grid__items">
      <MoleculesTeamMemberCard
        v-for="member in members"
        :key="member.id"
        :member="member"
        :show-bio="showBio"
        :show-contact="showContact"
      />
    </div>
  </div>
</template>
