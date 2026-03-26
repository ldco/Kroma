/**
 * Storybook Main Configuration
 *
 * Configures Storybook for Vue 3 + Vite with Nuxt compatibility.
 * Includes addons for accessibility, interactions, and essential tools.
 */
import type { StorybookConfig } from '@storybook/vue3-vite'
import { mergeConfig } from 'vite'
import path from 'path'

const config: StorybookConfig = {
  stories: [
    '../app/components/**/*.stories.@(js|jsx|ts|tsx|mdx)',
    '../stories/**/*.stories.@(js|jsx|ts|tsx|mdx)'
  ],

  addons: [
    '@storybook/addon-essentials',
    '@storybook/addon-interactions',
    '@storybook/addon-a11y'
  ],

  framework: {
    name: '@storybook/vue3-vite',
    options: {}
  },

  docs: {
    autodocs: 'tag'
  },

  core: {
    disableTelemetry: true
  },

  async viteFinal(config) {
    return mergeConfig(config, {
      resolve: {
        alias: {
          '~': path.resolve(__dirname, '../app'),
          '~~': path.resolve(__dirname, '..'),
          '@': path.resolve(__dirname, '../app'),
          '~icons': path.resolve(__dirname, '../node_modules/@iconify/json')
        }
      },
      css: {
        preprocessorOptions: {}
      }
    })
  }
}

export default config
