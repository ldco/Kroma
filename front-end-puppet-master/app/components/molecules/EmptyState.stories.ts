/**
 * Empty State Stories
 *
 * Showcases the EmptyState component for different scenarios.
 */
import type { Meta, StoryObj } from '@storybook/vue3'
import { fn } from '@storybook/test'
import EmptyState from './EmptyState.vue'

const meta: Meta<typeof EmptyState> = {
  title: 'Molecules/EmptyState',
  component: EmptyState,
  tags: ['autodocs'],

  argTypes: {
    title: {
      control: 'text',
      description: 'Main heading text'
    },
    description: {
      control: 'text',
      description: 'Supporting description text'
    },
    actionLabel: {
      control: 'text',
      description: 'Action button label'
    },
    actionHref: {
      control: 'text',
      description: 'Action button href (makes it a link)'
    },
    variant: {
      control: 'select',
      options: ['default', 'compact', 'large'],
      description: 'Size variant'
    },
    illustration: {
      control: 'select',
      options: ['empty-box', 'search', 'error', 'success', 'custom'],
      description: 'Built-in illustration type'
    }
  },

  args: {
    title: 'No items found',
    onAction: fn()
  }
}

export default meta
type Story = StoryObj<typeof EmptyState>

export const Default: Story = {
  args: {
    title: 'No items yet',
    description: 'Create your first item to get started.',
    actionLabel: 'Create Item'
  }
}

export const SearchNoResults: Story = {
  args: {
    title: 'No results found',
    description: 'Try adjusting your search or filter to find what you\'re looking for.',
    illustration: 'search',
    actionLabel: 'Clear Filters'
  }
}

export const ErrorState: Story = {
  args: {
    title: 'Something went wrong',
    description: 'We couldn\'t load the content. Please try again.',
    illustration: 'error',
    actionLabel: 'Retry'
  }
}

export const SuccessState: Story = {
  args: {
    title: 'All caught up!',
    description: 'You\'ve completed all your tasks for today.',
    illustration: 'success'
  }
}

export const CompactVariant: Story = {
  args: {
    title: 'No messages',
    description: 'Your inbox is empty.',
    variant: 'compact',
    illustration: 'empty-box'
  }
}

export const LargeVariant: Story = {
  args: {
    title: 'Welcome to Your Dashboard',
    description: 'This is where you\'ll see your activity, notifications, and updates. Get started by exploring the menu.',
    variant: 'large',
    actionLabel: 'Take a Tour'
  }
}

export const WithLinkAction: Story = {
  args: {
    title: 'Need help?',
    description: 'Check out our documentation for guides and tutorials.',
    illustration: 'empty-box',
    actionLabel: 'View Documentation',
    actionHref: '/docs'
  }
}

export const NoAction: Story = {
  args: {
    title: 'No notifications',
    description: 'You\'re all caught up! Check back later for updates.'
  }
}

export const AllIllustrations: Story = {
  render: () => ({
    components: { EmptyState },
    template: `
      <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 2rem;">
        <EmptyState
          title="Empty Box"
          description="Default empty state illustration"
          illustration="empty-box"
        />
        <EmptyState
          title="Search"
          description="Search-related empty state"
          illustration="search"
        />
        <EmptyState
          title="Error"
          description="Error state illustration"
          illustration="error"
        />
        <EmptyState
          title="Success"
          description="Success state illustration"
          illustration="success"
        />
      </div>
    `
  })
}

export const InboxExample: Story = {
  args: {
    title: 'Your inbox is empty',
    description: 'Messages from customers and team members will appear here when they contact you.',
    illustration: 'empty-box',
    actionLabel: 'Send a Message'
  }
}

export const ProductsExample: Story = {
  args: {
    title: 'No products added',
    description: 'Add your first product to start selling. You can import products in bulk or add them one by one.',
    illustration: 'empty-box',
    actionLabel: 'Add Product'
  }
}
