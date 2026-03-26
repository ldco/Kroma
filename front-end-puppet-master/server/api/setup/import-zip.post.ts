/**
 * Upload Zip for Import API
 *
 * POST /api/setup/import-zip
 * Body: FormData with 'file' field
 * Returns: { files: string[], extractionMethod: 'system' | 'node' }
 *
 * Extracts zip to ./import/ folder for brownfield migration.
 * Cross-platform: uses system unzip first, falls back to adm-zip.
 * Security: Only accessible when pmMode is 'unconfigured'
 */
import { createWriteStream, existsSync, mkdirSync, readdirSync, rmSync, unlinkSync, statSync } from 'fs'
import { resolve, join, normalize, relative } from 'path'
import AdmZip from 'adm-zip'
import { requireSetupAccess } from '../../utils/setup-guard'

// ═══════════════════════════════════════════════════════════════════════════════
// SECURITY CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════
const MAX_ZIP_SIZE_MB = 100 // Maximum compressed zip size
const MAX_UNCOMPRESSED_SIZE_MB = 500 // Maximum total uncompressed size
const MAX_FILE_COUNT = 10000 // Maximum number of files in archive
const MAX_SINGLE_FILE_SIZE_MB = 50 // Maximum size of any single file

// ═══════════════════════════════════════════════════════════════════════════════
// SECURITY HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Check if a path is safe (no path traversal / zip-slip attack)
 * Returns true if the resolved path stays within the target directory
 */
function isSafePath(targetDir: string, entryPath: string): boolean {
  // Normalize and resolve the full path
  const normalizedEntry = normalize(entryPath)

  // Reject paths with .. components
  if (normalizedEntry.includes('..')) {
    return false
  }

  // Reject absolute paths
  if (normalizedEntry.startsWith('/') || /^[a-zA-Z]:/.test(normalizedEntry)) {
    return false
  }

  // Resolve the full path and check it's within target
  const fullPath = resolve(targetDir, normalizedEntry)
  const relativePath = relative(targetDir, fullPath)

  // If relative path starts with .., it escapes the target directory
  if (relativePath.startsWith('..') || relativePath.startsWith('/')) {
    return false
  }

  return true
}

/**
 * Validate ZIP archive before extraction
 * Checks for:
 * - Total uncompressed size
 * - Number of files
 * - Individual file sizes
 * - Path traversal attempts (zip-slip)
 */
function validateZipArchive(zip: AdmZip, targetDir: string): void {
  const entries = zip.getEntries()

  // Check file count
  if (entries.length > MAX_FILE_COUNT) {
    throw createError({
      statusCode: 400,
      statusMessage: 'PM_ZIP_006: Too many files',
      message: `ZIP archive contains ${entries.length} files. Maximum allowed is ${MAX_FILE_COUNT}.`
    })
  }

  let totalUncompressedSize = 0
  const maxUncompressedBytes = MAX_UNCOMPRESSED_SIZE_MB * 1024 * 1024
  const maxSingleFileBytes = MAX_SINGLE_FILE_SIZE_MB * 1024 * 1024

  for (const entry of entries) {
    const entryPath = entry.entryName

    // Check for path traversal (zip-slip vulnerability)
    if (!isSafePath(targetDir, entryPath)) {
      throw createError({
        statusCode: 400,
        statusMessage: 'PM_ZIP_007: Invalid path detected',
        message: `ZIP archive contains potentially unsafe path: "${entryPath}". Path traversal is not allowed.`
      })
    }

    // Skip directories for size checks
    if (entry.isDirectory) continue

    // Check individual file size
    const fileSize = entry.header.size
    if (fileSize > maxSingleFileBytes) {
      throw createError({
        statusCode: 400,
        statusMessage: 'PM_ZIP_008: File too large',
        message: `File "${entryPath}" is ${Math.round(fileSize / 1024 / 1024)}MB. Maximum allowed is ${MAX_SINGLE_FILE_SIZE_MB}MB.`
      })
    }

    // Accumulate total size
    totalUncompressedSize += fileSize

    // Early exit if total exceeds limit
    if (totalUncompressedSize > maxUncompressedBytes) {
      throw createError({
        statusCode: 400,
        statusMessage: 'PM_ZIP_009: Archive too large',
        message: `ZIP archive uncompressed size exceeds ${MAX_UNCOMPRESSED_SIZE_MB}MB limit.`
      })
    }
  }
}

/**
 * Safe extraction using adm-zip with validation
 */
function safeExtract(zip: AdmZip, targetDir: string): void {
  const entries = zip.getEntries()

  for (const entry of entries) {
    const entryPath = entry.entryName

    // Double-check path safety during extraction
    if (!isSafePath(targetDir, entryPath)) {
      console.warn(`Skipping unsafe entry: ${entryPath}`)
      continue
    }

    const fullPath = resolve(targetDir, entryPath)

    if (entry.isDirectory) {
      // Create directory if it doesn't exist
      if (!existsSync(fullPath)) {
        mkdirSync(fullPath, { recursive: true })
      }
    } else {
      // Ensure parent directory exists
      const parentDir = resolve(fullPath, '..')
      if (!existsSync(parentDir)) {
        mkdirSync(parentDir, { recursive: true })
      }

      // Extract file
      zip.extractEntryTo(entry, targetDir, true, true)
    }
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

export default defineEventHandler(async (event) => {
  // Security: Only allow zip upload when project is unconfigured
  requireSetupAccess(event)

  // Security: REQUIRE Content-Length header to prevent chunked transfer bypass
  // Chunked transfers don't have a declared size, so they could exhaust memory
  // by streaming unlimited data before any size checks run
  const contentLength = getHeader(event, 'content-length')
  const maxRequestBytes = (MAX_ZIP_SIZE_MB + 1) * 1024 * 1024 // +1MB for multipart overhead

  if (!contentLength) {
    throw createError({
      statusCode: 411,
      statusMessage: 'PM_ZIP_012: Content-Length required',
      message: 'Content-Length header is required for file uploads. Chunked transfer encoding is not supported for security reasons.'
    })
  }

  const requestSizeBytes = parseInt(contentLength, 10)
  if (isNaN(requestSizeBytes) || requestSizeBytes < 0) {
    throw createError({
      statusCode: 400,
      statusMessage: 'PM_ZIP_013: Invalid Content-Length',
      message: 'Invalid Content-Length header value.'
    })
  }

  if (requestSizeBytes > maxRequestBytes) {
    throw createError({
      statusCode: 413,
      statusMessage: 'PM_ZIP_011: Request too large',
      message: `Request body is ${Math.round(requestSizeBytes / 1024 / 1024)}MB. Maximum allowed is ${MAX_ZIP_SIZE_MB}MB. Please compress your project further or split it into smaller parts.`
    })
  }

  const formData = await readMultipartFormData(event)

  if (!formData || formData.length === 0) {
    throw createError({
      statusCode: 400,
      statusMessage: 'PM_ZIP_001: No file uploaded',
      message: 'No file uploaded. Please select a ZIP file to import.'
    })
  }

  const file = formData.find(f => f.name === 'file')
  if (!file || !file.data) {
    throw createError({
      statusCode: 400,
      statusMessage: 'PM_ZIP_002: No file data found',
      message: 'No file data found. Please try uploading again.'
    })
  }

  if (!file.filename?.endsWith('.zip')) {
    throw createError({
      statusCode: 400,
      statusMessage: 'PM_ZIP_003: Invalid file type',
      message: 'Only .zip files are allowed. Please compress your project as a ZIP file.'
    })
  }

  // Security: Check compressed file size
  const fileSizeBytes = file.data.length
  const maxZipBytes = MAX_ZIP_SIZE_MB * 1024 * 1024
  if (fileSizeBytes > maxZipBytes) {
    throw createError({
      statusCode: 400,
      statusMessage: 'PM_ZIP_010: Upload too large',
      message: `ZIP file is ${Math.round(fileSizeBytes / 1024 / 1024)}MB. Maximum allowed is ${MAX_ZIP_SIZE_MB}MB.`
    })
  }

  const importDir = resolve(process.cwd(), 'import')
  const tempZipPath = resolve(process.cwd(), 'data', 'temp-import.zip')
  let extractionMethod: 'system' | 'node' = 'node' // Default to node for safety

  try {
    // Ensure data directory exists
    const dataDir = resolve(process.cwd(), 'data')
    if (!existsSync(dataDir)) {
      mkdirSync(dataDir, { recursive: true })
    }

    // Clear existing import folder
    if (existsSync(importDir)) {
      rmSync(importDir, { recursive: true, force: true })
    }
    mkdirSync(importDir, { recursive: true })

    // Write zip to temp file
    const writeStream = createWriteStream(tempZipPath)
    writeStream.write(file.data)
    writeStream.end()

    // Wait for write to complete
    await new Promise((resolve, reject) => {
      writeStream.on('finish', resolve)
      writeStream.on('error', reject)
    })

    // Always use adm-zip for validation and safe extraction
    // System unzip doesn't allow us to pre-validate entries
    try {
      const zip = new AdmZip(tempZipPath)

      // Validate archive before extraction (security checks)
      validateZipArchive(zip, importDir)

      // Safe extraction with path validation
      safeExtract(zip, importDir)
      extractionMethod = 'node'
    } catch (e: any) {
      // Re-throw createError instances
      if (e.statusCode) throw e

      throw createError({
        statusCode: 500,
        statusMessage: 'PM_ZIP_004: Extraction failed',
        message: `Failed to extract ZIP file: ${e.message}. The file may be corrupted or use unsupported compression.`
      })
    }

    // Clean up temp file
    if (existsSync(tempZipPath)) {
      unlinkSync(tempZipPath)
    }

    // Get list of extracted files
    const files = getFilesRecursive(importDir)

    return {
      success: true,
      data: {
        files: files.map(f => f.replace(importDir + '/', '')),
        extractionMethod,
        fileCount: files.length
      }
    }
  } catch (e: any) {
    // Clean up on error
    if (existsSync(tempZipPath)) {
      try {
        unlinkSync(tempZipPath)
      } catch {
        // Ignore temporary file cleanup errors.
      }
    }
    if (existsSync(importDir)) {
      try {
        rmSync(importDir, { recursive: true, force: true })
      } catch {
        // Ignore import directory cleanup errors.
      }
    }

    // Re-throw createError instances
    if (e.statusCode) {
      throw e
    }

    throw createError({
      statusCode: 500,
      statusMessage: 'PM_ZIP_005: Processing failed',
      message: e.message || 'Failed to process zip file'
    })
  }
})

// Recursively get all files in a directory
function getFilesRecursive(dir: string): string[] {
  const files: string[] = []
  const entries = readdirSync(dir, { withFileTypes: true })

  for (const entry of entries) {
    const fullPath = join(dir, entry.name)
    if (entry.isDirectory()) {
      // Skip hidden directories and node_modules
      if (!entry.name.startsWith('.') && entry.name !== 'node_modules') {
        files.push(...getFilesRecursive(fullPath))
      }
    } else {
      // Skip hidden files
      if (!entry.name.startsWith('.')) {
        files.push(fullPath)
      }
    }
  }

  return files
}
