import { createHmac, timingSafeEqual } from 'node:crypto'

export type S3VisibilityMode = 'public' | 'private'

const S3_PROXY_BASE_PATH = '/api/media/s3'
const MAX_S3_KEY_LENGTH = 1024

function getS3SigningSecret(): string {
  const secret = process.env.S3_PROXY_SIGNING_KEY || process.env.S3_SECRET_KEY
  if (!secret) {
    throw new Error('S3 signing key missing. Set S3_PROXY_SIGNING_KEY or S3_SECRET_KEY.')
  }
  return secret
}

function signRawKey(key: string): string {
  return createHmac('sha256', getS3SigningSecret()).update(key).digest('hex')
}

export function getS3VisibilityMode(): S3VisibilityMode {
  const explicit = process.env.S3_VISIBILITY?.trim().toLowerCase()
  if (!explicit) {
    return process.env.S3_PUBLIC_URL ? 'public' : 'private'
  }

  if (explicit !== 'public' && explicit !== 'private') {
    throw new Error('Invalid S3_VISIBILITY value. Expected "public" or "private".')
  }

  return explicit
}

export function normalizeS3ObjectKey(rawKey: string): string {
  const key = rawKey.trim().replace(/^\/+/, '')

  if (!key) {
    throw new Error('S3 object key is required')
  }

  if (key.length > MAX_S3_KEY_LENGTH) {
    throw new Error('S3 object key is too long')
  }

  if (key.includes('\0') || key.includes('..')) {
    throw new Error('Invalid S3 object key')
  }

  return key
}

export function encodeS3ObjectKeyForPath(key: string): string {
  return normalizeS3ObjectKey(key)
    .split('/')
    .map(segment => encodeURIComponent(segment))
    .join('/')
}

export function decodeS3ObjectKeyFromPath(pathKey: string): string {
  const decoded = pathKey
    .split('/')
    .map(segment => decodeURIComponent(segment))
    .join('/')

  return normalizeS3ObjectKey(decoded)
}

export function signS3ObjectKey(key: string): string {
  return signRawKey(normalizeS3ObjectKey(key))
}

export function verifyS3ObjectKeySignature(key: string, signature: string): boolean {
  if (!signature) {
    return false
  }

  const expected = signRawKey(normalizeS3ObjectKey(key))
  const expectedBuffer = Buffer.from(expected, 'hex')
  const receivedBuffer = Buffer.from(signature, 'hex')

  if (expectedBuffer.length !== receivedBuffer.length) {
    return false
  }

  return timingSafeEqual(expectedBuffer, receivedBuffer)
}

export function getS3PublicBaseUrl(): string {
  const publicUrl = process.env.S3_PUBLIC_URL?.trim().replace(/\/+$/, '')
  if (!publicUrl) {
    throw new Error('S3_PUBLIC_URL is required when S3 visibility mode is "public".')
  }
  return publicUrl
}

export function buildPrivateS3ProxyUrl(key: string): string {
  const normalized = normalizeS3ObjectKey(key)
  const encodedKey = encodeS3ObjectKeyForPath(normalized)
  const signature = signS3ObjectKey(normalized)
  return `${S3_PROXY_BASE_PATH}/${encodedKey}?sig=${signature}`
}

export function buildS3ObjectUrl(key: string): string {
  const normalized = normalizeS3ObjectKey(key)
  const visibility = getS3VisibilityMode()

  if (visibility === 'public') {
    return `${getS3PublicBaseUrl()}/${normalized}`
  }

  return buildPrivateS3ProxyUrl(normalized)
}
