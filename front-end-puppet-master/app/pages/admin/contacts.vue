<script setup lang="ts">
/**
 * Admin Contacts Page
 *
 * View and manage contact form submissions.
 * Uses global CSS: .card, .badge, .btn, .flex, .grid utilities
 */
import IconTrash from '~icons/tabler/trash'
import IconMail from '~icons/tabler/mail'
import IconMailOpened from '~icons/tabler/mail-opened'
import type { ContactSubmission } from '~/types'

definePageMeta({
  layout: 'admin',
  middleware: 'auth',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('admin.navContacts')} | Admin`
})

const { confirm } = useConfirm()
const { toast } = useToast()

// Shared unread count for nav badge
const { decrementUnread, incrementUnread, fetchUnreadCount } = useUnreadCount()

// Fetch contacts - pass cookies for SSR auth
const headers = useRequestHeaders(['cookie'])
const { data, pending, refresh } = await useFetch<{
  items: ContactSubmission[]
  unreadCount: number
}>('/api/admin/contacts', {
  headers
})

const items = computed(() => data.value?.items || [])
const localUnreadCount = computed(() => data.value?.unreadCount || 0)

// Selected message for detail view
const selectedMessage = ref<ContactSubmission | null>(null)

async function toggleRead(item: ContactSubmission) {
  const wasUnread = !item.read
  const itemId = item.id
  try {
    await $fetch(`/api/admin/contacts/${item.id}`, {
      method: 'PUT',
      body: { read: !item.read }
    })
    // Update nav badge immediately (optimistic)
    if (wasUnread) {
      decrementUnread()
    } else {
      incrementUnread()
    }
    await refresh()
    // Update selectedMessage to point to the refreshed item
    if (selectedMessage.value?.id === itemId) {
      selectedMessage.value = items.value.find(i => i.id === itemId) || null
    }
  } catch (e: unknown) {
    // Revert on error
    fetchUnreadCount()
    const errorMsg = e instanceof Error ? e.message : 'Failed to update'
    toast.error(errorMsg)
  }
}

const deleting = ref<number | null>(null)

async function deleteMessage(item: ContactSubmission) {
  const confirmed = await confirm(t('admin.confirmDelete'), {
    title: t('common.delete'),
    confirmText: t('common.delete'),
    variant: 'danger'
  })
  if (!confirmed) return

  const wasUnread = !item.read
  deleting.value = item.id
  try {
    await $fetch(`/api/admin/contacts/${item.id}`, { method: 'DELETE' })
    // Update nav badge if deleted message was unread
    if (wasUnread) {
      decrementUnread()
    }
    if (selectedMessage.value?.id === item.id) {
      selectedMessage.value = null
    }
    await refresh()
  } catch (e: unknown) {
    fetchUnreadCount() // Revert on error
    const errorMsg = e instanceof Error ? e.message : 'Failed to delete'
    toast.error(errorMsg)
  } finally {
    deleting.value = null
  }
}

function selectMessage(item: ContactSubmission) {
  // Toggle: click same message again to close
  if (selectedMessage.value?.id === item.id) {
    selectedMessage.value = null
    return
  }
  selectedMessage.value = item
  // Mark as read when opening
  if (!item.read) {
    toggleRead(item)
  }
}

function formatDate(timestamp: string | number | Date | null | undefined) {
  if (!timestamp) return ''
  return new Date(timestamp).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<template>
  <div class="admin-contacts">
    <!-- Header -->
    <div class="page-header">
      <h1 class="page-title">
        {{ t('admin.contacts') }}
        <span v-if="localUnreadCount" class="badge badge-primary">{{ localUnreadCount }}</span>
      </h1>
    </div>

    <!-- Loading -->
    <div v-if="pending" class="loading-state">{{ t('common.loading') }}</div>

    <!-- Empty state -->
    <div v-else-if="!items.length" class="empty-state">
      <IconMail class="icon-xl" />
      <p>{{ t('admin.noMessages') }}</p>
    </div>

    <!-- Messages: Desktop = side-by-side cards, Mobile = accordion cards -->
    <div v-else class="inbox-layout">
      <!-- Messages list - each item is a card -->
      <div class="inbox-list">
        <div
          v-for="item in items"
          :key="item.id"
          class="card contact-item"
          :class="{
            'is-selected': selectedMessage?.id === item.id,
            'is-unread': !item.read
          }"
        >
          <!-- Message preview (always visible) -->
          <button type="button" class="contact-preview" @click="selectMessage(item)">
            <div class="contact-preview-header">
              <span class="contact-name">{{ item.name }}</span>
              <span class="contact-date">{{ formatDate(item.createdAt) }}</span>
            </div>
            <div class="contact-subject">{{ item.subject || t('admin.noSubject') }}</div>
            <div class="contact-excerpt">
              {{ item.message.slice(0, 60) }}{{ item.message.length > 60 ? '...' : '' }}
            </div>
          </button>

          <!-- Expanded content (mobile only, inline accordion) -->
          <div v-if="selectedMessage?.id === item.id" class="contact-expanded">
            <div class="contact-expanded-actions">
              <button
                type="button"
                class="btn btn-icon btn-ghost"
                @click.stop="toggleRead(item)"
                :title="item.read ? t('admin.markUnread') : t('admin.markRead')"
              >
                <IconMailOpened v-if="item.read" />
                <IconMail v-else />
              </button>
              <button
                type="button"
                class="btn btn-icon btn-ghost btn-danger"
                @click.stop="deleteMessage(item)"
                :disabled="deleting === item.id"
                :title="t('common.delete')"
                :aria-label="t('common.delete')"
              >
                <IconTrash />
              </button>
            </div>
            <div class="contact-expanded-meta">
              <p>
                <strong>{{ t('admin.from') }}:</strong>
                {{ item.name }} &lt;{{ item.email }}&gt;
              </p>
              <p v-if="item.phone">
                <strong>{{ t('admin.phone') }}:</strong>
                {{ item.phone }}
              </p>
            </div>
            <div class="contact-expanded-body">{{ item.message }}</div>
          </div>
        </div>
      </div>

      <!-- Message detail panel (desktop only) - separate card -->
      <div class="card inbox-detail">
        <template v-if="selectedMessage">
          <div class="card-header">
            <h2>{{ selectedMessage.subject || t('admin.noSubject') }}</h2>
            <div class="flex gap-2">
              <button
                type="button"
                class="btn btn-icon btn-ghost"
                @click="toggleRead(selectedMessage)"
                :title="selectedMessage.read ? t('admin.markUnread') : t('admin.markRead')"
              >
                <IconMailOpened v-if="selectedMessage.read" />
                <IconMail v-else />
              </button>
              <button
                type="button"
                class="btn btn-icon btn-ghost btn-danger"
                @click="deleteMessage(selectedMessage)"
                :disabled="deleting === selectedMessage.id"
                :title="t('common.delete')"
                :aria-label="t('common.delete')"
              >
                <IconTrash />
              </button>
            </div>
          </div>
          <div class="inbox-detail-content">
            <div class="inbox-detail-meta">
              <p>
                <strong>{{ t('admin.from') }}:</strong>
                {{ selectedMessage.name }} &lt;{{ selectedMessage.email }}&gt;
              </p>
              <p v-if="selectedMessage.phone">
                <strong>{{ t('admin.phone') }}:</strong>
                {{ selectedMessage.phone }}
              </p>
              <p>
                <strong>{{ t('admin.date') }}:</strong>
                {{ formatDate(selectedMessage.createdAt) }}
              </p>
            </div>
            <div class="inbox-detail-body">{{ selectedMessage.message }}</div>
          </div>
        </template>
        <div v-else class="inbox-detail-empty">
          <IconMail class="icon-xl opacity-50" />
          <p>{{ t('admin.selectMessage') }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<!--
  Uses global CSS classes:
  - ui/content: .card, .card-body, .card-header, .badge
  - ui/buttons: .btn, .btn-icon, .btn-ghost
  - common/utilities: .flex, .grid, .gap-*, .items-center, .justify-between
  - typography: .text-*, .font-*, .truncate
-->
