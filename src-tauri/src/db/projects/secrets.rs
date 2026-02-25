use std::collections::BTreeMap;
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
    ensure_column, fetch_project_by_slug, normalize_optional_text, normalize_provider_code,
    normalize_required_text, normalize_slug, now_iso, ProjectsRepoError, ProjectsStore,
};

const DEFAULT_MASTER_KEY_FILE: &str = "var/backend/master.key";
const SECRET_CIPHERTEXT_PREFIX: &str = "enc:v1:";
const DEFAULT_SECRET_KEY_REF: &str = "local-master";

#[derive(Debug, Clone, Serialize)]
pub struct SecretSummary {
    pub project_id: String,
    pub provider_code: String,
    pub secret_name: String,
    pub has_value: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretKeyRefCount {
    pub key_ref: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretEncryptionStatus {
    pub total: usize,
    pub encrypted: usize,
    pub plaintext: usize,
    pub empty: usize,
    pub key_refs: Vec<SecretKeyRefCount>,
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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RotateSecretsInput {
    #[serde(default)]
    pub from_key_ref: Option<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotateSecretsResult {
    pub scanned: usize,
    pub rotated: usize,
    pub skipped_empty: usize,
    pub skipped_current_key_ref: usize,
    pub plaintext_reencrypted: usize,
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

    pub fn get_project_secret_encryption_status(
        &self,
        slug: &str,
    ) -> Result<SecretEncryptionStatus, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT secret_value, key_ref
                FROM project_secrets
                WHERE project_id = ?1
            ",
            )?;
            let mut rows = stmt.query(params![project.id.as_str()])?;

            let mut status = SecretEncryptionStatus {
                total: 0,
                encrypted: 0,
                plaintext: 0,
                empty: 0,
                key_refs: Vec::new(),
            };
            let mut key_ref_counts = BTreeMap::<String, usize>::new();
            while let Some(row) = rows.next()? {
                status.total += 1;
                let secret_value = row
                    .get::<_, Option<String>>("secret_value")?
                    .unwrap_or_default();
                let key_ref = row
                    .get::<_, Option<String>>("key_ref")?
                    .unwrap_or_else(|| String::from(DEFAULT_SECRET_KEY_REF));
                *key_ref_counts.entry(key_ref).or_insert(0) += 1;

                if secret_value.trim().is_empty() {
                    status.empty += 1;
                } else if secret_value.starts_with(SECRET_CIPHERTEXT_PREFIX) {
                    status.encrypted += 1;
                } else {
                    status.plaintext += 1;
                }
            }
            status.key_refs = key_ref_counts
                .into_iter()
                .map(|(key_ref, count)| SecretKeyRefCount { key_ref, count })
                .collect();
            Ok(status)
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
        let key_ref = current_secret_key_ref();

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
                    key_ref,
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

    pub fn rotate_project_secrets(
        &self,
        slug: &str,
        input: RotateSecretsInput,
    ) -> Result<RotateSecretsResult, ProjectsRepoError> {
        let target_key_ref = current_secret_key_ref();
        let from_key_ref = normalize_optional_text(input.from_key_ref.as_deref());
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = if from_key_ref.is_some() {
                conn.prepare(
                    "
                    SELECT provider_code, secret_name, secret_value, key_ref
                    FROM project_secrets
                    WHERE project_id = ?1 AND key_ref = ?2
                    ORDER BY provider_code ASC, secret_name ASC
                ",
                )?
            } else {
                conn.prepare(
                    "
                    SELECT provider_code, secret_name, secret_value, key_ref
                    FROM project_secrets
                    WHERE project_id = ?1
                    ORDER BY provider_code ASC, secret_name ASC
                ",
                )?
            };

            let mut rows = if let Some(ref_key) = from_key_ref.as_deref() {
                stmt.query(params![project.id.as_str(), ref_key])?
            } else {
                stmt.query(params![project.id.as_str()])?
            };

            let mut secrets = Vec::new();
            while let Some(row) = rows.next()? {
                secrets.push(StoredSecretRow {
                    provider_code: row.get("provider_code")?,
                    secret_name: row.get("secret_name")?,
                    secret_value: row
                        .get::<_, Option<String>>("secret_value")?
                        .unwrap_or_default(),
                    key_ref: row
                        .get::<_, Option<String>>("key_ref")?
                        .unwrap_or_else(|| String::from(DEFAULT_SECRET_KEY_REF)),
                });
            }
            drop(rows);
            drop(stmt);

            let mut result = RotateSecretsResult {
                scanned: secrets.len(),
                rotated: 0,
                skipped_empty: 0,
                skipped_current_key_ref: 0,
                plaintext_reencrypted: 0,
            };

            let candidates: Vec<StoredSecretRow> = secrets
                .into_iter()
                .filter_map(|secret| {
                    if secret.secret_value.trim().is_empty() {
                        result.skipped_empty += 1;
                        return None;
                    }
                    if !input.force && secret.key_ref == target_key_ref {
                        result.skipped_current_key_ref += 1;
                        return None;
                    }
                    Some(secret)
                })
                .collect();
            if candidates.is_empty() {
                return Ok(result);
            }

            let current_key = load_or_create_master_key_bytes(self.repo_root.as_path(), false)?;
            let mut decryption_keys = vec![current_key];
            let previous_keys = parse_previous_master_keys()?;
            decryption_keys.extend(previous_keys);

            let tx = conn.transaction()?;
            let now = now_iso();
            for secret in candidates {
                let plaintext = if secret.secret_value.starts_with(SECRET_CIPHERTEXT_PREFIX) {
                    decrypt_secret_value_with_candidates(
                        secret.secret_value.as_str(),
                        decryption_keys.as_slice(),
                    )?
                } else {
                    result.plaintext_reencrypted += 1;
                    secret.secret_value
                };
                let ciphertext = encrypt_secret_value_with_key(plaintext.as_str(), &current_key)?;
                tx.execute(
                    "
                    UPDATE project_secrets
                    SET secret_value = ?1,
                        key_ref = ?2,
                        updated_at = ?3
                    WHERE project_id = ?4
                      AND provider_code = ?5
                      AND secret_name = ?6
                ",
                    params![
                        ciphertext,
                        target_key_ref,
                        now,
                        project.id.as_str(),
                        secret.provider_code.as_str(),
                        secret.secret_name.as_str(),
                    ],
                )?;
                result.rotated += 1;
            }
            tx.commit()?;

            Ok(result)
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
    encrypt_secret_value_with_key(secret_value, &key_bytes)
}

fn encrypt_secret_value_with_key(
    secret_value: &str,
    key_bytes: &[u8; 32],
) -> Result<String, ProjectsRepoError> {
    let mut nonce_bytes = [0u8; 12];
    SystemRandom::new().fill(&mut nonce_bytes).map_err(|_| {
        ProjectsRepoError::Internal(String::from("Failed to generate secret nonce"))
    })?;
    let unbound = UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|_| {
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

fn decrypt_secret_value_with_candidates(
    secret_value: &str,
    key_candidates: &[[u8; 32]],
) -> Result<String, ProjectsRepoError> {
    for key_bytes in key_candidates {
        if let Ok(plaintext) = decrypt_secret_value_with_key(secret_value, key_bytes) {
            return Ok(plaintext);
        }
    }
    Err(ProjectsRepoError::Internal(String::from(
        "Unable to decrypt secret with configured rotation keys",
    )))
}

fn decrypt_secret_value_with_key(
    secret_value: &str,
    key_bytes: &[u8; 32],
) -> Result<String, ProjectsRepoError> {
    let encoded = secret_value
        .strip_prefix(SECRET_CIPHERTEXT_PREFIX)
        .ok_or_else(|| {
            ProjectsRepoError::Internal(String::from("Unsupported encrypted secret format"))
        })?;
    let payload = URL_SAFE.decode(encoded.as_bytes()).map_err(|_| {
        ProjectsRepoError::Internal(String::from("Invalid encrypted secret payload"))
    })?;
    if payload.len() <= 12 {
        return Err(ProjectsRepoError::Internal(String::from(
            "Encrypted secret payload is truncated",
        )));
    }

    let (nonce_raw, ciphertext) = payload.split_at(12);
    let nonce = Nonce::assume_unique_for_key(nonce_raw.try_into().map_err(|_| {
        ProjectsRepoError::Internal(String::from("Encrypted secret nonce is invalid"))
    })?);
    let unbound = UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|_| {
        ProjectsRepoError::Internal(String::from("Failed to initialize secret decryption key"))
    })?;
    let key = LessSafeKey::new(unbound);
    let mut in_out = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| ProjectsRepoError::Internal(String::from("Secret decryption failed")))?;
    String::from_utf8(plaintext.to_vec()).map_err(|_| {
        ProjectsRepoError::Internal(String::from("Decrypted secret is not valid UTF-8"))
    })
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

fn current_secret_key_ref() -> String {
    std::env::var("IAT_MASTER_KEY_REF")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| String::from(DEFAULT_SECRET_KEY_REF))
}

fn parse_previous_master_keys() -> Result<Vec<[u8; 32]>, ProjectsRepoError> {
    let raw = std::env::var("IAT_MASTER_KEY_PREVIOUS").unwrap_or_default();
    let mut out = Vec::new();
    for candidate in raw.split(',') {
        let value = candidate.trim();
        if value.is_empty() {
            continue;
        }
        out.push(decode_master_key(value)?);
    }
    Ok(out)
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

#[derive(Debug, Clone)]
struct StoredSecretRow {
    provider_code: String,
    secret_name: String,
    secret_value: String,
    key_ref: String,
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;
    use crate::db::projects::{
        ProjectsStore, RotateSecretsInput, UpsertProjectInput, UpsertSecretInput,
    };

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
        assert_eq!(row.1, DEFAULT_SECRET_KEY_REF);
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

    #[test]
    fn rotate_secret_reencrypts_with_current_key_and_ref() {
        let root =
            std::env::temp_dir().join(format!("kroma_secrets_rotate_{}", uuid::Uuid::new_v4()));
        let db = root.join("var/backend/app.db");
        let store = ProjectsStore::new(db.clone(), root.clone());
        store.initialize().expect("store should initialize");
        let created = store
            .upsert_project(UpsertProjectInput {
                name: String::from("Secrets Rotate"),
                ..UpsertProjectInput::default()
            })
            .expect("project upsert should succeed");
        let slug = created.project.slug;

        with_env_vars(
            &[
                (
                    "IAT_MASTER_KEY",
                    Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
                ),
                ("IAT_MASTER_KEY_REF", Some("local-master-v1")),
                ("IAT_MASTER_KEY_PREVIOUS", None),
            ],
            || {
                store
                    .upsert_project_secret(
                        slug.as_str(),
                        UpsertSecretInput {
                            provider_code: String::from("openai"),
                            secret_name: String::from("api_key"),
                            secret_value: String::from("sk-test-rotate-me"),
                        },
                    )
                    .expect("secret upsert should succeed");
            },
        );

        let before = Connection::open(db.as_path())
            .expect("db should open")
            .query_row(
                "SELECT secret_value, key_ref FROM project_secrets WHERE provider_code = 'openai' AND secret_name = 'api_key'",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .expect("secret row should exist");
        assert_eq!(before.1, "local-master-v1");

        with_env_vars(
            &[
                (
                    "IAT_MASTER_KEY",
                    Some("AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE="),
                ),
                ("IAT_MASTER_KEY_REF", Some("local-master-v2")),
                (
                    "IAT_MASTER_KEY_PREVIOUS",
                    Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
                ),
            ],
            || {
                let rotation = store
                    .rotate_project_secrets(
                        slug.as_str(),
                        RotateSecretsInput {
                            from_key_ref: Some(String::from("local-master-v1")),
                            force: false,
                        },
                    )
                    .expect("rotation should succeed");
                assert_eq!(rotation.scanned, 1);
                assert_eq!(rotation.rotated, 1);
                assert_eq!(rotation.skipped_empty, 0);
                assert_eq!(rotation.skipped_current_key_ref, 0);
                assert_eq!(rotation.plaintext_reencrypted, 0);
            },
        );

        let after = Connection::open(db.as_path())
            .expect("db should open")
            .query_row(
                "SELECT secret_value, key_ref FROM project_secrets WHERE provider_code = 'openai' AND secret_name = 'api_key'",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .expect("secret row should exist");
        assert_eq!(after.1, "local-master-v2");
        assert!(after.0.starts_with(SECRET_CIPHERTEXT_PREFIX));
        assert_ne!(before.0, after.0);
    }

    #[test]
    fn encryption_status_counts_plaintext_encrypted_and_empty_rows() {
        let root =
            std::env::temp_dir().join(format!("kroma_secrets_status_{}", uuid::Uuid::new_v4()));
        let db = root.join("var/backend/app.db");
        let store = ProjectsStore::new(db.clone(), root.clone());
        store.initialize().expect("store should initialize");
        let created = store
            .upsert_project(UpsertProjectInput {
                name: String::from("Secrets Status"),
                ..UpsertProjectInput::default()
            })
            .expect("project upsert should succeed");
        let slug = created.project.slug.clone();

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
                            secret_value: String::from("sk-test-status"),
                        },
                    )
                    .expect("secret upsert should succeed");
            },
        );

        let conn = Connection::open(db.as_path()).expect("db should open");
        let project_id = conn
            .query_row(
                "SELECT id FROM projects WHERE slug = ?1 LIMIT 1",
                params![slug.as_str()],
                |row| row.get::<_, String>(0),
            )
            .expect("project should exist");
        let now = now_iso();
        conn.execute(
            "
            INSERT INTO project_secrets
              (project_id, provider_code, secret_name, secret_value, key_ref, created_at, updated_at)
            VALUES
              (?1, 'legacy', 'legacy_plain', 'legacy-plain-text', 'legacy-key', ?2, ?2)
        ",
            params![project_id.as_str(), now.as_str()],
        )
        .expect("legacy plaintext row insert should succeed");
        conn.execute(
            "
            INSERT INTO project_secrets
              (project_id, provider_code, secret_name, secret_value, key_ref, created_at, updated_at)
            VALUES
              (?1, 'meta', 'empty_secret', '', 'local-master', ?2, ?2)
        ",
            params![project_id.as_str(), now.as_str()],
        )
        .expect("empty row insert should succeed");

        let status = store
            .get_project_secret_encryption_status(slug.as_str())
            .expect("status should load");
        assert_eq!(status.total, 3);
        assert_eq!(status.encrypted, 1);
        assert_eq!(status.plaintext, 1);
        assert_eq!(status.empty, 1);
        assert_eq!(status.key_refs.len(), 2);
    }
}
