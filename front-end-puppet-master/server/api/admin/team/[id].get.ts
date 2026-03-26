/**
 * GET /api/admin/team/:id
 *
 * Get a single team member with all translations.
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { safeJsonParseOrNull } from '../../../utils/json'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({
      statusCode: 400,
      message: 'Invalid ID'
    })
  }

  // Get team member
  const member = db
    .select()
    .from(schema.teamMembers)
    .where(eq(schema.teamMembers.id, id))
    .get()

  if (!member) {
    throw createError({
      statusCode: 404,
      message: 'Team member not found'
    })
  }

  // Get translations
  const translations = db
    .select()
    .from(schema.teamMemberTranslations)
    .where(eq(schema.teamMemberTranslations.memberId, id))
    .all()

  // Group translations by locale
  const translationsByLocale = translations.reduce<
    Record<string, { position: string | null; bio: string | null }>
  >((acc, t) => {
    acc[t.locale] = {
      position: t.position,
      bio: t.bio
    }
    return acc
  }, {})

  return {
    success: true,
    data: {
      ...member,
      socialLinks: safeJsonParseOrNull(member.socialLinks),
      translations: translationsByLocale
    }
  }
})
