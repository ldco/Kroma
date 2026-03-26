/**
 * System Translation Prefixes
 *
 * Keys starting with these prefixes are "system" translations:
 * - Editable only by master role in admin panel
 * - UI labels used throughout the app
 *
 * All other keys are "content" translations:
 * - Editable by admin+ roles
 * - Site-specific content (hero text, about, etc.)
 */

export const SYSTEM_PREFIXES = [
  'common.',
  'nav.',
  'auth.',
  'admin.',
  'theme.',
  'footer.',
  'validation.'
] as const

/**
 * Check if a translation key is a system key
 * System keys are only editable by master role
 */
export function isSystemKey(key: string): boolean {
  return SYSTEM_PREFIXES.some(prefix => key.startsWith(prefix))
}
