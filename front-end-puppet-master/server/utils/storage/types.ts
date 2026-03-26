/**
 * Storage Adapter Types
 *
 * Abstract interface for file storage operations.
 * Implementations: LocalStorage, S3Storage
 */

export interface UploadResult {
  id: string
  url: string
  thumbnailUrl?: string
  originalName?: string
  size: number
  mimeType: string
  width?: number
  height?: number
  duration?: number // For videos, in seconds
}

export interface StorageAdapter {
  /**
   * Upload a file (image or video)
   * @param buffer - File data buffer
   * @param options - Upload options
   * @returns Upload result with URLs
   */
  upload(buffer: Buffer, options: UploadOptions): Promise<UploadResult>

  /**
   * Delete a file by its ID
   * @param id - File ID (UUID)
   */
  delete(id: string): Promise<void>

  /**
   * Get public URL for a file
   * @param id - File ID
   * @param variant - 'original' | 'thumbnail'
   */
  getUrl(id: string, variant?: 'original' | 'thumbnail'): string
}

export interface UploadOptions {
  type: 'image' | 'video'
  originalName?: string
  mimeType: string
}

export interface ImageProcessingOptions {
  maxWidth: number
  maxHeight: number
  quality: number
  thumbnailWidth: number
  thumbnailHeight: number
  thumbnailQuality: number
}

export interface VideoProcessingOptions {
  maxWidth: number
  videoBitrate: string
  audioBitrate: string
  fps: number
  outputFormat: 'mp4' | 'webm'
}
