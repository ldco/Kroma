/**
 * Local Storage Adapter
 *
 * Stores files in public/uploads/ directory.
 * Good for small/medium sites without S3 infrastructure.
 */
import { writeFile, unlink, mkdir } from 'fs/promises'
import { existsSync } from 'fs'
import { randomUUID } from 'crypto'
import sharp from 'sharp'
import type {
  StorageAdapter,
  UploadResult,
  UploadOptions,
  ImageProcessingOptions,
  VideoProcessingOptions
} from './types'

const UPLOAD_DIR = './public/uploads'

export class LocalStorage implements StorageAdapter {
  private imageOptions: ImageProcessingOptions
  private videoOptions: VideoProcessingOptions

  constructor(imageOptions: ImageProcessingOptions, videoOptions: VideoProcessingOptions) {
    this.imageOptions = imageOptions
    this.videoOptions = videoOptions
  }

  async upload(buffer: Buffer, options: UploadOptions): Promise<UploadResult> {
    // Ensure upload directory exists
    if (!existsSync(UPLOAD_DIR)) {
      await mkdir(UPLOAD_DIR, { recursive: true })
    }

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

    // Process main image: resize + convert to WebP
    const mainImage = await sharp(buffer)
      .resize(maxWidth, maxHeight, {
        fit: 'inside',
        withoutEnlargement: true
      })
      .webp({ quality })
      .toBuffer()

    // Generate thumbnail
    const thumbnail = await sharp(buffer)
      .resize(thumbnailWidth, thumbnailHeight, {
        fit: 'cover',
        position: 'center'
      })
      .webp({ quality: thumbnailQuality })
      .toBuffer()

    // Save files
    await writeFile(`${UPLOAD_DIR}/${id}.webp`, mainImage)
    await writeFile(`${UPLOAD_DIR}/${id}-thumb.webp`, thumbnail)

    return {
      id,
      url: `/uploads/${id}.webp`,
      thumbnailUrl: `/uploads/${id}-thumb.webp`,
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
    // Video processing is handled separately due to FFmpeg complexity
    // This is a placeholder - actual implementation in video.post.ts
    const ext = this.videoOptions.outputFormat
    const outputPath = `${UPLOAD_DIR}/${id}.${ext}`

    // For now, save raw file - FFmpeg processing done in upload endpoint
    await writeFile(outputPath, buffer)

    return {
      id,
      url: `/uploads/${id}.${ext}`,
      originalName: options.originalName,
      size: buffer.length,
      mimeType: options.mimeType
    }
  }

  async delete(id: string): Promise<void> {
    const extensions = ['webp', 'mp4', 'webm']
    const variants = ['', '-thumb']

    for (const ext of extensions) {
      for (const variant of variants) {
        const path = `${UPLOAD_DIR}/${id}${variant}.${ext}`
        if (existsSync(path)) {
          await unlink(path)
        }
      }
    }
  }

  getUrl(id: string, variant: 'original' | 'thumbnail' = 'original'): string {
    const suffix = variant === 'thumbnail' ? '-thumb' : ''
    // Default to webp, but could be mp4/webm for videos
    return `/uploads/${id}${suffix}.webp`
  }
}
