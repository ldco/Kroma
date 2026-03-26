/**
 * Content Types
 *
 * Re-exports database-derived types for content entities.
 * This file exists for backward compatibility with existing imports.
 *
 * IMPORTANT: These types are now sourced from database schema.
 * Edit server/database/schema.ts to modify entity definitions.
 */

// Re-export everything from database types (single source of truth)
export type {
  DateLike,
  Portfolio,
  PortfolioItem,
  PortfolioWithItems,
  ContactSubmission,
  PortfolioType,
  PortfolioItemType
} from './database'

export {
  PORTFOLIO_TYPES,
  PORTFOLIO_ITEM_TYPES
} from './database'
