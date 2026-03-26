/**
 * ESLint Plugin for Puppet Master SSR Safety
 *
 * Custom rules to enforce SSR-safe patterns in Vue components.
 */

// Rule: Warn on direct config access in template or non-computed script
// Allows import if config is only accessed through computed() wrappers or for method calls
const noDirectConfigAccess = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow direct access to config object in Vue components (use useSafeConfig instead)',
      category: 'SSR Safety',
      recommended: 'warn',
      url: 'https://github.com/ldco/puppet-master/docs/ssr-safety.md'
    },
    schema: [],
    messages: {
      directAccess:
        'Direct config access can cause SSR hydration errors. Use `useSafeConfig()` composable instead. Example: `const { isOnepager } = useSafeConfig()`'
    }
  },
  create(context) {
    const filename = context.getFilename()

    // Skip non-Vue files (composables, plugins, middleware, types, utils)
    // These files legitimately need direct config access
    const skipPatterns = [
      /\/composables\//,
      /\/plugins\//,
      /\/middleware\//,
      /\/types\//,
      /\/utils\//,
      /\/server\//,
      /\.config\.ts$/,
      /drizzle\.config\.ts$/,
      /nuxt\.config\.ts$/
    ]

    if (skipPatterns.some(pattern => pattern.test(filename))) {
      return {}
    }

    // Only apply to Vue files
    if (!filename.endsWith('.vue')) {
      return {}
    }

    let configVarName = null
    let unsafeAccesses = []

    return {
      // Check for import from '~/puppet-master.config'
      ImportDeclaration(node) {
        if (node.source.value === '~/puppet-master.config') {
          const configSpecifier = node.specifiers.find(
            s => s.type === 'ImportDefaultSpecifier'
          )
          if (configSpecifier) {
            configVarName = configSpecifier.local.name
          }
        }
      },

      // Check for direct config property access (config.xxx)
      MemberExpression(node) {
        if (!configVarName) return

        // Check if this is config.xxx access
        if (node.object.type === 'Identifier' && node.object.name === configVarName) {
          const propertyName = node.property.name

          // Skip method calls - these are safe (e.g., config.getAdminSectionsForRole())
          if (node.parent.type === 'CallExpression' && node.parent.callee === node) {
            return
          }

          // Skip config.settings, config.sections, config.locales, config.admin, config.logo
          // when used inside computed() - check parent chain
          let current = node.parent
          let insideComputed = false
          let insideVariableDeclaratorInit = false

          while (current) {
            // Check for computed(() => ...) wrapper
            if (
              current.type === 'CallExpression' &&
              current.callee.type === 'Identifier' &&
              current.callee.name === 'computed'
            ) {
              insideComputed = true
              break
            }
            // Check if we're in the init part of a variable declarator (const x = computed(...))
            if (
              current.type === 'VariableDeclarator' &&
              current.init &&
              current.init.type === 'CallExpression' &&
              current.init.callee.type === 'Identifier' &&
              current.init.callee.name === 'computed'
            ) {
              insideVariableDeclaratorInit = true
              // Continue checking - we need to see if this is the init of a computed
            }
            current = current.parent
          }

          // Safe if inside computed()
          if (insideComputed) {
            return
          }

          // Track as potentially unsafe
          unsafeAccesses.push({
            node,
            propertyName,
            line: node.loc.start.line
          })
        }
      },

      // Report at Program exit if unsafe access was found
      'Program:exit'() {
        if (unsafeAccesses.length > 0 && configVarName) {
          // Only report if there are unsafe accesses NOT inside computed
          context.report({
            loc: { line: 1, column: 0 },
            messageId: 'directAccess'
          })
        }
      }
    }
  }
}

export default {
  rules: {
    'no-direct-config-access': noDirectConfigAccess
  }
}
