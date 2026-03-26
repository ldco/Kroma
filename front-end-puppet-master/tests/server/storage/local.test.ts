import { describe, it, expect } from 'vitest'
import { LocalStorage } from '../../../server/utils/storage/local'

// Create instance with default options for testing
const defaultImageOptions = {
  maxWidth: 1920,
  maxHeight: 1080,
  quality: 85,
  thumbnailWidth: 300,
  thumbnailHeight: 300,
  thumbnailQuality: 75
}

const defaultVideoOptions = {
  maxWidth: 1920,
  videoBitrate: '2M',
  audioBitrate: '128k',
  fps: 30,
  outputFormat: 'mp4' as const
}

describe('LocalStorage.getUrl', () => {
  const storage = new LocalStorage(defaultImageOptions, defaultVideoOptions)

  describe('original variant', () => {
    it('should return correct URL for original image', () => {
      expect(storage.getUrl('abc123', 'original')).toBe('/uploads/abc123.webp')
    })

    it('should default to original when no variant specified', () => {
      expect(storage.getUrl('abc123')).toBe('/uploads/abc123.webp')
    })

    it('should handle UUID-style IDs', () => {
      const uuid = '550e8400-e29b-41d4-a716-446655440000'
      expect(storage.getUrl(uuid)).toBe(`/uploads/${uuid}.webp`)
    })
  })

  describe('thumbnail variant', () => {
    it('should return correct URL for thumbnail', () => {
      expect(storage.getUrl('abc123', 'thumbnail')).toBe('/uploads/abc123-thumb.webp')
    })

    it('should handle UUID-style IDs for thumbnails', () => {
      const uuid = '550e8400-e29b-41d4-a716-446655440000'
      expect(storage.getUrl(uuid, 'thumbnail')).toBe(`/uploads/${uuid}-thumb.webp`)
    })
  })

  describe('edge cases', () => {
    it('should handle empty ID', () => {
      expect(storage.getUrl('')).toBe('/uploads/.webp')
    })

    it('should handle ID with special characters', () => {
      // This shouldn't happen in practice, but good to test
      expect(storage.getUrl('file-name_123')).toBe('/uploads/file-name_123.webp')
    })
  })
})
