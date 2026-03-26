/**
 * Module Access Composable
 *
 * Provides access to module configuration and state.
 * Uses the modules config from puppet-master.config.ts.
 */
import config from '~/puppet-master.config'
import {
  type ModuleId,
  type ModuleConfigMap,
  MODULE_REGISTRY,
  getModuleDefaults
} from '~/types/modules'

/**
 * Get module configuration and helpers
 */
export function useModule<T extends ModuleId>(moduleId: T) {
  const moduleConfig = config.modules?.[moduleId]
  const definition = MODULE_REGISTRY[moduleId]

  /**
   * Whether this module is enabled in config
   */
  const isEnabled = computed(() => moduleConfig?.enabled ?? false)

  /**
   * Whether this module is implemented (has code)
   */
  const isImplemented = computed(() => definition.implemented)

  /**
   * Whether this module is both enabled and implemented
   */
  const isAvailable = computed(() => isEnabled.value && isImplemented.value)

  /**
   * Get merged config (defaults + user config)
   */
  const mergedConfig = computed((): ModuleConfigMap[T] => {
    const defaults = getModuleDefaults(moduleId)
    const userConfig = moduleConfig?.config ?? {}
    return { ...defaults, ...userConfig } as ModuleConfigMap[T]
  })

  /**
   * Get a specific config value
   */
  function getConfig<K extends keyof ModuleConfigMap[T]>(
    key: K
  ): ModuleConfigMap[T][K] {
    return mergedConfig.value[key]
  }

  return {
    // State
    isEnabled,
    isImplemented,
    isAvailable,

    // Config
    config: mergedConfig,
    getConfig,

    // Definition
    definition,
    routes: definition.routes,
    adminSections: definition.adminSections
  }
}

/**
 * Get all enabled modules
 */
export function useEnabledModules() {
  const enabledModules = computed(() => {
    const modules = config.modules ?? {}
    return (Object.keys(modules) as ModuleId[]).filter(
      (id) => modules[id]?.enabled && MODULE_REGISTRY[id]?.implemented
    )
  })

  const enabledAdminSections = computed(() => {
    return enabledModules.value.flatMap((id) => MODULE_REGISTRY[id].adminSections)
  })

  return {
    enabledModules,
    enabledAdminSections
  }
}

/**
 * Check if a specific module is enabled
 */
export function isModuleEnabled(moduleId: ModuleId): boolean {
  return config.modules?.[moduleId]?.enabled ?? false
}
