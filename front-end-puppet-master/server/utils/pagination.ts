/**
 * Database-Level Pagination Utility (HIGH-02)
 *
 * Provides efficient cursor-based and offset pagination
 * at the database level instead of in-memory.
 *
 * Features:
 * - Offset-based pagination (page/limit)
 * - Cursor-based pagination (for infinite scroll)
 * - Sorting support
 * - Filtering helpers
 * - Consistent response format
 *
 * Usage:
 * const params = parsePaginationParams(getQuery(event))
 * const result = await paginate(db, schema.users, params)
 * return result
 */
import { sql, asc, desc, and, gt, lt, eq } from 'drizzle-orm'
import type { SQL, SQLWrapper } from 'drizzle-orm'
import type { BetterSQLite3Database } from 'drizzle-orm/better-sqlite3'

export interface PaginationParams {
  page?: number
  limit?: number
  cursor?: number | string // For cursor-based pagination
  sortBy?: string
  sortOrder?: 'asc' | 'desc'
  filters?: Record<string, unknown>
}

export interface PaginatedResult<T> {
  items: T[]
  meta: PaginationMeta
}

export interface PaginationMeta {
  total: number
  page: number
  limit: number
  totalPages: number
  hasNextPage: boolean
  hasPreviousPage: boolean
  nextCursor?: number | string
  previousCursor?: number | string
}

// Default and max limits
const DEFAULT_LIMIT = 20
const MAX_LIMIT = 100

/**
 * Parse and validate pagination parameters from query string
 */
export function parsePaginationParams(query: Record<string, unknown>): PaginationParams {
  const page = Math.max(1, parseInt(String(query.page || '1'), 10) || 1)
  const limit = Math.min(
    MAX_LIMIT,
    Math.max(1, parseInt(String(query.limit || DEFAULT_LIMIT), 10) || DEFAULT_LIMIT)
  )
  const cursor = query.cursor ? String(query.cursor) : undefined
  const sortBy = query.sortBy ? String(query.sortBy) : undefined
  const sortOrder = (query.sortOrder === 'asc' ? 'asc' : 'desc') as 'asc' | 'desc'

  // Parse filters from query (filter[field]=value format)
  const filters: Record<string, unknown> = {}
  for (const [key, value] of Object.entries(query)) {
    const filterMatch = key.match(/^filter\[(\w+)\]$/)
    if (filterMatch && value !== undefined && value !== '') {
      filters[filterMatch[1]] = value
    }
  }

  return { page, limit, cursor, sortBy, sortOrder, filters }
}

/**
 * Calculate offset from page and limit
 */
export function calculateOffset(page: number, limit: number): number {
  return (page - 1) * limit
}

/**
 * Build pagination metadata
 */
export function buildPaginationMeta(
  total: number,
  page: number,
  limit: number,
  items: unknown[],
  cursorField?: string
): PaginationMeta {
  const totalPages = Math.ceil(total / limit)
  const hasNextPage = page < totalPages
  const hasPreviousPage = page > 1

  const meta: PaginationMeta = {
    total,
    page,
    limit,
    totalPages,
    hasNextPage,
    hasPreviousPage
  }

  // Add next cursor for cursor-based pagination
  if (cursorField && items.length > 0) {
    const lastItem = items[items.length - 1] as Record<string, unknown>
    const firstItem = items[0] as Record<string, unknown>

    if (lastItem && cursorField in lastItem && hasNextPage) {
      meta.nextCursor = lastItem[cursorField] as number | string
    }
    if (firstItem && cursorField in firstItem && hasPreviousPage) {
      meta.previousCursor = firstItem[cursorField] as number | string
    }
  }

  return meta
}

/**
 * Apply pagination to a SQL query
 * Returns the SQL clauses to add (LIMIT and OFFSET)
 */
export function paginationClauses(
  page: number,
  limit: number
): { limitClause: number; offsetClause: number } {
  return {
    limitClause: limit,
    offsetClause: calculateOffset(page, limit)
  }
}

/**
 * Create a count query SQL
 */
export function countSql(tableName: string, where?: SQL): SQL {
  if (where) {
    return sql`SELECT COUNT(*) as count FROM ${sql.identifier(tableName)} WHERE ${where}`
  }
  return sql`SELECT COUNT(*) as count FROM ${sql.identifier(tableName)}`
}

/**
 * Get sort direction function
 */
export function getSortDirection(order: 'asc' | 'desc' = 'desc') {
  return order === 'asc' ? asc : desc
}

/**
 * Build where clause from filters
 */
export function buildFilterConditions<T extends Record<string, SQLWrapper>>(
  schema: T,
  filters: Record<string, unknown>
): SQL | undefined {
  const conditions: SQL[] = []

  for (const [field, value] of Object.entries(filters)) {
    if (value === undefined || value === null || value === '') continue

    const column = schema[field as keyof T]
    if (!column) continue

    // Handle different value types
    if (typeof value === 'boolean' || value === 'true' || value === 'false') {
      const boolValue = value === true || value === 'true'
      conditions.push(eq(column, boolValue))
    } else if (typeof value === 'number' || !isNaN(Number(value))) {
      conditions.push(eq(column, Number(value)))
    } else if (typeof value === 'string') {
      // Check for comparison operators
      if (value.startsWith('>=')) {
        conditions.push(sql`${column} >= ${value.slice(2)}`)
      } else if (value.startsWith('<=')) {
        conditions.push(sql`${column} <= ${value.slice(2)}`)
      } else if (value.startsWith('>')) {
        conditions.push(gt(column, value.slice(1)))
      } else if (value.startsWith('<')) {
        conditions.push(lt(column, value.slice(1)))
      } else if (value.includes('%')) {
        // LIKE query
        conditions.push(sql`${column} LIKE ${value}`)
      } else {
        conditions.push(eq(column, value))
      }
    }
  }

  if (conditions.length === 0) return undefined
  if (conditions.length === 1) return conditions[0]
  return and(...conditions)
}

/**
 * Format pagination response for API
 */
export function formatPaginatedResponse<T>(
  items: T[],
  meta: PaginationMeta
): PaginatedResult<T> {
  return { items, meta }
}

/**
 * Create pagination links for HATEOAS
 */
export function createPaginationLinks(
  baseUrl: string,
  params: PaginationParams,
  meta: PaginationMeta
): {
  self: string
  first: string
  last: string
  next?: string
  prev?: string
} {
  const buildUrl = (page: number) => {
    const url = new URL(baseUrl)
    url.searchParams.set('page', String(page))
    url.searchParams.set('limit', String(params.limit || DEFAULT_LIMIT))
    if (params.sortBy) url.searchParams.set('sortBy', params.sortBy)
    if (params.sortOrder) url.searchParams.set('sortOrder', params.sortOrder)
    return url.toString()
  }

  const links: ReturnType<typeof createPaginationLinks> = {
    self: buildUrl(meta.page),
    first: buildUrl(1),
    last: buildUrl(meta.totalPages || 1)
  }

  if (meta.hasNextPage) {
    links.next = buildUrl(meta.page + 1)
  }
  if (meta.hasPreviousPage) {
    links.prev = buildUrl(meta.page - 1)
  }

  return links
}

/**
 * Legacy export for backward compatibility
 */
export { buildPaginationMeta as buildPaginationMetaLegacy }

// Re-export types
export type { SQL }
