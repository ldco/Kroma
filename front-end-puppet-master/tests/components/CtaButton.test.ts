/**
 * CtaButton Component Tests
 *
 * Tests for the CTA Button atom component.
 * Uses @nuxt/test-utils for Nuxt-aware component testing.
 */
import { describe, it, expect } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import CtaButton from '~/components/atoms/CtaButton.vue'

describe('CtaButton', () => {
  // ─────────────────────────────────────────────────────────────────────────
  // Rendering Tests
  // ─────────────────────────────────────────────────────────────────────────

  it('renders as a button by default', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      slots: {
        default: 'Click me'
      }
    })

    expect(wrapper.find('button').exists()).toBe(true)
    expect(wrapper.text()).toBe('Click me')
  })

  it('renders as NuxtLink when "to" prop is provided', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: {
        to: '/about'
      },
      slots: {
        default: 'Go to About'
      }
    })

    // NuxtLink renders as <a> in test environment
    expect(wrapper.find('a').exists()).toBe(true)
    expect(wrapper.find('button').exists()).toBe(false)
  })

  it('renders as external link when "href" prop is provided', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: {
        href: 'https://example.com'
      },
      slots: {
        default: 'External Link'
      }
    })

    const link = wrapper.find('a')
    expect(link.exists()).toBe(true)
    expect(link.attributes('href')).toBe('https://example.com')
    expect(link.attributes('target')).toBe('_blank')
    expect(link.attributes('rel')).toBe('noopener noreferrer')
  })

  // ─────────────────────────────────────────────────────────────────────────
  // Variant Tests
  // ─────────────────────────────────────────────────────────────────────────

  it('applies primary variant by default', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      slots: { default: 'Button' }
    })

    expect(wrapper.find('button').classes()).toContain('btn')
    expect(wrapper.find('button').classes()).toContain('btn-primary')
  })

  it('applies secondary variant when specified', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: { variant: 'secondary' },
      slots: { default: 'Button' }
    })

    expect(wrapper.find('button').classes()).toContain('btn-secondary')
  })

  it('applies ghost variant when specified', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: { variant: 'ghost' },
      slots: { default: 'Button' }
    })

    expect(wrapper.find('button').classes()).toContain('btn-ghost')
  })

  it('applies outline variant when specified', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: { variant: 'outline' },
      slots: { default: 'Button' }
    })

    expect(wrapper.find('button').classes()).toContain('btn-outline')
  })

  // ─────────────────────────────────────────────────────────────────────────
  // Size Tests
  // ─────────────────────────────────────────────────────────────────────────

  it('applies size class when specified', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: { size: 'lg' },
      slots: { default: 'Large Button' }
    })

    expect(wrapper.find('button').classes()).toContain('btn-lg')
  })

  it('applies full width class when fullWidth is true', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: { fullWidth: true },
      slots: { default: 'Full Width' }
    })

    expect(wrapper.find('button').classes()).toContain('btn-full')
  })

  // ─────────────────────────────────────────────────────────────────────────
  // State Tests
  // ─────────────────────────────────────────────────────────────────────────

  it('applies disabled attribute when disabled', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: { disabled: true },
      slots: { default: 'Disabled' }
    })

    expect(wrapper.find('button').attributes('disabled')).toBeDefined()
  })

  it('uses correct button type', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      props: { type: 'submit' },
      slots: { default: 'Submit' }
    })

    expect(wrapper.find('button').attributes('type')).toBe('submit')
  })

  it('defaults to type="button"', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      slots: { default: 'Button' }
    })

    expect(wrapper.find('button').attributes('type')).toBe('button')
  })

  // ─────────────────────────────────────────────────────────────────────────
  // Event Tests
  // ─────────────────────────────────────────────────────────────────────────

  it('emits click event when clicked', async () => {
    const wrapper = await mountSuspended(CtaButton, {
      slots: { default: 'Click me' }
    })

    await wrapper.find('button').trigger('click')

    expect(wrapper.emitted('click')).toHaveLength(1)
    expect(wrapper.emitted('click')![0][0]).toBeInstanceOf(MouseEvent)
  })
})
