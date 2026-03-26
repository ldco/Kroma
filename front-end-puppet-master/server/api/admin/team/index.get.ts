/**
 * GET /api/admin/team
 *
 * Get all team members with translations for admin.
 * Content stored in centralized translations table with key pattern: team.{id}.{field}
 */
import { asc, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { safeJsonParseOrNull } from '../../../utils/json'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async () => {
  const db = useDatabase()
  const defaultLocale = config.defaultLocale || 'en'

  // Get all team members ordered by order
  const members = db
    .select()
    .from(schema.teamMembers)
    .orderBy(asc(schema.teamMembers.order), asc(schema.teamMembers.name))
    .all()

  // Get all team translations from centralized table
  const allTranslations = db
    .select()
    .from(schema.translations)
    .where(like(schema.translations.key, 'team.%'))
    .all()

  // Return members with translations
  return {
    success: true,
    data: members.map(member => {
      // Build translations map for this member
      const translationsMap: Record<string, { position: string | null; bio: string | null }> = {}
      const positionKey = `team.${member.id}.position`
      const bioKey = `team.${member.id}.bio`

      // Group translations by locale
      const locales = new Set(
        allTranslations
          .filter(t => t.key === positionKey || t.key === bioKey)
          .map(t => t.locale)
      )

      locales.forEach(locale => {
        const position = allTranslations.find(t => t.key === positionKey && t.locale === locale)?.value || null
        const bio = allTranslations.find(t => t.key === bioKey && t.locale === locale)?.value || null
        translationsMap[locale] = { position, bio }
      })

      // Get position/bio from default locale for the form
      const defaultPosition = translationsMap[defaultLocale]?.position || ''
      const defaultBio = translationsMap[defaultLocale]?.bio || ''

      return {
        ...member,
        position: defaultPosition,
        bio: defaultBio,
        socialLinks: safeJsonParseOrNull(member.socialLinks),
        translations: translationsMap
      }
    })
  }
})
