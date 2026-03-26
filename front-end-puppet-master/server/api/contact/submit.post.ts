/**
 * POST /api/contact/submit
 *
 * Submits a contact form.
 * Public endpoint with rate limiting and validation.
 *
 * Side effects:
 * - Saves to database
 * - Sends confirmation email to user (if SMTP configured)
 * - Sends Telegram notification to admin (if bot configured)
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { sendContactConfirmation } from '../../utils/email'
import { notifyNewContact } from '../../utils/telegram'
import { contactRateLimiter } from '../../utils/rateLimit'
import { escapeHtml } from '../../utils/sanitize'
import { logger } from '../../utils/logger'
import { contactSchema } from '../../utils/validation'
import config from '~~/app/puppet-master.config'

export default defineEventHandler(async event => {
  // Get client IP for rate limiting
  const ip = getHeader(event, 'x-forwarded-for') || getHeader(event, 'x-real-ip') || 'unknown'

  // Check rate limit
  if (!contactRateLimiter.checkRateLimit(ip)) {
    throw createError({
      statusCode: 429,
      statusMessage: 'Too many requests. Please try again later.'
    })
  }

  // Parse and validate request body
  const body = await readBody(event)
  const result = contactSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const data = result.data
  const db = useDatabase()

  // Sanitize user input before storing
  const sanitizedData = {
    name: escapeHtml(data.name),
    email: data.email, // Email is validated by zod, no HTML escaping needed
    phone: data.phone ? escapeHtml(data.phone) : null,
    subject: data.subject ? escapeHtml(data.subject) : null,
    message: escapeHtml(data.message)
  }

  // Insert contact submission
  const submission = db
    .insert(schema.contactSubmissions)
    .values({
      name: sanitizedData.name,
      email: sanitizedData.email,
      phone: sanitizedData.phone,
      subject: sanitizedData.subject,
      message: sanitizedData.message,
      read: false
    })
    .returning()
    .get()

  // Send notifications (don't await - fire and forget, don't block response)
  // Only if enabled in config

  // Email confirmation to user
  if (config.features.contactEmailConfirmation) {
    // Get site name from settings for email (legal.companyName → seo.title → fallback)
    const companyNameSetting = db
      .select()
      .from(schema.settings)
      .where(eq(schema.settings.key, 'legal.companyName'))
      .get()
    const seoTitleSetting = db
      .select()
      .from(schema.settings)
      .where(eq(schema.settings.key, 'seo.title'))
      .get()
    const siteName = companyNameSetting?.value || seoTitleSetting?.value || 'Puppet Master'

    sendContactConfirmation(data.email, data.name, siteName).catch((e: unknown) => {
      logger.error(
        { error: e instanceof Error ? e.message : String(e) },
        'Email notification failed'
      )
    })
  }

  // Telegram notification to admin
  if (config.features.contactTelegramNotify) {
    notifyNewContact({
      name: data.name,
      email: data.email,
      phone: data.phone,
      message: data.message
    }).catch((e: unknown) => {
      logger.error(
        { error: e instanceof Error ? e.message : String(e) },
        'Telegram notification failed'
      )
    })
  }

  return {
    success: true,
    data: {
      message: 'Thank you for your message. We will get back to you soon.',
      id: submission.id
    }
  }
})
