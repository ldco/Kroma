/**
 * Storage Factory
 *
 * Returns the appropriate storage adapter based on config.
 * Usage: const storage = useFileStorage()
 */
import config from '../../../app/puppet-master.config'
import { LocalStorage } from './local'
import { S3Storage } from './s3'
import type { StorageAdapter, ImageProcessingOptions, VideoProcessingOptions } from './types'

let storageInstance: StorageAdapter | null = null

/**
 * Get storage adapter singleton
 * Reads provider from config.storage.provider
 * Named useFileStorage to avoid conflict with Nitro's useStorage
 */
export function useFileStorage(): StorageAdapter {
  if (storageInstance) {
    return storageInstance
  }

  const { storage } = config

  const imageOptions: ImageProcessingOptions = {
    maxWidth: storage.image.maxWidth,
    maxHeight: storage.image.maxHeight,
    quality: storage.image.quality,
    thumbnailWidth: storage.image.thumbnailWidth,
    thumbnailHeight: storage.image.thumbnailHeight,
    thumbnailQuality: storage.image.thumbnailQuality
  }

  const videoOptions: VideoProcessingOptions = {
    maxWidth: storage.video.compression.maxWidth,
    videoBitrate: storage.video.compression.videoBitrate,
    audioBitrate: storage.video.compression.audioBitrate,
    fps: storage.video.compression.fps,
    outputFormat: storage.video.outputFormat
  }

  if (storage.provider === 's3') {
    storageInstance = new S3Storage(imageOptions, videoOptions)
  } else {
    storageInstance = new LocalStorage(imageOptions, videoOptions)
  }

  return storageInstance
}
