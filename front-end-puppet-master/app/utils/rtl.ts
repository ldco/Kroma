/**
 * RTL Language Detection
 * As per BRIEF Section 12 - Internationalization
 */

const RTL_LANGUAGES = ['he', 'ar', 'fa', 'ur']

export function isRtlLanguage(locale: string): boolean {
  return RTL_LANGUAGES.includes(locale)
}
