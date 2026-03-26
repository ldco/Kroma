<script setup lang="ts">
/**
 * Run Status Badge Atom
 *
 * Displays run status with color-coded badge.
 * Supports: pending, running, completed, failed, cancelled
 */
import type { RunStatus } from '~/types/kroma'

const props = defineProps<{
  status: RunStatus
  showLabel?: boolean
  size?: 'sm' | 'md' | 'lg'
}>()

const statusConfig: Record<RunStatus, { label: string; class: string; icon: string }> = {
  pending: {
    label: 'Pending',
    class: 'status-badge--pending',
    icon: 'lucide:clock'
  },
  running: {
    label: 'Running',
    class: 'status-badge--running',
    icon: 'lucide:loader'
  },
  completed: {
    label: 'Completed',
    class: 'status-badge--completed',
    icon: 'lucide:check-circle'
  },
  failed: {
    label: 'Failed',
    class: 'status-badge--failed',
    icon: 'lucide:x-circle'
  },
  cancelled: {
    label: 'Cancelled',
    class: 'status-badge--cancelled',
    icon: 'lucide:ban'
  }
}

const config = computed(() => statusConfig[props.status] || statusConfig.pending)

const showLabel = computed(() => props.showLabel ?? true)
const size = computed(() => props.size || 'md')
</script>

<template>
  <!-- Uses global classes from ui/runs/run-status.css -->
  <span class="run-status-badge" :class="[config.class, `run-status-badge--${size}`]">
    <Icon :name="config.icon" class="run-status-badge__icon" :class="`icon-${size}`" />
    <span v-if="showLabel" class="run-status-badge__label">{{ config.label }}</span>
  </span>
</template>

<!-- No scoped styles - uses ui/runs/run-status.css -->
