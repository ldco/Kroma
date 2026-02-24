use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    ensure_user, fetch_project_by_slug, fetch_project_by_slug_for_user, normalize_optional_text,
    normalize_slug, now_iso, ProjectsRepoError, ProjectsStore,
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateApiTokenInput {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub project_slug: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateApiTokenResult {
    pub id: String,
    pub token: String,
    pub token_prefix: String,
    pub label: String,
    pub project_id: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiTokenSummary {
    pub id: String,
    pub user_id: String,
    pub project_id: Option<String>,
    pub token_prefix: String,
    pub label: String,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct ApiTokenAuthContext {
    pub token_id: String,
    pub user_id: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppendAuditEventInput {
    pub project_slug: Option<String>,
    pub actor_user_id: Option<String>,
    pub event_code: String,
    pub payload_json: Value,
}

impl ProjectsStore {
    pub fn authorize_project_slug_access(
        &self,
        slug: &str,
        user_id: &str,
        token_project_id: Option<&str>,
    ) -> Result<bool, ProjectsRepoError> {
        let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
        let safe_user_id = normalize_optional_text(Some(user_id))
            .ok_or_else(|| ProjectsRepoError::Validation(String::from("user_id is required")))?;
        let scoped_project_id = normalize_optional_text(token_project_id);

        self.with_connection(|conn| {
            // Current data access paths resolve by slug only, so deny ambiguous slugs
            // until all lookups are fully user-scoped end-to-end.
            let same_slug_count = conn.query_row(
                "SELECT COUNT(*) FROM projects WHERE slug = ?1",
                [safe_slug.as_str()],
                |row| row.get::<_, i64>(0),
            )?;
            if same_slug_count > 1 {
                return Err(ProjectsRepoError::Validation(String::from(
                    "Ambiguous project slug for multi-tenant access",
                )));
            }

            let allowed_count = if let Some(project_id) = scoped_project_id.as_deref() {
                conn.query_row(
                    "
                    SELECT COUNT(*)
                    FROM projects
                    WHERE slug = ?1
                      AND COALESCE(owner_user_id, user_id) = ?2
                      AND id = ?3
                ",
                    params![safe_slug, safe_user_id, project_id],
                    |row| row.get::<_, i64>(0),
                )?
            } else {
                conn.query_row(
                    "
                    SELECT COUNT(*)
                    FROM projects
                    WHERE slug = ?1
                      AND COALESCE(owner_user_id, user_id) = ?2
                ",
                    params![safe_slug, safe_user_id],
                    |row| row.get::<_, i64>(0),
                )?
            };

            Ok(allowed_count > 0)
        })
    }

    pub fn create_api_token_local(
        &self,
        input: CreateApiTokenInput,
    ) -> Result<CreateApiTokenResult, ProjectsRepoError> {
        let label = normalize_optional_text(input.label.as_deref())
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect::<String>();
        let expires_at = normalize_optional_text(input.expires_at.as_deref());

        self.with_connection_mut(|conn| {
            let user_id = ensure_user(conn, "local", "Local User")?;
            let project_id = if let Some(project_slug) = input.project_slug.as_deref() {
                let safe_slug = normalize_slug(project_slug).ok_or(ProjectsRepoError::NotFound)?;
                let project = fetch_project_by_slug_for_user(conn, safe_slug.as_str(), user_id.as_str())?
                    .ok_or(ProjectsRepoError::NotFound)?;
                Some(project.id)
            } else {
                None
            };

            let token_id = Uuid::new_v4().to_string();
            let secret = format!("kroma_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
            let token_prefix = secret.chars().take(12).collect::<String>();
            let token_hash = sha256_hex(secret.as_bytes());
            let created_at = now_iso();
            conn.execute(
                "
                INSERT INTO api_tokens
                  (id, user_id, project_id, token_hash, token_prefix, label, expires_at, revoked_at, last_used_at, created_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, ?8)
                ",
                params![
                    token_id,
                    user_id,
                    project_id,
                    token_hash,
                    token_prefix,
                    label,
                    expires_at,
                    created_at
                ],
            )?;

            Ok(CreateApiTokenResult {
                id: token_id,
                token: secret,
                token_prefix,
                label,
                project_id,
                created_at,
                expires_at,
            })
        })
    }

    pub fn list_api_tokens(&self) -> Result<Vec<ApiTokenSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  user_id,
                  project_id,
                  token_prefix,
                  label,
                  expires_at,
                  revoked_at,
                  last_used_at,
                  created_at
                FROM api_tokens
                ORDER BY created_at DESC
                ",
            )?;
            let rows = stmt.query_map([], |row| {
                let revoked_at = row.get::<_, Option<String>>("revoked_at")?;
                let expires_at = row.get::<_, Option<String>>("expires_at")?;
                let now = now_iso();
                let active = revoked_at.is_none()
                    && expires_at
                        .as_deref()
                        .map(|v| v > now.as_str())
                        .unwrap_or(true);
                Ok(ApiTokenSummary {
                    id: row.get("id")?,
                    user_id: row.get("user_id")?,
                    project_id: row.get("project_id")?,
                    token_prefix: row.get("token_prefix")?,
                    label: row.get::<_, Option<String>>("label")?.unwrap_or_default(),
                    expires_at,
                    revoked_at,
                    last_used_at: row.get("last_used_at")?,
                    created_at: row.get("created_at")?,
                    active,
                })
            })?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn revoke_api_token(&self, token_id: &str) -> Result<(), ProjectsRepoError> {
        let token_id = normalize_optional_text(Some(token_id))
            .ok_or_else(|| ProjectsRepoError::Validation(String::from("token_id is required")))?;
        self.with_connection_mut(|conn| {
            let now = now_iso();
            let affected = conn.execute(
                "UPDATE api_tokens SET revoked_at = COALESCE(revoked_at, ?2) WHERE id = ?1",
                params![token_id, now],
            )?;
            if affected == 0 {
                Err(ProjectsRepoError::NotFound)
            } else {
                Ok(())
            }
        })
    }

    pub fn validate_api_token(
        &self,
        raw_token: &str,
    ) -> Result<Option<ApiTokenAuthContext>, ProjectsRepoError> {
        let raw = normalize_optional_text(Some(raw_token)).ok_or_else(|| {
            ProjectsRepoError::Validation(String::from("Authorization token is required"))
        })?;
        let token_hash = sha256_hex(raw.as_bytes());
        self.with_connection_mut(|conn| {
            let now = now_iso();
            let found = conn
                .query_row(
                    "
                    SELECT id, user_id, project_id
                    FROM api_tokens
                    WHERE token_hash = ?1
                      AND revoked_at IS NULL
                      AND (expires_at IS NULL OR expires_at > ?2)
                    LIMIT 1
                    ",
                    params![token_hash, now],
                    |row| {
                        Ok(ApiTokenAuthContext {
                            token_id: row.get("id")?,
                            user_id: row.get("user_id")?,
                            project_id: row.get("project_id")?,
                        })
                    },
                )
                .optional()?;

            if let Some(ctx) = found.as_ref() {
                let _ = conn.execute(
                    "UPDATE api_tokens SET last_used_at = ?2 WHERE id = ?1",
                    params![ctx.token_id, now],
                );
            }
            Ok(found)
        })
    }

    pub fn append_audit_event(
        &self,
        input: AppendAuditEventInput,
    ) -> Result<String, ProjectsRepoError> {
        let event_code = normalize_optional_text(Some(input.event_code.as_str()))
            .ok_or_else(|| ProjectsRepoError::Validation(String::from("event_code is required")))?;
        self.with_connection_mut(|conn| {
            let project_id = if let Some(project_slug) = input.project_slug.as_deref() {
                let safe_slug = normalize_slug(project_slug).ok_or(ProjectsRepoError::NotFound)?;
                Some(
                    fetch_project_by_slug(conn, safe_slug.as_str())?
                        .ok_or(ProjectsRepoError::NotFound)?
                        .id,
                )
            } else {
                None
            };
            let id = Uuid::new_v4().to_string();
            let created_at = now_iso();
            let payload_json =
                serde_json::to_string(&input.payload_json).unwrap_or_else(|_| "{}".into());
            conn.execute(
                "
                INSERT INTO audit_events
                  (id, project_id, actor_user_id, event_code, payload_json, created_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6)
                ",
                params![
                    id,
                    project_id,
                    input.actor_user_id,
                    event_code,
                    payload_json,
                    created_at
                ],
            )?;
            Ok(id)
        })
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}
