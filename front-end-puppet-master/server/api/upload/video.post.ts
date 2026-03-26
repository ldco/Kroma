/**
 * POST /api/upload/video
 *
 * Uploads and compresses video using FFmpeg.
 * - Compresses to web-optimized MP4/WebM
 * - Generates thumbnail from first frame
 * - Uses storage adapter (local or S3)
 *
 * Security:
 * - Validates file content by magic bytes (not just MIME type)
 * - Requires admin authentication
 */
import { randomUUID } from 'crypto'
import { writeFile, unlink, mkdir } from 'fs/promises'
import { existsSync } from 'fs'
import ffmpeg from 'fluent-ffmpeg'
import ffmpegStatic from 'ffmpeg-static'
import config from '../../../app/puppet-master.config'
import { useFileStorage } from '../../utils/storage'
import { requireAnyContentAdminSectionAccess } from '../../utils/admin-rbac'
import { validateVideoFile } from '../../utils/fileValidation'
import { scanFileForViruses } from '../../utils/virusScanning'
import { logger } from '../../utils/logger'

// Set FFmpeg path
if (ffmpegStatic) {
  ffmpeg.setFfmpegPath(ffmpegStatic)
}

const TEMP_DIR = './data/temp'

export default defineEventHandler(async event => {
  // Check if video uploads are enabled
  if (!config.storage.video.enabled) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Video uploads are disabled'
    })
  }

  await requireAnyContentAdminSectionAccess(event.context.user)

  // Parse multipart form data
  const formData = await readMultipartFormData(event)
  if (!formData) {
    throw createError({
      statusCode: 400,
      statusMessage: 'No form data provided'
    })
  }

  // Find the video file
  const file = formData.find(f => f.name === 'video')
  if (!file?.data) {
    throw createError({
      statusCode: 400,
      statusMessage: 'No video provided'
    })
  }

  // Validate file size first (before expensive validation)
  const maxSize = config.storage.video.maxSizeMB * 1024 * 1024
  if (file.data.length > maxSize) {
    throw createError({
      statusCode: 400,
      statusMessage: `File too large. Maximum size: ${config.storage.video.maxSizeMB}MB`
    })
  }

  // Validate file by magic bytes (not just client-provided MIME type)
  const validation = validateVideoFile(file.data)
  if (!validation.valid) {
    throw createError({
      statusCode: 400,
      statusMessage: validation.error || 'Invalid file type'
    })
  }

  // Note: detectedMime (validation.detectedMime) is available but not used
  // since we always convert to the configured output format

  // Scan for viruses
  const scanResult = await scanFileForViruses(file.data, file.filename || 'video')
  if (scanResult.isInfected) {
    throw createError({
      statusCode: 400,
      statusMessage: `File rejected: ${scanResult.viruses?.join(', ') || 'Malware detected'}`
    })
  }

  // Ensure temp directory exists
  if (!existsSync(TEMP_DIR)) {
    await mkdir(TEMP_DIR, { recursive: true })
  }

  const id = randomUUID()
  const inputPath = `${TEMP_DIR}/${id}-input`
  const outputFormat = config.storage.video.outputFormat
  const outputPath = `${TEMP_DIR}/${id}.${outputFormat}`
  const thumbPath = `${TEMP_DIR}/${id}-thumb.jpg`

  try {
    // Write input file to temp
    await writeFile(inputPath, file.data)

    // Compress video with FFmpeg
    await compressVideo(inputPath, outputPath)

    // Generate thumbnail
    await generateThumbnail(inputPath, thumbPath)

    // Read compressed files
    const { readFile } = await import('fs/promises')
    const videoBuffer = await readFile(outputPath)
    const thumbBuffer = await readFile(thumbPath)

    // Upload using storage adapter
    const storage = useFileStorage()
    const videoResult = await storage.upload(videoBuffer, {
      type: 'video',
      originalName: file.filename,
      mimeType: outputFormat === 'mp4' ? 'video/mp4' : 'video/webm'
    })

    // Upload thumbnail as image
    const thumbResult = await storage.upload(thumbBuffer, {
      type: 'image',
      originalName: `${file.filename}-thumb.jpg`,
      mimeType: 'image/jpeg'
    })

    // Cleanup temp files
    await Promise.all([
      unlink(inputPath).catch(() => {}),
      unlink(outputPath).catch(() => {}),
      unlink(thumbPath).catch(() => {})
    ])

    return {
      success: true,
      data: {
        id: videoResult.id,
        url: videoResult.url,
        thumbnailUrl: thumbResult.url,
        originalName: file.filename,
        size: videoBuffer.length
      }
    }
  } catch (error) {
    // Cleanup on error
    await Promise.all([
      unlink(inputPath).catch(() => {}),
      unlink(outputPath).catch(() => {}),
      unlink(thumbPath).catch(() => {})
    ])

    logger.error({ error: error instanceof Error ? error.message : String(error) }, 'Video upload error')
    throw createError({
      statusCode: 500,
      statusMessage: 'Failed to process video'
    })
  }
})

function compressVideo(input: string, output: string): Promise<void> {
  const { compression, outputFormat } = config.storage.video

  return new Promise((resolve, reject) => {
    let command = ffmpeg(input).outputOptions([
      `-vf scale='min(${compression.maxWidth},iw)':-2`,
      `-b:v ${compression.videoBitrate}`,
      `-b:a ${compression.audioBitrate}`,
      `-r ${compression.fps}`,
      '-movflags +faststart' // Enable streaming
    ])

    if (outputFormat === 'mp4') {
      command = command.videoCodec('libx264').audioCodec('aac')
    } else {
      command = command.videoCodec('libvpx-vp9').audioCodec('libopus')
    }

    command
      .output(output)
      .on('end', () => resolve())
      .on('error', reject)
      .run()
  })
}

function generateThumbnail(input: string, output: string): Promise<void> {
  return new Promise((resolve, reject) => {
    ffmpeg(input)
      .screenshots({
        timestamps: ['00:00:01'],
        filename: output.split('/').pop(),
        folder: TEMP_DIR,
        size: '400x300'
      })
      .on('end', () => resolve())
      .on('error', reject)
  })
}
