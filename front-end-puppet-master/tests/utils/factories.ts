/**
 * Test Data Factories
 *
 * Provides functions to generate test data for users, roles, posts, etc.
 * Uses consistent patterns for predictable testing.
 */

import type {
  NewUser,
  NewRole,
  NewBlogPost,
  NewContactSubmission,
  NewTeamMember
} from '../../server/database/schema'

let userIdCounter = 1000
let roleIdCounter = 100
let postIdCounter = 500
let contactIdCounter = 200
let teamMemberIdCounter = 300

/**
 * Reset all ID counters (call in beforeEach if needed)
 */
export function resetFactoryCounters(): void {
  userIdCounter = 1000
  roleIdCounter = 100
  postIdCounter = 500
  contactIdCounter = 200
  teamMemberIdCounter = 300
}

/**
 * Create a test user object
 */
export function createTestUser(overrides: Partial<NewUser> = {}): NewUser & { id: number } {
  const id = userIdCounter++
  return {
    id,
    email: `test-user-${id}@example.com`,
    passwordHash: 'hashed_password_placeholder',
    name: `Test User ${id}`,
    role: 'editor',
    roleId: null,
    failedLoginAttempts: 0,
    lockedUntil: null,
    lastFailedLogin: null,
    twoFactorEnabled: false,
    ...overrides
  }
}

/**
 * Create a master user for testing admin operations
 */
export function createMasterUser(overrides: Partial<NewUser> = {}): NewUser & { id: number } {
  return createTestUser({
    email: 'test-master@example.com',
    name: 'Test Master',
    role: 'master',
    ...overrides
  })
}

/**
 * Create an admin user for testing
 */
export function createAdminUser(overrides: Partial<NewUser> = {}): NewUser & { id: number } {
  return createTestUser({
    email: 'test-admin@example.com',
    name: 'Test Admin',
    role: 'admin',
    ...overrides
  })
}

/**
 * Create a test role object
 */
export function createTestRole(overrides: Partial<NewRole> = {}): NewRole & { id: number } {
  const id = roleIdCounter++
  return {
    id,
    name: `Test Role ${id}`,
    slug: `test-role-${id}`,
    description: `Test role description ${id}`,
    permissions: JSON.stringify({ users: false, settings: false }),
    level: 0,
    isBuiltIn: false,
    color: 'secondary',
    icon: 'pencil',
    ...overrides
  }
}

/**
 * Create a built-in role for testing
 */
export function createBuiltInRole(overrides: Partial<NewRole> = {}): NewRole & { id: number } {
  return createTestRole({
    name: 'Built-In Role',
    slug: 'built-in-role',
    isBuiltIn: true,
    ...overrides
  })
}

/**
 * Create a test blog post object
 */
export function createTestBlogPost(overrides: Partial<NewBlogPost> = {}): NewBlogPost & { id: number } {
  const id = postIdCounter++
  return {
    id,
    slug: `test-post-${id}`,
    authorId: null,
    categoryId: null,
    coverImageUrl: null,
    coverImageAlt: null,
    published: false,
    publishedAt: null,
    readingTimeMinutes: 5,
    viewCount: 0,
    ...overrides
  }
}

/**
 * Create a published blog post for testing
 */
export function createPublishedBlogPost(overrides: Partial<NewBlogPost> = {}): NewBlogPost & { id: number } {
  return createTestBlogPost({
    published: true,
    publishedAt: new Date(),
    ...overrides
  })
}

/**
 * Create a test contact submission
 */
export function createTestContact(overrides: Partial<NewContactSubmission> = {}): NewContactSubmission & { id: number } {
  const id = contactIdCounter++
  return {
    id,
    name: `Test Contact ${id}`,
    email: `contact-${id}@example.com`,
    phone: '+1234567890',
    subject: `Test Subject ${id}`,
    message: `This is a test message from contact ${id}. It needs to be at least 10 characters.`,
    read: false,
    ...overrides
  }
}

/**
 * Create a test team member
 */
export function createTestTeamMember(overrides: Partial<NewTeamMember> = {}): NewTeamMember & { id: number } {
  const id = teamMemberIdCounter++
  return {
    id,
    slug: `team-member-${id}`,
    name: `Team Member ${id}`,
    position: 'Developer',
    bio: `Bio for team member ${id}`,
    photoUrl: null,
    hoverPhotoUrl: null,
    email: `team-${id}@example.com`,
    phone: '+1234567890',
    department: 'Engineering',
    socialLinks: null,
    published: true,
    order: id,
    ...overrides
  }
}

/**
 * Create valid login credentials
 */
export function createLoginCredentials(email = 'master@example.com', password = 'master123') {
  return {
    email,
    password,
    rememberMe: false
  }
}

/**
 * Create invalid login credentials
 */
export function createInvalidLoginCredentials() {
  return {
    email: 'invalid@example.com',
    password: 'wrongpassword'
  }
}

/**
 * Create test contact form data
 */
export function createContactFormData(overrides: Record<string, unknown> = {}) {
  return {
    name: 'Test Contact',
    email: 'contact@example.com',
    phone: '+1234567890',
    subject: 'Test Subject',
    message: 'This is a test message that is at least 10 characters long.',
    ...overrides
  }
}

/**
 * Create test pagination params
 */
export function createPaginationParams(page = 1, limit = 10, sort = 'createdAt', order: 'asc' | 'desc' = 'desc') {
  return { page, limit, sort, order }
}
