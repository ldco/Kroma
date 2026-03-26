import { GetObjectCommand } from '@aws-sdk/client-s3'
import { sendStream } from 'h3'
import { Readable } from 'node:stream'
import { createS3ClientFromEnv } from '../../../utils/storage/s3'
import {
  decodeS3ObjectKeyFromPath,
  getS3VisibilityMode,
  verifyS3ObjectKeySignature
} from '../../../utils/storage/s3-private-url'

function toNodeReadable(body: unknown): Readable | null {
  if (!body) {
    return null
  }

  if (body instanceof Readable) {
    return body
  }

  const webStreamBody = body as { transformToWebStream?: () => ReadableStream<Uint8Array> }
  if (typeof webStreamBody.transformToWebStream === 'function') {
    return Readable.fromWeb(webStreamBody.transformToWebStream())
  }

  return null
}

export default defineEventHandler(async event => {
  if (getS3VisibilityMode() !== 'private') {
    throw createError({
      statusCode: 404,
      statusMessage: 'Not found'
    })
  }

  const rawKey = event.context.params?.key
  if (!rawKey) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Missing object key'
    })
  }

  const signature = getQuery(event).sig
  if (typeof signature !== 'string' || signature.length === 0) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Missing media signature'
    })
  }

  const key = decodeS3ObjectKeyFromPath(rawKey)
  if (!verifyS3ObjectKeySignature(key, signature)) {
    throw createError({
      statusCode: 403,
      statusMessage: 'Invalid media signature'
    })
  }

  const { client, bucket } = createS3ClientFromEnv()
  const range = getHeader(event, 'range')

  try {
    const object = await client.send(
      new GetObjectCommand({
        Bucket: bucket,
        Key: key,
        ...(range ? { Range: range } : {})
      })
    )

    if (object.ContentType) setHeader(event, 'Content-Type', object.ContentType)
    if (object.ContentLength !== undefined) {
      setHeader(event, 'Content-Length', String(object.ContentLength))
    }
    if (object.ETag) setHeader(event, 'ETag', object.ETag)
    if (object.LastModified) setHeader(event, 'Last-Modified', object.LastModified.toUTCString())
    if (object.AcceptRanges) setHeader(event, 'Accept-Ranges', object.AcceptRanges)
    if (object.ContentRange) {
      setHeader(event, 'Content-Range', object.ContentRange)
      setResponseStatus(event, 206)
    }
    setHeader(event, 'Cache-Control', 'private, max-age=300')

    const stream = toNodeReadable(object.Body)
    if (!stream) {
      throw createError({
        statusCode: 502,
        statusMessage: 'S3 response body is not streamable'
      })
    }

    return sendStream(event, stream)
  } catch (error) {
    const statusCode = (error as { $metadata?: { httpStatusCode?: number } })?.$metadata
      ?.httpStatusCode
    const errorName = (error as { name?: string })?.name

    if (statusCode === 404 || errorName === 'NoSuchKey') {
      throw createError({
        statusCode: 404,
        statusMessage: 'Media not found'
      })
    }

    throw createError({
      statusCode: 502,
      statusMessage: 'Failed to fetch media from storage'
    })
  }
})
