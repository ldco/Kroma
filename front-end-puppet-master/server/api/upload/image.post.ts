/**
 * POST /api/upload/image
 *
 * Uploads and processes an image using Sharp.
 * Uses storage adapter (local or S3) based on config.
 *
 * Security:
 * - Validates file content by magic bytes (not just MIME type)
 * - Requires admin authentication
 *
 * Returns URLs for the processed image and thumbnail.
 */
import config from '../../../app/puppet-master.config'
import { useFileStorage } from '../../utils/storage'
import { requireAnyContentAdminSectionAccess } from '../../utils/admin-rbac'
import { validateImageFile } from '../../utils/fileValidation'
import { scanFileForViruses } from '../../utils/virusScanning'
import { logger } from '../../utils/logger'

export default defineEventHandler(async event => {
  await requireAnyContentAdminSectionAccess(event.context.user)

  // Parse multipart form data
  const formData = await readMultipartFormData(event)
  if (!formData) {
    throw createError({
      statusCode: 400,
      statusMessage: 'No form data provided'
    })
  }

  // Find the image file
  const file = formData.find(f => f.name === 'image')
  if (!file?.data) {
    throw createError({
      statusCode: 400,
      statusMessage: 'No image provided'
    })
  }

  // Validate file size first (before expensive validation)
  const maxSize = config.storage.image.maxSizeMB * 1024 * 1024
  if (file.data.length > maxSize) {
    throw createError({
      statusCode: 400,
      statusMessage: `File too large. Maximum size: ${config.storage.image.maxSizeMB}MB`
    })
  }

  // Validate file by magic bytes (not just client-provided MIME type)
  const validation = validateImageFile(file.data)
  if (!validation.valid) {
    throw createError({
      statusCode: 400,
      statusMessage: validation.error || 'Invalid file type'
    })
  }

  // Use detected MIME type (from magic bytes) instead of client-provided
  const mimeType = validation.detectedMime!

  // Scan for viruses
  const scanResult = await scanFileForViruses(file.data, file.filename || 'image')
  if (scanResult.isInfected) {
    throw createError({
      statusCode: 400,
      statusMessage: `File rejected: ${scanResult.viruses?.join(', ') || 'Malware detected'}`
    })
  }

  try {
    const storage = useFileStorage()
    const result = await storage.upload(file.data, {
      type: 'image',
      originalName: file.filename,
      mimeType
    })

    return {
      success: true,
      data: result
    }
  } catch (error) {
    logger.error({ error: error instanceof Error ? error.message : String(error) }, 'Image upload error')
    throw createError({
      statusCode: 500,
      statusMessage: 'Failed to process image'
    })
  }
})
