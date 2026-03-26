/**
 * Unread Messages Count Composable
 *
 * Shared state for unread message count across admin panel.
 * Can be refreshed from anywhere (e.g., when marking message as read).
 */

const unreadCount = ref(0)
const isLoading = ref(false)

async function fetchUnreadCount() {
  if (isLoading.value) return

  isLoading.value = true
  try {
    const data = await $fetch<{ unreadMessages: number }>('/api/admin/stats')
    unreadCount.value = data.unreadMessages
  } catch {
    // Silently fail - badge just won't show
  } finally {
    isLoading.value = false
  }
}

export function useUnreadCount() {
  return {
    unreadCount: readonly(unreadCount),
    fetchUnreadCount,
    // Decrement locally without refetching (optimistic update)
    decrementUnread: () => {
      if (unreadCount.value > 0) {
        unreadCount.value--
      }
    },
    // Increment locally (when new message arrives via SSE/polling)
    incrementUnread: () => {
      unreadCount.value++
    }
  }
}
