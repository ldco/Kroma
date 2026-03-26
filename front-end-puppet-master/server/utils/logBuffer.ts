/**
 * Log Ring Buffer
 *
 * In-memory circular buffer for storing recent log entries.
 * Used by the admin health page to display recent logs without
 * external log aggregation services.
 *
 * Features:
 * - Fixed size buffer (500 entries by default)
 * - FIFO eviction when full
 * - Thread-safe for single-process Node.js
 * - ~50KB memory for 500 entries
 */

import { Writable } from 'stream'

export interface LogEntry {
  time: string
  level: number
  levelLabel: string
  msg: string
  context?: Record<string, unknown>
}

// Log level mappings (Pino levels)
const LEVEL_LABELS: Record<number, string> = {
  10: 'trace',
  20: 'debug',
  30: 'info',
  40: 'warn',
  50: 'error',
  60: 'fatal'
}

// Buffer configuration
const MAX_BUFFER_SIZE = 500

// PII and sensitive fields to exclude from logs
const SENSITIVE_FIELDS = [
  'password', 'passwd', 'secret', 'token', 'apiKey', 'api_key',
  'authorization', 'auth', 'cookie', 'session', 'csrf',
  'creditCard', 'credit_card', 'cardNumber', 'card_number', 'cvv', 'cvc',
  'ssn', 'socialSecurity', 'social_security',
  'email', 'phone', 'address', 'dob', 'dateOfBirth', 'date_of_birth',
  'ip', 'ipAddress', 'ip_address', 'userAgent', 'user_agent',
  'body', 'requestBody', 'request_body' // Request bodies may contain sensitive data
]

// The ring buffer
const buffer: LogEntry[] = []

/**
 * Check if a field name is sensitive (case-insensitive)
 */
function isSensitiveField(fieldName: string): boolean {
  const lowerField = fieldName.toLowerCase()
  return SENSITIVE_FIELDS.some(sensitive => lowerField.includes(sensitive.toLowerCase()))
}

/**
 * Recursively filter sensitive fields from an object
 */
function filterSensitiveData(obj: Record<string, unknown>): Record<string, unknown> {
  const filtered: Record<string, unknown> = {}
  for (const [key, value] of Object.entries(obj)) {
    if (isSensitiveField(key)) {
      filtered[key] = '[REDACTED]'
    } else if (value && typeof value === 'object' && !Array.isArray(value)) {
      filtered[key] = filterSensitiveData(value as Record<string, unknown>)
    } else {
      filtered[key] = value
    }
  }
  return filtered
}

/**
 * Add a log entry to the buffer
 */
export function addLogEntry(entry: {
  level: number
  time: number | string
  msg: string
  [key: string]: unknown
}): void {
  // Extract known fields
  const { level, time, msg, ...rest } = entry

  // Remove pino internal fields from context
  const context = { ...rest }
  delete context.pid
  delete context.hostname
  delete context.service
  delete context.version
  delete context.environment

  // Filter out PII and sensitive data
  const sanitizedContext = filterSensitiveData(context)

  // Only include context if it has fields
  const hasContext = Object.keys(sanitizedContext).length > 0

  const logEntry: LogEntry = {
    time: typeof time === 'number' ? new Date(time).toISOString() : String(time),
    level,
    levelLabel: LEVEL_LABELS[level] || 'unknown',
    msg: msg || '',
    ...(hasContext && { context: sanitizedContext })
  }

  buffer.push(logEntry)

  // Evict oldest if over limit
  if (buffer.length > MAX_BUFFER_SIZE) {
    buffer.shift()
  }
}

/**
 * Get recent log entries
 * @param limit - Maximum number of entries to return (default: 100)
 * @param minLevel - Minimum log level to include (default: 0 = all)
 */
export function getRecentLogs(limit = 100, minLevel = 0): LogEntry[] {
  const filtered = minLevel > 0 ? buffer.filter(entry => entry.level >= minLevel) : buffer

  // Return most recent entries (newest last in array, we want newest first for display)
  return filtered.slice(-limit).reverse()
}

/**
 * Get buffer statistics
 */
export function getBufferStats(): { size: number; maxSize: number; oldestEntry: string | null } {
  const firstEntry = buffer[0]
  return {
    size: buffer.length,
    maxSize: MAX_BUFFER_SIZE,
    oldestEntry: firstEntry ? firstEntry.time : null
  }
}

/**
 * Clear the buffer (useful for testing)
 */
export function clearBuffer(): void {
  buffer.length = 0
}

/**
 * Create a Pino destination stream that also writes to the buffer
 */
export function createBufferDestination(): NodeJS.WritableStream {
  return new Writable({
    write(chunk: Buffer, _encoding: string, callback: () => void) {
      try {
        const str = chunk.toString()
        // Pino writes newline-delimited JSON
        const lines = str.split('\n').filter(Boolean)
        for (const line of lines) {
          try {
            const parsed = JSON.parse(line)
            addLogEntry(parsed)
          } catch {
            // Not JSON, ignore
          }
        }
      } catch {
        // Ignore parse errors
      }
      callback()
    }
  })
}
