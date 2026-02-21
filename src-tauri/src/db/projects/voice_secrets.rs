use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ensure_column, fetch_project_by_slug, normalize_optional_storage_field,
    normalize_provider_code, normalize_required_text, normalize_slug, now_iso, ProjectsRepoError,
    ProjectsStore,
};

#[derive(Debug, Clone, Serialize)]
pub struct VoiceRequestSummary {
    pub id: String,
    pub project_id: String,
    pub request_type: String,
    pub status: String,
    pub input_text: String,
    pub output_text: String,
    pub audio_uri: String,
    pub error_message: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretSummary {
    pub project_id: String,
    pub provider_code: String,
    pub secret_name: String,
    pub has_value: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateVoiceSttInput {
    #[serde(default)]
    pub audio_uri: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub mock_transcript: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateVoiceTtsInput {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub voice: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
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
    pub fn create_voice_stt_request(
        &self,
        slug: &str,
        input: CreateVoiceSttInput,
    ) -> Result<VoiceRequestSummary, ProjectsRepoError> {
        let audio_uri = input
            .audio_uri
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let language = input
            .language
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let output_text = input
            .mock_transcript
            .as_deref()
            .and_then(normalize_optional_storage_field)
            .or_else(|| {
                if audio_uri.is_some() {
                    Some(String::from("transcription_unavailable"))
                } else {
                    None
                }
            })
            .unwrap_or_default();

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project =
                fetch_project_by_slug(conn, safe_slug.as_str())?.ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO voice_requests
                  (id, project_id, request_type, status, input_text, output_text, audio_uri, error_message, created_at, updated_at)
                VALUES
                  (?1, ?2, 'stt', 'completed', ?3, ?4, ?5, '', ?6, ?6)
            ",
                params![
                    id,
                    project.id,
                    language.unwrap_or_default(),
                    output_text,
                    audio_uri,
                    now
                ],
            )?;

            fetch_voice_request_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn create_voice_tts_request(
        &self,
        slug: &str,
        input: CreateVoiceTtsInput,
    ) -> Result<VoiceRequestSummary, ProjectsRepoError> {
        let text = normalize_required_text(input.text.as_str(), "text")?;
        let voice = input
            .voice
            .as_deref()
            .and_then(normalize_optional_storage_field)
            .unwrap_or_else(|| String::from("default"));
        let format = input
            .format
            .as_deref()
            .and_then(normalize_optional_storage_field)
            .unwrap_or_else(|| String::from("wav"));

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project =
                fetch_project_by_slug(conn, safe_slug.as_str())?.ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            let audio_uri = format!("voice://{voice}/{id}.{format}");
            conn.execute(
                "
                INSERT INTO voice_requests
                  (id, project_id, request_type, status, input_text, output_text, audio_uri, error_message, created_at, updated_at)
                VALUES
                  (?1, ?2, 'tts', 'completed', ?3, '', ?4, '', ?5, ?5)
            ",
                params![id, project.id, text, audio_uri, now],
            )?;

            fetch_voice_request_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_voice_request_detail(
        &self,
        slug: &str,
        request_id: &str,
    ) -> Result<VoiceRequestSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_voice_request_by_id(conn, project.id.as_str(), request_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

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

pub(super) fn ensure_voice_and_secret_tables(conn: &Connection) -> Result<(), ProjectsRepoError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS voice_requests (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          request_type TEXT NOT NULL,
          status TEXT NOT NULL,
          input_text TEXT,
          output_text TEXT,
          audio_uri TEXT,
          error_message TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

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

pub(super) fn ensure_voice_and_secret_columns(conn: &Connection) -> Result<(), ProjectsRepoError> {
    ensure_column(conn, "voice_requests", "project_id", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "voice_requests",
        "request_type",
        "TEXT NOT NULL DEFAULT 'stt'",
    )?;
    ensure_column(
        conn,
        "voice_requests",
        "status",
        "TEXT NOT NULL DEFAULT 'pending'",
    )?;
    ensure_column(conn, "voice_requests", "input_text", "TEXT")?;
    ensure_column(conn, "voice_requests", "output_text", "TEXT")?;
    ensure_column(conn, "voice_requests", "audio_uri", "TEXT")?;
    ensure_column(conn, "voice_requests", "error_message", "TEXT")?;
    ensure_column(
        conn,
        "voice_requests",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "voice_requests",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

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

fn row_to_voice_request_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<VoiceRequestSummary> {
    Ok(VoiceRequestSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        request_type: row.get("request_type")?,
        status: row.get("status")?,
        input_text: row
            .get::<_, Option<String>>("input_text")?
            .unwrap_or_default(),
        output_text: row
            .get::<_, Option<String>>("output_text")?
            .unwrap_or_default(),
        audio_uri: row
            .get::<_, Option<String>>("audio_uri")?
            .unwrap_or_default(),
        error_message: row
            .get::<_, Option<String>>("error_message")?
            .unwrap_or_default(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_voice_request_by_id(
    conn: &Connection,
    project_id: &str,
    request_id: &str,
) -> Result<Option<VoiceRequestSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT
          id,
          project_id,
          request_type,
          status,
          input_text,
          output_text,
          audio_uri,
          error_message,
          created_at,
          updated_at
        FROM voice_requests
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![request_id, project_id],
        row_to_voice_request_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
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
