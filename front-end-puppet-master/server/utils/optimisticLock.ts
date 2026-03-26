/**
 * Optimistic Locking Utility (MED-03)
 *
 * Prevents concurrent edit conflicts using version checking.
 * Client sends the last-known version (updatedAt timestamp).
 * Server rejects updates if the resource was modified.
 *
 * Usage:
 * const version = body.expectedVersion  // ISO string or timestamp
 * checkVersion(existing.updatedAt, version)  // Throws 409 if conflict
 *
 * Client-side:
 * - Store `updatedAt` when fetching resource
 * - Send as `expectedVersion` when updating
 * - Handle 409 by showing conflict UI and refetching
 */

/**
 * Check if the expected version matches the current version
 * @throws 409 Conflict if versions don't match
 */
export function checkVersion(
  currentVersion: Date | number | null | undefined,
  expectedVersion: string | number | Date | null | undefined
): void {
  // If no expected version provided, skip check (backwards compatibility)
  if (expectedVersion === undefined || expectedVersion === null) {
    return
  }

  // If no current version exists, skip check (new record edge case)
  if (!currentVersion) {
    return
  }

  // Normalize both to timestamps for comparison
  const currentTs =
    currentVersion instanceof Date
      ? currentVersion.getTime()
      : typeof currentVersion === 'number'
        ? currentVersion
        : new Date(currentVersion).getTime()

  const expectedTs =
    expectedVersion instanceof Date
      ? expectedVersion.getTime()
      : typeof expectedVersion === 'number'
        ? expectedVersion
        : new Date(expectedVersion).getTime()

  // Check for invalid dates
  if (isNaN(currentTs) || isNaN(expectedTs)) {
    return // Skip check if dates are invalid
  }

  // Compare timestamps (allow 1 second tolerance for rounding issues)
  if (Math.abs(currentTs - expectedTs) > 1000) {
    throw createError({
      statusCode: 409,
      statusMessage: 'Conflict: Resource was modified by another user',
      data: {
        currentVersion: new Date(currentTs).toISOString(),
        expectedVersion: new Date(expectedTs).toISOString(),
        message: 'Please refresh the page and try again.'
      }
    })
  }
}

/**
 * Create version info for response
 * Include this in API responses so clients can track versions
 */
export function versionInfo(record: { updatedAt?: Date | null }) {
  return {
    version: record.updatedAt?.toISOString() || null
  }
}
