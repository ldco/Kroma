/**
 * Storybook Preview Configuration
 *
 * Global decorators, parameters, and setup for all stories.
 * Imports the main CSS and provides theme switching.
 */
import type { Preview } from '@storybook/vue3'
import { setup } from '@storybook/vue3'
import { createPinia } from 'pinia'

// Import global styles
import '../app/assets/css/main.css'

// Setup Pinia for state management
const pinia = createPinia()
setup(app => {
  app.use(pinia)
})

const preview: Preview = {
  parameters: {
    actions: { argTypesRegex: '^on[A-Z].*' },

    controls: {
      expanded: true,
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i
      }
    },

    backgrounds: {
      default: 'light',
      values: [
        { name: 'light', value: '#ffffff' },
        { name: 'dark', value: '#1a1a1a' }
      ]
    },

    viewport: {
      viewports: {
        mobile: {
          name: 'Mobile',
          styles: { width: '375px', height: '667px' }
        },
        tablet: {
          name: 'Tablet',
          styles: { width: '768px', height: '1024px' }
        },
        desktop: {
          name: 'Desktop',
          styles: { width: '1280px', height: '800px' }
        }
      }
    },

    a11y: {
      element: '#storybook-root',
      config: {},
      options: {},
      manual: false
    }
  },

  globalTypes: {
    theme: {
      name: 'Theme',
      description: 'Global theme for components',
      defaultValue: 'light',
      toolbar: {
        icon: 'circlehollow',
        items: [
          { value: 'light', icon: 'sun', title: 'Light' },
          { value: 'dark', icon: 'moon', title: 'Dark' }
        ],
        showName: true
      }
    },

    locale: {
      name: 'Locale',
      description: 'Internationalization locale',
      defaultValue: 'en',
      toolbar: {
        icon: 'globe',
        items: [
          { value: 'en', title: 'English' },
          { value: 'fr', title: 'Français' },
          { value: 'de', title: 'Deutsch' }
        ],
        showName: true
      }
    }
  },

  decorators: [
    (story, context) => {
      const theme = context.globals.theme || 'light'

      return {
        components: { story },
        template: `
          <div :class="['storybook-wrapper', themeClass]" :data-theme="theme">
            <story />
          </div>
        `,
        data() {
          return {
            theme,
            themeClass: theme === 'dark' ? 'dark-mode' : 'light-mode'
          }
        }
      }
    }
  ]
}

export default preview
