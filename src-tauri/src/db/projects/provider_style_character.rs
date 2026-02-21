use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{
    ensure_column, fetch_project_by_slug, is_unique_constraint_error,
    normalize_optional_storage_field, normalize_provider_code, normalize_required_text,
    normalize_slug, now_iso, parse_json_value, ProjectsRepoError, ProjectsStore,
};

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

impl ProjectsStore {
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
}

pub(super) fn ensure_provider_style_character_tables(
    conn: &Connection,
) -> Result<(), ProjectsRepoError> {
    conn.execute_batch(
        "
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
    ",
    )?;

    Ok(())
}

pub(super) fn ensure_provider_style_character_columns(
    conn: &Connection,
) -> Result<(), ProjectsRepoError> {
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

    Ok(())
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
