<script setup lang="ts">
/**
 * Admin Health Page
 *
 * Server health metrics dashboard for master users only.
 * Displays: system status, recent logs, audit trail, external monitoring link.
 *
 * Uses global CSS: .card, .badge, .data-table, .btn, .flex, .grid utilities
 */
import IconRefresh from '~icons/tabler/refresh'
import IconExternalLink from '~icons/tabler/external-link'
import IconCheck from '~icons/tabler/check'
import IconAlertTriangle from '~icons/tabler/alert-triangle'
import IconX from '~icons/tabler/x'
import IconCopy from '~icons/tabler/copy'
import type { HealthCheck, HealthData, LogEntry, AuditLogEntry } from '~/types'

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('admin.navHealth')} | Admin`
})

const { toast } = useToast()
const config = useRuntimeConfig()

// Auto-refresh toggle
const autoRefresh = ref(true)
const refreshInterval = ref<ReturnType<typeof setInterval> | null>(null)

// Fetch health data
const headers = useRequestHeaders(['cookie'])
const {
  data: healthData,
  pending: healthPending,
  refresh: refreshHealth
} = await useFetch<HealthData>('/api/admin/health', {
  headers
})

// Fetch logs
const logLevel = ref('all')
const {
  data: logsData,
  pending: logsPending,
  refresh: refreshLogs
} = await useFetch<{ logs: LogEntry[]; meta: Record<string, unknown> }>('/api/admin/logs', {
  headers,
  query: computed(() => ({
    limit: 100,
    level: logLevel.value === 'all' ? undefined : logLevel.value
  }))
})

// Fetch audit logs
const {
  data: auditData,
  pending: auditPending,
  refresh: refreshAudit
} = await useFetch<{ logs: AuditLogEntry[]; meta: Record<string, unknown> }>(
  '/api/admin/audit-logs',
  { headers }
)

// Computed values
const logs = computed(() => logsData.value?.logs || [])
const auditLogs = computed(() => auditData.value?.logs || [])

// Format uptime
function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)

  if (days > 0) {
    return `${days}d ${hours}h ${minutes}m`
  }
  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }
  return `${minutes}m`
}

// Format timestamp
function formatTime(isoString: string): string {
  const date = new Date(isoString)
  return date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  })
}

// Format date
function formatDate(isoString: string): string {
  const date = new Date(isoString)
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// Get status badge class
function getStatusBadgeClass(status: string): string {
  switch (status) {
    case 'ok':
      return 'badge-success'
    case 'warning':
    case 'degraded':
      return 'badge-warning'
    case 'error':
    case 'unhealthy':
      return 'badge-error'
    default:
      return 'badge-secondary'
  }
}

// Get log level badge class
function getLogLevelClass(level: number): string {
  if (level >= 50) return 'badge-error' // error, fatal
  if (level >= 40) return 'badge-warning' // warn
  if (level >= 30) return 'badge-info' // info
  return 'badge-secondary' // debug, trace
}

// Refresh all data
async function refreshAll() {
  await Promise.all([refreshHealth(), refreshLogs(), refreshAudit()])
}

// Copy logs to clipboard
async function copyLogs() {
  const logText = logs.value
    .map(log => `${log.time} [${log.levelLabel.toUpperCase()}] ${log.msg}`)
    .join('\n')

  try {
    await navigator.clipboard.writeText(logText)
    toast.success('Logs copied to clipboard')
  } catch {
    toast.error('Failed to copy logs')
  }
}

// Auto-refresh logic
watch(autoRefresh, enabled => {
  if (enabled) {
    refreshInterval.value = setInterval(refreshAll, 30000)
  } else if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
    refreshInterval.value = null
  }
})

onMounted(() => {
  if (autoRefresh.value) {
    refreshInterval.value = setInterval(refreshAll, 30000)
  }
})

onUnmounted(() => {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
  }
})

// Last updated display
const lastUpdated = computed(() => {
  if (!healthData.value?.timestamp) return ''
  return formatTime(healthData.value.timestamp)
})

// Uptime Kuma URL from config
const uptimeKumaUrl = computed(() => config.public.uptimeKumaUrl || '')
</script>

<template>
  <div>
    <!-- Page Header -->
    <div class="page-header">
      <h1 class="page-title">{{ t('admin.health') }}</h1>
      <div class="page-actions">
        <label class="toggle toggle--sm">
          <input v-model="autoRefresh" type="checkbox" />
          <span class="toggle-track"></span>
          <span class="toggle-label">Auto-refresh</span>
        </label>
        <button class="btn btn-sm btn-outline" :disabled="healthPending" @click="refreshAll">
          <IconRefresh :class="{ spin: healthPending }" />
          Refresh
        </button>
      </div>
    </div>

    <div class="admin-page-content">
      <!-- System Status -->
      <section class="card mb-6">
        <div class="card-header">
          <h2 class="card-title">System Status</h2>
          <span v-if="lastUpdated" class="text-sm text-secondary">
            Last updated: {{ lastUpdated }}
          </span>
        </div>
        <div class="card-body">
          <div v-if="healthPending" class="loading-state">
            <p>Loading...</p>
          </div>
          <div v-else-if="healthData" class="health-grid">
            <!-- Overall Status -->
            <div class="health-item">
              <span class="health-label">Status</span>
              <span :class="['badge', getStatusBadgeClass(healthData.status)]">
                <IconCheck v-if="healthData.status === 'ok'" class="icon-xs" />
                <IconAlertTriangle v-else-if="healthData.status === 'degraded'" class="icon-xs" />
                <IconX v-else class="icon-xs" />
                {{ healthData.status }}
              </span>
            </div>

            <!-- Uptime -->
            <div class="health-item">
              <span class="health-label">Uptime</span>
              <span class="health-value">{{ formatUptime(healthData.uptime) }}</span>
            </div>

            <!-- Version -->
            <div class="health-item">
              <span class="health-label">Version</span>
              <span class="health-value">{{ healthData.version }}</span>
            </div>

            <!-- Node Version -->
            <div class="health-item">
              <span class="health-label">Node.js</span>
              <span class="health-value">{{ healthData.nodeVersion }}</span>
            </div>

            <!-- Environment -->
            <div class="health-item">
              <span class="health-label">Environment</span>
              <span class="badge badge-secondary">{{ healthData.environment }}</span>
            </div>

            <!-- Response Time -->
            <div class="health-item">
              <span class="health-label">Response Time</span>
              <span class="health-value">{{ healthData.responseTime }}ms</span>
            </div>

            <!-- Health Checks -->
            <div v-for="check in healthData.checks" :key="check.name" class="health-item">
              <span class="health-label">{{ check.name }}</span>
              <span :class="['badge', getStatusBadgeClass(check.status)]">
                {{ check.status }}
                <template v-if="check.latency"> ({{ check.latency }}ms)</template>
              </span>
              <span v-if="check.details?.percentage" class="text-sm text-secondary ml-2">
                {{ check.details.percentage }}%
              </span>
            </div>
          </div>
        </div>
      </section>

      <!-- External Monitoring -->
      <section v-if="uptimeKumaUrl" class="card mb-6">
        <div class="card-header">
          <h2 class="card-title">External Monitoring</h2>
        </div>
        <div class="card-body">
          <a :href="uptimeKumaUrl" target="_blank" rel="noopener" class="btn btn-outline">
            <IconExternalLink />
            Open Uptime Kuma
          </a>
        </div>
      </section>

      <!-- Recent Logs -->
      <section class="card mb-6">
        <div class="card-header">
          <h2 class="card-title">Recent Logs</h2>
          <div class="flex gap-sm align-center">
            <select v-model="logLevel" class="input input-sm">
              <option value="all">All Levels</option>
              <option value="error">Errors Only</option>
              <option value="warn">Warnings+</option>
              <option value="info">Info+</option>
            </select>
            <button class="btn btn-sm btn-ghost" title="Copy logs" @click="copyLogs">
              <IconCopy />
            </button>
          </div>
        </div>
        <div class="card-body log-container">
          <div v-if="logsPending" class="loading-state">
            <p>Loading...</p>
          </div>
          <div v-else-if="logs.length === 0" class="empty-state">
            <p>No logs available</p>
            <p class="text-sm">Logs are captured in production mode only</p>
          </div>
          <div v-else class="log-list">
            <div v-for="(log, index) in logs" :key="index" class="log-entry">
              <span class="log-time">{{ formatTime(log.time) }}</span>
              <span :class="['badge badge-sm', getLogLevelClass(log.level)]">
                {{ log.levelLabel }}
              </span>
              <span class="log-message">{{ log.msg }}</span>
            </div>
          </div>
        </div>
      </section>

      <!-- Audit Trail -->
      <section class="card">
        <div class="card-header">
          <h2 class="card-title">Audit Trail</h2>
          <span class="text-sm text-secondary">
            {{ auditData?.meta?.total || 0 }} total events
          </span>
        </div>
        <div v-if="auditPending" class="card-body">
          <div class="loading-state">
            <p>Loading...</p>
          </div>
        </div>
        <div v-else-if="auditLogs.length === 0" class="card-body">
          <div class="empty-state">
            <p>No audit events recorded</p>
          </div>
        </div>
        <table v-else class="data-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>Action</th>
              <th>User</th>
              <th>IP Address</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="log in auditLogs" :key="log.id">
              <td data-label="Time">{{ formatDate(log.createdAt) }}</td>
              <td data-label="Action">
                <code>{{ log.action }}</code>
              </td>
              <td data-label="User">
                {{ log.actorName || log.actorEmail || 'System' }}
              </td>
              <td data-label="IP">
                <code v-if="log.ipAddress">{{ log.ipAddress }}</code>
                <span v-else class="text-secondary">-</span>
              </td>
              <td data-label="Status">
                <span :class="['badge', log.success ? 'badge-success' : 'badge-error']">
                  {{ log.success ? 'Success' : 'Failed' }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </section>
    </div>
  </div>
</template>
