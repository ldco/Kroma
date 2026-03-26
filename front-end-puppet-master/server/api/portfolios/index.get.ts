/**
 * GET /api/portfolios
 *
 * Returns list of portfolios.
 * Public endpoint - returns only published portfolios unless admin.
 *
 * Query params:
 *   - all: boolean - Include unpublished (admin only)
 *   - type: 'gallery' | 'case_study' - Filter by type
 *   - limit: number - Limit results
 *   - offset: number - Pagination offset
 *   - includeItems: boolean - Include portfolio items for each portfolio
 *   - locale: string - Return translations for specific locale (public)
 *   - itemsLimit: number - Limit items per portfolio (defaults to config.itemsPerPage)
 */
import { eq, desc, and, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { safeJsonParse } from '../../utils/json'
import { hasAdminSectionAccess } from '../../utils/admin-rbac'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)
  const user = event.context.user

  // Check if requesting all (including unpublished)
  const requestedAll = query.all === 'true'
  let includeAll = false
  if (requestedAll) {
    if (!user || !(await hasAdminSectionAccess(user, 'portfolios'))) {
      throw createError({
        statusCode: 403,
        statusMessage: 'Access denied: unpublished portfolios require portfolios section access'
      })
    }
    includeAll = true
  }
  const includeItems = query.includeItems === 'true'
  const requestedLocale = query.locale as string | undefined
  const defaultLocale = config.defaultLocale || 'en'

  // Comment 10: Get items limit from query or config
  const itemsLimit = query.itemsLimit
    ? Math.min(parseInt(query.itemsLimit as string), 50) // Cap at 50 for safety
    : config.modules?.portfolio?.config?.itemsPerPage || 12

  // Build query conditions for portfolios
  const conditions = []
  if (!includeAll) {
    conditions.push(eq(schema.portfolios.published, true))
  }

  if (query.type && (query.type === 'gallery' || query.type === 'case_study')) {
    conditions.push(eq(schema.portfolios.type, query.type))
  }

  // Parse pagination params
  const limit = query.limit ? parseInt(query.limit as string) : undefined
  const offset = query.offset ? parseInt(query.offset as string) : 0

  // Execute query with database-level pagination
  const whereClause =
    conditions.length === 0
      ? undefined
      : conditions.length === 1
        ? conditions[0]
        : and(...conditions)

  const baseQuery = db
    .select()
    .from(schema.portfolios)
    .orderBy(desc(schema.portfolios.order), desc(schema.portfolios.createdAt))

  let portfolios
  if (whereClause) {
    if (limit) {
      portfolios = baseQuery.where(whereClause).limit(limit).offset(offset).all()
    } else {
      portfolios = baseQuery.where(whereClause).all()
    }
  } else {
    if (limit) {
      portfolios = baseQuery.limit(limit).offset(offset).all()
    } else {
      portfolios = baseQuery.all()
    }
  }

  // Fetch translations for portfolios
  const portfolioTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'portfolio.%'))
    .all()

  // Build translation maps for portfolios
  const getPortfolioTranslations = (portfolioId: number) => {
    const nameKey = `portfolio.${portfolioId}.name`
    const descKey = `portfolio.${portfolioId}.description`
    const translationsMap: Record<string, { name: string | null; description: string | null }> = {}

    const locales = new Set(
      portfolioTranslations
        .filter(t => t.key === nameKey || t.key === descKey)
        .map(t => t.locale)
    )

    locales.forEach(locale => {
      const name = portfolioTranslations.find(t => t.key === nameKey && t.locale === locale)?.value || null
      const description = portfolioTranslations.find(t => t.key === descKey && t.locale === locale)?.value || null
      translationsMap[locale] = { name, description }
    })

    return translationsMap
  }

  // If includeItems, fetch items for each portfolio
  if (includeItems && portfolios.length > 0) {
    const portfolioIds = portfolios.map(p => p.id)

    // Comment 2: Fetch enough items for all portfolios (N * limit) to ensure each gets up to itemsLimit
    const dbLimit = portfolioIds.length * itemsLimit

    // Fetch all items for these portfolios
    const itemConditions = includeAll
      ? [] // Admin sees all items
      : [eq(schema.portfolioItems.published, true)]

    // Comment 2: Apply per-portfolio limit by fetching N*limit items, then capping during grouping
    const allItems = itemConditions.length > 0
      ? db
          .select()
          .from(schema.portfolioItems)
          .where(and(...itemConditions))
          .orderBy(desc(schema.portfolioItems.order), desc(schema.portfolioItems.createdAt))
          .limit(dbLimit)
          .all()
      : db
          .select()
          .from(schema.portfolioItems)
          .orderBy(desc(schema.portfolioItems.order), desc(schema.portfolioItems.createdAt))
          .limit(dbLimit)
          .all()

    // Fetch item translations
    const itemTranslations = db
      .select()
      .from(schema.translations)
      .where(like(schema.translations.key, 'portfolio_item.%'))
      .all()

    // Build translation map for items
    const getItemTranslations = (itemId: number) => {
      const titleKey = `portfolio_item.${itemId}.title`
      const descKey = `portfolio_item.${itemId}.description`
      const contentKey = `portfolio_item.${itemId}.content`
      const categoryKey = `portfolio_item.${itemId}.category`
      const translationsMap: Record<string, { title: string | null; description: string | null; content: string | null; category: string | null }> = {}

      const locales = new Set(
        itemTranslations
          .filter(t => t.key === titleKey || t.key === descKey || t.key === contentKey || t.key === categoryKey)
          .map(t => t.locale)
      )

      locales.forEach(locale => {
        const title = itemTranslations.find(t => t.key === titleKey && t.locale === locale)?.value || null
        const description = itemTranslations.find(t => t.key === descKey && t.locale === locale)?.value || null
        const content = itemTranslations.find(t => t.key === contentKey && t.locale === locale)?.value || null
        const category = itemTranslations.find(t => t.key === categoryKey && t.locale === locale)?.value || null
        translationsMap[locale] = { title, description, content, category }
      })

      return translationsMap
    }

    // Filter items by portfolio IDs and group by portfolioId
    // Comment 2: Enforce per-portfolio limit during grouping
    const itemsByPortfolio = new Map<number, typeof allItems>()
    for (const item of allItems) {
      if (portfolioIds.includes(item.portfolioId)) {
        const existing = itemsByPortfolio.get(item.portfolioId) || []
        // Comment 2: Stop adding items once we hit the per-portfolio limit
        if (existing.length >= itemsLimit) {
          continue
        }
        const itemTrans = getItemTranslations(item.id)
        const locale = requestedLocale || defaultLocale
        // Parse tags JSON and add translations
        const parsedItem = {
          ...item,
          tags: safeJsonParse(item.tags, []),
          // For public: return translated values, for admin: return translations map
          ...(includeAll
            ? { translations: itemTrans }
            : {
                title: itemTrans[locale]?.title || item.title,
                description: itemTrans[locale]?.description || item.description,
                content: itemTrans[locale]?.content || item.content,
                category: itemTrans[locale]?.category || item.category
              })
        }
        existing.push(parsedItem)
        itemsByPortfolio.set(item.portfolioId, existing)
      }
    }

    // Attach items to each portfolio and return in envelope format (Comment 9)
    const portfoliosWithItems = portfolios.map(portfolio => {
      const trans = getPortfolioTranslations(portfolio.id)
      const locale = requestedLocale || defaultLocale
      return {
        ...portfolio,
        // For public: return translated values, for admin: return translations map
        ...(includeAll
          ? { translations: trans }
          : {
              name: trans[locale]?.name || portfolio.name,
              description: trans[locale]?.description || portfolio.description
            }),
        items: itemsByPortfolio.get(portfolio.id) || []
      }
    })

    // Comment 9: Return strict envelope format
    return {
      success: true,
      data: portfoliosWithItems
    }
  }

  // Return portfolios with translations
  return {
    success: true,
    data: portfolios.map(portfolio => {
      const trans = getPortfolioTranslations(portfolio.id)
      const locale = requestedLocale || defaultLocale
      return {
        ...portfolio,
        // For public: return translated values, for admin: return translations map
        ...(includeAll
          ? { translations: trans }
          : {
              name: trans[locale]?.name || portfolio.name,
              description: trans[locale]?.description || portfolio.description
            })
      }
    })
  }
})
