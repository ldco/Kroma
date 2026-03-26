/**
 * EmptyState Component Tests
 *
 * Tests for the EmptyState molecule component.
 * Displays placeholder content when lists or sections are empty.
 */
import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import EmptyState from '~/components/molecules/EmptyState.vue'

describe('EmptyState', () => {
  // ─────────────────────────────────────────────────────────────────────────
  // Basic Rendering
  // ─────────────────────────────────────────────────────────────────────────

  it('renders with default props', async () => {
    const wrapper = await mountSuspended(EmptyState)

    expect(wrapper.find('.empty-state').exists()).toBe(true)
  })

  it('renders title when provided', async () => {
    const wrapper = await mountSuspended(EmptyState, {
      props: {
        title: 'No items found'
      }
    })

    expect(wrapper.text()).toContain('No items found')
  })

  it('renders description when provided', async () => {
    const wrapper = await mountSuspended(EmptyState, {
      props: {
        title: 'Empty',
        description: 'Start by adding your first item'
      }
    })

    expect(wrapper.text()).toContain('Start by adding your first item')
  })

  // ─────────────────────────────────────────────────────────────────────────
  // Illustration Variants
  // ─────────────────────────────────────────────────────────────────────────

  it('renders default illustration when no variant specified', async () => {
    const wrapper = await mountSuspended(EmptyState)

    // Should have illustration container
    expect(wrapper.find('.empty-state__illustration').exists()).toBe(true)
  })

  it('renders inbox variant illustration', async () => {
    const wrapper = await mountSuspended(EmptyState, {
      props: {
        variant: 'inbox',
        title: 'No messages'
      }
    })

    expect(wrapper.find('.empty-state').exists()).toBe(true)
  })

  it('renders search variant illustration', async () => {
    const wrapper = await mountSuspended(EmptyState, {
      props: {
        variant: 'search',
        title: 'No results'
      }
    })

    expect(wrapper.find('.empty-state').exists()).toBe(true)
  })

  // ─────────────────────────────────────────────────────────────────────────
  // Action Button (via actionLabel prop)
  // ─────────────────────────────────────────────────────────────────────────

  it('renders action button when actionLabel provided', async () => {
    const wrapper = await mountSuspended(EmptyState, {
      props: {
        title: 'No items',
        actionLabel: 'Add Item'
      }
    })

    expect(wrapper.find('.empty-state__action button').exists()).toBe(true)
    expect(wrapper.find('.empty-state__action button').text()).toBe('Add Item')
  })

  // ─────────────────────────────────────────────────────────────────────────
  // Variant Classes
  // ─────────────────────────────────────────────────────────────────────────

  it('applies compact class when variant is compact', async () => {
    const wrapper = await mountSuspended(EmptyState, {
      props: {
        variant: 'compact',
        title: 'Compact'
      }
    })

    expect(wrapper.find('.empty-state').classes()).toContain('empty-state--compact')
  })

  it('applies large class when variant is large', async () => {
    const wrapper = await mountSuspended(EmptyState, {
      props: {
        variant: 'large',
        title: 'Large'
      }
    })

    expect(wrapper.find('.empty-state').classes()).toContain('empty-state--large')
  })
})
