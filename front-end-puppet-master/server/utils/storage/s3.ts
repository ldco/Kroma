/**
 * S3 Storage Adapter
 *
 * Stores files in S3-compatible bucket (AWS S3, Cloudflare R2, MinIO).
 * For production sites with CDN needs or large media libraries.
 */
import { S3Client, PutObjectCommand, DeleteObjectCommand } from '@aws-sdk/client-s3'
import { randomUUID } from 'crypto'
import sharp from 'sharp'
import { buildS3ObjectUrl } from './s3-private-url'
import type {
  StorageAdapter,
  UploadResult,
  UploadOptions,
  ImageProcessingOptions,
  VideoProcessingOptions
} from './types'

export interface S3RuntimeConfig {
  endpoint: string
  accessKeyId: string
  secretAccessKey: string
  region: string
  bucket: string
}

export function getS3RuntimeConfig(): S3RuntimeConfig {
  const endpoint = process.env.S3_ENDPOINT
  const accessKeyId = process.env.S3_ACCESS_KEY
  const secretAccessKey = process.env.S3_SECRET_KEY
  const region = process.env.S3_REGION || 'auto'

  if (!endpoint || !accessKeyId || !secretAccessKey) {
    throw new Error(
      'S3 credentials not configured. Set S3_ENDPOINT, S3_ACCESS_KEY, S3_SECRET_KEY in .env'
    )
  }

  return {
    endpoint,
    accessKeyId,
    secretAccessKey,
    region,
    bucket: process.env.S3_BUCKET || 'uploads'
  }
}

export function createS3Client(config: S3RuntimeConfig): S3Client {
  return new S3Client({
    endpoint: config.endpoint,
    region: config.region,
    credentials: {
      accessKeyId: config.accessKeyId,
      secretAccessKey: config.secretAccessKey
    },
    forcePathStyle: true // Required for MinIO and some S3-compatible services
  })
}

export function createS3ClientFromEnv(): { client: S3Client; bucket: string } {
  const runtimeConfig = getS3RuntimeConfig()
  return {
    client: createS3Client(runtimeConfig),
    bucket: runtimeConfig.bucket
  }
}

export class S3Storage implements StorageAdapter {
  private client: S3Client
  private bucket: string
  private imageOptions: ImageProcessingOptions
  private videoOptions: VideoProcessingOptions

  constructor(imageOptions: ImageProcessingOptions, videoOptions: VideoProcessingOptions) {
    this.imageOptions = imageOptions
    this.videoOptions = videoOptions

    const runtimeConfig = getS3RuntimeConfig()
    this.bucket = runtimeConfig.bucket
    this.client = createS3Client(runtimeConfig)
  }

  async upload(buffer: Buffer, options: UploadOptions): Promise<UploadResult> {
    const id = randomUUID()

    if (options.type === 'image') {
      return this.uploadImage(buffer, id, options)
    } else {
      return this.uploadVideo(buffer, id, options)
    }
  }

  private async uploadImage(
    buffer: Buffer,
    id: string,
    options: UploadOptions
  ): Promise<UploadResult> {
    const { maxWidth, maxHeight, quality, thumbnailWidth, thumbnailHeight, thumbnailQuality } =
      this.imageOptions

    // Get original metadata
    const metadata = await sharp(buffer).metadata()

    // Process main image
    const mainImage = await sharp(buffer)
      .resize(maxWidth, maxHeight, { fit: 'inside', withoutEnlargement: true })
      .webp({ quality })
      .toBuffer()

    // Generate thumbnail
    const thumbnail = await sharp(buffer)
      .resize(thumbnailWidth, thumbnailHeight, { fit: 'cover', position: 'center' })
      .webp({ quality: thumbnailQuality })
      .toBuffer()

    // Upload to S3
    await Promise.all([
      this.putObject(`${id}.webp`, mainImage, 'image/webp'),
      this.putObject(`${id}-thumb.webp`, thumbnail, 'image/webp')
    ])

    return {
      id,
      url: buildS3ObjectUrl(`${id}.webp`),
      thumbnailUrl: buildS3ObjectUrl(`${id}-thumb.webp`),
      originalName: options.originalName,
      size: mainImage.length,
      mimeType: 'image/webp',
      width: metadata.width,
      height: metadata.height
    }
  }

  private async uploadVideo(
    buffer: Buffer,
    id: string,
    options: UploadOptions
  ): Promise<UploadResult> {
    const ext = this.videoOptions.outputFormat
    const key = `${id}.${ext}`
    const contentType = ext === 'mp4' ? 'video/mp4' : 'video/webm'

    await this.putObject(key, buffer, contentType)

    return {
      id,
      url: buildS3ObjectUrl(key),
      originalName: options.originalName,
      size: buffer.length,
      mimeType: contentType
    }
  }

  private async putObject(key: string, buffer: Buffer, contentType: string): Promise<void> {
    await this.client.send(
      new PutObjectCommand({
        Bucket: this.bucket,
        Key: key,
        Body: buffer,
        ContentType: contentType
      })
    )
  }

  async delete(id: string): Promise<void> {
    const keys = [`${id}.webp`, `${id}-thumb.webp`, `${id}.mp4`, `${id}.webm`, `${id}-thumb.jpg`]

    await Promise.all(
      keys.map(key =>
        this.client.send(new DeleteObjectCommand({ Bucket: this.bucket, Key: key })).catch(() => {
          /* Ignore if file doesn't exist */
        })
      )
    )
  }

  getUrl(id: string, variant: 'original' | 'thumbnail' = 'original'): string {
    const suffix = variant === 'thumbnail' ? '-thumb' : ''
    return buildS3ObjectUrl(`${id}${suffix}.webp`)
  }
}
