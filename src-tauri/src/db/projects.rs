use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

const DEFAULT_PROJECTS_BASE_DIR: &str = "var/projects";
const DEFAULT_S3_PREFIX: &str = "iat-projects";

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

            let storage = self.with_connection(|conn_ro| {
                resolve_storage(conn_ro, self.repo_root.as_path(), &project)
            })?;
            Ok(ProjectStoragePayload {
                project: ProjectStorageProject {
                    id: project.id,
                    slug: project.slug,
                    name: project.name,
                },
                storage,
            })
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
          project_id TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS run_jobs (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS assets (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL
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
