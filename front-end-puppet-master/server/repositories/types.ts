/**
 * Base Repository Types
 * Common interfaces and types shared across all repositories
 */

/**
 * Common query options for list operations
 */
export interface QueryOptions {
  limit?: number
  offset?: number
  orderBy?: string
  orderDir?: 'asc' | 'desc'
}

/**
 * Paginated result wrapper
 */
export interface PaginatedResult<T> {
  items: T[]
  total: number
  limit: number
  offset: number
}

/**
 * Base repository interface (not all repositories implement all methods)
 */
export interface Repository<T, CreateT, UpdateT> {
  findById(id: number | string): Promise<T | null>
  findAll(options?: QueryOptions): Promise<T[]>
  create(data: CreateT): Promise<T>
  update(id: number | string, data: UpdateT): Promise<T>
  delete(id: number | string): Promise<void>
}
