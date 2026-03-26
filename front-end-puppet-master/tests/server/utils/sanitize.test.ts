import { describe, it, expect } from 'vitest'
import { containsUnsafeHtml, escapeHtml, sanitizeHtml } from '../../../server/utils/sanitize'

describe('escapeHtml', () => {
  it('should escape ampersand', () => {
    expect(escapeHtml('Tom & Jerry')).toBe('Tom &amp; Jerry')
  })

  it('should escape less than', () => {
    expect(escapeHtml('a < b')).toBe('a &lt; b')
  })

  it('should escape greater than', () => {
    expect(escapeHtml('a > b')).toBe('a &gt; b')
  })

  it('should escape all HTML special characters together', () => {
    expect(escapeHtml('<script>alert("XSS")</script>')).toBe(
      '&lt;script&gt;alert(&quot;XSS&quot;)&lt;/script&gt;'
    )
  })

  it('should handle multiple occurrences', () => {
    expect(escapeHtml('a & b & c < d > e')).toBe('a &amp; b &amp; c &lt; d &gt; e')
  })

  it('should return empty string for empty input', () => {
    expect(escapeHtml('')).toBe('')
  })

  it('should not modify text without special characters', () => {
    expect(escapeHtml('Hello World')).toBe('Hello World')
  })

  it('should handle unicode characters', () => {
    expect(escapeHtml('Привет <мир> & 世界')).toBe('Привет &lt;мир&gt; &amp; 世界')
  })

  it('should preserve newlines', () => {
    expect(escapeHtml('Line 1\nLine 2')).toBe('Line 1\nLine 2')
  })

  it('should handle HTML-like injection attempts', () => {
    expect(escapeHtml('<b>bold</b>')).toBe('&lt;b&gt;bold&lt;/b&gt;')
    expect(escapeHtml('<a href="http://evil.com">Click</a>')).toBe(
      '&lt;a href=&quot;http://evil.com&quot;&gt;Click&lt;/a&gt;'
    )
  })
})

describe('sanitizeHtml', () => {
  it('removes full double-quoted event handler attributes', () => {
    const input = '<div onclick="alert(1)" class="safe">Click</div>'
    const output = sanitizeHtml(input)

    expect(output).toContain('<div')
    expect(output).toContain('class="safe"')
    expect(output).not.toContain('onclick')
    expect(output).not.toContain('alert(1)"')
  })

  it('removes full single-quoted event handler attributes', () => {
    const input = "<span onmouseover='doBadThing()'>Hover</span>"
    const output = sanitizeHtml(input)

    expect(output).toContain('<span')
    expect(output).not.toContain('onmouseover')
    expect(output).not.toContain("doBadThing()'")
  })

  it('removes full unquoted event handler attributes', () => {
    const input = '<p onfocus=runBadStuff()>Hello</p>'
    const output = sanitizeHtml(input)

    expect(output).toContain('<p')
    expect(output).not.toContain('onfocus')
    expect(output).not.toContain('runBadStuff()')
  })
})

describe('containsUnsafeHtml', () => {
  it('detects inline event handlers', () => {
    expect(containsUnsafeHtml('<div onclick="alert(1)">X</div>')).toBe(true)
  })

  it('returns false for plain safe markup', () => {
    expect(containsUnsafeHtml('<div><p>Safe content</p></div>')).toBe(false)
  })
})
