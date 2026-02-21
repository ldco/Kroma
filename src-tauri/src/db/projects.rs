use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

const DEFAULT_PROJECTS_BASE_DIR: &str = "var/projects";
const DEFAULT_S3_PREFIX: &str = "iat-projects";
const ASSET_LINK_TYPE_DERIVED_FROM: &str = "derived_from";
const ASSET_LINK_TYPE_VARIANT_OF: &str = "variant_of";
const ASSET_LINK_TYPE_MASK_FOR: &str = "mask_for";
const ASSET_LINK_TYPE_REFERENCE_OF: &str = "reference_of";

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSummary {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectCounts {
    pub runs: i64,
    pub jobs: i64,
    pub assets: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageLocal {
    pub base_dir: String,
    pub project_root: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageS3 {
    pub enabled: bool,
    pub bucket: String,
    pub prefix: String,
    pub region: String,
    pub profile: String,
    pub endpoint_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageConfig {
    pub local: StorageLocal,
    pub s3: StorageS3,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDetail {
    pub project: ProjectInfo,
    pub counts: ProjectCounts,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectStorageProject {
    pub id: String,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectStoragePayload {
    pub project: ProjectStorageProject,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub id: String,
    pub project_id: String,
    pub run_mode: String,
    pub status: String,
    pub stage: String,
    pub time_of_day: String,
    pub weather: String,
    pub model_name: String,
    pub provider_code: String,
    pub settings_snapshot_json: Value,
    pub started_at: String,
    pub finished_at: String,
    pub created_at: String,
    pub run_log_path: String,
    pub image_size: String,
    pub image_quality: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunCandidateSummary {
    pub id: String,
    pub job_id: String,
    pub candidate_index: i64,
    pub status: String,
    pub output_asset_id: String,
    pub final_asset_id: String,
    pub output_path: String,
    pub final_output_path: String,
    pub rank_hard_failures: i64,
    pub rank_soft_warnings: i64,
    pub rank_avg_chroma_exceed: f64,
    pub meta_json: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunJobSummary {
    pub id: String,
    pub run_id: String,
    pub job_key: String,
    pub status: String,
    pub prompt_text: String,
    pub selected_candidate_index: Option<i64>,
    pub final_asset_id: String,
    pub final_output: String,
    pub meta_json: Value,
    pub created_at: String,
    pub candidates: Vec<RunCandidateSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetSummary {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub asset_kind: String,
    pub storage_uri: String,
    pub rel_path: String,
    pub storage_backend: String,
    pub mime_type: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub sha256: String,
    pub run_id: String,
    pub job_id: String,
    pub candidate_id: String,
    pub metadata_json: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetLinkSummary {
    pub id: String,
    pub project_id: String,
    pub parent_asset_id: String,
    pub child_asset_id: String,
    pub link_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct QualityReportSummary {
    pub id: String,
    pub project_id: String,
    pub run_id: String,
    pub asset_id: String,
    pub report_type: String,
    pub grade: String,
    pub hard_failures: i64,
    pub soft_warnings: i64,
    pub avg_chroma_exceed: f64,
    pub summary_json: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostEventSummary {
    pub id: String,
    pub project_id: String,
    pub run_id: String,
    pub job_id: String,
    pub provider_code: String,
    pub model_name: String,
    pub event_type: String,
    pub units: f64,
    pub unit_cost_usd: f64,
    pub total_cost_usd: f64,
    pub currency: String,
    pub meta_json: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectExportSummary {
    pub id: String,
    pub project_id: String,
    pub run_id: String,
    pub status: String,
    pub export_format: String,
    pub storage_uri: String,
    pub rel_path: String,
    pub file_size_bytes: i64,
    pub checksum_sha256: String,
    pub manifest_json: Value,
    pub created_at: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptTemplateSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub template_text: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderAccountSummary {
    pub project_id: String,
    pub provider_code: String,
    pub display_name: String,
    pub account_ref: String,
    pub base_url: String,
    pub enabled: bool,
    pub config_json: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StyleGuideSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub instructions: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharacterSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub prompt_text: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferenceSetSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferenceSetItemSummary {
    pub id: String,
    pub project_id: String,
    pub reference_set_id: String,
    pub label: String,
    pub content_uri: String,
    pub content_text: String,
    pub sort_order: i64,
    pub metadata_json: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatSessionSummary {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageSummary {
    pub id: String,
    pub project_id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpsertProjectInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub user_display_name: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateStorageLocalInput {
    #[serde(default)]
    pub base_dir: Option<String>,
    #[serde(default)]
    pub project_root: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateStorageS3Input {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub bucket: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub endpoint_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateAssetLinkInput {
    #[serde(default)]
    pub parent_asset_id: String,
    #[serde(default)]
    pub child_asset_id: String,
    #[serde(default)]
    pub link_type: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateAssetLinkInput {
    #[serde(default)]
    pub parent_asset_id: Option<String>,
    #[serde(default)]
    pub child_asset_id: Option<String>,
    #[serde(default)]
    pub link_type: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreatePromptTemplateInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub template_text: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdatePromptTemplateInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub template_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpsertProviderAccountInput {
    #[serde(default)]
    pub provider_code: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub account_ref: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub config_json: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateProviderAccountInput {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub account_ref: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub config_json: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateStyleGuideInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateStyleGuideInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateCharacterInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub prompt_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateCharacterInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub prompt_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateReferenceSetInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateReferenceSetInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateReferenceSetItemInput {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub content_uri: Option<String>,
    #[serde(default)]
    pub content_text: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub metadata_json: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateReferenceSetItemInput {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub content_uri: Option<String>,
    #[serde(default)]
    pub content_text: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub metadata_json: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateChatSessionInput {
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateChatMessageInput {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Error)]
pub enum ProjectsRepoError {
    #[error("project not found")]
    NotFound,

    #[error("{0}")]
    Validation(String),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone)]
pub struct ProjectsStore {
    db_path: PathBuf,
    repo_root: PathBuf,
}

impl ProjectsStore {
    pub fn new(db_path: impl Into<PathBuf>, repo_root: impl Into<PathBuf>) -> Self {
        Self {
            db_path: db_path.into(),
            repo_root: repo_root.into(),
        }
    }

    pub fn initialize(&self) -> Result<(), ProjectsRepoError> {
        self.with_connection(|_| Ok(()))
    }

    fn with_connection<T, F>(&self, func: F) -> Result<T, ProjectsRepoError>
    where
        F: FnOnce(&Connection) -> Result<T, ProjectsRepoError>,
    {
        if let Some(parent) = self.db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(self.db_path.as_path())?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        ensure_schema(&conn)?;
        func(&conn)
    }

    fn with_connection_mut<T, F>(&self, func: F) -> Result<T, ProjectsRepoError>
    where
        F: FnOnce(&mut Connection) -> Result<T, ProjectsRepoError>,
    {
        if let Some(parent) = self.db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut conn = Connection::open(self.db_path.as_path())?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        ensure_schema(&conn)?;
        func(&mut conn)
    }
}

impl ProjectsStore {
    pub fn list_projects(
        &self,
        username: Option<&str>,
    ) -> Result<Vec<ProjectSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let mut sql = String::from(
                "
                SELECT
                  p.id,
                  p.slug,
                  p.name,
                  p.description,
                  p.status,
                  p.created_at,
                  p.updated_at,
                  COALESCE(au.username, lu.username, '') AS username
                FROM projects p
                LEFT JOIN app_users au ON au.id = COALESCE(p.owner_user_id, p.user_id)
                LEFT JOIN users lu ON lu.id = COALESCE(p.owner_user_id, p.user_id)
            ",
            );

            let mut filter_username = None::<String>;
            if let Some(raw) = username {
                if let Some(cleaned) = normalize_slug(raw) {
                    sql.push_str(" WHERE COALESCE(au.username, lu.username) = ?1 ");
                    filter_username = Some(cleaned);
                }
            }
            sql.push_str(" ORDER BY p.updated_at DESC, p.created_at DESC");

            let mut stmt = conn.prepare(sql.as_str())?;
            let mut out = Vec::new();

            let mut rows = if let Some(value) = filter_username {
                stmt.query([value])?
            } else {
                stmt.query([])?
            };
            while let Some(row) = rows.next()? {
                out.push(ProjectSummary {
                    id: row.get("id")?,
                    slug: row.get("slug")?,
                    name: row.get("name")?,
                    description: row
                        .get::<_, Option<String>>("description")?
                        .unwrap_or_default(),
                    status: row
                        .get::<_, Option<String>>("status")?
                        .unwrap_or_else(|| String::from("active")),
                    username: row
                        .get::<_, Option<String>>("username")?
                        .unwrap_or_default(),
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                });
            }

            Ok(out)
        })
    }

    pub fn get_project_detail(&self, slug: &str) -> Result<ProjectDetail, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?.ok_or(ProjectsRepoError::NotFound)?;

            let runs = query_count(conn, "SELECT COUNT(*) FROM runs WHERE project_id = ?1", [&project.id])?;
            let jobs = query_count(
                conn,
                "SELECT COUNT(*) FROM run_jobs WHERE run_id IN (SELECT id FROM runs WHERE project_id = ?1)",
                [&project.id],
            )?;
            let assets = query_count(conn, "SELECT COUNT(*) FROM assets WHERE project_id = ?1", [&project.id])?;
            let storage = resolve_storage(conn, self.repo_root.as_path(), &project)?;

            Ok(ProjectDetail {
                project: ProjectInfo {
                    id: project.id,
                    slug: project.slug,
                    name: project.name,
                    description: project.description,
                    status: project.status,
                    created_at: project.created_at,
                    updated_at: project.updated_at,
                },
                counts: ProjectCounts { runs, jobs, assets },
                storage,
            })
        })
    }

    pub fn upsert_project(
        &self,
        input: UpsertProjectInput,
    ) -> Result<ProjectStoragePayload, ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let name = input.name.trim().to_string();
            if name.is_empty() {
                return Err(ProjectsRepoError::Validation(String::from(
                    "Field 'name' is required",
                )));
            }

            let username = input
                .username
                .as_deref()
                .and_then(normalize_slug)
                .unwrap_or_else(|| String::from("local"));
            let display_name = input
                .user_display_name
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| String::from("Local User"));
            let description = input
                .description
                .map(|v| v.trim().to_string())
                .unwrap_or_default();
            let slug = input
                .slug
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .and_then(|v| normalize_slug(v.as_str()))
                .unwrap_or_else(|| slugify(name.as_str()));

            let tx = conn.transaction()?;
            let user_id = ensure_user(&tx, username.as_str(), display_name.as_str())?;
            let project = ensure_project(
                &tx,
                user_id.as_str(),
                slug.as_str(),
                name.as_str(),
                description.as_str(),
            )?;
            ensure_project_storage_defaults(&tx, &project)?;
            tx.commit()?;

            self.with_connection(|conn_ro| {
                project_storage_payload(conn_ro, self.repo_root.as_path(), &project)
            })
        })
    }

    pub fn get_project_storage(
        &self,
        slug: &str,
    ) -> Result<ProjectStoragePayload, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            project_storage_payload(conn, self.repo_root.as_path(), &project)
        })
    }

    pub fn update_project_storage_local(
        &self,
        slug: &str,
        input: UpdateStorageLocalInput,
    ) -> Result<ProjectStoragePayload, ProjectsRepoError> {
        if input.base_dir.is_none() && input.project_root.is_none() {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: base_dir, project_root",
            )));
        }

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            ensure_project_storage_defaults(conn, &project)?;

            if let Some(raw_base_dir) = input.base_dir {
                let base_dir_update = normalize_required_storage_field(raw_base_dir.as_str())
                    .map_err(ProjectsRepoError::Validation)?;
                conn.execute(
                    "
                    UPDATE project_storage
                    SET local_base_dir = ?1, updated_at = ?2
                    WHERE project_id = ?3
                ",
                    params![base_dir_update, now_iso(), project.id],
                )?;
            }

            if let Some(raw_project_root) = input.project_root {
                let normalized_root = normalize_optional_storage_field(raw_project_root.as_str());
                conn.execute(
                    "
                    UPDATE project_storage
                    SET local_project_root = ?1, updated_at = ?2
                    WHERE project_id = ?3
                ",
                    params![normalized_root, now_iso(), project.id],
                )?;
            }

            project_storage_payload(conn, self.repo_root.as_path(), &project)
        })
    }

    pub fn update_project_storage_s3(
        &self,
        slug: &str,
        input: UpdateStorageS3Input,
    ) -> Result<ProjectStoragePayload, ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            ensure_project_storage_defaults(conn, &project)?;

            if let Some(enabled) = input.enabled {
                conn.execute(
                    "
                    UPDATE project_storage
                    SET s3_enabled = ?1, updated_at = ?2
                    WHERE project_id = ?3
                ",
                    params![if enabled { 1 } else { 0 }, now_iso(), project.id],
                )?;
            }

            if let Some(raw) = input.bucket {
                conn.execute(
                    "
                    UPDATE project_storage
                    SET s3_bucket = ?1, updated_at = ?2
                    WHERE project_id = ?3
                ",
                    params![
                        normalize_optional_storage_field(raw.as_str()),
                        now_iso(),
                        project.id
                    ],
                )?;
            }

            if let Some(raw) = input.prefix {
                conn.execute(
                    "
                    UPDATE project_storage
                    SET s3_prefix = ?1, updated_at = ?2
                    WHERE project_id = ?3
                ",
                    params![
                        normalize_optional_storage_field(raw.as_str()),
                        now_iso(),
                        project.id
                    ],
                )?;
            }

            if let Some(raw) = input.region {
                conn.execute(
                    "
                    UPDATE project_storage
                    SET s3_region = ?1, updated_at = ?2
                    WHERE project_id = ?3
                ",
                    params![
                        normalize_optional_storage_field(raw.as_str()),
                        now_iso(),
                        project.id
                    ],
                )?;
            }

            if let Some(raw) = input.profile {
                conn.execute(
                    "
                    UPDATE project_storage
                    SET s3_profile = ?1, updated_at = ?2
                    WHERE project_id = ?3
                ",
                    params![
                        normalize_optional_storage_field(raw.as_str()),
                        now_iso(),
                        project.id
                    ],
                )?;
            }

            if let Some(raw) = input.endpoint_url {
                conn.execute(
                    "
                    UPDATE project_storage
                    SET s3_endpoint_url = ?1, updated_at = ?2
                    WHERE project_id = ?3
                ",
                    params![
                        normalize_optional_storage_field(raw.as_str()),
                        now_iso(),
                        project.id
                    ],
                )?;
            }

            project_storage_payload(conn, self.repo_root.as_path(), &project)
        })
    }

    pub fn list_runs(&self, slug: &str, limit: i64) -> Result<Vec<RunSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 1000);
            let mut stmt = conn.prepare(
                "
                SELECT *
                FROM runs
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows = stmt.query_map(params![project.id, capped], row_to_run_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn get_run_detail(
        &self,
        slug: &str,
        run_id: &str,
    ) -> Result<(RunSummary, Vec<RunJobSummary>), ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let run = conn
                .query_row(
                    "
                    SELECT *
                    FROM runs
                    WHERE id = ?1 AND project_id = ?2
                    LIMIT 1
                ",
                    params![run_id, project.id],
                    row_to_run_summary,
                )
                .optional()?
                .ok_or(ProjectsRepoError::NotFound)?;

            let jobs = fetch_jobs_with_candidates(conn, run_id)?;
            Ok((run, jobs))
        })
    }

    pub fn list_run_jobs(
        &self,
        slug: &str,
        run_id: &str,
    ) -> Result<Vec<RunJobSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let run_exists = conn
                .query_row(
                    "SELECT id FROM runs WHERE id = ?1 AND project_id = ?2 LIMIT 1",
                    params![run_id, project.id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .is_some();
            if !run_exists {
                return Err(ProjectsRepoError::NotFound);
            }

            fetch_jobs_with_candidates(conn, run_id)
        })
    }

    pub fn list_assets(
        &self,
        slug: &str,
        limit: i64,
    ) -> Result<Vec<AssetSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 2000);

            let mut stmt = conn.prepare(
                "
                SELECT *
                FROM assets
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows = stmt.query_map(params![project.id, capped], row_to_asset_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn get_asset_detail(
        &self,
        slug: &str,
        asset_id: &str,
    ) -> Result<AssetSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            conn.query_row(
                "
                SELECT *
                FROM assets
                WHERE id = ?1 AND project_id = ?2
                LIMIT 1
            ",
                params![asset_id, project.id],
                row_to_asset_summary,
            )
            .optional()?
            .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn list_asset_links(
        &self,
        slug: &str,
        asset_id: Option<&str>,
        link_type: Option<&str>,
    ) -> Result<Vec<AssetLinkSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut sql = String::from(
                "
                SELECT
                  id,
                  project_id,
                  parent_asset_id,
                  child_asset_id,
                  link_type,
                  created_at,
                  updated_at
                FROM asset_links
                WHERE project_id = ?1
            ",
            );
            let mut values = vec![project.id];

            if let Some(raw_asset_id) = asset_id.map(str::trim).filter(|value| !value.is_empty()) {
                let index = values.len() + 1;
                sql.push_str(
                    format!(" AND (parent_asset_id = ?{index} OR child_asset_id = ?{index})")
                        .as_str(),
                );
                values.push(raw_asset_id.to_string());
            }

            if let Some(raw_link_type) = link_type {
                let normalized = normalize_asset_link_type(raw_link_type)?;
                let index = values.len() + 1;
                sql.push_str(format!(" AND link_type = ?{index}").as_str());
                values.push(normalized);
            }

            sql.push_str(" ORDER BY COALESCE(updated_at, '') DESC, id DESC");
            let mut stmt = conn.prepare(sql.as_str())?;
            let rows = stmt.query_map(
                rusqlite::params_from_iter(values.iter().map(String::as_str)),
                row_to_asset_link_summary,
            )?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_asset_link(
        &self,
        slug: &str,
        input: CreateAssetLinkInput,
    ) -> Result<AssetLinkSummary, ProjectsRepoError> {
        let parent_asset_id =
            normalize_required_identifier(input.parent_asset_id.as_str(), "parent_asset_id")?;
        let child_asset_id =
            normalize_required_identifier(input.child_asset_id.as_str(), "child_asset_id")?;
        if parent_asset_id == child_asset_id {
            return Err(ProjectsRepoError::Validation(String::from(
                "parent_asset_id and child_asset_id must differ",
            )));
        }
        let link_type = normalize_asset_link_type(
            input
                .link_type
                .as_deref()
                .unwrap_or(ASSET_LINK_TYPE_DERIVED_FROM),
        )?;

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            ensure_asset_belongs_to_project(conn, project.id.as_str(), parent_asset_id.as_str())?;
            ensure_asset_belongs_to_project(conn, project.id.as_str(), child_asset_id.as_str())?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            let insert = conn.execute(
                "
                INSERT INTO asset_links
                  (id, project_id, parent_asset_id, child_asset_id, link_type, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?6)
            ",
                params![
                    id,
                    project.id,
                    parent_asset_id,
                    child_asset_id,
                    link_type,
                    now
                ],
            );

            if let Err(source) = insert {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Asset link already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_asset_link_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_asset_link_detail(
        &self,
        slug: &str,
        link_id: &str,
    ) -> Result<AssetLinkSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_asset_link_by_id(conn, project.id.as_str(), link_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn update_asset_link(
        &self,
        slug: &str,
        link_id: &str,
        input: UpdateAssetLinkInput,
    ) -> Result<AssetLinkSummary, ProjectsRepoError> {
        if input.parent_asset_id.is_none()
            && input.child_asset_id.is_none()
            && input.link_type.is_none()
        {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: parent_asset_id, child_asset_id, link_type",
            )));
        }

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing = fetch_asset_link_by_id(conn, project.id.as_str(), link_id)?
                .ok_or(ProjectsRepoError::NotFound)?;

            let parent_asset_id = if let Some(raw) = input.parent_asset_id.as_deref() {
                normalize_required_identifier(raw, "parent_asset_id")?
            } else {
                existing.parent_asset_id.clone()
            };
            let child_asset_id = if let Some(raw) = input.child_asset_id.as_deref() {
                normalize_required_identifier(raw, "child_asset_id")?
            } else {
                existing.child_asset_id.clone()
            };
            if parent_asset_id == child_asset_id {
                return Err(ProjectsRepoError::Validation(String::from(
                    "parent_asset_id and child_asset_id must differ",
                )));
            }

            let link_type = if let Some(raw) = input.link_type.as_deref() {
                normalize_asset_link_type(raw)?
            } else {
                existing.link_type.clone()
            };

            ensure_asset_belongs_to_project(conn, project.id.as_str(), parent_asset_id.as_str())?;
            ensure_asset_belongs_to_project(conn, project.id.as_str(), child_asset_id.as_str())?;

            let update = conn.execute(
                "
                UPDATE asset_links
                SET parent_asset_id = ?1,
                    child_asset_id = ?2,
                    link_type = ?3,
                    updated_at = ?4
                WHERE id = ?5 AND project_id = ?6
            ",
                params![
                    parent_asset_id,
                    child_asset_id,
                    link_type,
                    now_iso(),
                    link_id,
                    project.id
                ],
            );

            if let Err(source) = update {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Asset link already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_asset_link_by_id(conn, project.id.as_str(), link_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn delete_asset_link(&self, slug: &str, link_id: &str) -> Result<(), ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let affected = conn.execute(
                "DELETE FROM asset_links WHERE id = ?1 AND project_id = ?2",
                params![link_id, project.id],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }

    pub fn list_quality_reports(
        &self,
        slug: &str,
        limit: i64,
    ) -> Result<Vec<QualityReportSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 2000);

            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  project_id,
                  run_id,
                  asset_id,
                  report_type,
                  grade,
                  hard_failures,
                  soft_warnings,
                  avg_chroma_exceed,
                  summary_json,
                  created_at
                FROM quality_reports
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows =
                stmt.query_map(params![project.id, capped], row_to_quality_report_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn list_cost_events(
        &self,
        slug: &str,
        limit: i64,
    ) -> Result<Vec<CostEventSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 2000);

            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  project_id,
                  run_id,
                  job_id,
                  provider_code,
                  model_name,
                  event_type,
                  units,
                  unit_cost_usd,
                  total_cost_usd,
                  currency,
                  meta_json,
                  created_at
                FROM cost_events
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows = stmt.query_map(params![project.id, capped], row_to_cost_event_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn list_project_exports(
        &self,
        slug: &str,
        limit: i64,
    ) -> Result<Vec<ProjectExportSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 2000);

            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  project_id,
                  run_id,
                  status,
                  export_format,
                  storage_uri,
                  rel_path,
                  file_size_bytes,
                  checksum_sha256,
                  manifest_json,
                  created_at,
                  completed_at
                FROM project_exports
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows =
                stmt.query_map(params![project.id, capped], row_to_project_export_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn get_project_export_detail(
        &self,
        slug: &str,
        export_id: &str,
    ) -> Result<ProjectExportSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            conn.query_row(
                "
                SELECT
                  id,
                  project_id,
                  run_id,
                  status,
                  export_format,
                  storage_uri,
                  rel_path,
                  file_size_bytes,
                  checksum_sha256,
                  manifest_json,
                  created_at,
                  completed_at
                FROM project_exports
                WHERE id = ?1 AND project_id = ?2
                LIMIT 1
            ",
                params![export_id, project.id],
                row_to_project_export_summary,
            )
            .optional()?
            .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn list_prompt_templates(
        &self,
        slug: &str,
    ) -> Result<Vec<PromptTemplateSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, name, template_text, created_at, updated_at
                FROM prompt_templates
                WHERE project_id = ?1
                ORDER BY COALESCE(updated_at, '') DESC, id DESC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_prompt_template_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_prompt_template(
        &self,
        slug: &str,
        input: CreatePromptTemplateInput,
    ) -> Result<PromptTemplateSummary, ProjectsRepoError> {
        let name = normalize_required_text(input.name.as_str(), "name")?;
        let template_text = normalize_required_text(input.template_text.as_str(), "template_text")?;

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            let insert = conn.execute(
                "
                INSERT INTO prompt_templates
                  (id, project_id, name, template_text, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?5)
            ",
                params![id, project.id, name, template_text, now],
            );
            if let Err(source) = insert {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Prompt template name already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_prompt_template_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_prompt_template_detail(
        &self,
        slug: &str,
        template_id: &str,
    ) -> Result<PromptTemplateSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_prompt_template_by_id(conn, project.id.as_str(), template_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn update_prompt_template(
        &self,
        slug: &str,
        template_id: &str,
        input: UpdatePromptTemplateInput,
    ) -> Result<PromptTemplateSummary, ProjectsRepoError> {
        if input.name.is_none() && input.template_text.is_none() {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: name, template_text",
            )));
        }

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing = fetch_prompt_template_by_id(conn, project.id.as_str(), template_id)?
                .ok_or(ProjectsRepoError::NotFound)?;

            let name = if let Some(raw) = input.name.as_deref() {
                normalize_required_text(raw, "name")?
            } else {
                existing.name
            };
            let template_text = if let Some(raw) = input.template_text.as_deref() {
                normalize_required_text(raw, "template_text")?
            } else {
                existing.template_text
            };

            let update = conn.execute(
                "
                UPDATE prompt_templates
                SET name = ?1, template_text = ?2, updated_at = ?3
                WHERE id = ?4 AND project_id = ?5
            ",
                params![name, template_text, now_iso(), template_id, project.id],
            );
            if let Err(source) = update {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Prompt template name already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_prompt_template_by_id(conn, project.id.as_str(), template_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn delete_prompt_template(
        &self,
        slug: &str,
        template_id: &str,
    ) -> Result<(), ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let affected = conn.execute(
                "DELETE FROM prompt_templates WHERE id = ?1 AND project_id = ?2",
                params![template_id, project.id],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }

    pub fn list_provider_accounts(
        &self,
        slug: &str,
    ) -> Result<Vec<ProviderAccountSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT
                  project_id,
                  provider_code,
                  display_name,
                  account_ref,
                  base_url,
                  enabled,
                  config_json,
                  created_at,
                  updated_at
                FROM provider_accounts
                WHERE project_id = ?1
                ORDER BY provider_code ASC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_provider_account_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn upsert_provider_account(
        &self,
        slug: &str,
        input: UpsertProviderAccountInput,
    ) -> Result<ProviderAccountSummary, ProjectsRepoError> {
        let provider_code = normalize_provider_code(input.provider_code.as_str())?;
        let display_name = input
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| provider_code.clone());
        let account_ref = input
            .account_ref
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let base_url = input
            .base_url
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let enabled = input.enabled.unwrap_or(true);
        let config_json = serde_json::to_string(
            &input
                .config_json
                .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
        )
        .unwrap_or_else(|_| String::from("{}"));

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let now = now_iso();
            conn.execute(
                "
                INSERT INTO provider_accounts (
                    project_id, provider_code, display_name, account_ref, base_url,
                    enabled, config_json, created_at, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
                ON CONFLICT(project_id, provider_code) DO UPDATE SET
                    display_name = excluded.display_name,
                    account_ref = excluded.account_ref,
                    base_url = excluded.base_url,
                    enabled = excluded.enabled,
                    config_json = excluded.config_json,
                    updated_at = excluded.updated_at
            ",
                params![
                    project.id,
                    provider_code,
                    display_name,
                    account_ref,
                    base_url,
                    if enabled { 1 } else { 0 },
                    config_json,
                    now
                ],
            )?;

            fetch_provider_account_by_code(conn, project.id.as_str(), provider_code.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_provider_account_detail(
        &self,
        slug: &str,
        provider_code: &str,
    ) -> Result<ProviderAccountSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let safe_provider_code =
                normalize_slug(provider_code).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_provider_account_by_code(conn, project.id.as_str(), safe_provider_code.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn update_provider_account(
        &self,
        slug: &str,
        provider_code: &str,
        input: UpdateProviderAccountInput,
    ) -> Result<ProviderAccountSummary, ProjectsRepoError> {
        if input.display_name.is_none()
            && input.account_ref.is_none()
            && input.base_url.is_none()
            && input.enabled.is_none()
            && input.config_json.is_none()
        {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: display_name, account_ref, base_url, enabled, config_json",
            )));
        }

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let safe_provider_code =
                normalize_slug(provider_code).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing = fetch_provider_account_by_code(
                conn,
                project.id.as_str(),
                safe_provider_code.as_str(),
            )?
            .ok_or(ProjectsRepoError::NotFound)?;

            let display_name = if let Some(raw) = input.display_name.as_deref() {
                normalize_required_text(raw, "display_name")?
            } else {
                existing.display_name
            };
            let account_ref = if let Some(raw) = input.account_ref.as_deref() {
                normalize_optional_storage_field(raw)
            } else if existing.account_ref.trim().is_empty() {
                None
            } else {
                Some(existing.account_ref)
            };
            let base_url = if let Some(raw) = input.base_url.as_deref() {
                normalize_optional_storage_field(raw)
            } else if existing.base_url.trim().is_empty() {
                None
            } else {
                Some(existing.base_url)
            };
            let enabled = input.enabled.unwrap_or(existing.enabled);
            let config_json_value = input.config_json.unwrap_or(existing.config_json);
            let config_json =
                serde_json::to_string(&config_json_value).unwrap_or_else(|_| String::from("{}"));

            conn.execute(
                "
                UPDATE provider_accounts
                SET display_name = ?1,
                    account_ref = ?2,
                    base_url = ?3,
                    enabled = ?4,
                    config_json = ?5,
                    updated_at = ?6
                WHERE project_id = ?7 AND provider_code = ?8
            ",
                params![
                    display_name,
                    account_ref,
                    base_url,
                    if enabled { 1 } else { 0 },
                    config_json,
                    now_iso(),
                    project.id,
                    safe_provider_code
                ],
            )?;

            fetch_provider_account_by_code(conn, project.id.as_str(), safe_provider_code.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn delete_provider_account(
        &self,
        slug: &str,
        provider_code: &str,
    ) -> Result<(), ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let safe_provider_code =
                normalize_slug(provider_code).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let affected = conn.execute(
                "DELETE FROM provider_accounts WHERE project_id = ?1 AND provider_code = ?2",
                params![project.id, safe_provider_code],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }

    pub fn list_style_guides(
        &self,
        slug: &str,
    ) -> Result<Vec<StyleGuideSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, name, instructions, notes, created_at, updated_at
                FROM style_guides
                WHERE project_id = ?1
                ORDER BY COALESCE(updated_at, '') DESC, id DESC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_style_guide_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_style_guide(
        &self,
        slug: &str,
        input: CreateStyleGuideInput,
    ) -> Result<StyleGuideSummary, ProjectsRepoError> {
        let name = normalize_required_text(input.name.as_str(), "name")?;
        let instructions = normalize_required_text(input.instructions.as_str(), "instructions")?;
        let notes = input
            .notes
            .as_deref()
            .and_then(normalize_optional_storage_field);

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            let insert = conn.execute(
                "
                INSERT INTO style_guides
                  (id, project_id, name, instructions, notes, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?6)
            ",
                params![id, project.id, name, instructions, notes, now],
            );
            if let Err(source) = insert {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Style guide name already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_style_guide_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_style_guide_detail(
        &self,
        slug: &str,
        style_guide_id: &str,
    ) -> Result<StyleGuideSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_style_guide_by_id(conn, project.id.as_str(), style_guide_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn update_style_guide(
        &self,
        slug: &str,
        style_guide_id: &str,
        input: UpdateStyleGuideInput,
    ) -> Result<StyleGuideSummary, ProjectsRepoError> {
        if input.name.is_none() && input.instructions.is_none() && input.notes.is_none() {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: name, instructions, notes",
            )));
        }

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing = fetch_style_guide_by_id(conn, project.id.as_str(), style_guide_id)?
                .ok_or(ProjectsRepoError::NotFound)?;

            let name = if let Some(raw) = input.name.as_deref() {
                normalize_required_text(raw, "name")?
            } else {
                existing.name
            };
            let instructions = if let Some(raw) = input.instructions.as_deref() {
                normalize_required_text(raw, "instructions")?
            } else {
                existing.instructions
            };
            let notes = if let Some(raw) = input.notes.as_deref() {
                normalize_optional_storage_field(raw)
            } else if existing.notes.trim().is_empty() {
                None
            } else {
                Some(existing.notes)
            };

            let update = conn.execute(
                "
                UPDATE style_guides
                SET name = ?1, instructions = ?2, notes = ?3, updated_at = ?4
                WHERE id = ?5 AND project_id = ?6
            ",
                params![
                    name,
                    instructions,
                    notes,
                    now_iso(),
                    style_guide_id,
                    project.id
                ],
            );
            if let Err(source) = update {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Style guide name already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_style_guide_by_id(conn, project.id.as_str(), style_guide_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn delete_style_guide(
        &self,
        slug: &str,
        style_guide_id: &str,
    ) -> Result<(), ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let affected = conn.execute(
                "DELETE FROM style_guides WHERE id = ?1 AND project_id = ?2",
                params![style_guide_id, project.id],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }

    pub fn list_characters(&self, slug: &str) -> Result<Vec<CharacterSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, name, description, prompt_text, created_at, updated_at
                FROM characters
                WHERE project_id = ?1
                ORDER BY COALESCE(updated_at, '') DESC, id DESC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_character_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_character(
        &self,
        slug: &str,
        input: CreateCharacterInput,
    ) -> Result<CharacterSummary, ProjectsRepoError> {
        let name = normalize_required_text(input.name.as_str(), "name")?;
        let description = input
            .description
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let prompt_text = input
            .prompt_text
            .as_deref()
            .and_then(normalize_optional_storage_field);

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            let insert = conn.execute(
                "
                INSERT INTO characters
                  (id, project_id, name, description, prompt_text, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?6)
            ",
                params![id, project.id, name, description, prompt_text, now],
            );
            if let Err(source) = insert {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Character name already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_character_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_character_detail(
        &self,
        slug: &str,
        character_id: &str,
    ) -> Result<CharacterSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            fetch_character_by_id(conn, project.id.as_str(), character_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn update_character(
        &self,
        slug: &str,
        character_id: &str,
        input: UpdateCharacterInput,
    ) -> Result<CharacterSummary, ProjectsRepoError> {
        if input.name.is_none() && input.description.is_none() && input.prompt_text.is_none() {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: name, description, prompt_text",
            )));
        }

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing = fetch_character_by_id(conn, project.id.as_str(), character_id)?
                .ok_or(ProjectsRepoError::NotFound)?;

            let name = if let Some(raw) = input.name.as_deref() {
                normalize_required_text(raw, "name")?
            } else {
                existing.name
            };
            let description = if let Some(raw) = input.description.as_deref() {
                normalize_optional_storage_field(raw)
            } else if existing.description.trim().is_empty() {
                None
            } else {
                Some(existing.description)
            };
            let prompt_text = if let Some(raw) = input.prompt_text.as_deref() {
                normalize_optional_storage_field(raw)
            } else if existing.prompt_text.trim().is_empty() {
                None
            } else {
                Some(existing.prompt_text)
            };

            let update = conn.execute(
                "
                UPDATE characters
                SET name = ?1, description = ?2, prompt_text = ?3, updated_at = ?4
                WHERE id = ?5 AND project_id = ?6
            ",
                params![
                    name,
                    description,
                    prompt_text,
                    now_iso(),
                    character_id,
                    project.id
                ],
            );
            if let Err(source) = update {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Character name already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_character_by_id(conn, project.id.as_str(), character_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn delete_character(
        &self,
        slug: &str,
        character_id: &str,
    ) -> Result<(), ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let affected = conn.execute(
                "DELETE FROM characters WHERE id = ?1 AND project_id = ?2",
                params![character_id, project.id],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }

    pub fn list_reference_sets(
        &self,
        slug: &str,
    ) -> Result<Vec<ReferenceSetSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, name, description, created_at, updated_at
                FROM reference_sets
                WHERE project_id = ?1
                ORDER BY COALESCE(updated_at, '') DESC, id DESC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_reference_set_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_reference_set(
        &self,
        slug: &str,
        input: CreateReferenceSetInput,
    ) -> Result<ReferenceSetSummary, ProjectsRepoError> {
        let name = normalize_required_text(input.name.as_str(), "name")?;
        let description = input
            .description
            .as_deref()
            .and_then(normalize_optional_storage_field);

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            let insert = conn.execute(
                "
                INSERT INTO reference_sets
                  (id, project_id, name, description, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?5)
            ",
                params![id, project.id, name, description, now],
            );
            if let Err(source) = insert {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Reference set name already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_reference_set_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_reference_set_detail(
        &self,
        slug: &str,
        reference_set_id: &str,
    ) -> Result<ReferenceSetSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            fetch_reference_set_by_id(conn, project.id.as_str(), reference_set_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn update_reference_set(
        &self,
        slug: &str,
        reference_set_id: &str,
        input: UpdateReferenceSetInput,
    ) -> Result<ReferenceSetSummary, ProjectsRepoError> {
        if input.name.is_none() && input.description.is_none() {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: name, description",
            )));
        }

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing = fetch_reference_set_by_id(conn, project.id.as_str(), reference_set_id)?
                .ok_or(ProjectsRepoError::NotFound)?;

            let name = if let Some(raw) = input.name.as_deref() {
                normalize_required_text(raw, "name")?
            } else {
                existing.name
            };
            let description = if let Some(raw) = input.description.as_deref() {
                normalize_optional_storage_field(raw)
            } else if existing.description.trim().is_empty() {
                None
            } else {
                Some(existing.description)
            };

            let update = conn.execute(
                "
                UPDATE reference_sets
                SET name = ?1, description = ?2, updated_at = ?3
                WHERE id = ?4 AND project_id = ?5
            ",
                params![name, description, now_iso(), reference_set_id, project.id],
            );
            if let Err(source) = update {
                if is_unique_constraint_error(&source) {
                    return Err(ProjectsRepoError::Validation(String::from(
                        "Reference set name already exists",
                    )));
                }
                return Err(ProjectsRepoError::Sqlite(source));
            }

            fetch_reference_set_by_id(conn, project.id.as_str(), reference_set_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn delete_reference_set(
        &self,
        slug: &str,
        reference_set_id: &str,
    ) -> Result<(), ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let affected = conn.execute(
                "DELETE FROM reference_sets WHERE id = ?1 AND project_id = ?2",
                params![reference_set_id, project.id],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }

    pub fn list_reference_set_items(
        &self,
        slug: &str,
        reference_set_id: &str,
    ) -> Result<Vec<ReferenceSetItemSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let exists =
                fetch_reference_set_by_id(conn, project.id.as_str(), reference_set_id)?.is_some();
            if !exists {
                return Err(ProjectsRepoError::NotFound);
            }

            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  project_id,
                  reference_set_id,
                  label,
                  content_uri,
                  content_text,
                  sort_order,
                  metadata_json,
                  created_at,
                  updated_at
                FROM reference_set_items
                WHERE project_id = ?1 AND reference_set_id = ?2
                ORDER BY sort_order ASC, COALESCE(updated_at, '') DESC, id DESC
            ",
            )?;
            let rows = stmt.query_map(
                params![project.id, reference_set_id],
                row_to_reference_set_item_summary,
            )?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_reference_set_item(
        &self,
        slug: &str,
        reference_set_id: &str,
        input: CreateReferenceSetItemInput,
    ) -> Result<ReferenceSetItemSummary, ProjectsRepoError> {
        let label = normalize_required_text(input.label.as_str(), "label")?;
        let content_uri = input
            .content_uri
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let content_text = input
            .content_text
            .as_deref()
            .and_then(normalize_optional_storage_field);
        if content_uri.is_none() && content_text.is_none() {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: content_uri, content_text",
            )));
        }
        let sort_order = input.sort_order.unwrap_or(0);
        let metadata_json = serde_json::to_string(
            &input
                .metadata_json
                .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
        )
        .unwrap_or_else(|_| String::from("{}"));

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let exists =
                fetch_reference_set_by_id(conn, project.id.as_str(), reference_set_id)?.is_some();
            if !exists {
                return Err(ProjectsRepoError::NotFound);
            }

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO reference_set_items
                  (id, project_id, reference_set_id, label, content_uri, content_text, sort_order, metadata_json, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
            ",
                params![
                    id,
                    project.id,
                    reference_set_id,
                    label,
                    content_uri,
                    content_text,
                    sort_order,
                    metadata_json,
                    now
                ],
            )?;

            fetch_reference_set_item_by_id(conn, project.id.as_str(), reference_set_id, id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_reference_set_item_detail(
        &self,
        slug: &str,
        reference_set_id: &str,
        item_id: &str,
    ) -> Result<ReferenceSetItemSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_reference_set_item_by_id(conn, project.id.as_str(), reference_set_id, item_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn update_reference_set_item(
        &self,
        slug: &str,
        reference_set_id: &str,
        item_id: &str,
        input: UpdateReferenceSetItemInput,
    ) -> Result<ReferenceSetItemSummary, ProjectsRepoError> {
        if input.label.is_none()
            && input.content_uri.is_none()
            && input.content_text.is_none()
            && input.sort_order.is_none()
            && input.metadata_json.is_none()
        {
            return Err(ProjectsRepoError::Validation(String::from(
                "Provide at least one of: label, content_uri, content_text, sort_order, metadata_json",
            )));
        }

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing = fetch_reference_set_item_by_id(
                conn,
                project.id.as_str(),
                reference_set_id,
                item_id,
            )?
            .ok_or(ProjectsRepoError::NotFound)?;

            let label = if let Some(raw) = input.label.as_deref() {
                normalize_required_text(raw, "label")?
            } else {
                existing.label
            };
            let content_uri = if let Some(raw) = input.content_uri.as_deref() {
                normalize_optional_storage_field(raw)
            } else if existing.content_uri.trim().is_empty() {
                None
            } else {
                Some(existing.content_uri)
            };
            let content_text = if let Some(raw) = input.content_text.as_deref() {
                normalize_optional_storage_field(raw)
            } else if existing.content_text.trim().is_empty() {
                None
            } else {
                Some(existing.content_text)
            };
            if content_uri.is_none() && content_text.is_none() {
                return Err(ProjectsRepoError::Validation(String::from(
                    "Provide at least one of: content_uri, content_text",
                )));
            }

            let sort_order = input.sort_order.unwrap_or(existing.sort_order);
            let metadata_json_value = input.metadata_json.unwrap_or(existing.metadata_json);
            let metadata_json =
                serde_json::to_string(&metadata_json_value).unwrap_or_else(|_| String::from("{}"));

            conn.execute(
                "
                UPDATE reference_set_items
                SET label = ?1,
                    content_uri = ?2,
                    content_text = ?3,
                    sort_order = ?4,
                    metadata_json = ?5,
                    updated_at = ?6
                WHERE id = ?7 AND project_id = ?8 AND reference_set_id = ?9
            ",
                params![
                    label,
                    content_uri,
                    content_text,
                    sort_order,
                    metadata_json,
                    now_iso(),
                    item_id,
                    project.id,
                    reference_set_id
                ],
            )?;

            fetch_reference_set_item_by_id(conn, project.id.as_str(), reference_set_id, item_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn delete_reference_set_item(
        &self,
        slug: &str,
        reference_set_id: &str,
        item_id: &str,
    ) -> Result<(), ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let affected = conn.execute(
                "
                DELETE FROM reference_set_items
                WHERE id = ?1 AND project_id = ?2 AND reference_set_id = ?3
            ",
                params![item_id, project.id, reference_set_id],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }

    pub fn list_chat_sessions(
        &self,
        slug: &str,
    ) -> Result<Vec<ChatSessionSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, title, status, created_at, updated_at
                FROM chat_sessions
                WHERE project_id = ?1
                ORDER BY COALESCE(updated_at, '') DESC, id DESC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_chat_session_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_chat_session(
        &self,
        slug: &str,
        input: CreateChatSessionInput,
    ) -> Result<ChatSessionSummary, ProjectsRepoError> {
        let title = input
            .title
            .as_deref()
            .and_then(normalize_optional_storage_field)
            .unwrap_or_else(|| String::from("New Session"));

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO chat_sessions
                  (id, project_id, title, status, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, 'active', ?4, ?4)
            ",
                params![id, project.id, title, now],
            )?;

            fetch_chat_session_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_chat_session_detail(
        &self,
        slug: &str,
        session_id: &str,
    ) -> Result<ChatSessionSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_chat_session_by_id(conn, project.id.as_str(), session_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn list_chat_messages(
        &self,
        slug: &str,
        session_id: &str,
    ) -> Result<Vec<ChatMessageSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let session_exists =
                fetch_chat_session_by_id(conn, project.id.as_str(), session_id)?.is_some();
            if !session_exists {
                return Err(ProjectsRepoError::NotFound);
            }

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, session_id, role, content, created_at
                FROM chat_messages
                WHERE project_id = ?1 AND session_id = ?2
                ORDER BY rowid ASC
            ",
            )?;
            let rows =
                stmt.query_map(params![project.id, session_id], row_to_chat_message_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_chat_message(
        &self,
        slug: &str,
        session_id: &str,
        input: CreateChatMessageInput,
    ) -> Result<ChatMessageSummary, ProjectsRepoError> {
        let role = normalize_chat_role(input.role.as_str())?;
        let content = normalize_required_text(input.content.as_str(), "content")?;

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let session_exists =
                fetch_chat_session_by_id(conn, project.id.as_str(), session_id)?.is_some();
            if !session_exists {
                return Err(ProjectsRepoError::NotFound);
            }

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO chat_messages
                  (id, project_id, session_id, role, content, created_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6)
            ",
                params![id, project.id, session_id, role, content, now],
            )?;

            fetch_chat_message_by_id(conn, project.id.as_str(), session_id, id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }
}

#[derive(Debug, Clone)]
struct ProjectRow {
    id: String,
    slug: String,
    name: String,
    description: String,
    status: String,
    settings_json: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct StorageDefaults {
    local_base_dir: String,
    local_project_root_cfg: Option<String>,
    s3_enabled: bool,
    s3_bucket: String,
    s3_prefix: String,
    s3_region: String,
    s3_profile: String,
    s3_endpoint_url: String,
}

fn ensure_schema(conn: &Connection) -> Result<(), ProjectsRepoError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
          id TEXT PRIMARY KEY,
          username TEXT NOT NULL UNIQUE,
          display_name TEXT NOT NULL,
          email TEXT,
          is_active INTEGER NOT NULL DEFAULT 1,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_users (
          id TEXT PRIMARY KEY,
          username TEXT NOT NULL UNIQUE,
          display_name TEXT NOT NULL,
          email TEXT,
          is_active INTEGER NOT NULL DEFAULT 1,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL,
          slug TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT NOT NULL DEFAULT '',
          status TEXT NOT NULL DEFAULT 'active',
          settings_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          owner_user_id TEXT,
          UNIQUE(user_id, slug),
          FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS runs (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          status TEXT,
          stage TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS run_jobs (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          job_key TEXT,
          status TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS run_job_candidates (
          id TEXT PRIMARY KEY,
          job_id TEXT NOT NULL,
          candidate_index INTEGER,
          status TEXT,
          output_path TEXT,
          final_output_path TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS run_candidates (
          id TEXT PRIMARY KEY,
          job_id TEXT NOT NULL,
          candidate_index INTEGER,
          status TEXT,
          output_asset_id TEXT,
          final_asset_id TEXT,
          output_path TEXT,
          final_output_path TEXT,
          rank_hard_failures INTEGER,
          rank_soft_warnings INTEGER,
          rank_avg_chroma_exceed REAL,
          meta_json TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS assets (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          kind TEXT,
          storage_uri TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS asset_links (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          parent_asset_id TEXT NOT NULL,
          child_asset_id TEXT NOT NULL,
          link_type TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, parent_asset_id, child_asset_id, link_type),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS quality_reports (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          asset_id TEXT,
          report_type TEXT,
          grade TEXT,
          hard_failures INTEGER,
          soft_warnings INTEGER,
          avg_chroma_exceed REAL,
          summary_json TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS cost_events (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          job_id TEXT,
          provider_code TEXT,
          model_name TEXT,
          event_type TEXT,
          units REAL,
          unit_cost_usd REAL,
          total_cost_usd REAL,
          currency TEXT,
          meta_json TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS project_exports (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          status TEXT,
          export_format TEXT,
          storage_uri TEXT,
          rel_path TEXT,
          file_size_bytes INTEGER,
          checksum_sha256 TEXT,
          manifest_json TEXT,
          created_at TEXT,
          completed_at TEXT
        );

        CREATE TABLE IF NOT EXISTS prompt_templates (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          template_text TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );

        CREATE TABLE IF NOT EXISTS provider_accounts (
          project_id TEXT NOT NULL,
          provider_code TEXT NOT NULL,
          display_name TEXT NOT NULL,
          account_ref TEXT,
          base_url TEXT,
          enabled INTEGER NOT NULL DEFAULT 1,
          config_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY(project_id, provider_code)
        );

        CREATE TABLE IF NOT EXISTS style_guides (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          instructions TEXT NOT NULL,
          notes TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );

        CREATE TABLE IF NOT EXISTS characters (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT,
          prompt_text TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );

        CREATE TABLE IF NOT EXISTS reference_sets (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );

        CREATE TABLE IF NOT EXISTS reference_set_items (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          reference_set_id TEXT NOT NULL,
          label TEXT NOT NULL,
          content_uri TEXT,
          content_text TEXT,
          sort_order INTEGER NOT NULL DEFAULT 0,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chat_sessions (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          title TEXT NOT NULL,
          status TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chat_messages (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          session_id TEXT NOT NULL,
          role TEXT NOT NULL,
          content TEXT NOT NULL,
          created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS project_storage (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL UNIQUE,
          total_bytes INTEGER NOT NULL DEFAULT 0,
          used_bytes INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          local_base_dir TEXT NOT NULL DEFAULT 'var/projects',
          local_project_root TEXT,
          s3_enabled INTEGER NOT NULL DEFAULT 0,
          s3_bucket TEXT,
          s3_prefix TEXT,
          s3_region TEXT,
          s3_profile TEXT,
          s3_endpoint_url TEXT,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );
    ",
    )?;

    ensure_column(conn, "projects", "owner_user_id", "TEXT")?;
    ensure_column(
        conn,
        "project_storage",
        "local_base_dir",
        "TEXT NOT NULL DEFAULT 'var/projects'",
    )?;
    ensure_column(conn, "project_storage", "local_project_root", "TEXT")?;
    ensure_column(
        conn,
        "project_storage",
        "s3_enabled",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(conn, "project_storage", "s3_bucket", "TEXT")?;
    ensure_column(conn, "project_storage", "s3_prefix", "TEXT")?;
    ensure_column(conn, "project_storage", "s3_region", "TEXT")?;
    ensure_column(conn, "project_storage", "s3_profile", "TEXT")?;
    ensure_column(conn, "project_storage", "s3_endpoint_url", "TEXT")?;

    ensure_column(conn, "runs", "run_log_path", "TEXT")?;
    ensure_column(conn, "runs", "run_mode", "TEXT")?;
    ensure_column(conn, "runs", "mode", "TEXT")?;
    ensure_column(conn, "runs", "stage", "TEXT")?;
    ensure_column(conn, "runs", "time_of_day", "TEXT")?;
    ensure_column(conn, "runs", "weather", "TEXT")?;
    ensure_column(conn, "runs", "model_name", "TEXT")?;
    ensure_column(conn, "runs", "model", "TEXT")?;
    ensure_column(conn, "runs", "provider_code", "TEXT")?;
    ensure_column(conn, "runs", "settings_snapshot_json", "TEXT")?;
    ensure_column(conn, "runs", "meta_json", "TEXT")?;
    ensure_column(conn, "runs", "started_at", "TEXT")?;
    ensure_column(conn, "runs", "finished_at", "TEXT")?;
    ensure_column(conn, "runs", "status", "TEXT")?;
    ensure_column(conn, "runs", "image_size", "TEXT")?;
    ensure_column(conn, "runs", "image_quality", "TEXT")?;
    ensure_column(conn, "runs", "created_at", "TEXT")?;

    ensure_column(conn, "run_jobs", "job_key", "TEXT")?;
    ensure_column(conn, "run_jobs", "status", "TEXT")?;
    ensure_column(conn, "run_jobs", "prompt_text", "TEXT")?;
    ensure_column(conn, "run_jobs", "selected_candidate", "INTEGER")?;
    ensure_column(conn, "run_jobs", "selected_candidate_index", "INTEGER")?;
    ensure_column(conn, "run_jobs", "final_asset_id", "TEXT")?;
    ensure_column(conn, "run_jobs", "final_output", "TEXT")?;
    ensure_column(conn, "run_jobs", "meta_json", "TEXT")?;
    ensure_column(conn, "run_jobs", "created_at", "TEXT")?;

    ensure_column(conn, "run_job_candidates", "candidate_index", "INTEGER")?;
    ensure_column(conn, "run_job_candidates", "status", "TEXT")?;
    ensure_column(conn, "run_job_candidates", "output_path", "TEXT")?;
    ensure_column(conn, "run_job_candidates", "final_output_path", "TEXT")?;
    ensure_column(conn, "run_job_candidates", "rank_hard_failures", "INTEGER")?;
    ensure_column(conn, "run_job_candidates", "rank_soft_warnings", "INTEGER")?;
    ensure_column(conn, "run_job_candidates", "rank_avg_chroma_exceed", "REAL")?;
    ensure_column(conn, "run_job_candidates", "meta_json", "TEXT")?;
    ensure_column(conn, "run_job_candidates", "created_at", "TEXT")?;

    ensure_column(conn, "run_candidates", "candidate_index", "INTEGER")?;
    ensure_column(conn, "run_candidates", "status", "TEXT")?;
    ensure_column(conn, "run_candidates", "output_asset_id", "TEXT")?;
    ensure_column(conn, "run_candidates", "final_asset_id", "TEXT")?;
    ensure_column(conn, "run_candidates", "output_path", "TEXT")?;
    ensure_column(conn, "run_candidates", "final_output_path", "TEXT")?;
    ensure_column(conn, "run_candidates", "rank_hard_failures", "INTEGER")?;
    ensure_column(conn, "run_candidates", "rank_soft_warnings", "INTEGER")?;
    ensure_column(conn, "run_candidates", "rank_avg_chroma_exceed", "REAL")?;
    ensure_column(conn, "run_candidates", "meta_json", "TEXT")?;
    ensure_column(conn, "run_candidates", "created_at", "TEXT")?;

    ensure_column(conn, "assets", "kind", "TEXT")?;
    ensure_column(conn, "assets", "asset_kind", "TEXT")?;
    ensure_column(conn, "assets", "storage_uri", "TEXT")?;
    ensure_column(conn, "assets", "rel_path", "TEXT")?;
    ensure_column(conn, "assets", "storage_backend", "TEXT")?;
    ensure_column(conn, "assets", "mime_type", "TEXT")?;
    ensure_column(conn, "assets", "width", "INTEGER")?;
    ensure_column(conn, "assets", "height", "INTEGER")?;
    ensure_column(conn, "assets", "sha256", "TEXT")?;
    ensure_column(conn, "assets", "run_id", "TEXT")?;
    ensure_column(conn, "assets", "job_id", "TEXT")?;
    ensure_column(conn, "assets", "candidate_id", "TEXT")?;
    ensure_column(conn, "assets", "metadata_json", "TEXT")?;
    ensure_column(conn, "assets", "meta_json", "TEXT")?;
    ensure_column(conn, "assets", "created_at", "TEXT")?;

    ensure_column(conn, "asset_links", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "asset_links", "parent_asset_id", "TEXT NOT NULL")?;
    ensure_column(conn, "asset_links", "child_asset_id", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "asset_links",
        "link_type",
        "TEXT NOT NULL DEFAULT 'derived_from'",
    )?;
    ensure_column(
        conn,
        "asset_links",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "asset_links",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "quality_reports", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "quality_reports", "run_id", "TEXT")?;
    ensure_column(conn, "quality_reports", "asset_id", "TEXT")?;
    ensure_column(conn, "quality_reports", "report_type", "TEXT")?;
    ensure_column(conn, "quality_reports", "grade", "TEXT")?;
    ensure_column(conn, "quality_reports", "hard_failures", "INTEGER")?;
    ensure_column(conn, "quality_reports", "soft_warnings", "INTEGER")?;
    ensure_column(conn, "quality_reports", "avg_chroma_exceed", "REAL")?;
    ensure_column(conn, "quality_reports", "summary_json", "TEXT")?;
    ensure_column(conn, "quality_reports", "meta_json", "TEXT")?;
    ensure_column(conn, "quality_reports", "created_at", "TEXT")?;

    ensure_column(conn, "cost_events", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "cost_events", "run_id", "TEXT")?;
    ensure_column(conn, "cost_events", "job_id", "TEXT")?;
    ensure_column(conn, "cost_events", "provider_code", "TEXT")?;
    ensure_column(conn, "cost_events", "model_name", "TEXT")?;
    ensure_column(conn, "cost_events", "event_type", "TEXT")?;
    ensure_column(conn, "cost_events", "units", "REAL")?;
    ensure_column(conn, "cost_events", "unit_cost_usd", "REAL")?;
    ensure_column(conn, "cost_events", "total_cost_usd", "REAL")?;
    ensure_column(conn, "cost_events", "currency", "TEXT")?;
    ensure_column(conn, "cost_events", "meta_json", "TEXT")?;
    ensure_column(conn, "cost_events", "created_at", "TEXT")?;

    ensure_column(conn, "project_exports", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "project_exports", "run_id", "TEXT")?;
    ensure_column(conn, "project_exports", "status", "TEXT")?;
    ensure_column(conn, "project_exports", "export_format", "TEXT")?;
    ensure_column(conn, "project_exports", "storage_uri", "TEXT")?;
    ensure_column(conn, "project_exports", "rel_path", "TEXT")?;
    ensure_column(conn, "project_exports", "file_size_bytes", "INTEGER")?;
    ensure_column(conn, "project_exports", "checksum_sha256", "TEXT")?;
    ensure_column(conn, "project_exports", "manifest_json", "TEXT")?;
    ensure_column(conn, "project_exports", "meta_json", "TEXT")?;
    ensure_column(conn, "project_exports", "created_at", "TEXT")?;
    ensure_column(conn, "project_exports", "completed_at", "TEXT")?;

    ensure_column(conn, "prompt_templates", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "prompt_templates", "name", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "prompt_templates",
        "template_text",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "prompt_templates",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "prompt_templates",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "provider_accounts", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "provider_accounts", "provider_code", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "provider_accounts",
        "display_name",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(conn, "provider_accounts", "account_ref", "TEXT")?;
    ensure_column(conn, "provider_accounts", "base_url", "TEXT")?;
    ensure_column(
        conn,
        "provider_accounts",
        "enabled",
        "INTEGER NOT NULL DEFAULT 1",
    )?;
    ensure_column(
        conn,
        "provider_accounts",
        "config_json",
        "TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column(
        conn,
        "provider_accounts",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "provider_accounts",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "style_guides", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "style_guides", "name", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "style_guides",
        "instructions",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(conn, "style_guides", "notes", "TEXT")?;
    ensure_column(
        conn,
        "style_guides",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "style_guides",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "characters", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "characters", "name", "TEXT NOT NULL")?;
    ensure_column(conn, "characters", "description", "TEXT")?;
    ensure_column(conn, "characters", "prompt_text", "TEXT")?;
    ensure_column(conn, "characters", "created_at", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(conn, "characters", "updated_at", "TEXT NOT NULL DEFAULT ''")?;

    ensure_column(conn, "reference_sets", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "reference_sets", "name", "TEXT NOT NULL")?;
    ensure_column(conn, "reference_sets", "description", "TEXT")?;
    ensure_column(
        conn,
        "reference_sets",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "reference_sets",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "reference_set_items", "project_id", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "reference_set_items",
        "reference_set_id",
        "TEXT NOT NULL",
    )?;
    ensure_column(
        conn,
        "reference_set_items",
        "label",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(conn, "reference_set_items", "content_uri", "TEXT")?;
    ensure_column(conn, "reference_set_items", "content_text", "TEXT")?;
    ensure_column(
        conn,
        "reference_set_items",
        "sort_order",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "reference_set_items",
        "metadata_json",
        "TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column(
        conn,
        "reference_set_items",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "reference_set_items",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "chat_sessions", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "chat_sessions", "title", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(
        conn,
        "chat_sessions",
        "status",
        "TEXT NOT NULL DEFAULT 'active'",
    )?;
    ensure_column(
        conn,
        "chat_sessions",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "chat_sessions",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "chat_messages", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "chat_messages", "session_id", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "chat_messages",
        "role",
        "TEXT NOT NULL DEFAULT 'user'",
    )?;
    ensure_column(conn, "chat_messages", "content", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(
        conn,
        "chat_messages",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    Ok(())
}

fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), ProjectsRepoError> {
    if table_has_column(conn, table, column)? {
        return Ok(());
    }
    conn.execute(
        format!("ALTER TABLE {table} ADD COLUMN {column} {definition}").as_str(),
        [],
    )?;
    Ok(())
}

fn table_has_column(
    conn: &Connection,
    table: &str,
    column: &str,
) -> Result<bool, ProjectsRepoError> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(pragma.as_str())?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get("name")?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn fetch_project_by_id(
    conn: &Connection,
    project_id: &str,
) -> Result<Option<ProjectRow>, ProjectsRepoError> {
    let mut stmt = conn.prepare(
        "
        SELECT id, slug, name, description, status, settings_json, created_at, updated_at
        FROM projects
        WHERE id = ?1
        LIMIT 1
    ",
    )?;
    stmt.query_row([project_id], row_to_project)
        .optional()
        .map_err(ProjectsRepoError::from)
}

fn fetch_project_by_slug(
    conn: &Connection,
    slug: &str,
) -> Result<Option<ProjectRow>, ProjectsRepoError> {
    let mut stmt = conn.prepare(
        "
        SELECT id, slug, name, description, status, settings_json, created_at, updated_at
        FROM projects
        WHERE slug = ?1
        LIMIT 1
    ",
    )?;
    stmt.query_row([slug], row_to_project)
        .optional()
        .map_err(ProjectsRepoError::from)
}

fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRow> {
    Ok(ProjectRow {
        id: row.get("id")?,
        slug: row.get("slug")?,
        name: row.get("name")?,
        description: row
            .get::<_, Option<String>>("description")?
            .unwrap_or_default(),
        status: row
            .get::<_, Option<String>>("status")?
            .unwrap_or_else(|| String::from("active")),
        settings_json: row
            .get::<_, Option<String>>("settings_json")?
            .unwrap_or_else(|| String::from("{}")),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn row_to_run_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunSummary> {
    let settings_snapshot_json = parse_json_value(
        row.get::<_, Option<String>>("settings_snapshot_json")?
            .or(row.get::<_, Option<String>>("meta_json")?),
    );

    Ok(RunSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        run_mode: row_string_from_columns(row, &["run_mode", "mode"])?,
        status: row_string_from_columns(row, &["status"])?,
        stage: row_string_from_columns(row, &["stage"])?,
        time_of_day: row_string_from_columns(row, &["time_of_day"])?,
        weather: row_string_from_columns(row, &["weather"])?,
        model_name: row_string_from_columns(row, &["model_name", "model"])?,
        provider_code: row_string_from_columns(row, &["provider_code"])?,
        settings_snapshot_json,
        started_at: row_string_from_columns(row, &["started_at"])?,
        finished_at: row_string_from_columns(row, &["finished_at"])?,
        created_at: row_string_from_columns(row, &["created_at"])?,
        run_log_path: row_string_from_columns(row, &["run_log_path"])?,
        image_size: row_string_from_columns(row, &["image_size"])?,
        image_quality: row_string_from_columns(row, &["image_quality"])?,
    })
}

fn row_to_run_candidate_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunCandidateSummary> {
    Ok(RunCandidateSummary {
        id: row.get("id")?,
        job_id: row.get("job_id")?,
        candidate_index: row
            .get::<_, Option<i64>>("candidate_index")?
            .unwrap_or_default(),
        status: row.get::<_, Option<String>>("status")?.unwrap_or_default(),
        output_asset_id: row
            .get::<_, Option<String>>("output_asset_id")?
            .unwrap_or_default(),
        final_asset_id: row
            .get::<_, Option<String>>("final_asset_id")?
            .unwrap_or_default(),
        output_path: row
            .get::<_, Option<String>>("output_path")?
            .unwrap_or_default(),
        final_output_path: row
            .get::<_, Option<String>>("final_output_path")?
            .unwrap_or_default(),
        rank_hard_failures: row
            .get::<_, Option<i64>>("rank_hard_failures")?
            .unwrap_or_default(),
        rank_soft_warnings: row
            .get::<_, Option<i64>>("rank_soft_warnings")?
            .unwrap_or_default(),
        rank_avg_chroma_exceed: row
            .get::<_, Option<f64>>("rank_avg_chroma_exceed")?
            .unwrap_or(0.0),
        meta_json: parse_json_value(row.get::<_, Option<String>>("meta_json")?),
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
    })
}

fn row_to_asset_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssetSummary> {
    let kind = row_string_from_columns(row, &["kind", "asset_kind"])?;
    let asset_kind = row_string_from_columns(row, &["asset_kind", "kind"])?;
    let metadata_json = parse_json_value(
        row.get::<_, Option<String>>("metadata_json")?
            .or(row.get::<_, Option<String>>("meta_json")?),
    );

    Ok(AssetSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        kind,
        asset_kind,
        storage_uri: row
            .get::<_, Option<String>>("storage_uri")?
            .unwrap_or_default(),
        rel_path: row
            .get::<_, Option<String>>("rel_path")?
            .unwrap_or_default(),
        storage_backend: row
            .get::<_, Option<String>>("storage_backend")?
            .unwrap_or_default(),
        mime_type: row
            .get::<_, Option<String>>("mime_type")?
            .unwrap_or_default(),
        width: row.get::<_, Option<i64>>("width")?,
        height: row.get::<_, Option<i64>>("height")?,
        sha256: row.get::<_, Option<String>>("sha256")?.unwrap_or_default(),
        run_id: row.get::<_, Option<String>>("run_id")?.unwrap_or_default(),
        job_id: row.get::<_, Option<String>>("job_id")?.unwrap_or_default(),
        candidate_id: row
            .get::<_, Option<String>>("candidate_id")?
            .unwrap_or_default(),
        metadata_json,
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
    })
}

fn row_to_asset_link_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssetLinkSummary> {
    Ok(AssetLinkSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        parent_asset_id: row.get("parent_asset_id")?,
        child_asset_id: row.get("child_asset_id")?,
        link_type: row.get("link_type")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_asset_link_by_id(
    conn: &Connection,
    project_id: &str,
    link_id: &str,
) -> Result<Option<AssetLinkSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT
          id,
          project_id,
          parent_asset_id,
          child_asset_id,
          link_type,
          created_at,
          updated_at
        FROM asset_links
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![link_id, project_id],
        row_to_asset_link_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_quality_report_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<QualityReportSummary> {
    Ok(QualityReportSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        run_id: row.get::<_, Option<String>>("run_id")?.unwrap_or_default(),
        asset_id: row
            .get::<_, Option<String>>("asset_id")?
            .unwrap_or_default(),
        report_type: row
            .get::<_, Option<String>>("report_type")?
            .unwrap_or_default(),
        grade: row.get::<_, Option<String>>("grade")?.unwrap_or_default(),
        hard_failures: row
            .get::<_, Option<i64>>("hard_failures")?
            .unwrap_or_default(),
        soft_warnings: row
            .get::<_, Option<i64>>("soft_warnings")?
            .unwrap_or_default(),
        avg_chroma_exceed: row
            .get::<_, Option<f64>>("avg_chroma_exceed")?
            .unwrap_or(0.0),
        summary_json: parse_json_value(row.get::<_, Option<String>>("summary_json")?),
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
    })
}

fn row_to_cost_event_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<CostEventSummary> {
    Ok(CostEventSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        run_id: row.get::<_, Option<String>>("run_id")?.unwrap_or_default(),
        job_id: row.get::<_, Option<String>>("job_id")?.unwrap_or_default(),
        provider_code: row
            .get::<_, Option<String>>("provider_code")?
            .unwrap_or_default(),
        model_name: row
            .get::<_, Option<String>>("model_name")?
            .unwrap_or_default(),
        event_type: row
            .get::<_, Option<String>>("event_type")?
            .unwrap_or_default(),
        units: row.get::<_, Option<f64>>("units")?.unwrap_or(0.0),
        unit_cost_usd: row.get::<_, Option<f64>>("unit_cost_usd")?.unwrap_or(0.0),
        total_cost_usd: row.get::<_, Option<f64>>("total_cost_usd")?.unwrap_or(0.0),
        currency: row
            .get::<_, Option<String>>("currency")?
            .unwrap_or_default(),
        meta_json: parse_json_value(row.get::<_, Option<String>>("meta_json")?),
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
    })
}

fn row_to_project_export_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<ProjectExportSummary> {
    Ok(ProjectExportSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        run_id: row.get::<_, Option<String>>("run_id")?.unwrap_or_default(),
        status: row.get::<_, Option<String>>("status")?.unwrap_or_default(),
        export_format: row
            .get::<_, Option<String>>("export_format")?
            .unwrap_or_default(),
        storage_uri: row
            .get::<_, Option<String>>("storage_uri")?
            .unwrap_or_default(),
        rel_path: row
            .get::<_, Option<String>>("rel_path")?
            .unwrap_or_default(),
        file_size_bytes: row
            .get::<_, Option<i64>>("file_size_bytes")?
            .unwrap_or_default(),
        checksum_sha256: row
            .get::<_, Option<String>>("checksum_sha256")?
            .unwrap_or_default(),
        manifest_json: parse_json_value(row.get::<_, Option<String>>("manifest_json")?),
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
        completed_at: row
            .get::<_, Option<String>>("completed_at")?
            .unwrap_or_default(),
    })
}

fn row_to_prompt_template_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PromptTemplateSummary> {
    Ok(PromptTemplateSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        template_text: row.get("template_text")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_prompt_template_by_id(
    conn: &Connection,
    project_id: &str,
    template_id: &str,
) -> Result<Option<PromptTemplateSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT id, project_id, name, template_text, created_at, updated_at
        FROM prompt_templates
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![template_id, project_id],
        row_to_prompt_template_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_provider_account_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<ProviderAccountSummary> {
    Ok(ProviderAccountSummary {
        project_id: row.get("project_id")?,
        provider_code: row.get("provider_code")?,
        display_name: row.get("display_name")?,
        account_ref: row
            .get::<_, Option<String>>("account_ref")?
            .unwrap_or_default(),
        base_url: row
            .get::<_, Option<String>>("base_url")?
            .unwrap_or_default(),
        enabled: row.get::<_, Option<i64>>("enabled")?.unwrap_or(1) != 0,
        config_json: parse_json_value(row.get::<_, Option<String>>("config_json")?),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_provider_account_by_code(
    conn: &Connection,
    project_id: &str,
    provider_code: &str,
) -> Result<Option<ProviderAccountSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT
          project_id,
          provider_code,
          display_name,
          account_ref,
          base_url,
          enabled,
          config_json,
          created_at,
          updated_at
        FROM provider_accounts
        WHERE project_id = ?1 AND provider_code = ?2
        LIMIT 1
    ",
        params![project_id, provider_code],
        row_to_provider_account_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_style_guide_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<StyleGuideSummary> {
    Ok(StyleGuideSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        instructions: row.get("instructions")?,
        notes: row.get::<_, Option<String>>("notes")?.unwrap_or_default(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_style_guide_by_id(
    conn: &Connection,
    project_id: &str,
    style_guide_id: &str,
) -> Result<Option<StyleGuideSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT id, project_id, name, instructions, notes, created_at, updated_at
        FROM style_guides
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![style_guide_id, project_id],
        row_to_style_guide_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_character_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<CharacterSummary> {
    Ok(CharacterSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        description: row
            .get::<_, Option<String>>("description")?
            .unwrap_or_default(),
        prompt_text: row
            .get::<_, Option<String>>("prompt_text")?
            .unwrap_or_default(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_character_by_id(
    conn: &Connection,
    project_id: &str,
    character_id: &str,
) -> Result<Option<CharacterSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT id, project_id, name, description, prompt_text, created_at, updated_at
        FROM characters
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![character_id, project_id],
        row_to_character_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_reference_set_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReferenceSetSummary> {
    Ok(ReferenceSetSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        description: row
            .get::<_, Option<String>>("description")?
            .unwrap_or_default(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_reference_set_by_id(
    conn: &Connection,
    project_id: &str,
    reference_set_id: &str,
) -> Result<Option<ReferenceSetSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT id, project_id, name, description, created_at, updated_at
        FROM reference_sets
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![reference_set_id, project_id],
        row_to_reference_set_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_reference_set_item_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<ReferenceSetItemSummary> {
    Ok(ReferenceSetItemSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        reference_set_id: row.get("reference_set_id")?,
        label: row.get("label")?,
        content_uri: row
            .get::<_, Option<String>>("content_uri")?
            .unwrap_or_default(),
        content_text: row
            .get::<_, Option<String>>("content_text")?
            .unwrap_or_default(),
        sort_order: row.get::<_, Option<i64>>("sort_order")?.unwrap_or_default(),
        metadata_json: parse_json_value(row.get::<_, Option<String>>("metadata_json")?),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_reference_set_item_by_id(
    conn: &Connection,
    project_id: &str,
    reference_set_id: &str,
    item_id: &str,
) -> Result<Option<ReferenceSetItemSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT
          id,
          project_id,
          reference_set_id,
          label,
          content_uri,
          content_text,
          sort_order,
          metadata_json,
          created_at,
          updated_at
        FROM reference_set_items
        WHERE id = ?1 AND project_id = ?2 AND reference_set_id = ?3
        LIMIT 1
    ",
        params![item_id, project_id, reference_set_id],
        row_to_reference_set_item_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_chat_session_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatSessionSummary> {
    Ok(ChatSessionSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_chat_session_by_id(
    conn: &Connection,
    project_id: &str,
    session_id: &str,
) -> Result<Option<ChatSessionSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT id, project_id, title, status, created_at, updated_at
        FROM chat_sessions
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![session_id, project_id],
        row_to_chat_session_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_chat_message_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatMessageSummary> {
    Ok(ChatMessageSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        session_id: row.get("session_id")?,
        role: row.get("role")?,
        content: row.get("content")?,
        created_at: row.get("created_at")?,
    })
}

fn fetch_chat_message_by_id(
    conn: &Connection,
    project_id: &str,
    session_id: &str,
    message_id: &str,
) -> Result<Option<ChatMessageSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT id, project_id, session_id, role, content, created_at
        FROM chat_messages
        WHERE id = ?1 AND project_id = ?2 AND session_id = ?3
        LIMIT 1
    ",
        params![message_id, project_id, session_id],
        row_to_chat_message_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn fetch_jobs_with_candidates(
    conn: &Connection,
    run_id: &str,
) -> Result<Vec<RunJobSummary>, ProjectsRepoError> {
    let mut candidates_by_job: HashMap<String, Vec<RunCandidateSummary>> = HashMap::new();

    let mut candidate_stmt = conn.prepare(
        "
        SELECT
          id,
          job_id,
          candidate_index,
          status,
          output_asset_id,
          final_asset_id,
          output_path,
          final_output_path,
          rank_hard_failures,
          rank_soft_warnings,
          rank_avg_chroma_exceed,
          meta_json,
          created_at
        FROM run_candidates
        WHERE job_id IN (
          SELECT id FROM run_jobs WHERE run_id = ?1
        )
        ORDER BY COALESCE(candidate_index, 0) ASC, id ASC
    ",
    )?;
    let mut candidate_rows = candidate_stmt.query([run_id])?;
    while let Some(row) = candidate_rows.next()? {
        let candidate = row_to_run_candidate_summary(row)?;
        candidates_by_job
            .entry(candidate.job_id.clone())
            .or_default()
            .push(candidate);
    }

    let mut stmt = conn.prepare(
        "
        SELECT
          id,
          run_id,
          job_key,
          status,
          prompt_text,
          selected_candidate_index,
          selected_candidate,
          final_asset_id,
          final_output,
          meta_json,
          created_at
        FROM run_jobs
        WHERE run_id = ?1
        ORDER BY COALESCE(created_at, '') ASC, id ASC
    ",
    )?;

    let mut rows = stmt.query([run_id])?;
    let mut out = Vec::new();

    while let Some(row) = rows.next()? {
        let job_id: String = row.get("id")?;
        let selected_candidate_index = row
            .get::<_, Option<i64>>("selected_candidate_index")?
            .or(row.get::<_, Option<i64>>("selected_candidate")?);

        out.push(RunJobSummary {
            id: job_id.clone(),
            run_id: row.get("run_id")?,
            job_key: row.get::<_, Option<String>>("job_key")?.unwrap_or_default(),
            status: row.get::<_, Option<String>>("status")?.unwrap_or_default(),
            prompt_text: row
                .get::<_, Option<String>>("prompt_text")?
                .unwrap_or_default(),
            selected_candidate_index,
            final_asset_id: row
                .get::<_, Option<String>>("final_asset_id")?
                .unwrap_or_default(),
            final_output: row
                .get::<_, Option<String>>("final_output")?
                .unwrap_or_default(),
            meta_json: parse_json_value(row.get::<_, Option<String>>("meta_json")?),
            created_at: row
                .get::<_, Option<String>>("created_at")?
                .unwrap_or_default(),
            candidates: candidates_by_job
                .remove(job_id.as_str())
                .unwrap_or_default(),
        });
    }

    Ok(out)
}

fn row_string_from_columns(row: &rusqlite::Row<'_>, columns: &[&str]) -> rusqlite::Result<String> {
    for column in columns {
        if let Some(value) = row.get::<_, Option<String>>(*column)? {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    Ok(String::new())
}

fn parse_json_value(raw: Option<String>) -> Value {
    raw.and_then(|text| serde_json::from_str::<Value>(text.as_str()).ok())
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()))
}

fn ensure_user(
    conn: &Connection,
    username: &str,
    display_name: &str,
) -> Result<String, ProjectsRepoError> {
    let safe_username = slugify(username);
    let now = now_iso();

    let existing_app_user_id = conn
        .query_row(
            "SELECT id FROM app_users WHERE username = ?1 LIMIT 1",
            [safe_username.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    let existing_legacy_user_id = conn
        .query_row(
            "SELECT id FROM users WHERE username = ?1 LIMIT 1",
            [safe_username.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    let user_id = existing_app_user_id
        .or(existing_legacy_user_id)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    conn.execute(
        "
        INSERT INTO app_users (id, username, display_name, email, is_active, created_at, updated_at)
        VALUES (?1, ?2, ?3, NULL, 1, ?4, ?4)
        ON CONFLICT(id) DO UPDATE SET
          username = excluded.username,
          display_name = excluded.display_name,
          email = NULL,
          is_active = 1,
          updated_at = excluded.updated_at
    ",
        params![user_id, safe_username, display_name, now],
    )?;
    conn.execute(
        "
        INSERT INTO users (id, username, display_name, email, is_active, created_at, updated_at)
        VALUES (?1, ?2, ?3, NULL, 1, ?4, ?4)
        ON CONFLICT(id) DO UPDATE SET
          username = excluded.username,
          display_name = excluded.display_name,
          email = NULL,
          is_active = 1,
          updated_at = excluded.updated_at
    ",
        params![user_id, safe_username, display_name, now],
    )?;

    Ok(user_id)
}

fn ensure_project(
    conn: &Connection,
    owner_user_id: &str,
    slug: &str,
    name: &str,
    description: &str,
) -> Result<ProjectRow, ProjectsRepoError> {
    let safe_slug = slugify(slug);
    let now = now_iso();

    let existing_id = conn
        .query_row(
            "SELECT id FROM projects WHERE owner_user_id = ?1 AND slug = ?2 LIMIT 1",
            params![owner_user_id, safe_slug],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .or(conn
            .query_row(
                "SELECT id FROM projects WHERE user_id = ?1 AND slug = ?2 LIMIT 1",
                params![owner_user_id, safe_slug],
                |row| row.get::<_, String>(0),
            )
            .optional()?);

    if let Some(project_id) = existing_id {
        conn.execute(
            "
            UPDATE projects
            SET name = ?1,
                description = ?2,
                status = 'active',
                owner_user_id = ?3,
                user_id = COALESCE(user_id, ?3),
                updated_at = ?4
            WHERE id = ?5
        ",
            params![name, description, owner_user_id, now, project_id],
        )?;
        return fetch_project_by_id(conn, project_id.as_str())?.ok_or(ProjectsRepoError::NotFound);
    }

    let project_id = Uuid::new_v4().to_string();
    conn.execute(
        "
        INSERT INTO projects
          (id, owner_user_id, user_id, slug, name, description, status, settings_json, created_at, updated_at)
        VALUES
          (?1, ?2, ?2, ?3, ?4, ?5, 'active', '{}', ?6, ?6)
    ",
        params![project_id, owner_user_id, safe_slug, name, description, now],
    )?;
    fetch_project_by_id(conn, project_id.as_str())?.ok_or(ProjectsRepoError::NotFound)
}

fn ensure_project_storage_defaults(
    conn: &Connection,
    project: &ProjectRow,
) -> Result<(), ProjectsRepoError> {
    let exists = conn
        .query_row(
            "SELECT id FROM project_storage WHERE project_id = ?1 LIMIT 1",
            [&project.id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();
    if exists {
        return Ok(());
    }

    let defaults = parse_storage_defaults(project.settings_json.as_str());
    let now = now_iso();
    conn.execute(
        "
        INSERT INTO project_storage
          (id, project_id, total_bytes, used_bytes, created_at, updated_at, local_base_dir, local_project_root, s3_enabled, s3_bucket, s3_prefix, s3_region, s3_profile, s3_endpoint_url)
        VALUES
          (?1, ?2, 0, 0, ?3, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
    ",
        params![
            Uuid::new_v4().to_string(),
            project.id,
            now,
            defaults.local_base_dir,
            defaults.local_project_root_cfg,
            if defaults.s3_enabled { 1 } else { 0 },
            none_if_empty(defaults.s3_bucket),
            none_if_empty(defaults.s3_prefix),
            none_if_empty(defaults.s3_region),
            none_if_empty(defaults.s3_profile),
            none_if_empty(defaults.s3_endpoint_url),
        ],
    )?;
    Ok(())
}

fn resolve_storage(
    conn: &Connection,
    repo_root: &Path,
    project: &ProjectRow,
) -> Result<StorageConfig, ProjectsRepoError> {
    let mut defaults = parse_storage_defaults(project.settings_json.as_str());

    let storage_row = conn
        .query_row(
            "
            SELECT
              local_base_dir,
              local_project_root,
              s3_enabled,
              s3_bucket,
              s3_prefix,
              s3_region,
              s3_profile,
              s3_endpoint_url
            FROM project_storage
            WHERE project_id = ?1
            LIMIT 1
        ",
            [&project.id],
            |row| {
                Ok(StorageDefaults {
                    local_base_dir: row
                        .get::<_, Option<String>>("local_base_dir")?
                        .filter(|v| !v.trim().is_empty())
                        .unwrap_or_else(|| String::from(DEFAULT_PROJECTS_BASE_DIR)),
                    local_project_root_cfg: row
                        .get::<_, Option<String>>("local_project_root")?
                        .map(|v| v.trim().to_string())
                        .filter(|v| !v.is_empty()),
                    s3_enabled: row.get::<_, Option<i64>>("s3_enabled")?.unwrap_or(0) != 0,
                    s3_bucket: row
                        .get::<_, Option<String>>("s3_bucket")?
                        .unwrap_or_default(),
                    s3_prefix: row
                        .get::<_, Option<String>>("s3_prefix")?
                        .filter(|v| !v.trim().is_empty())
                        .unwrap_or_else(|| String::from(DEFAULT_S3_PREFIX)),
                    s3_region: row
                        .get::<_, Option<String>>("s3_region")?
                        .unwrap_or_default(),
                    s3_profile: row
                        .get::<_, Option<String>>("s3_profile")?
                        .unwrap_or_default(),
                    s3_endpoint_url: row
                        .get::<_, Option<String>>("s3_endpoint_url")?
                        .unwrap_or_default(),
                })
            },
        )
        .optional()?;

    if let Some(storage) = storage_row {
        defaults = storage;
    }

    let project_root = if let Some(raw_root) = defaults.local_project_root_cfg.as_deref() {
        let cfg_path = PathBuf::from(raw_root);
        if cfg_path.is_absolute() {
            cfg_path
        } else {
            repo_root.join(cfg_path)
        }
    } else {
        let base = PathBuf::from(defaults.local_base_dir.as_str());
        let base_abs = if base.is_absolute() {
            base
        } else {
            repo_root.join(base)
        };
        base_abs.join(project.slug.as_str())
    };

    Ok(StorageConfig {
        local: StorageLocal {
            base_dir: defaults.local_base_dir,
            project_root: project_root.to_string_lossy().to_string(),
        },
        s3: StorageS3 {
            enabled: defaults.s3_enabled,
            bucket: defaults.s3_bucket,
            prefix: defaults.s3_prefix,
            region: defaults.s3_region,
            profile: defaults.s3_profile,
            endpoint_url: defaults.s3_endpoint_url,
        },
    })
}

fn project_storage_payload(
    conn: &Connection,
    repo_root: &Path,
    project: &ProjectRow,
) -> Result<ProjectStoragePayload, ProjectsRepoError> {
    let storage = resolve_storage(conn, repo_root, project)?;
    Ok(ProjectStoragePayload {
        project: ProjectStorageProject {
            id: project.id.clone(),
            slug: project.slug.clone(),
            name: project.name.clone(),
        },
        storage,
    })
}

fn normalize_required_storage_field(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(String::from("Field 'base_dir' must not be empty"))
    } else {
        Ok(trimmed.to_string())
    }
}

fn normalize_optional_storage_field(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_required_identifier(
    value: &str,
    field_name: &str,
) -> Result<String, ProjectsRepoError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(ProjectsRepoError::Validation(format!(
            "Field '{field_name}' is required"
        )))
    } else {
        Ok(trimmed.to_string())
    }
}

fn normalize_required_text(value: &str, field_name: &str) -> Result<String, ProjectsRepoError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(ProjectsRepoError::Validation(format!(
            "Field '{field_name}' is required"
        )))
    } else {
        Ok(trimmed.to_string())
    }
}

fn normalize_provider_code(value: &str) -> Result<String, ProjectsRepoError> {
    normalize_slug(value).ok_or_else(|| {
        ProjectsRepoError::Validation(String::from("Field 'provider_code' is required"))
    })
}

fn normalize_chat_role(value: &str) -> Result<String, ProjectsRepoError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(ProjectsRepoError::Validation(String::from(
            "Field 'role' is required",
        )));
    }
    if matches!(
        normalized.as_str(),
        "user" | "assistant" | "system" | "tool"
    ) {
        Ok(normalized)
    } else {
        Err(ProjectsRepoError::Validation(String::from(
            "Field 'role' must be one of: user, assistant, system, tool",
        )))
    }
}

fn normalize_asset_link_type(value: &str) -> Result<String, ProjectsRepoError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(ProjectsRepoError::Validation(String::from(
            "Field 'link_type' is required",
        )));
    }

    if is_valid_asset_link_type(normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(ProjectsRepoError::Validation(String::from(
            "Field 'link_type' must be one of: derived_from, variant_of, mask_for, reference_of",
        )))
    }
}

fn is_valid_asset_link_type(value: &str) -> bool {
    matches!(
        value,
        ASSET_LINK_TYPE_DERIVED_FROM
            | ASSET_LINK_TYPE_VARIANT_OF
            | ASSET_LINK_TYPE_MASK_FOR
            | ASSET_LINK_TYPE_REFERENCE_OF
    )
}

fn ensure_asset_belongs_to_project(
    conn: &Connection,
    project_id: &str,
    asset_id: &str,
) -> Result<(), ProjectsRepoError> {
    let exists = conn
        .query_row(
            "
            SELECT id
            FROM assets
            WHERE id = ?1 AND project_id = ?2
            LIMIT 1
        ",
            params![asset_id, project_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();

    if exists {
        Ok(())
    } else {
        Err(ProjectsRepoError::Validation(format!(
            "Asset '{asset_id}' was not found in this project"
        )))
    }
}

fn is_unique_constraint_error(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::ConstraintViolation,
                ..
            },
            _
        )
    )
}

fn parse_storage_defaults(settings_json: &str) -> StorageDefaults {
    let parsed: Value = serde_json::from_str(settings_json).unwrap_or(Value::Null);
    let storage = parsed
        .get("storage")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let local = storage
        .get("local")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let s3 = storage
        .get("s3")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    StorageDefaults {
        local_base_dir: local
            .get("base_dir")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(DEFAULT_PROJECTS_BASE_DIR)
            .to_string(),
        local_project_root_cfg: local
            .get("project_root")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToOwned::to_owned),
        s3_enabled: s3.get("enabled").and_then(Value::as_bool).unwrap_or(false),
        s3_bucket: s3
            .get("bucket")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .to_string(),
        s3_prefix: s3
            .get("prefix")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(DEFAULT_S3_PREFIX)
            .to_string(),
        s3_region: s3
            .get("region")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .to_string(),
        s3_profile: s3
            .get("profile")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .to_string(),
        s3_endpoint_url: s3
            .get("endpoint_url")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .to_string(),
    }
}

fn query_count<P>(conn: &Connection, sql: &str, params: P) -> Result<i64, ProjectsRepoError>
where
    P: rusqlite::Params,
{
    conn.query_row(sql, params, |row| row.get::<_, i64>(0))
        .map_err(ProjectsRepoError::from)
}

fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn none_if_empty(value: String) -> Option<String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub fn slugify(value: &str) -> String {
    normalize_slug(value).unwrap_or_else(|| String::from("project"))
}

pub fn normalize_slug(value: &str) -> Option<String> {
    let mut out = String::new();
    let mut last_underscore = false;
    for ch in value.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
            last_underscore = false;
            continue;
        }
        if !last_underscore {
            out.push('_');
            last_underscore = true;
        }
    }
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_repo() -> ProjectsStore {
        let suffix = Uuid::new_v4().to_string();
        let root = std::env::temp_dir().join(format!("kroma_repo_{suffix}"));
        let db = root.join("var/backend/app.db");
        std::fs::create_dir_all(root.as_path()).expect("temp root must be creatable");
        ProjectsStore::new(db, root)
    }

    #[test]
    fn slugify_normalizes_values() {
        assert_eq!(slugify("My Fancy Project"), "my_fancy_project");
        assert_eq!(slugify("___"), "project");
        assert_eq!(slugify("hero-01"), "hero-01");
        assert_eq!(normalize_slug("___"), None);
    }

    #[test]
    fn upsert_and_fetch_project_flow() {
        let repo = temp_repo();
        let created = repo
            .upsert_project(UpsertProjectInput {
                name: String::from("Hero Book"),
                slug: None,
                description: Some(String::from("Main production project")),
                username: Some(String::from("local")),
                user_display_name: Some(String::from("Local User")),
            })
            .expect("project should be created");

        assert_eq!(created.project.slug, "hero_book");
        let list = repo.list_projects(None).expect("projects should list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].slug, "hero_book");

        let detail = repo
            .get_project_detail("hero_book")
            .expect("project detail should load");
        assert_eq!(detail.project.name, "Hero Book");
        assert_eq!(detail.counts.runs, 0);
        assert_eq!(detail.counts.jobs, 0);
        assert_eq!(detail.counts.assets, 0);
    }

    #[test]
    fn list_filter_ignores_non_normalizable_username() {
        let repo = temp_repo();
        let _ = repo
            .upsert_project(UpsertProjectInput {
                name: String::from("Gamma"),
                slug: None,
                description: None,
                username: Some(String::from("local")),
                user_display_name: Some(String::from("Local User")),
            })
            .expect("project should be created");

        let unfiltered = repo.list_projects(None).expect("projects should list");
        let invalid_filter = repo
            .list_projects(Some("!!!"))
            .expect("invalid filter should not fail");

        assert_eq!(unfiltered.len(), 1);
        assert_eq!(invalid_filter.len(), 1);
    }
}
