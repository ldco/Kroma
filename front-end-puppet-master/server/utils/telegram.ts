/**
 * Telegram Bot Service
 *
 * Sends notifications to a Telegram chat/group.
 * Configure via:
 * - Admin settings: contact.telegramBotToken
 * - Environment: TELEGRAM_CHAT_ID (infrastructure-owned destination)
 */
import { escapeHtml } from './sanitize'
import { logger } from './logger'
import { getSettingValue } from './site-settings'

interface TelegramConfig {
  botToken: string
  chatId: string
}

function getConfig(): TelegramConfig | null {
  const runtimeConfig = useRuntimeConfig()
  const telegramBotToken = getSettingValue('contact.telegramBotToken') || runtimeConfig.telegramBotToken
  const telegramChatId = runtimeConfig.telegramChatId

  if (!telegramBotToken || !telegramChatId) {
    logger.debug('Telegram bot not configured - notifications will be skipped')
    return null
  }

  return {
    botToken: telegramBotToken,
    chatId: telegramChatId
  }
}

/**
 * Send a message to the configured Telegram chat
 * Supports HTML formatting
 */
export async function sendTelegramMessage(
  text: string,
  parseMode: 'HTML' | 'Markdown' = 'HTML'
): Promise<boolean> {
  const config = getConfig()
  if (!config) return false

  const url = `https://api.telegram.org/bot${config.botToken}/sendMessage`

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        chat_id: config.chatId,
        text,
        parse_mode: parseMode
      })
    })

    const result = await response.json()

    if (!result.ok) {
      logger.error({ description: result.description }, 'Telegram API error')
      return false
    }

    logger.info('Telegram message sent successfully')
    return true
  } catch (error) {
    logger.error({ error }, 'Failed to send Telegram message')
    return false
  }
}

/**
 * Send contact form notification to admin
 */
export async function notifyNewContact(data: {
  name: string
  email: string
  phone?: string | null
  message: string
}): Promise<boolean> {
  const text = `
📩 <b>New Contact Form Submission</b>

<b>Name:</b> ${escapeHtml(data.name)}
<b>Email:</b> ${escapeHtml(data.email)}${data.phone ? `\n<b>Phone:</b> ${escapeHtml(data.phone)}` : ''}

<b>Message:</b>
${escapeHtml(data.message)}
`.trim()

  return sendTelegramMessage(text)
}
