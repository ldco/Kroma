import { describe, expect, it } from 'vitest'
import {
  PRIVATE_TEACHING_LIMITS,
  normalizePrivateGroupSize
} from '../../../server/utils/group-size'

describe('normalizePrivateGroupSize', () => {
  it('returns default value for empty input', () => {
    expect(normalizePrivateGroupSize(undefined)).toBe(PRIVATE_TEACHING_LIMITS.defaultGroupSize)
    expect(normalizePrivateGroupSize(null)).toBe(PRIVATE_TEACHING_LIMITS.defaultGroupSize)
    expect(normalizePrivateGroupSize('')).toBe(PRIVATE_TEACHING_LIMITS.defaultGroupSize)
  })

  it('clamps values to the domain range', () => {
    expect(normalizePrivateGroupSize(0)).toBe(PRIVATE_TEACHING_LIMITS.minGroupSize)
    expect(normalizePrivateGroupSize(999)).toBe(PRIVATE_TEACHING_LIMITS.maxGroupSize)
  })

  it('accepts strings and rounds numeric values', () => {
    expect(normalizePrivateGroupSize('4')).toBe(4)
    expect(normalizePrivateGroupSize(3.6)).toBe(4)
  })

  it('falls back to default for invalid values', () => {
    expect(normalizePrivateGroupSize('abc')).toBe(PRIVATE_TEACHING_LIMITS.defaultGroupSize)
    expect(normalizePrivateGroupSize(Number.NaN)).toBe(PRIVATE_TEACHING_LIMITS.defaultGroupSize)
  })
})
