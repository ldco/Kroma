/**
 * File Validation Utility
 *
 * Validates file uploads by checking magic bytes (file signatures).
 * This prevents MIME type spoofing attacks where malicious files
 * are uploaded with fake MIME types.
 *
 * Security note: Never trust client-provided MIME types.
 */

/**
 * File signature type
 */
interface FileSignature {
  signatures: Array<{ bytes: number[]; offset?: number }>
  mime: string
  multiCheck?: boolean
}

/**
 * Known file signatures (magic bytes)
 * Format: { signature: hex bytes, offset?: start position, mime: MIME type }
 */
const FILE_SIGNATURES: Record<string, FileSignature> = {
  // Images
  jpeg: {
    signatures: [
      { bytes: [0xff, 0xd8, 0xff] } // JPEG/JPG
    ],
    mime: 'image/jpeg'
  },
  png: {
    signatures: [
      { bytes: [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a] } // PNG
    ],
    mime: 'image/png'
  },
  gif: {
    signatures: [
      { bytes: [0x47, 0x49, 0x46, 0x38, 0x37, 0x61] }, // GIF87a
      { bytes: [0x47, 0x49, 0x46, 0x38, 0x39, 0x61] } // GIF89a
    ],
    mime: 'image/gif'
  },
  webp: {
    signatures: [
      { bytes: [0x52, 0x49, 0x46, 0x46], offset: 0 }, // RIFF
      { bytes: [0x57, 0x45, 0x42, 0x50], offset: 8 } // WEBP at offset 8
    ],
    mime: 'image/webp',
    multiCheck: true // Both signatures must match
  },

  // Videos
  mp4: {
    signatures: [
      { bytes: [0x66, 0x74, 0x79, 0x70], offset: 4 } // ftyp at offset 4
    ],
    mime: 'video/mp4'
  },
  webm: {
    signatures: [
      { bytes: [0x1a, 0x45, 0xdf, 0xa3] } // WebM/MKV
    ],
    mime: 'video/webm'
  },
  mov: {
    signatures: [
      { bytes: [0x66, 0x74, 0x79, 0x70, 0x71, 0x74], offset: 4 }, // ftypqt (QuickTime)
      { bytes: [0x6d, 0x6f, 0x6f, 0x76], offset: 4 } // moov
    ],
    mime: 'video/quicktime'
  },
  avi: {
    signatures: [
      { bytes: [0x52, 0x49, 0x46, 0x46] }, // RIFF (AVI starts with RIFF)
      { bytes: [0x41, 0x56, 0x49, 0x20], offset: 8 } // AVI  at offset 8
    ],
    mime: 'video/x-msvideo',
    multiCheck: true
  }
}

/**
 * Allowed MIME types for images
 */
const ALLOWED_IMAGE_MIMES = new Set(['image/jpeg', 'image/png', 'image/webp', 'image/gif'])

/**
 * Allowed MIME types for videos
 */
const ALLOWED_VIDEO_MIMES = new Set([
  'video/mp4',
  'video/webm',
  'video/quicktime',
  'video/x-msvideo'
])

interface ValidationResult {
  valid: boolean
  detectedMime: string | null
  error?: string
}

/**
 * Check if buffer matches a signature at a given offset
 */
function matchesSignature(buffer: Buffer, bytes: number[], offset: number = 0): boolean {
  if (buffer.length < offset + bytes.length) return false

  for (let i = 0; i < bytes.length; i++) {
    if (buffer[offset + i] !== bytes[i]) return false
  }
  return true
}

/**
 * Detect file type from buffer using magic bytes
 */
export function detectFileType(buffer: Buffer): string | null {
  for (const [, fileType] of Object.entries(FILE_SIGNATURES)) {
    if (fileType.multiCheck) {
      // All signatures must match (for formats like WebP, AVI)
      const allMatch = fileType.signatures.every(sig =>
        matchesSignature(buffer, sig.bytes, sig.offset || 0)
      )
      if (allMatch) return fileType.mime
    } else {
      // Any signature can match
      for (const sig of fileType.signatures) {
        if (matchesSignature(buffer, sig.bytes, sig.offset || 0)) {
          return fileType.mime
        }
      }
    }
  }

  // Special check for MP4 variants (ftyp can have different brands)
  // ftyp is at offset 4, check for common MP4 brands
  if (buffer.length >= 12) {
    const ftypAt4 = matchesSignature(buffer, [0x66, 0x74, 0x79, 0x70], 4)
    if (ftypAt4) {
      // Check brand at offset 8: isom, mp41, mp42, M4V, etc.
      const brand = buffer.slice(8, 12).toString('ascii')
      const mp4Brands = ['isom', 'iso2', 'mp41', 'mp42', 'M4V ', 'M4A ', 'avc1', 'iso5', 'iso6']
      if (mp4Brands.some(b => brand.startsWith(b.trim()))) {
        return 'video/mp4'
      }
      // QuickTime brand
      if (brand === 'qt  ') {
        return 'video/quicktime'
      }
    }
  }

  return null
}

/**
 * Validate an image file by checking its magic bytes
 */
export function validateImageFile(buffer: Buffer): ValidationResult {
  const detectedMime = detectFileType(buffer)

  if (!detectedMime) {
    return {
      valid: false,
      detectedMime: null,
      error: 'Could not detect file type. File may be corrupted or unsupported.'
    }
  }

  if (!ALLOWED_IMAGE_MIMES.has(detectedMime)) {
    return {
      valid: false,
      detectedMime,
      error: `Detected file type "${detectedMime}" is not allowed. Allowed types: ${[...ALLOWED_IMAGE_MIMES].join(', ')}`
    }
  }

  return {
    valid: true,
    detectedMime
  }
}

/**
 * Validate a video file by checking its magic bytes
 */
export function validateVideoFile(buffer: Buffer): ValidationResult {
  const detectedMime = detectFileType(buffer)

  if (!detectedMime) {
    return {
      valid: false,
      detectedMime: null,
      error: 'Could not detect file type. File may be corrupted or unsupported.'
    }
  }

  if (!ALLOWED_VIDEO_MIMES.has(detectedMime)) {
    return {
      valid: false,
      detectedMime,
      error: `Detected file type "${detectedMime}" is not allowed. Allowed types: ${[...ALLOWED_VIDEO_MIMES].join(', ')}`
    }
  }

  return {
    valid: true,
    detectedMime
  }
}

/**
 * Get allowed MIME types for each category
 */
export function getAllowedMimes() {
  return {
    image: [...ALLOWED_IMAGE_MIMES],
    video: [...ALLOWED_VIDEO_MIMES]
  }
}
