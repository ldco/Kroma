use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::URL_SAFE;
use base64::Engine as _;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{
    ensure_column, fetch_project_by_slug, normalize_provider_code, normalize_required_text,
    normalize_slug, now_iso, ProjectsRepoError, ProjectsStore,
};

const DEFAULT_MASTER_KEY_FILE: &str = "var/backend/master.key";
const SECRET_CIPHERTEXT_PREFIX: &str = "enc:v1:";
const SECRET_KEY_REF: &str = "local-master";

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
        let secret_ciphertext =
            encrypt_secret_value(secret_value.as_str(), self.repo_root.as_path())?;

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO project_secrets
                  (project_id, provider_code, secret_name, secret_value, key_ref, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?6)
                ON CONFLICT(project_id, provider_code, secret_name) DO UPDATE SET
                  secret_value = excluded.secret_value,
                  key_ref = excluded.key_ref,
                  updated_at = excluded.updated_at
            ",
                params![
                    project.id,
                    provider_code,
                    secret_name,
                    secret_ciphertext,
                    SECRET_KEY_REF,
                    now
                ],
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
          key_ref TEXT NOT NULL DEFAULT 'local-master',
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
        "key_ref",
        "TEXT NOT NULL DEFAULT 'local-master'",
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
        has_value: secret_has_value(secret_value.as_str()),
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

fn secret_has_value(secret_value: &str) -> bool {
    !secret_value.trim().is_empty()
}

fn encrypt_secret_value(secret_value: &str, repo_root: &Path) -> Result<String, ProjectsRepoError> {
    if secret_value.trim().is_empty() {
        return Err(ProjectsRepoError::Validation(String::from(
            "Field 'secret_value' is required",
        )));
    }
    let key_bytes = load_or_create_master_key_bytes(repo_root, true)?;
    let mut nonce_bytes = [0u8; 12];
    SystemRandom::new().fill(&mut nonce_bytes).map_err(|_| {
        ProjectsRepoError::Internal(String::from("Failed to generate secret nonce"))
    })?;
    let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|_| {
        ProjectsRepoError::Internal(String::from("Failed to initialize secret encryption key"))
    })?;
    let key = LessSafeKey::new(unbound);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut ciphertext = secret_value.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
        .map_err(|_| ProjectsRepoError::Internal(String::from("Secret encryption failed")))?;

    let mut payload = nonce_bytes.to_vec();
    payload.extend_from_slice(ciphertext.as_slice());
    Ok(format!(
        "{SECRET_CIPHERTEXT_PREFIX}{}",
        URL_SAFE.encode(payload)
    ))
}

fn load_or_create_master_key_bytes(
    repo_root: &Path,
    allow_create: bool,
) -> Result<[u8; 32], ProjectsRepoError> {
    let raw_key = load_or_create_master_key(repo_root, allow_create)?;
    decode_master_key(raw_key.as_str())
}

fn load_or_create_master_key(
    repo_root: &Path,
    allow_create: bool,
) -> Result<String, ProjectsRepoError> {
    if let Some(value) = std::env::var("IAT_MASTER_KEY")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        return Ok(value);
    }

    let key_file = master_key_file_path(repo_root);
    if key_file.is_file() {
        let existing = fs::read_to_string(key_file.as_path())
            .map_err(|e| ProjectsRepoError::Internal(format!("Master key read failed: {e}")))?;
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    if !allow_create {
        return Err(ProjectsRepoError::Internal(String::from(
            "Master key not found. Set IAT_MASTER_KEY or IAT_MASTER_KEY_FILE.",
        )));
    }

    let mut bytes = [0u8; 32];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| ProjectsRepoError::Internal(String::from("Failed to generate master key")))?;
    let generated = URL_SAFE.encode(bytes);
    if let Some(parent) = key_file.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            ProjectsRepoError::Internal(format!("Master key dir create failed: {e}"))
        })?;
    }
    fs::write(key_file.as_path(), format!("{generated}\n"))
        .map_err(|e| ProjectsRepoError::Internal(format!("Master key write failed: {e}")))?;
    #[cfg(unix)]
    {
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(key_file.as_path(), permissions).map_err(|e| {
            ProjectsRepoError::Internal(format!("Master key permissions update failed: {e}"))
        })?;
    }
    Ok(generated)
}

fn master_key_file_path(repo_root: &Path) -> PathBuf {
    let key_file_raw = std::env::var("IAT_MASTER_KEY_FILE")
        .unwrap_or_else(|_| String::from(DEFAULT_MASTER_KEY_FILE));
    let path = PathBuf::from(key_file_raw);
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

fn decode_master_key(raw: &str) -> Result<[u8; 32], ProjectsRepoError> {
    let decoded = URL_SAFE.decode(raw.as_bytes()).map_err(|_| {
        ProjectsRepoError::Internal(String::from(
            "IAT master key is invalid (expected base64url 32-byte key)",
        ))
    })?;
    decoded.try_into().map_err(|_| {
        ProjectsRepoError::Internal(String::from(
            "IAT master key must decode to exactly 32 bytes",
        ))
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;
    use crate::db::projects::{ProjectsStore, UpsertProjectInput, UpsertSecretInput};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_env_var(key: &str, value: Option<&str>, run: impl FnOnce()) {
        with_env_vars(&[(key, value)], run);
    }

    fn with_env_vars(vars: &[(&str, Option<&str>)], run: impl FnOnce()) {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let originals = vars
            .iter()
            .map(|(key, _)| ((*key).to_string(), std::env::var(key).ok()))
            .collect::<Vec<_>>();
        for (key, value) in vars {
            match value {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
        run();
        for (key, original) in originals {
            if let Some(v) = original {
                unsafe { std::env::set_var(&key, v) };
            } else {
                unsafe { std::env::remove_var(&key) };
            }
        }
    }

    #[test]
    fn upsert_secret_encrypts_value_at_rest() {
        let root =
            std::env::temp_dir().join(format!("kroma_secrets_encrypt_{}", uuid::Uuid::new_v4()));
        let db = root.join("var/backend/app.db");
        let store = ProjectsStore::new(db.clone(), root.clone());
        store.initialize().expect("store should initialize");
        let created = store
            .upsert_project(UpsertProjectInput {
                name: String::from("Secrets Encrypt"),
                ..UpsertProjectInput::default()
            })
            .expect("project upsert should succeed");
        let slug = created.project.slug;

        with_env_var(
            "IAT_MASTER_KEY",
            Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
            || {
                store
                    .upsert_project_secret(
                        slug.as_str(),
                        UpsertSecretInput {
                            provider_code: String::from("openai"),
                            secret_name: String::from("api_key"),
                            secret_value: String::from("sk-test-plaintext"),
                        },
                    )
                    .expect("secret upsert should succeed");
            },
        );

        let conn = Connection::open(db.as_path()).expect("db should open");
        let row = conn
            .query_row(
                "SELECT secret_value, key_ref FROM project_secrets WHERE provider_code = 'openai' AND secret_name = 'api_key'",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .expect("secret row should exist");
        assert_ne!(row.0, "sk-test-plaintext");
        assert!(row.0.starts_with(SECRET_CIPHERTEXT_PREFIX));
        assert_eq!(row.1, SECRET_KEY_REF);
    }

    #[test]
    fn upsert_secret_creates_master_key_file_when_missing() {
        let root =
            std::env::temp_dir().join(format!("kroma_secrets_keyfile_{}", uuid::Uuid::new_v4()));
        let db = root.join("var/backend/app.db");
        let store = ProjectsStore::new(db, root.clone());
        store.initialize().expect("store should initialize");
        let created = store
            .upsert_project(UpsertProjectInput {
                name: String::from("Secrets Keyfile"),
                ..UpsertProjectInput::default()
            })
            .expect("project upsert should succeed");

        with_env_vars(
            &[
                ("IAT_MASTER_KEY", None),
                ("IAT_MASTER_KEY_FILE", Some("var/backend/master.key")),
            ],
            || {
                store
                    .upsert_project_secret(
                        created.project.slug.as_str(),
                        UpsertSecretInput {
                            provider_code: String::from("openai"),
                            secret_name: String::from("api_key"),
                            secret_value: String::from("sk-test-generated"),
                        },
                    )
                    .expect("secret upsert should succeed");
            },
        );

        assert!(root.join("var/backend/master.key").is_file());
    }
}
