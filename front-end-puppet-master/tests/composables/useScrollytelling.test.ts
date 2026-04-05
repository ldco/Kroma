import { describe, expect, it } from 'vitest'

/**
 * Pagination utility tests
 *
 * Tests for pure pagination helper functions used in the app.
 * These don't require Nuxt runtime context.
 */

// Pure pagination helpers (extracted from usePagination for testability)
function calculatePageNumbers(currentPage: number, totalPages: number): (number | '...')[] {
  const pages: (number | '...')[] = []

  if (totalPages <= 7) {
    for (let i = 1; i <= totalPages; i++) {
      pages.push(i)
    }
  } else {
    pages.push(1)
    if (currentPage > 3) {
      pages.push('...')
    }
    const start = Math.max(2, currentPage - 1)
    const end = Math.min(totalPages - 1, currentPage + 1)
    for (let i = start; i <= end; i++) {
      pages.push(i)
    }
    if (currentPage < totalPages - 2) {
      pages.push('...')
    }
    pages.push(totalPages)
  }

  return pages
}

function buildOffset(page: number, limit: number): number {
  return (page - 1) * limit
}

describe('pagination utilities', () => {
  describe('calculatePageNumbers', () => {
    it('shows all pages when total is <= 7', () => {
      expect(calculatePageNumbers(1, 5)).toEqual([1, 2, 3, 4, 5])
      expect(calculatePageNumbers(3, 7)).toEqual([1, 2, 3, 4, 5, 6, 7])
    })

    it('shows ellipsis for large page counts', () => {
      const pages = calculatePageNumbers(10, 20)
      expect(pages).toContain(1)
      expect(pages).toContain('...')
      expect(pages).toContain(20)
    })

    it('shows pages around current page', () => {
      const pages = calculatePageNumbers(10, 20)
      expect(pages).toContain(9)
      expect(pages).toContain(10)
      expect(pages).toContain(11)
    })

    it('handles single page', () => {
      expect(calculatePageNumbers(1, 1)).toEqual([1])
    })

    it('handles zero pages', () => {
      expect(calculatePageNumbers(0, 0)).toEqual([])
    })
  })

  describe('buildOffset', () => {
    it('calculates offset correctly', () => {
      expect(buildOffset(1, 10)).toBe(0)
      expect(buildOffset(2, 10)).toBe(10)
      expect(buildOffset(3, 20)).toBe(40)
    })

    it('handles page 1', () => {
      expect(buildOffset(1, 25)).toBe(0)
    })
  })
})
