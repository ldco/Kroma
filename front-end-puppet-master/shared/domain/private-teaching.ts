/**
 * Private teaching domain limits shared across app/server/tests.
 */

export const PRIVATE_TEACHING_LIMITS = {
  minGroupSize: 1,
  maxGroupSize: 10,
  defaultGroupSize: 1
} as const

/**
 * Normalize private-group size into a safe integer range.
 */
export function normalizePrivateGroupSize(value: unknown): number {
  if (value === null || value === undefined || value === '') {
    return PRIVATE_TEACHING_LIMITS.defaultGroupSize
  }

  const numeric =
    typeof value === 'number' ? value : Number.parseInt(String(value), 10)

  if (!Number.isFinite(numeric)) {
    return PRIVATE_TEACHING_LIMITS.defaultGroupSize
  }

  const rounded = Math.round(numeric)
  return Math.min(
    PRIVATE_TEACHING_LIMITS.maxGroupSize,
    Math.max(PRIVATE_TEACHING_LIMITS.minGroupSize, rounded)
  )
}
