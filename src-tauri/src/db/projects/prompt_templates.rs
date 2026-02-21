use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ensure_column, fetch_project_by_slug, is_unique_constraint_error, normalize_required_text,
    normalize_slug, now_iso, ProjectsRepoError, ProjectsStore,
};

#[derive(Debug, Clone, Serialize)]
pub struct PromptTemplateSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub template_text: String,
    pub created_at: String,
    pub updated_at: String,
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

impl ProjectsStore {
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
}

pub(super) fn ensure_prompt_template_tables(conn: &Connection) -> Result<(), ProjectsRepoError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS prompt_templates (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          template_text TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, name)
        );
    ",
    )?;

    Ok(())
}

pub(super) fn ensure_prompt_template_columns(conn: &Connection) -> Result<(), ProjectsRepoError> {
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

    Ok(())
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
