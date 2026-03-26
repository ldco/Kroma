/**
 * Pricing Card Stories
 *
 * Showcases the PricingCard component for different pricing tiers.
 */
import type { Meta, StoryObj } from '@storybook/vue3'
import PricingCard from './PricingCard.vue'

const mockFeatures = [
  { id: 1, text: '5 projects', included: true },
  { id: 2, text: '10GB storage', included: true },
  { id: 3, text: 'Email support', included: true },
  { id: 4, text: 'API access', included: false },
  { id: 5, text: 'Custom domain', included: false }
]

const mockProFeatures = [
  { id: 1, text: '25 projects', included: true },
  { id: 2, text: '100GB storage', included: true },
  { id: 3, text: 'Priority support', included: true },
  { id: 4, text: 'API access', included: true },
  { id: 5, text: 'Custom domain', included: true }
]

const mockEnterpriseFeatures = [
  { id: 1, text: 'Unlimited projects', included: true },
  { id: 2, text: 'Unlimited storage', included: true },
  { id: 3, text: 'Dedicated support', included: true },
  { id: 4, text: 'Full API access', included: true },
  { id: 5, text: 'Custom integrations', included: true },
  { id: 6, text: 'SLA guarantee', included: true }
]

const meta: Meta<typeof PricingCard> = {
  title: 'Molecules/PricingCard',
  component: PricingCard,
  tags: ['autodocs'],

  argTypes: {
    highlighted: {
      control: 'boolean',
      description: 'Highlight the card (additional to tier.featured)'
    },
    billingPeriod: {
      control: 'select',
      options: ['month', 'year'],
      description: 'Billing period for price calculation'
    },
    yearlyDiscount: {
      control: 'number',
      description: 'Discount percentage for yearly billing'
    },
    currency: {
      control: 'select',
      options: ['USD', 'EUR', 'GBP', 'RUB', 'ILS'],
      description: 'Currency override'
    }
  },

  args: {
    billingPeriod: 'month'
  }
}

export default meta
type Story = StoryObj<typeof PricingCard>

export const Free: Story = {
  args: {
    tier: {
      id: 1,
      slug: 'free',
      name: 'Free',
      description: 'Perfect for getting started',
      price: 0,
      currency: 'USD',
      period: 'month',
      featured: false,
      ctaText: null,
      ctaUrl: '/signup',
      features: mockFeatures
    }
  }
}

export const Basic: Story = {
  args: {
    tier: {
      id: 2,
      slug: 'basic',
      name: 'Basic',
      description: 'For individuals and small teams',
      price: 19,
      currency: 'USD',
      period: 'month',
      featured: false,
      ctaText: 'Start Free Trial',
      ctaUrl: '/signup?plan=basic',
      features: mockFeatures
    }
  }
}

export const Pro: Story = {
  args: {
    tier: {
      id: 3,
      slug: 'pro',
      name: 'Pro',
      description: 'For growing businesses',
      price: 49,
      currency: 'USD',
      period: 'month',
      featured: true,
      ctaText: null,
      ctaUrl: '/signup?plan=pro',
      features: mockProFeatures
    }
  }
}

export const Enterprise: Story = {
  args: {
    tier: {
      id: 4,
      slug: 'enterprise',
      name: 'Enterprise',
      description: 'For large organizations',
      price: null,
      currency: 'USD',
      period: null,
      featured: false,
      ctaText: 'Contact Sales',
      ctaUrl: '/contact',
      features: mockEnterpriseFeatures
    }
  }
}

export const HighlightedNonFeatured: Story = {
  args: {
    tier: {
      id: 2,
      slug: 'basic',
      name: 'Basic',
      description: 'Recommended for your needs',
      price: 19,
      currency: 'USD',
      period: 'month',
      featured: false,
      ctaText: null,
      ctaUrl: '/signup?plan=basic',
      features: mockFeatures
    },
    highlighted: true
  }
}

export const YearlyBilling: Story = {
  args: {
    tier: {
      id: 3,
      slug: 'pro',
      name: 'Pro',
      description: 'Save 20% with annual billing',
      price: 49,
      currency: 'USD',
      period: 'month',
      featured: true,
      ctaText: null,
      ctaUrl: '/signup?plan=pro&billing=yearly',
      features: mockProFeatures
    },
    billingPeriod: 'year',
    yearlyDiscount: 20
  }
}

export const EuroCurrency: Story = {
  args: {
    tier: {
      id: 3,
      slug: 'pro',
      name: 'Pro',
      description: 'For European customers',
      price: 45,
      currency: 'EUR',
      period: 'month',
      featured: true,
      ctaText: null,
      ctaUrl: '/signup?plan=pro',
      features: mockProFeatures
    },
    currency: 'EUR'
  }
}

export const AllTiers: Story = {
  render: () => ({
    components: { PricingCard },
    setup() {
      const tiers = [
        {
          id: 1,
          slug: 'free',
          name: 'Free',
          description: 'Perfect for getting started',
          price: 0,
          currency: 'USD',
          period: 'month' as const,
          featured: false,
          ctaText: 'Get Started',
          ctaUrl: '/signup',
          features: mockFeatures
        },
        {
          id: 2,
          slug: 'pro',
          name: 'Pro',
          description: 'For growing businesses',
          price: 49,
          currency: 'USD',
          period: 'month' as const,
          featured: true,
          ctaText: null,
          ctaUrl: '/signup?plan=pro',
          features: mockProFeatures
        },
        {
          id: 3,
          slug: 'enterprise',
          name: 'Enterprise',
          description: 'For large organizations',
          price: null,
          currency: 'USD',
          period: null,
          featured: false,
          ctaText: 'Contact Sales',
          ctaUrl: '/contact',
          features: mockEnterpriseFeatures
        }
      ]
      return { tiers }
    },
    template: `
      <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 2rem; max-width: 900px;">
        <PricingCard v-for="tier in tiers" :key="tier.id" :tier="tier" />
      </div>
    `
  })
}
