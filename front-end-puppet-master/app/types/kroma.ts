/**
 * Kroma Types
 *
 * TypeScript type definitions for Kroma app domain models.
 */

// ═══════════════════════════════════════════════════════════════════════════
// PROJECTS
// ═══════════════════════════════════════════════════════════════════════════

export interface Project {
  id: string
  slug: string
  name: string
  description: string | null
  created_at: string
  updated_at: string
  owner_id: string
}

export interface ProjectSummary extends Project {
  run_count: number
  asset_count: number
}

export interface ProjectDetail {
  project: Project
  counts: ProjectCounts
  storage: StorageConfig
}

export interface ProjectCounts {
  runs: number
  assets: number
  characters: number
  style_guides: number
  reference_sets: number
}

export interface CreateProjectInput {
  name: string
  description?: string
  storage_provider?: 'local' | 's3'
  storage_config?: LocalStorageConfig | S3StorageConfig
}

// ═══════════════════════════════════════════════════════════════════════════
// STORAGE
// ═══════════════════════════════════════════════════════════════════════════

export interface StorageConfig {
  provider: 'local' | 's3'
  config: LocalStorageConfig | S3StorageConfig
}

export interface LocalStorageConfig {
  root_path: string
}

export interface S3StorageConfig {
  bucket: string
  region: string
  endpoint: string | null
}

// ═══════════════════════════════════════════════════════════════════════════
// RUNS
// ═══════════════════════════════════════════════════════════════════════════

export interface Run {
  id: string
  project_slug: string
  status: RunStatus
  prompt: string
  negative_prompt: string | null
  candidates: Candidate[]
  created_at: string
  updated_at: string
  completed_at: string | null
}

export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface Candidate {
  id: string
  run_id: string
  image_url: string
  score: number | null
  is_winner: boolean
  metadata: CandidateMetadata
}

export interface CandidateMetadata {
  seed: number
  steps: number
  guidance_scale: number
  width: number
  height: number
}

export interface RunConfig {
  prompt: string
  negative_prompt?: string
  model?: string
  width?: number
  height?: number
  steps?: number
  guidance_scale?: number
  seed?: number
  candidate_count?: number
  reference_set_ids?: string[]
  character_ids?: string[]
  style_guide_ids?: string[]
}

// ═══════════════════════════════════════════════════════════════════════════
// ASSETS
// ═══════════════════════════════════════════════════════════════════════════

export interface Asset {
  id: string
  project_slug: string
  run_id: string | null
  file_path: string
  file_size: number
  mime_type: string
  width: number
  height: number
  metadata: AssetMetadata
  lineage: string[] // parent asset IDs
  created_at: string
}

export interface AssetMetadata {
  prompt: string | null
  negative_prompt: string | null
  seed: number | null
  model: string | null
  tags: string[]
}

// ═══════════════════════════════════════════════════════════════════════════
// CHARACTERS
// ═══════════════════════════════════════════════════════════════════════════

export interface Character {
  id: string
  project_slug: string
  name: string
  description: string | null
  reference_images: string[]
  created_at: string
  updated_at: string
}

// ═══════════════════════════════════════════════════════════════════════════
// STYLE GUIDES
// ═══════════════════════════════════════════════════════════════════════════

export interface StyleGuide {
  id: string
  project_slug: string
  name: string
  description: string | null
  constraints: StyleConstraint[]
  reference_images: string[]
  created_at: string
  updated_at: string
}

export interface StyleConstraint {
  type: 'color' | 'composition' | 'lighting' | 'mood' | 'other'
  description: string
  weight: number
}

// ═══════════════════════════════════════════════════════════════════════════
// REFERENCE SETS
// ═══════════════════════════════════════════════════════════════════════════

export interface ReferenceSet {
  id: string
  project_slug: string
  name: string
  description: string | null
  items: ReferenceItem[]
  created_at: string
  updated_at: string
}

export interface ReferenceItem {
  id: string
  reference_set_id: string
  asset_id: string | null
  external_url: string | null
  description: string | null
  tags: string[]
}

// ═══════════════════════════════════════════════════════════════════════════
// PROVIDERS
// ═══════════════════════════════════════════════════════════════════════════

export interface ProviderAccount {
  provider_code: string
  name: string
  provider_type: ProviderType
  api_key_encrypted: string
  health_status: HealthStatus
  created_at: string
  updated_at: string
}

export interface CreateProviderInput {
  name: string
  provider_type: ProviderType
  api_key: string
  config?: Record<string, any>
}

export type ProviderType = 'openai' | 'stability' | 'anthropic' | 'midjourney' | 'other'

export interface HealthStatus {
  status: 'healthy' | 'unhealthy' | 'unknown'
  last_checked: string | null
  error_message: string | null
}

// ═══════════════════════════════════════════════════════════════════════════
// AUTH (re-export from existing types)
// ═══════════════════════════════════════════════════════════════════════════

export type { User, UserRole, LoginCredentials, RolePermissions } from '~/types/auth'
