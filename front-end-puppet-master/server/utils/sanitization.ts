/**
 * Sanitization Utilities
 *
 * Shared helpers for sanitizing user input to prevent XSS and injection attacks.
 * Centralizes sanitization logic to avoid duplication across API routes.
 *
 * Usage:
 *   import { sanitizeQuestionPayload, sanitizeText, sanitizeHtml } from '../utils/sanitization'
 *
 *   const cleanPayload = sanitizeQuestionPayload(body)
 */
import sanitizeHtmlLib from 'sanitize-html'

/**
 * Options for HTML sanitization - allows safe formatting tags only
 */
const HTML_SANITIZE_OPTIONS: sanitizeHtmlLib.IOptions = {
  allowedTags: ['b', 'i', 'em', 'strong', 'u', 'br', 'p', 'ul', 'ol', 'li', 'code', 'pre'],
  allowedAttributes: {},
  allowedSchemes: ['http', 'https'],
  disallowedTagsMode: 'discard',
  selfClosing: ['br'],
  excludeTags: ['script', 'style', 'iframe', 'object', 'embed', 'form', 'input', 'textarea'],
  textFilter: (text) => text.replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>')
}

/**
 * Sanitize HTML content - allows only safe formatting tags
 */
export function sanitizeHtml(dirty: string): string {
  return sanitizeHtmlLib(dirty, HTML_SANITIZE_OPTIONS)
}

/**
 * Sanitize plain text - strips all HTML tags and trims whitespace
 */
export function sanitizeText(dirty: string): string {
  return dirty.replace(/<[^>]*>/g, '').trim()
}

/**
 * Question payload interface
 */
export interface QuestionPayload {
  id?: number
  text: string
  explanation?: string
  order?: number
  type?: 'multiple-choice' | 'open-ended' | 'true-false'
  options?: Array<{
    id?: number
    text: string
    isCorrect?: boolean
    order?: number
  }>
}

/**
 * Sanitize a question payload to prevent XSS
 *
 * Sanitizes:
 * - question text (HTML allowed but sanitized)
 * - explanation (HTML allowed but sanitized)
 * - option texts (HTML allowed but sanitized)
 *
 * @example
 * const cleanPayload = sanitizeQuestionPayload({
 *   text: '<script>alert("xss")</script>What is 2+2?',
 *   explanation: 'Basic <b>arithmetic</b>',
 *   options: [{ text: '<b>4</b>', isCorrect: true }]
 * })
 * // Returns: { text: 'What is 2+2?', explanation: '<b>arithmetic</b>', options: [{ text: '<b>4</b>', isCorrect: true }] }
 */
export function sanitizeQuestionPayload(payload: QuestionPayload): QuestionPayload {
  const sanitized: QuestionPayload = {
    text: sanitizeHtml(payload.text),
    type: payload.type,
    order: payload.order
  }

  if (payload.id !== undefined) {
    sanitized.id = payload.id
  }

  if (payload.explanation !== undefined) {
    sanitized.explanation = sanitizeHtml(payload.explanation)
  }

  if (payload.options && Array.isArray(payload.options)) {
    sanitized.options = payload.options.map((option, index) => ({
      id: option.id,
      text: sanitizeHtml(option.text),
      isCorrect: option.isCorrect,
      order: option.order ?? index
    }))
  }

  return sanitized
}

/**
 * Assignment payload interface
 */
export interface AssignmentPayload {
  id?: number
  title: string
  description?: string
  instructions?: string
  dueDate?: string
  maxScore?: number
}

/**
 * Sanitize an assignment payload to prevent XSS
 */
export function sanitizeAssignmentPayload(payload: AssignmentPayload): AssignmentPayload {
  const sanitized: AssignmentPayload = {
    title: sanitizeText(payload.title),
    maxScore: payload.maxScore
  }

  if (payload.id !== undefined) {
    sanitized.id = payload.id
  }

  if (payload.description !== undefined) {
    sanitized.description = sanitizeHtml(payload.description)
  }

  if (payload.instructions !== undefined) {
    sanitized.instructions = sanitizeHtml(payload.instructions)
  }

  if (payload.dueDate !== undefined) {
    sanitized.dueDate = payload.dueDate
  }

  return sanitized
}

/**
 * Blog post payload interface
 */
export interface BlogPostPayload {
  id?: number
  title: string
  slug?: string
  content: string
  excerpt?: string
  authorId?: number
  publishedAt?: string
  tags?: string[]
}

/**
 * Sanitize a blog post payload to prevent XSS
 */
export function sanitizeBlogPostPayload(payload: BlogPostPayload): BlogPostPayload {
  const sanitized: BlogPostPayload = {
    title: sanitizeText(payload.title),
    content: sanitizeHtml(payload.content)
  }

  if (payload.id !== undefined) {
    sanitized.id = payload.id
  }

  if (payload.slug !== undefined) {
    sanitized.slug = payload.slug
  }

  if (payload.excerpt !== undefined) {
    sanitized.excerpt = sanitizeHtml(payload.excerpt)
  }

  if (payload.authorId !== undefined) {
    sanitized.authorId = payload.authorId
  }

  if (payload.publishedAt !== undefined) {
    sanitized.publishedAt = payload.publishedAt
  }

  if (payload.tags && Array.isArray(payload.tags)) {
    sanitized.tags = payload.tags.map(tag => sanitizeText(tag))
  }

  return sanitized
}

/**
 * Contact form payload interface
 */
export interface ContactPayload {
  name: string
  email: string
  subject?: string
  message: string
}

/**
 * Sanitize a contact form payload to prevent XSS
 */
export function sanitizeContactPayload(payload: ContactPayload): ContactPayload {
  return {
    name: sanitizeText(payload.name),
    email: payload.email.trim().toLowerCase(),
    subject: payload.subject ? sanitizeText(payload.subject) : undefined,
    message: sanitizeHtml(payload.message)
  }
}

/**
 * Testimonial payload interface
 */
export interface TestimonialPayload {
  id?: number
  author: string
  role?: string
  content: string
  rating?: number
  companyId?: number
}

/**
 * Sanitize a testimonial payload to prevent XSS
 */
export function sanitizeTestimonialPayload(payload: TestimonialPayload): TestimonialPayload {
  const sanitized: TestimonialPayload = {
    author: sanitizeText(payload.author),
    content: sanitizeHtml(payload.content)
  }

  if (payload.id !== undefined) {
    sanitized.id = payload.id
  }

  if (payload.role !== undefined) {
    sanitized.role = sanitizeText(payload.role)
  }

  if (payload.rating !== undefined) {
    sanitized.rating = payload.rating
  }

  if (payload.companyId !== undefined) {
    sanitized.companyId = payload.companyId
  }

  return sanitized
}

/**
 * Validate email format (basic check)
 */
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(email)
}

/**
 * Validate URL format (basic check)
 */
export function isValidUrl(url: string): boolean {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

/**
 * Sanitize and validate a file name (prevent path traversal)
 */
export function sanitizeFileName(fileName: string): string {
  // Remove path components
  const baseName = fileName.split(/[\\/]/).pop() || 'unnamed'

  // Remove special characters that could be dangerous
  const sanitized = baseName.replace(/[^a-zA-Z0-9._-]/g, '_')

  // Limit length
  return sanitized.slice(0, 255)
}
