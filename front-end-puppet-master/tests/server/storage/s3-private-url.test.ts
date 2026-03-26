import { afterEach, describe, expect, it } from 'vitest'
import {
  buildPrivateS3ProxyUrl,
  buildS3ObjectUrl,
  decodeS3ObjectKeyFromPath,
  encodeS3ObjectKeyForPath,
  getS3VisibilityMode,
  signS3ObjectKey,
  verifyS3ObjectKeySignature
} from '../../../server/utils/storage/s3-private-url'

const originalEnv = {
  S3_VISIBILITY: process.env.S3_VISIBILITY,
  S3_PUBLIC_URL: process.env.S3_PUBLIC_URL,
  S3_SECRET_KEY: process.env.S3_SECRET_KEY,
  S3_PROXY_SIGNING_KEY: process.env.S3_PROXY_SIGNING_KEY
}

afterEach(() => {
  process.env.S3_VISIBILITY = originalEnv.S3_VISIBILITY
  process.env.S3_PUBLIC_URL = originalEnv.S3_PUBLIC_URL
  process.env.S3_SECRET_KEY = originalEnv.S3_SECRET_KEY
  process.env.S3_PROXY_SIGNING_KEY = originalEnv.S3_PROXY_SIGNING_KEY
})

describe('s3-private-url', () => {
  it('defaults to private visibility when explicit visibility is not set', () => {
    delete process.env.S3_VISIBILITY
    delete process.env.S3_PUBLIC_URL
    expect(getS3VisibilityMode()).toBe('private')
  })

  it('switches to public visibility when public URL exists and visibility is not set', () => {
    delete process.env.S3_VISIBILITY
    process.env.S3_PUBLIC_URL = 'https://cdn.example.com'
    expect(getS3VisibilityMode()).toBe('public')
  })

  it('builds public object URL in public mode', () => {
    process.env.S3_VISIBILITY = 'public'
    process.env.S3_PUBLIC_URL = 'https://cdn.example.com/'

    expect(buildS3ObjectUrl('media/example.webp')).toBe('https://cdn.example.com/media/example.webp')
  })

  it('builds signed private proxy URL in private mode', () => {
    process.env.S3_VISIBILITY = 'private'
    process.env.S3_SECRET_KEY = 'test-secret'

    const url = buildS3ObjectUrl('media/example.webp')
    expect(url.startsWith('/api/media/s3/media/example.webp?sig=')).toBe(true)
  })

  it('encodes and decodes nested object keys', () => {
    const key = 'folder name/image file.webp'
    const encoded = encodeS3ObjectKeyForPath(key)

    expect(encoded).toBe('folder%20name/image%20file.webp')
    expect(decodeS3ObjectKeyFromPath(encoded)).toBe(key)
  })

  it('validates signatures with timing-safe comparison', () => {
    process.env.S3_SECRET_KEY = 'test-secret'

    const key = 'private/path/file.webp'
    const signature = signS3ObjectKey(key)

    expect(verifyS3ObjectKeySignature(key, signature)).toBe(true)
    expect(verifyS3ObjectKeySignature(key, `${signature.slice(0, -1)}0`)).toBe(false)
  })

  it('rejects invalid object key patterns', () => {
    process.env.S3_SECRET_KEY = 'test-secret'
    expect(() => buildPrivateS3ProxyUrl('../etc/passwd')).toThrow('Invalid S3 object key')
  })
})
