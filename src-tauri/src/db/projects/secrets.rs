use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{
    ensure_column, fetch_project_by_slug, normalize_provider_code, normalize_required_text,
    normalize_slug, now_iso, ProjectsRepoError, ProjectsStore,
};

#[derive(Debug, Clone, Serialize)]
pub struct SecretSummary {
    pub project_id: String,
    pub provider_code: String,
    pub secret_name: String,
    pub has_value: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpsertSecretInput {
    #[serde(default)]
    pub provider_code: String,
    #[serde(default)]
    pub secret_name: String,
    #[serde(default)]
    pub secret_value: String,
}

impl ProjectsStore {
    pub fn list_project_secrets(
        &self,
        slug: &str,
    ) -> Result<Vec<SecretSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT project_id, provider_code, secret_name, secret_value, updated_at
                FROM project_secrets
                WHERE project_id = ?1
                ORDER BY provider_code ASC, secret_name ASC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_secret_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn upsert_project_secret(
        &self,
        slug: &str,
        input: UpsertSecretInput,
    ) -> Result<SecretSummary, ProjectsRepoError> {
        let provider_code = normalize_provider_code(input.provider_code.as_str())?;
        let secret_name = normalize_required_text(input.secret_name.as_str(), "secret_name")?;
        let secret_value = normalize_required_text(input.secret_value.as_str(), "secret_value")?;

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO project_secrets
                  (project_id, provider_code, secret_name, secret_value, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?5)
                ON CONFLICT(project_id, provider_code, secret_name) DO UPDATE SET
                  secret_value = excluded.secret_value,
                  updated_at = excluded.updated_at
            ",
                params![project.id, provider_code, secret_name, secret_value, now],
            )?;

            fetch_project_secret_by_key(
                conn,
                project.id.as_str(),
                provider_code.as_str(),
                secret_name.as_str(),
            )?
            .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn delete_project_secret(
        &self,
        slug: &str,
        provider_code: &str,
        secret_name: &str,
    ) -> Result<(), ProjectsRepoError> {
        let safe_provider_code = normalize_provider_code(provider_code)?;
        let safe_secret_name = normalize_required_text(secret_name, "secret_name")?;

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let affected = conn.execute(
                "
                DELETE FROM project_secrets
                WHERE project_id = ?1 AND provider_code = ?2 AND secret_name = ?3
            ",
                params![project.id, safe_provider_code, safe_secret_name],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }
}

pub(super) fn ensure_secret_tables(conn: &Connection) -> Result<(), ProjectsRepoError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS project_secrets (
          project_id TEXT NOT NULL,
          provider_code TEXT NOT NULL,
          secret_name TEXT NOT NULL,
          secret_value TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY(project_id, provider_code, secret_name)
        );
    ",
    )?;

    Ok(())
}

pub(super) fn ensure_secret_columns(conn: &Connection) -> Result<(), ProjectsRepoError> {
    ensure_column(conn, "project_secrets", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "project_secrets", "provider_code", "TEXT NOT NULL")?;
    ensure_column(conn, "project_secrets", "secret_name", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "project_secrets",
        "secret_value",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "project_secrets",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "project_secrets",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    Ok(())
}

fn row_to_secret_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<SecretSummary> {
    let secret_value = row
        .get::<_, Option<String>>("secret_value")?
        .unwrap_or_default();
    Ok(SecretSummary {
        project_id: row.get("project_id")?,
        provider_code: row.get("provider_code")?,
        secret_name: row.get("secret_name")?,
        has_value: !secret_value.trim().is_empty(),
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_project_secret_by_key(
    conn: &Connection,
    project_id: &str,
    provider_code: &str,
    secret_name: &str,
) -> Result<Option<SecretSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT project_id, provider_code, secret_name, secret_value, updated_at
        FROM project_secrets
        WHERE project_id = ?1 AND provider_code = ?2 AND secret_name = ?3
        LIMIT 1
    ",
        params![project_id, provider_code, secret_name],
        row_to_secret_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}
