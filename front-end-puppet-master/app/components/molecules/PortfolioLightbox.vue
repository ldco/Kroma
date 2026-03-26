<script setup lang="ts">
/**
 * Portfolio Lightbox
 *
 * Fancy fullscreen lightbox for portfolio images/videos.
 * Features: smooth animations, keyboard navigation, swipe gestures
 */
import IconX from '~icons/tabler/x'
import IconChevronLeft from '~icons/tabler/chevron-left'
import IconChevronRight from '~icons/tabler/chevron-right'
import type { PortfolioItem } from '~/types'

const props = defineProps<{
  items: PortfolioItem[]
  currentIndex: number
  isOpen: boolean
}>()

const emit = defineEmits<{
  close: []
  navigate: [index: number]
}>()

const currentItem = computed(() => props.items[props.currentIndex])
const hasPrev = computed(() => props.currentIndex > 0)
const hasNext = computed(() => props.currentIndex < props.items.length - 1)

function prev() {
  if (hasPrev.value) emit('navigate', props.currentIndex - 1)
}

function next() {
  if (hasNext.value) emit('navigate', props.currentIndex + 1)
}

// Keyboard navigation
function handleKeydown(e: KeyboardEvent) {
  if (!props.isOpen) return
  if (e.key === 'Escape') emit('close')
  if (e.key === 'ArrowLeft') prev()
  if (e.key === 'ArrowRight') next()
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))

// Lock body scroll and add class when open
watch(
  () => props.isOpen,
  open => {
    document.body.style.overflow = open ? 'hidden' : ''
    if (open) {
      document.body.classList.add('lightbox-open')
    } else {
      document.body.classList.remove('lightbox-open')
    }
  }
)
</script>

<template>
  <Teleport to="body">
    <Transition name="lightbox">
      <div v-if="isOpen && currentItem" class="lightbox" @click.self="emit('close')">
        <!-- Close button -->
        <button type="button" class="lightbox-close" aria-label="Close" @click="emit('close')">
          <IconX />
        </button>

        <!-- Navigation arrows -->
        <button
          v-if="hasPrev"
          type="button"
          class="lightbox-nav lightbox-nav--prev"
          aria-label="Previous"
          @click="prev"
        >
          <IconChevronLeft />
        </button>

        <button
          v-if="hasNext"
          type="button"
          class="lightbox-nav lightbox-nav--next"
          aria-label="Next"
          @click="next"
        >
          <IconChevronRight />
        </button>

        <!-- Content -->
        <div class="lightbox-content">
          <Transition name="lightbox-slide" mode="out-in">
            <div :key="currentItem.id" class="lightbox-media">
              <img
                v-if="currentItem.mediaUrl"
                :src="currentItem.mediaUrl"
                :alt="currentItem.title || ''"
                class="lightbox-image"
              />
            </div>
          </Transition>

          <!-- Info panel -->
          <div class="lightbox-info">
            <span v-if="currentItem.category" class="lightbox-category">
              {{ currentItem.category }}
            </span>
            <h3 class="lightbox-title">{{ currentItem.title }}</h3>
            <p v-if="currentItem.description" class="lightbox-description">
              {{ currentItem.description }}
            </p>
            <span class="lightbox-counter">{{ currentIndex + 1 }} / {{ items.length }}</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<!--
  Uses global CSS classes from ui/overlays/index.css:
  - .lightbox, .lightbox-close, .lightbox-nav, .lightbox-content
  - .lightbox-media, .lightbox-image, .lightbox-info
  - .lightbox-category, .lightbox-title, .lightbox-description, .lightbox-counter
  - .lightbox-* animation classes
-->
