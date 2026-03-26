/**
 * GET /api/team
 *
 * Returns list of published team members.
 * Content stored in centralized translations table with key pattern: team.{id}.{field}
 *
 * Query params:
 *   - locale: string - Get translations for this locale
 *   - department: string - Filter by department
 *   - limit: number - Limit results
 */
import { eq, asc, and, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { safeJsonParseOrNull } from '../../utils/json'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const query = getQuery(event)
  const locale = (query.locale as string) || config.defaultLocale || 'en'
  const defaultLocale = config.defaultLocale || 'en'

  // Build query conditions
  const conditions = [eq(schema.teamMembers.published, true)]

  if (query.department && typeof query.department === 'string') {
    conditions.push(eq(schema.teamMembers.department, query.department))
  }

  // Parse limit
  const limit = query.limit ? parseInt(query.limit as string) : undefined

  // Get team members
  let members = db
    .select()
    .from(schema.teamMembers)
    .where(and(...conditions))
    .orderBy(asc(schema.teamMembers.order), asc(schema.teamMembers.name))
    .all()

  if (limit) {
    members = members.slice(0, limit)
  }

  if (members.length === 0) return { success: true, data: [] }

  // Get all team translations from centralized translations table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'team.%'))
    .all()

  // Return members with translations
  return {
    success: true,
    data: members.map(member => {
      const positionKey = `team.${member.id}.position`
      const bioKey = `team.${member.id}.bio`

      // Try requested locale first, then fall back to default locale
      const positionTrans =
        allTranslations.find(t => t.key === positionKey && t.locale === locale) ||
        allTranslations.find(t => t.key === positionKey && t.locale === defaultLocale)

      const bioTrans =
        allTranslations.find(t => t.key === bioKey && t.locale === locale) ||
        allTranslations.find(t => t.key === bioKey && t.locale === defaultLocale)

      return {
        ...member,
        position: positionTrans?.value || null,
        bio: bioTrans?.value || null,
        socialLinks: safeJsonParseOrNull(member.socialLinks)
      }
    })
  }
})
