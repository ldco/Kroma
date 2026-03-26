/**
 * HTML Sanitization Utility (Client-side)
 *
 * Prevents XSS attacks by removing dangerous tags, event handlers,
 * and javascript: URLs from HTML content.
 */

// Tags that should be completely removed (including content)
const DANGEROUS_TAGS = [
  'script',
  'style',
  'iframe',
  'frame',
  'object',
  'embed',
  'form',
  'input',
  'meta',
  'link',
  'base'
]

// Event handler pattern (on*)
const EVENT_HANDLER_REGEX = /\s+on\w+\s*=/gi

// JavaScript/data URL pattern
const DANGEROUS_URL_REGEX = /^\s*(javascript|data|vbscript):/i

/**
 * Sanitize HTML content to prevent XSS
 */
export function sanitizeHtml(html: string | null | undefined): string {
  if (!html) return ''

  let result = html

  // 1. Remove dangerous tags with their content
  for (const tag of DANGEROUS_TAGS) {
    const regex = new RegExp(`<${tag}[^>]*>[\\s\\S]*?<\\/${tag}>`, 'gi')
    result = result.replace(regex, '')
    const selfClosing = new RegExp(`<${tag}[^>]*/?>`, 'gi')
    result = result.replace(selfClosing, '')
  }

  // 2. Remove event handlers from all tags
  result = result.replace(EVENT_HANDLER_REGEX, ' ')

  // 3. Sanitize href and src attributes
  result = result.replace(/(href|src)\s*=\s*["']([^"']*)["']/gi, (match, attr, url) => {
    if (DANGEROUS_URL_REGEX.test(url)) {
      return `${attr}="#"`
    }
    return match
  })

  // 4. Remove expression() in style attributes (IE CSS expression attack)
  result = result.replace(/style\s*=\s*["'][^"']*expression\s*\([^"']*["']/gi, 'style=""')

  return result.trim()
}
