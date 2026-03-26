/**
 * CTA Button Stories
 *
 * Showcases the CtaButton component variants, sizes, and states.
 */
import type { Meta, StoryObj } from '@storybook/vue3'
import CtaButton from './CtaButton.vue'

const meta: Meta<typeof CtaButton> = {
  title: 'Atoms/CtaButton',
  component: CtaButton,
  tags: ['autodocs'],

  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'secondary', 'ghost', 'outline'],
      description: 'Button visual variant'
    },
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Button size'
    },
    disabled: {
      control: 'boolean',
      description: 'Disabled state'
    },
    fullWidth: {
      control: 'boolean',
      description: 'Full width button'
    },
    to: {
      control: 'text',
      description: 'Internal route or anchor'
    },
    href: {
      control: 'text',
      description: 'External link URL'
    },
    type: {
      control: 'select',
      options: ['button', 'submit', 'reset'],
      description: 'Button type for forms'
    }
  },

  args: {
    variant: 'primary',
    disabled: false,
    fullWidth: false
  }
}

export default meta
type Story = StoryObj<typeof CtaButton>

export const Primary: Story = {
  args: {
    variant: 'primary'
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">Get Started</CtaButton>'
  })
}

export const Secondary: Story = {
  args: {
    variant: 'secondary'
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">Learn More</CtaButton>'
  })
}

export const Ghost: Story = {
  args: {
    variant: 'ghost'
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">Cancel</CtaButton>'
  })
}

export const Outline: Story = {
  args: {
    variant: 'outline'
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">View Details</CtaButton>'
  })
}

export const Small: Story = {
  args: {
    variant: 'primary',
    size: 'sm'
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">Small Button</CtaButton>'
  })
}

export const Large: Story = {
  args: {
    variant: 'primary',
    size: 'lg'
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">Large Button</CtaButton>'
  })
}

export const Disabled: Story = {
  args: {
    variant: 'primary',
    disabled: true
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">Disabled</CtaButton>'
  })
}

export const FullWidth: Story = {
  args: {
    variant: 'primary',
    fullWidth: true
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">Full Width Button</CtaButton>'
  })
}

export const ExternalLink: Story = {
  args: {
    variant: 'primary',
    href: 'https://example.com'
  },
  render: args => ({
    components: { CtaButton },
    setup() {
      return { args }
    },
    template: '<CtaButton v-bind="args">External Link ↗</CtaButton>'
  })
}

export const AllVariants: Story = {
  render: () => ({
    components: { CtaButton },
    template: `
      <div style="display: flex; gap: 1rem; flex-wrap: wrap; align-items: center;">
        <CtaButton variant="primary">Primary</CtaButton>
        <CtaButton variant="secondary">Secondary</CtaButton>
        <CtaButton variant="outline">Outline</CtaButton>
        <CtaButton variant="ghost">Ghost</CtaButton>
      </div>
    `
  })
}

export const AllSizes: Story = {
  render: () => ({
    components: { CtaButton },
    template: `
      <div style="display: flex; gap: 1rem; flex-wrap: wrap; align-items: center;">
        <CtaButton size="sm">Small</CtaButton>
        <CtaButton>Default</CtaButton>
        <CtaButton size="lg">Large</CtaButton>
      </div>
    `
  })
}
