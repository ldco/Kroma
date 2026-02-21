use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{
    ensure_column, fetch_project_by_slug, is_unique_constraint_error,
    normalize_optional_storage_field, normalize_required_text, normalize_slug, now_iso,
    parse_json_value, ProjectsRepoError, ProjectsStore,
};

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

impl ProjectsStore {
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
}

pub(super) fn ensure_reference_set_tables(conn: &Connection) -> Result<(), ProjectsRepoError> {
    conn.execute_batch(
        "
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
    ",
    )?;

    Ok(())
}

pub(super) fn ensure_reference_set_columns(conn: &Connection) -> Result<(), ProjectsRepoError> {
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

    Ok(())
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
