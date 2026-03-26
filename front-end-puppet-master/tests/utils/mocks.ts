/**
 * Test Mocks
 *
 * Mocking utilities for external dependencies and services.
 * Use these to isolate tests from external systems.
 */

import { vi } from 'vitest'

/**
 * Mock H3 event for testing API handlers
 */
export function createMockEvent(options: {
  method?: string
  path?: string
  body?: unknown
  headers?: Record<string, string>
  cookies?: Record<string, string>
  query?: Record<string, string>
} = {}) {
  const {
    method = 'GET',
    path = '/',
    body = null,
    headers = {},
    cookies = {},
    query = {}
  } = options

  return {
    node: {
      req: {
        method,
        url: path,
        headers: {
          ...headers,
          cookie: Object.entries(cookies)
            .map(([k, v]) => `${k}=${v}`)
            .join('; ')
        },
        socket: {
          remoteAddress: '127.0.0.1'
        }
      },
      res: {
        statusCode: 200,
        setHeader: vi.fn(),
        getHeader: vi.fn()
      }
    },
    context: {
      session: null,
      user: null
    },
    _body: body,
    _query: query,
    path
  }
}

/**
 * Mock authenticated event (with session context)
 */
export function createAuthenticatedMockEvent(
  userId: number,
  options: Parameters<typeof createMockEvent>[0] = {}
) {
  const event = createMockEvent(options)
  event.context.session = { userId }
  event.context.user = {
    id: userId,
    email: `user-${userId}@example.com`,
    name: `User ${userId}`,
    role: 'editor',
    roleId: null
  }
  return event
}

/**
 * Mock master user event
 */
export function createMasterMockEvent(options: Parameters<typeof createMockEvent>[0] = {}) {
  const event = createAuthenticatedMockEvent(1, options)
  event.context.user!.role = 'master'
  event.context.user!.email = 'master@example.com'
  return event
}

/**
 * Mock admin user event
 */
export function createAdminMockEvent(options: Parameters<typeof createMockEvent>[0] = {}) {
  const event = createAuthenticatedMockEvent(2, options)
  event.context.user!.role = 'admin'
  event.context.user!.email = 'admin@example.com'
  return event
}

/**
 * Mock email transport
 */
export function createMockEmailTransport() {
  return {
    sendMail: vi.fn().mockResolvedValue({ messageId: 'mock-message-id' })
  }
}

/**
 * Mock S3 client
 */
export function createMockS3Client() {
  return {
    send: vi.fn().mockResolvedValue({}),
    putObject: vi.fn().mockResolvedValue({}),
    getObject: vi.fn().mockResolvedValue({ Body: Buffer.from('mock-content') }),
    deleteObject: vi.fn().mockResolvedValue({})
  }
}

/**
 * Mock file upload
 */
export function createMockFileUpload(options: {
  filename?: string
  mimetype?: string
  size?: number
  content?: Buffer
} = {}) {
  const {
    filename = 'test-file.txt',
    mimetype = 'text/plain',
    size = 1024,
    content = Buffer.from('test content')
  } = options

  return {
    filename,
    mimetype,
    size,
    data: content,
    toBuffer: () => Promise.resolve(content)
  }
}

/**
 * Mock image file
 */
export function createMockImageFile(options: {
  filename?: string
  width?: number
  height?: number
} = {}) {
  const { filename = 'test-image.png', width = 800, height = 600 } = options

  // Create a minimal valid PNG header
  const pngHeader = Buffer.from([
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a
  ])

  return createMockFileUpload({
    filename,
    mimetype: 'image/png',
    size: pngHeader.length,
    content: pngHeader
  })
}

/**
 * Mock database transaction
 */
export function createMockTransaction() {
  const operations: Array<{ type: string; args: unknown[] }> = []

  return {
    insert: vi.fn((...args) => {
      operations.push({ type: 'insert', args })
      return { values: vi.fn().mockReturnThis(), run: vi.fn() }
    }),
    update: vi.fn((...args) => {
      operations.push({ type: 'update', args })
      return { set: vi.fn().mockReturnThis(), where: vi.fn().mockReturnThis(), run: vi.fn() }
    }),
    delete: vi.fn((...args) => {
      operations.push({ type: 'delete', args })
      return { where: vi.fn().mockReturnThis(), run: vi.fn() }
    }),
    select: vi.fn((...args) => {
      operations.push({ type: 'select', args })
      return {
        from: vi.fn().mockReturnThis(),
        where: vi.fn().mockReturnThis(),
        get: vi.fn(),
        all: vi.fn().mockReturnValue([])
      }
    }),
    getOperations: () => operations,
    clearOperations: () => operations.length = 0
  }
}

/**
 * Mock rate limiter
 */
export function createMockRateLimiter() {
  let allowedRequests = 100

  return {
    checkRateLimit: vi.fn(() => {
      if (allowedRequests > 0) {
        allowedRequests--
        return true
      }
      return false
    }),
    reset: () => { allowedRequests = 100 },
    setAllowedRequests: (n: number) => { allowedRequests = n },
    exhaust: () => { allowedRequests = 0 }
  }
}

/**
 * Mock console for suppressing logs in tests
 */
export function mockConsole() {
  const originalConsole = { ...console }

  beforeEach(() => {
    console.log = vi.fn()
    console.warn = vi.fn()
    console.error = vi.fn()
    console.info = vi.fn()
  })

  afterEach(() => {
    Object.assign(console, originalConsole)
  })

  return {
    getLogs: () => (console.log as ReturnType<typeof vi.fn>).mock.calls,
    getWarns: () => (console.warn as ReturnType<typeof vi.fn>).mock.calls,
    getErrors: () => (console.error as ReturnType<typeof vi.fn>).mock.calls
  }
}

/**
 * Mock date/time
 */
export function mockDate(date: Date | string | number) {
  const fixedDate = new Date(date)
  vi.useFakeTimers()
  vi.setSystemTime(fixedDate)

  return {
    restore: () => vi.useRealTimers(),
    advance: (ms: number) => vi.advanceTimersByTime(ms),
    advanceToNextTimer: () => vi.advanceTimersToNextTimer()
  }
}

/**
 * Mock fetch globally
 */
export function mockGlobalFetch() {
  const mockResponses = new Map<string, { status: number; body: unknown }>()

  global.fetch = vi.fn((url: string | URL | Request) => {
    const urlString = url.toString()
    const response = mockResponses.get(urlString)

    if (response) {
      return Promise.resolve({
        ok: response.status >= 200 && response.status < 300,
        status: response.status,
        json: () => Promise.resolve(response.body),
        text: () => Promise.resolve(JSON.stringify(response.body))
      } as Response)
    }

    return Promise.resolve({
      ok: false,
      status: 404,
      json: () => Promise.resolve({ error: 'Not found' }),
      text: () => Promise.resolve('Not found')
    } as Response)
  })

  return {
    setResponse: (url: string, status: number, body: unknown) => {
      mockResponses.set(url, { status, body })
    },
    clearResponses: () => mockResponses.clear(),
    getCalls: () => (global.fetch as ReturnType<typeof vi.fn>).mock.calls
  }
}
