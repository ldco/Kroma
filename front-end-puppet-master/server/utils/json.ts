/**
 * Safe JSON Utilities
 *
 * Provides type-safe JSON parsing with proper error handling
 * to prevent application crashes from malformed JSON data.
 */

/**
 * Safely parse JSON string with fallback value
 *
 * @param value - JSON string to parse
 * @param fallback - Value to return if parsing fails
 * @returns Parsed value or fallback
 *
 * @example
 * const data = safeJsonParse(jsonString, [])
 * const config = safeJsonParse<Config>(configString, { enabled: false })
 */
export function safeJsonParse<T>(value: string | null | undefined, fallback: T): T {
  if (!value) return fallback

  try {
    return JSON.parse(value) as T
  } catch {
    return fallback
  }
}

/**
 * Safely parse JSON string, returning null on failure
 *
 * @param value - JSON string to parse
 * @returns Parsed value or null
 *
 * @example
 * const data = safeJsonParseOrNull<User[]>(jsonString)
 * if (data) { ... }
 */
export function safeJsonParseOrNull<T>(value: string | null | undefined): T | null {
  if (!value) return null

  try {
    return JSON.parse(value) as T
  } catch {
    return null
  }
}
