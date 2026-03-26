/**
 * Type declarations for clamscan module
 *
 * Minimal typing for virus scanning functionality.
 * The clamscan package doesn't provide its own types.
 */
declare module 'clamscan' {
  interface ClamScanOptions {
    clamdscan?: {
      socket?: string
      host?: string
      port?: number
      timeout?: number
      localFallback?: boolean
      path?: string
      configFile?: string
      multiscan?: boolean
      reloadDb?: boolean
      active?: boolean
      bypassTest?: boolean
    }
    preference?: 'clamdscan' | 'clamscan'
    removeInfected?: boolean
    quarantineInfected?: string | boolean
    debugMode?: boolean
    scanRecursively?: boolean
    scanLog?: string | null
  }

  interface ScanResult {
    isInfected: boolean
    file: string
    viruses: string[]
  }

  interface ClamScan {
    isInfected(filePath: string): Promise<ScanResult>
    scanFile(filePath: string): Promise<ScanResult>
    scanDir(dirPath: string): Promise<ScanResult[]>
    scanStream(stream: NodeJS.ReadableStream): Promise<ScanResult>
    scanBuffer(buffer: Buffer, timeout?: number, filename?: string): Promise<ScanResult>
    passthrough(): NodeJS.Transform
    getVersion(): Promise<string>
  }

  class NodeClam {
    init(options?: ClamScanOptions): Promise<ClamScan>
  }

  export = NodeClam
}
