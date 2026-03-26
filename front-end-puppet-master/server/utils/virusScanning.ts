/// <reference path="../../types/clamscan.d.ts" />
/**
 * Virus Scanning Utility
 *
 * Uses ClamAV via clamscan package to scan uploaded files for malware.
 * Falls back gracefully if ClamAV is not installed (logs warning).
 *
 * Installation:
 * 1. Install ClamAV: sudo apt install clamav clamav-daemon
 * 2. Update virus database: sudo freshclam
 * 3. Start daemon: sudo systemctl start clamav-daemon
 * 4. Install npm package: npm install clamscan
 *
 * Environment:
 * - CLAMAV_ENABLED=true|false (default: true in production, false in dev)
 * - CLAMAV_SOCKET=/var/run/clamav/clamd.ctl (default)
 */
import { logger } from './logger'

interface ScanResult {
  isInfected: boolean
  viruses?: string[]
  error?: string
}

/** ClamScan instance interface (matches types/clamscan.d.ts) */
interface ClamScanInstance {
  isInfected(filePath: string): Promise<{ isInfected: boolean; viruses: string[] }>
  scanBuffer(buffer: Buffer, timeout?: number, filename?: string): Promise<{ isInfected: boolean; viruses: string[] }>
  getVersion(): Promise<string>
}

let clamScanInstance: ClamScanInstance | null = null

/**
 * Initialize ClamAV scanner (lazy)
 */
async function initClamScan(): Promise<ClamScanInstance | null> {
  if (clamScanInstance !== null) {
    return clamScanInstance
  }

  try {
    // Dynamically import clamscan (optional dependency)
    const { default: NodeClam } = await import('clamscan').catch(() => {
      logger.warn('clamscan package not installed. Run: npm install clamscan')
      throw new Error('clamscan not installed')
    })

    const socketPath =
      process.env.CLAMAV_SOCKET || '/var/run/clamav/clamd.ctl'

    clamScanInstance = await new NodeClam().init({
      clamdscan: {
        socket: socketPath,
        timeout: 60000, // 60 seconds
        localFallback: false // Don't fall back to local scanning (too slow)
      },
      preference: 'clamdscan' // Use daemon for speed
    }) as ClamScanInstance

    logger.info(
      { socket: socketPath },
      'ClamAV initialized successfully'
    )

    return clamScanInstance
  } catch (error: unknown) {
    const err = error as Error
    logger.warn(
      { error: err.message },
      'ClamAV not available - virus scanning disabled'
    )
    return null
  }
}

/**
 * Check if ClamAV is enabled and available
 */
function isClamAvEnabled(): boolean {
  const isProduction = process.env.NODE_ENV === 'production'
  const envEnabled = process.env.CLAMAV_ENABLED !== 'false'

  // Enable by default in production, disable in dev (unless explicitly enabled)
  return isProduction ? envEnabled : process.env.CLAMAV_ENABLED === 'true'
}

/**
 * Scan a file buffer for viruses
 *
 * @param buffer - File buffer to scan
 * @param filename - Original filename (for logging)
 * @returns Scan result with infection status
 */
export async function scanFileForViruses(
  buffer: Buffer,
  filename: string
): Promise<ScanResult> {
  // Check if scanning is enabled
  if (!isClamAvEnabled()) {
    logger.debug(
      { filename },
      'Virus scanning disabled (dev mode or CLAMAV_ENABLED=false)'
    )
    return { isInfected: false }
  }

  try {
    const clam = await initClamScan()

    if (!clam) {
      // ClamAV not available - log warning but allow upload
      logger.warn(
        { filename },
        'ClamAV unavailable - file uploaded without virus scan'
      )
      return {
        isInfected: false,
        error: 'Scanner unavailable'
      }
    }

    // Scan the buffer
    const { isInfected, viruses } = await clam.scanBuffer(buffer, 3000, filename)

    if (isInfected) {
      logger.warn(
        { filename, viruses },
        'Virus detected in uploaded file'
      )
      return {
        isInfected: true,
        viruses: viruses || ['Unknown virus']
      }
    }

    logger.debug({ filename }, 'File scanned - clean')
    return { isInfected: false }
  } catch (error: unknown) {
    // Scanning error - log but don't block upload (fail open)
    const err = error as Error
    logger.error(
      { filename, error: err.message },
      'Virus scanning failed'
    )
    return {
      isInfected: false,
      error: err.message
    }
  }
}

/**
 * Scan a file at a path for viruses
 *
 * @param filePath - Path to file to scan
 * @returns Scan result with infection status
 */
export async function scanFileAtPath(
  filePath: string
): Promise<ScanResult> {
  if (!isClamAvEnabled()) {
    return { isInfected: false }
  }

  try {
    const clam = await initClamScan()

    if (!clam) {
      return {
        isInfected: false,
        error: 'Scanner unavailable'
      }
    }

    const { isInfected, viruses } = await clam.isInfected(filePath)

    if (isInfected) {
      logger.warn(
        { filePath, viruses },
        'Virus detected in file'
      )
      return {
        isInfected: true,
        viruses: viruses || ['Unknown virus']
      }
    }

    return { isInfected: false }
  } catch (error: unknown) {
    const err = error as Error
    logger.error(
      { filePath, error: err.message },
      'Virus scanning failed'
    )
    return {
      isInfected: false,
      error: err.message
    }
  }
}

/**
 * Get ClamAV status for health checks
 */
export async function getClamAvStatus(): Promise<{
  available: boolean
  version?: string
  error?: string
}> {
  try {
    const clam = await initClamScan()

    if (!clam) {
      return {
        available: false,
        error: 'ClamAV daemon not running or clamscan package not installed'
      }
    }

    const version = await clam.getVersion()
    return {
      available: true,
      version
    }
  } catch (error: unknown) {
    const err = error as Error
    return {
      available: false,
      error: err.message
    }
  }
}
