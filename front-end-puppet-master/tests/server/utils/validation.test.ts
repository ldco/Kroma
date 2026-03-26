import { describe, it, expect } from 'vitest'
import {
  loginSchema,
  contactSchema,
  createUserSchema,
  portfolioItemSchema,
  validateInput
} from '../../../server/utils/validation'

describe('loginSchema', () => {
  it('should accept valid login input', () => {
    const result = loginSchema.safeParse({
      email: 'user@example.com',
      password: 'password123'
    })
    expect(result.success).toBe(true)
  })

  it('should accept login with rememberMe', () => {
    const result = loginSchema.safeParse({
      email: 'user@example.com',
      password: 'password123',
      rememberMe: true
    })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.rememberMe).toBe(true)
    }
  })

  it('should default rememberMe to false', () => {
    const result = loginSchema.safeParse({
      email: 'user@example.com',
      password: 'password123'
    })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.rememberMe).toBe(false)
    }
  })

  it('should reject invalid email', () => {
    const result = loginSchema.safeParse({
      email: 'not-an-email',
      password: 'password123'
    })
    expect(result.success).toBe(false)
  })

  it('should reject empty password', () => {
    const result = loginSchema.safeParse({
      email: 'user@example.com',
      password: ''
    })
    expect(result.success).toBe(false)
  })
})

describe('contactSchema', () => {
  it('should accept valid contact input', () => {
    const result = contactSchema.safeParse({
      name: 'John Doe',
      email: 'john@example.com',
      message: 'Hello, this is a test message.'
    })
    expect(result.success).toBe(true)
  })

  it('should accept contact with optional fields', () => {
    const result = contactSchema.safeParse({
      name: 'John Doe',
      email: 'john@example.com',
      phone: '+1234567890',
      subject: 'Inquiry',
      message: 'Hello, this is a test message.'
    })
    expect(result.success).toBe(true)
  })

  it('should reject name shorter than 2 characters', () => {
    const result = contactSchema.safeParse({
      name: 'J',
      email: 'john@example.com',
      message: 'Hello, this is a test message.'
    })
    expect(result.success).toBe(false)
  })

  it('should reject message shorter than 10 characters', () => {
    const result = contactSchema.safeParse({
      name: 'John Doe',
      email: 'john@example.com',
      message: 'Hi'
    })
    expect(result.success).toBe(false)
  })

  it('should reject invalid email', () => {
    const result = contactSchema.safeParse({
      name: 'John Doe',
      email: 'invalid',
      message: 'Hello, this is a test message.'
    })
    expect(result.success).toBe(false)
  })
})

describe('createUserSchema', () => {
  it('should accept valid user creation input', () => {
    const result = createUserSchema.safeParse({
      email: 'user@example.com',
      password: 'password123',
      role: 'editor'
    })
    expect(result.success).toBe(true)
  })

  it('should accept all valid roles', () => {
    for (const role of ['master', 'admin', 'editor']) {
      const result = createUserSchema.safeParse({
        email: 'user@example.com',
        password: 'password123',
        role
      })
      expect(result.success).toBe(true)
    }
  })

  it('should reject invalid role', () => {
    const result = createUserSchema.safeParse({
      email: 'user@example.com',
      password: 'password123',
      role: 'superadmin'
    })
    expect(result.success).toBe(false)
  })

  it('should reject password shorter than 8 characters', () => {
    const result = createUserSchema.safeParse({
      email: 'user@example.com',
      password: 'short',
      role: 'editor'
    })
    expect(result.success).toBe(false)
  })
})

describe('portfolioItemSchema', () => {
  it('should accept valid portfolio item', () => {
    const result = portfolioItemSchema.safeParse({
      slug: 'my-project',
      title: 'My Project'
    })
    expect(result.success).toBe(true)
  })

  it('should reject slug with uppercase', () => {
    const result = portfolioItemSchema.safeParse({
      slug: 'My-Project',
      title: 'My Project'
    })
    expect(result.success).toBe(false)
  })
})

describe('validateInput helper', () => {
  it('should return success with data for valid input', () => {
    const result = validateInput(loginSchema, {
      email: 'user@example.com',
      password: 'password123'
    })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.email).toBe('user@example.com')
    }
  })

  it('should return errors array for invalid input', () => {
    const result = validateInput(loginSchema, {
      email: 'invalid',
      password: ''
    })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.errors.length).toBeGreaterThan(0)
    }
  })
})
