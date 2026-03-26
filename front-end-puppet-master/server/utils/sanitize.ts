/**
 * HTML Sanitization Utility
 *
 * Server-side HTML sanitization to prevent XSS attacks.
 * Supports two modes:
 * - sanitize-html library (recommended for production)
 * - Regex-based fallback (used when library not installed)
 *
 * To enable library-based sanitization:
 * npm install sanitize-html @types/sanitize-html
 *
 * Usage:
 *   import { sanitizeHtml, escapeHtml } from '../utils/sanitize'
 *   const cleanContent = sanitizeHtml(userContent)
 *   const safeText = escapeHtml(userText)
 */
import { logger } from './logger'

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

// Tags that are safe for rich text content
const ALLOWED_TAGS = [
  'p', 'br', 'hr',
  'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'ul', 'ol', 'li',
  'blockquote', 'pre', 'code',
  'strong', 'b', 'em', 'i', 'u', 's', 'strike', 'del', 'ins',
  'a', 'img',
  'table', 'thead', 'tbody', 'tr', 'th', 'td',
  'div', 'span',
  'figure', 'figcaption',
  'sup', 'sub', 'mark', 'small'
]

// Attributes that are safe (exported for use in advanced sanitization)
export const ALLOWED_ATTRIBUTES: Record<string, string[]> = {
  '*': ['class', 'id', 'title'],
  a: ['href', 'target', 'rel'],
  img: ['src', 'alt', 'width', 'height', 'loading'],
  td: ['colspan', 'rowspan'],
  th: ['colspan', 'rowspan', 'scope']
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIBRARY-BASED SANITIZATION (PREFERRED)
// ═══════════════════════════════════════════════════════════════════════════════

// Lazy-loaded sanitize-html library
let sanitizeHtmlLib: ((dirty: string, options: object) => string) | null = null
let libInitialized = false

/**
 * Initialize sanitize-html library if available
 */
async function initSanitizeLib(): Promise<boolean> {
  if (libInitialized) return sanitizeHtmlLib !== null

  libInitialized = true

  try {
    const lib = await import('sanitize-html').then(m => m.default).catch(() => null)

    if (!lib) {
      logger.debug('sanitize-html not installed - using regex-based sanitization')
      return false
    }

    sanitizeHtmlLib = lib
    logger.info('HTML sanitization using sanitize-html library')
    return true
  } catch {
    return false
  }
}

// Try to initialize on module load (non-blocking)
initSanitizeLib()

/**
 * Sanitize HTML using sanitize-html library
 */
function sanitizeWithLibrary(html: string): string {
  if (!sanitizeHtmlLib) return sanitizeWithRegex(html)

  return sanitizeHtmlLib(html, {
    allowedTags: ALLOWED_TAGS,
    allowedAttributes: {
      '*': ['class', 'id', 'title'],
      a: ['href', 'target', 'rel'],
      img: ['src', 'alt', 'width', 'height', 'loading']
    },
    allowedSchemes: ['http', 'https', 'mailto', 'tel'],
    allowedSchemesByTag: {
      img: ['http', 'https', 'data']
    },
    transformTags: {
      a: (tagName: string, attribs: Record<string, string>) => {
        // Add rel="noopener noreferrer" to external links
        if (attribs.target === '_blank') {
          attribs.rel = 'noopener noreferrer'
        }
        return { tagName, attribs }
      }
    }
  })
}

// ═══════════════════════════════════════════════════════════════════════════════
// REGEX-BASED SANITIZATION (FALLBACK)
// ═══════════════════════════════════════════════════════════════════════════════

// Tags that should be completely removed (including content)
const DANGEROUS_TAGS = [
  'script', 'style', 'iframe', 'frame', 'frameset',
  'object', 'embed', 'applet',
  'form', 'input', 'button', 'select', 'textarea',
  'meta', 'link', 'base', 'noscript', 'template'
]

// Event handler attributes (on*) including quoted and unquoted values
const EVENT_HANDLER_ATTR_REGEX = /\s+on[a-z0-9_-]+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)/gi
const EVENT_HANDLER_DETECT_REGEX = /\bon[a-z0-9_-]+\s*=/i

// JavaScript/data URL pattern
const DANGEROUS_URL_REGEX = /^\s*(javascript|data|vbscript):/i

/**
 * Sanitize HTML using regex (fallback when library not available)
 */
function sanitizeWithRegex(html: string): string {
  let result = html

  // 1. Remove dangerous tags with their content
  for (const tag of DANGEROUS_TAGS) {
    const regex = new RegExp(`<${tag}[^>]*>[\\s\\S]*?<\\/${tag}>`, 'gi')
    result = result.replace(regex, '')
    const selfClosing = new RegExp(`<${tag}[^>]*\\/?>`, 'gi')
    result = result.replace(selfClosing, '')
  }

  // 2. Remove event handlers from all tags
  result = result.replace(EVENT_HANDLER_ATTR_REGEX, '')

  // 3. Sanitize href and src attributes
  result = result.replace(/(href|src)\s*=\s*["']([^"']*)["']/gi, (match, attr, url) => {
    if (DANGEROUS_URL_REGEX.test(url)) {
      return `${attr}="#"`
    }
    return match
  })

  // 4. Remove expression() in style attributes (IE CSS expression attack)
  result = result.replace(/style\s*=\s*["'][^"']*expression\s*\([^"']*["']/gi, 'style=""')

  // 5. Remove data-* attributes that could contain encoded scripts
  result = result.replace(/\s+data-[a-z-]+\s*=\s*["'][^"']*["']/gi, '')

  // 6. Remove any remaining disallowed tags (but keep content)
  const allTags = result.match(/<\/?([a-z][a-z0-9]*)[^>]*>/gi) || []
  for (const tagMatch of allTags) {
    const tagName = tagMatch.match(/<\/?([a-z][a-z0-9]*)/i)?.[1]?.toLowerCase()
    if (tagName && !ALLOWED_TAGS.includes(tagName)) {
      result = result.replace(tagMatch, '')
    }
  }

  // 7. Clean up extra whitespace from removals
  result = result.replace(/\n\s*\n\s*\n/g, '\n\n')

  return result.trim()
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Sanitize HTML content to prevent XSS
 * Uses sanitize-html library if available, regex fallback otherwise
 */
export function sanitizeHtml(html: string | null | undefined): string {
  if (!html) return ''

  // Use library if available, otherwise regex fallback
  if (sanitizeHtmlLib) {
    return sanitizeWithLibrary(html)
  }

  return sanitizeWithRegex(html)
}

/**
 * Async version that ensures library is loaded if available
 */
export async function sanitizeHtmlAsync(html: string | null | undefined): Promise<string> {
  if (!html) return ''

  await initSanitizeLib()
  return sanitizeHtml(html)
}

/**
 * Sanitize plain text (escapes HTML entities)
 * Use this for user-provided text that should NOT contain HTML
 */
export function escapeHtml(text: string | null | undefined): string {
  if (!text) return ''

  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
}

/**
 * Check if a string contains potentially dangerous HTML
 * Returns true if content should be sanitized
 */
export function containsUnsafeHtml(html: string): boolean {
  if (!html) return false

  // Check for dangerous tags
  for (const tag of DANGEROUS_TAGS) {
    if (new RegExp(`<${tag}[\\s>]`, 'i').test(html)) {
      return true
    }
  }

  // Check for event handlers
  if (EVENT_HANDLER_DETECT_REGEX.test(html)) {
    return true
  }

  // Check for dangerous URLs
  if (DANGEROUS_URL_REGEX.test(html)) {
    return true
  }

  return false
}

/**
 * Check if sanitize-html library is available
 */
export function isLibrarySanitizationAvailable(): boolean {
  return sanitizeHtmlLib !== null
}
