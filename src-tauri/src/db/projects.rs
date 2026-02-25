use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

mod analytics_exports;
mod auth_audit;
mod bootstrap;
mod chat_instructions;
mod pipeline_ingest;
mod prompt_templates;
mod provider_style_character;
mod reference_sets;
mod runs_assets;
mod secrets;

pub use analytics_exports::{CostEventSummary, ProjectExportSummary, QualityReportSummary};
pub use auth_audit::{
    ApiTokenAuthContext, ApiTokenSummary, AppendAuditEventInput, CreateApiTokenInput,
    CreateApiTokenResult,
};
pub use bootstrap::{
    ImportProjectBootstrapInput, ProjectBootstrapExport, ProjectBootstrapImportResult,
    ProjectBootstrapProject, ProjectBootstrapSettings,
};
pub use chat_instructions::{
    AgentInstructionActionInput, AgentInstructionEventSummary, AgentInstructionSummary,
    ChatMessageSummary, ChatSessionSummary, CreateAgentInstructionInput, CreateChatMessageInput,
    CreateChatSessionInput,
};
pub use pipeline_ingest::{IngestRunLogInput, IngestRunLogResult};
pub use prompt_templates::{
    CreatePromptTemplateInput, PromptTemplateSummary, UpdatePromptTemplateInput,
};
pub use provider_style_character::{
    CharacterSummary, CreateCharacterInput, CreateStyleGuideInput, ProviderAccountSummary,
    StyleGuideSummary, UpdateCharacterInput, UpdateProviderAccountInput, UpdateStyleGuideInput,
    UpsertProviderAccountInput,
};
pub use reference_sets::{
    CreateReferenceSetInput, CreateReferenceSetItemInput, ReferenceSetItemSummary,
    ReferenceSetSummary, UpdateReferenceSetInput, UpdateReferenceSetItemInput,
};
pub use runs_assets::{AssetSummary, RunCandidateSummary, RunJobSummary, RunSummary};
pub use secrets::{SecretSummary, UpsertSecretInput};

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
pub struct AssetLinkSummary {
    pub id: String,
    pub project_id: String,
    pub parent_asset_id: String,
    pub child_asset_id: String,
    pub link_type: String,
    pub created_at: String,
    pub updated_at: String,
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

#[derive(Debug, Error)]
pub enum ProjectsRepoError {
    #[error("project not found")]
    NotFound,

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Internal(String),

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
        configure_connection(&conn)?;
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
        configure_connection(&conn)?;
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

        CREATE TABLE IF NOT EXISTS api_tokens (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL,
          project_id TEXT,
          token_hash TEXT NOT NULL UNIQUE,
          token_prefix TEXT NOT NULL,
          label TEXT NOT NULL DEFAULT '',
          expires_at TEXT,
          revoked_at TEXT,
          last_used_at TEXT,
          created_at TEXT NOT NULL,
          FOREIGN KEY(user_id) REFERENCES app_users(id) ON DELETE CASCADE,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS audit_events (
          id TEXT PRIMARY KEY,
          project_id TEXT,
          actor_user_id TEXT,
          event_code TEXT NOT NULL,
          payload_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL,
          FOREIGN KEY(actor_user_id) REFERENCES app_users(id) ON DELETE SET NULL
        );
    ",
    )?;

    chat_instructions::ensure_chat_and_instruction_tables(conn)?;
    analytics_exports::ensure_analytics_export_tables(conn)?;
    prompt_templates::ensure_prompt_template_tables(conn)?;
    provider_style_character::ensure_provider_style_character_tables(conn)?;
    reference_sets::ensure_reference_set_tables(conn)?;
    secrets::ensure_secret_tables(conn)?;

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
    ensure_column(conn, "api_tokens", "user_id", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(conn, "api_tokens", "project_id", "TEXT")?;
    ensure_column(conn, "api_tokens", "token_hash", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(
        conn,
        "api_tokens",
        "token_prefix",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(conn, "api_tokens", "label", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(conn, "api_tokens", "expires_at", "TEXT")?;
    ensure_column(conn, "api_tokens", "revoked_at", "TEXT")?;
    ensure_column(conn, "api_tokens", "last_used_at", "TEXT")?;
    ensure_column(conn, "api_tokens", "created_at", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(conn, "audit_events", "project_id", "TEXT")?;
    ensure_column(conn, "audit_events", "actor_user_id", "TEXT")?;
    ensure_column(
        conn,
        "audit_events",
        "event_code",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "audit_events",
        "payload_json",
        "TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column(
        conn,
        "audit_events",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

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

    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_projects_slug ON projects(slug);
        CREATE INDEX IF NOT EXISTS idx_projects_owner_slug ON projects(owner_user_id, slug);
        CREATE INDEX IF NOT EXISTS idx_runs_project_created ON runs(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_run_jobs_run_created ON run_jobs(run_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_assets_project_created ON assets(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_api_tokens_user_created ON api_tokens(user_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_api_tokens_project_created ON api_tokens(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_audit_events_project_created ON audit_events(project_id, created_at);
    ",
    )?;

    analytics_exports::ensure_analytics_export_columns(conn)?;
    chat_instructions::ensure_chat_and_instruction_columns(conn)?;
    prompt_templates::ensure_prompt_template_columns(conn)?;
    provider_style_character::ensure_provider_style_character_columns(conn)?;
    reference_sets::ensure_reference_set_columns(conn)?;
    secrets::ensure_secret_columns(conn)?;
    Ok(())
}

fn configure_connection(conn: &Connection) -> Result<(), ProjectsRepoError> {
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
    ",
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

fn fetch_project_by_slug_for_user(
    conn: &Connection,
    slug: &str,
    user_id: &str,
) -> Result<Option<ProjectRow>, ProjectsRepoError> {
    let mut stmt = conn.prepare(
        "
        SELECT id, slug, name, description, status, settings_json, created_at, updated_at
        FROM projects
        WHERE slug = ?1
          AND COALESCE(owner_user_id, user_id) = ?2
        LIMIT 1
    ",
    )?;
    stmt.query_row(params![slug, user_id], row_to_project)
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

    let slug_taken_by_other_user = conn
        .query_row(
            "
            SELECT id
            FROM projects
            WHERE slug = ?1
              AND COALESCE(owner_user_id, user_id) != ?2
            LIMIT 1
        ",
            params![safe_slug, owner_user_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();
    if slug_taken_by_other_user {
        return Err(ProjectsRepoError::Validation(String::from(
            "Project slug is already used by another user. Choose a different slug.",
        )));
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

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
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

    #[test]
    fn provider_account_paths_validate_provider_code() {
        let repo = temp_repo();
        let created = repo
            .upsert_project(UpsertProjectInput {
                name: String::from("Delta"),
                slug: None,
                description: None,
                username: Some(String::from("local")),
                user_display_name: Some(String::from("Local User")),
            })
            .expect("project should be created");

        let slug = created.project.slug;
        let _ = repo
            .upsert_provider_account(
                slug.as_str(),
                UpsertProviderAccountInput {
                    provider_code: String::from("openai"),
                    ..Default::default()
                },
            )
            .expect("provider account should be upserted");

        let detail_err = repo
            .get_provider_account_detail(slug.as_str(), "!!!")
            .expect_err("invalid provider code should fail validation");
        assert!(matches!(detail_err, ProjectsRepoError::Validation(_)));

        let update_err = repo
            .update_provider_account(
                slug.as_str(),
                "!!!",
                UpdateProviderAccountInput {
                    display_name: Some(String::from("OpenAI Updated")),
                    ..Default::default()
                },
            )
            .expect_err("invalid provider code should fail validation");
        assert!(matches!(update_err, ProjectsRepoError::Validation(_)));

        let delete_err = repo
            .delete_provider_account(slug.as_str(), "!!!")
            .expect_err("invalid provider code should fail validation");
        assert!(matches!(delete_err, ProjectsRepoError::Validation(_)));
    }
}
