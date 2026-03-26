/**
 * Puppet Master Module Registry
 *
 * Re-exports from shared/modules.ts for backward compatibility.
 * The actual source of truth is now in /shared/modules.ts
 *
 * Used by: config-writer, init-cli
 */

// Re-export everything from the shared module
export {
  // Types
  type ModuleMetadata,
  type ModuleId,
  type LocaleInfo,
  // Module constants
  ALL_MODULES,
  MODULE_METADATA,
  // Module functions
  getModulesForWizard,
  isValidModuleId,
  filterValidModules,
  validateModuleIds,
  // Locale constants
  LOCALE_MAP,
  SUPPORTED_LOCALE_CODES,
  // Locale functions
  getLocalesForWizard,
  getLocaleInfo,
  parseLocales
} from '../../shared/modules'
