import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
import typescript from '@typescript-eslint/eslint-plugin'
import typescriptParser from '@typescript-eslint/parser'
import vueParser from 'vue-eslint-parser'
import ssrSafety from './eslint-plugin-ssr-safety.js'

export default [
  // Base JavaScript rules
  js.configs.recommended,

  // Ignore patterns
  {
    ignores: [
      '**/node_modules/**',
      '**/.nuxt/**',
      '**/.output/**',
      '**/dist/**',
      '**/coverage/**',
      '**/data/**',
      'public/**'
    ]
  },

  // TypeScript files
  {
    files: ['**/*.ts'],
    languageOptions: {
      parser: typescriptParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module'
      },
      globals: {
        // Node.js globals
        process: 'readonly',
        console: 'readonly',
        Buffer: 'readonly',
        __dirname: 'readonly',
        __filename: 'readonly',
        // Nuxt auto-imports
        defineEventHandler: 'readonly',
        getQuery: 'readonly',
        readBody: 'readonly',
        createError: 'readonly',
        getCookie: 'readonly',
        setCookie: 'readonly',
        deleteCookie: 'readonly',
        getHeader: 'readonly',
        setHeader: 'readonly',
        useRuntimeConfig: 'readonly',
        defineNuxtConfig: 'readonly',
        defineNuxtPlugin: 'readonly',
        defineNuxtRouteMiddleware: 'readonly',
        defineNitroPlugin: 'readonly',
        useDatabase: 'readonly'
      }
    },
    plugins: {
      '@typescript-eslint': typescript
    },
    rules: {
      // TypeScript specific rules
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
      'no-unused-vars': 'off', // Use TypeScript version instead

      // General rules
      'no-console': 'off',
      'no-undef': 'off' // TypeScript handles this
    }
  },

  // Vue files
  {
    files: ['**/*.vue'],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: typescriptParser,
        ecmaVersion: 'latest',
        sourceType: 'module',
        extraFileExtensions: ['.vue']
      },
      globals: {
        // Vue/Nuxt auto-imports
        ref: 'readonly',
        reactive: 'readonly',
        computed: 'readonly',
        watch: 'readonly',
        watchEffect: 'readonly',
        onMounted: 'readonly',
        onUnmounted: 'readonly',
        onBeforeMount: 'readonly',
        onBeforeUnmount: 'readonly',
        defineProps: 'readonly',
        defineEmits: 'readonly',
        defineExpose: 'readonly',
        withDefaults: 'readonly',
        useRoute: 'readonly',
        useRouter: 'readonly',
        useState: 'readonly',
        useFetch: 'readonly',
        useAsyncData: 'readonly',
        useLazyFetch: 'readonly',
        useHead: 'readonly',
        useSeoMeta: 'readonly',
        useRuntimeConfig: 'readonly',
        useI18n: 'readonly',
        useLocalePath: 'readonly',
        useLocaleHead: 'readonly',
        navigateTo: 'readonly',
        createError: 'readonly',
        showError: 'readonly',
        clearError: 'readonly',
        useSafeConfig: 'readonly',
        useHydration: 'readonly'
      }
    },
    plugins: {
      vue,
      'ssr-safety': ssrSafety
    },
    rules: {
      // Vue specific rules
      ...vue.configs['flat/recommended'].rules,
      'vue/multi-word-component-names': 'off', // Nuxt pages don't need multi-word names
      'vue/no-v-html': 'warn', // Warn but allow (needed for some use cases)
      'vue/require-default-prop': 'off', // TypeScript handles this
      'vue/no-setup-props-destructure': 'off', // Vue 3.3+ supports this

      // SSR Safety rules
      'ssr-safety/no-direct-config-access': 'warn',

      // General rules
      'no-unused-vars': 'off',
      'no-undef': 'off'
    }
  },

  // JavaScript files
  {
    files: ['**/*.js', '**/*.mjs'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: {
        process: 'readonly',
        console: 'readonly'
      }
    },
    rules: {
      'no-unused-vars': ['warn', { argsIgnorePattern: '^_' }]
    }
  }
]
