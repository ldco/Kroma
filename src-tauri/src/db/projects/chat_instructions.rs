use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ensure_column, fetch_project_by_slug, normalize_chat_role, normalize_optional_storage_field,
    normalize_required_text, normalize_slug, now_iso, ProjectsRepoError, ProjectsStore,
};

#[derive(Debug, Clone, Serialize)]
pub struct ChatSessionSummary {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageSummary {
    pub id: String,
    pub project_id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentInstructionSummary {
    pub id: String,
    pub project_id: String,
    pub instruction_text: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub confirmed_at: String,
    pub canceled_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentInstructionEventSummary {
    pub id: String,
    pub project_id: String,
    pub instruction_id: String,
    pub event_type: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateChatSessionInput {
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateChatMessageInput {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateAgentInstructionInput {
    #[serde(default)]
    pub instruction_text: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentInstructionActionInput {
    #[serde(default)]
    pub message: Option<String>,
}

impl ProjectsStore {
    pub fn list_chat_sessions(
        &self,
        slug: &str,
    ) -> Result<Vec<ChatSessionSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, title, status, created_at, updated_at
                FROM chat_sessions
                WHERE project_id = ?1
                ORDER BY COALESCE(updated_at, '') DESC, id DESC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_chat_session_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_chat_session(
        &self,
        slug: &str,
        input: CreateChatSessionInput,
    ) -> Result<ChatSessionSummary, ProjectsRepoError> {
        let title = input
            .title
            .as_deref()
            .and_then(normalize_optional_storage_field)
            .unwrap_or_else(|| String::from("New Session"));

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO chat_sessions
                  (id, project_id, title, status, created_at, updated_at)
                VALUES
                  (?1, ?2, ?3, 'active', ?4, ?4)
            ",
                params![id, project.id, title, now],
            )?;

            fetch_chat_session_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_chat_session_detail(
        &self,
        slug: &str,
        session_id: &str,
    ) -> Result<ChatSessionSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_chat_session_by_id(conn, project.id.as_str(), session_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn list_chat_messages(
        &self,
        slug: &str,
        session_id: &str,
    ) -> Result<Vec<ChatMessageSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let session_exists =
                fetch_chat_session_by_id(conn, project.id.as_str(), session_id)?.is_some();
            if !session_exists {
                return Err(ProjectsRepoError::NotFound);
            }

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, session_id, role, content, created_at
                FROM chat_messages
                WHERE project_id = ?1 AND session_id = ?2
                ORDER BY rowid ASC
            ",
            )?;
            let rows =
                stmt.query_map(params![project.id, session_id], row_to_chat_message_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_chat_message(
        &self,
        slug: &str,
        session_id: &str,
        input: CreateChatMessageInput,
    ) -> Result<ChatMessageSummary, ProjectsRepoError> {
        let role = normalize_chat_role(input.role.as_str())?;
        let content = normalize_required_text(input.content.as_str(), "content")?;

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let session_exists =
                fetch_chat_session_by_id(conn, project.id.as_str(), session_id)?.is_some();
            if !session_exists {
                return Err(ProjectsRepoError::NotFound);
            }

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO chat_messages
                  (id, project_id, session_id, role, content, created_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6)
            ",
                params![id, project.id, session_id, role, content, now],
            )?;

            fetch_chat_message_by_id(conn, project.id.as_str(), session_id, id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn list_agent_instructions(
        &self,
        slug: &str,
    ) -> Result<Vec<AgentInstructionSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  project_id,
                  instruction_text,
                  status,
                  created_at,
                  updated_at,
                  confirmed_at,
                  canceled_at
                FROM agent_instructions
                WHERE project_id = ?1
                ORDER BY COALESCE(updated_at, '') DESC, id DESC
            ",
            )?;
            let rows = stmt.query_map(params![project.id], row_to_agent_instruction_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn create_agent_instruction(
        &self,
        slug: &str,
        input: CreateAgentInstructionInput,
    ) -> Result<AgentInstructionSummary, ProjectsRepoError> {
        let instruction_text =
            normalize_required_text(input.instruction_text.as_str(), "instruction_text")?;

        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project =
                fetch_project_by_slug(conn, safe_slug.as_str())?.ok_or(ProjectsRepoError::NotFound)?;

            let id = Uuid::new_v4().to_string();
            let now = now_iso();
            conn.execute(
                "
                INSERT INTO agent_instructions
                  (id, project_id, instruction_text, status, created_at, updated_at, confirmed_at, canceled_at)
                VALUES
                  (?1, ?2, ?3, 'pending', ?4, ?4, NULL, NULL)
            ",
                params![id, project.id, instruction_text, now],
            )?;
            record_agent_instruction_event(
                conn,
                project.id.as_str(),
                id.as_str(),
                "created",
                Some(String::from("Instruction created")),
            )?;

            fetch_agent_instruction_by_id(conn, project.id.as_str(), id.as_str())?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn get_agent_instruction_detail(
        &self,
        slug: &str,
        instruction_id: &str,
    ) -> Result<AgentInstructionSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            fetch_agent_instruction_by_id(conn, project.id.as_str(), instruction_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn list_agent_instruction_events(
        &self,
        slug: &str,
        instruction_id: &str,
    ) -> Result<Vec<AgentInstructionEventSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let instruction_exists =
                fetch_agent_instruction_by_id(conn, project.id.as_str(), instruction_id)?.is_some();
            if !instruction_exists {
                return Err(ProjectsRepoError::NotFound);
            }

            let mut stmt = conn.prepare(
                "
                SELECT id, project_id, instruction_id, event_type, message, created_at
                FROM agent_instruction_events
                WHERE project_id = ?1 AND instruction_id = ?2
                ORDER BY rowid ASC
            ",
            )?;
            let rows = stmt.query_map(
                params![project.id, instruction_id],
                row_to_agent_instruction_event_summary,
            )?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn confirm_agent_instruction(
        &self,
        slug: &str,
        instruction_id: &str,
        input: AgentInstructionActionInput,
    ) -> Result<AgentInstructionSummary, ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing =
                fetch_agent_instruction_by_id(conn, project.id.as_str(), instruction_id)?
                    .ok_or(ProjectsRepoError::NotFound)?;

            if existing.status == "canceled" {
                return Err(ProjectsRepoError::Validation(String::from(
                    "Instruction is already canceled",
                )));
            }
            if existing.status == "confirmed" {
                return Ok(existing);
            }

            let now = now_iso();
            conn.execute(
                "
                UPDATE agent_instructions
                SET status = 'confirmed',
                    confirmed_at = ?1,
                    canceled_at = NULL,
                    updated_at = ?1
                WHERE id = ?2 AND project_id = ?3
            ",
                params![now, instruction_id, project.id],
            )?;
            record_agent_instruction_event(
                conn,
                project.id.as_str(),
                instruction_id,
                "confirmed",
                input
                    .message
                    .as_deref()
                    .and_then(normalize_optional_storage_field)
                    .or(Some(String::from("Instruction confirmed"))),
            )?;

            fetch_agent_instruction_by_id(conn, project.id.as_str(), instruction_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }

    pub fn cancel_agent_instruction(
        &self,
        slug: &str,
        instruction_id: &str,
        input: AgentInstructionActionInput,
    ) -> Result<AgentInstructionSummary, ProjectsRepoError> {
        self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let existing =
                fetch_agent_instruction_by_id(conn, project.id.as_str(), instruction_id)?
                    .ok_or(ProjectsRepoError::NotFound)?;

            if existing.status == "confirmed" {
                return Err(ProjectsRepoError::Validation(String::from(
                    "Instruction is already confirmed",
                )));
            }
            if existing.status == "canceled" {
                return Ok(existing);
            }

            let now = now_iso();
            conn.execute(
                "
                UPDATE agent_instructions
                SET status = 'canceled',
                    canceled_at = ?1,
                    updated_at = ?1
                WHERE id = ?2 AND project_id = ?3
            ",
                params![now, instruction_id, project.id],
            )?;
            record_agent_instruction_event(
                conn,
                project.id.as_str(),
                instruction_id,
                "canceled",
                input
                    .message
                    .as_deref()
                    .and_then(normalize_optional_storage_field)
                    .or(Some(String::from("Instruction canceled"))),
            )?;

            fetch_agent_instruction_by_id(conn, project.id.as_str(), instruction_id)?
                .ok_or(ProjectsRepoError::NotFound)
        })
    }
}

pub(super) fn ensure_chat_and_instruction_tables(
    conn: &Connection,
) -> Result<(), ProjectsRepoError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS chat_sessions (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          title TEXT NOT NULL,
          status TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chat_messages (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          session_id TEXT NOT NULL,
          role TEXT NOT NULL,
          content TEXT NOT NULL,
          created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS agent_instructions (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          instruction_text TEXT NOT NULL,
          status TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          confirmed_at TEXT,
          canceled_at TEXT
        );

        CREATE TABLE IF NOT EXISTS agent_instruction_events (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          instruction_id TEXT NOT NULL,
          event_type TEXT NOT NULL,
          message TEXT,
          created_at TEXT NOT NULL
        );
    ",
    )?;

    Ok(())
}

pub(super) fn ensure_chat_and_instruction_columns(
    conn: &Connection,
) -> Result<(), ProjectsRepoError> {
    ensure_column(conn, "chat_sessions", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "chat_sessions", "title", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(
        conn,
        "chat_sessions",
        "status",
        "TEXT NOT NULL DEFAULT 'active'",
    )?;
    ensure_column(
        conn,
        "chat_sessions",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "chat_sessions",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "chat_messages", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "chat_messages", "session_id", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "chat_messages",
        "role",
        "TEXT NOT NULL DEFAULT 'user'",
    )?;
    ensure_column(conn, "chat_messages", "content", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(
        conn,
        "chat_messages",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    ensure_column(conn, "agent_instructions", "project_id", "TEXT NOT NULL")?;
    ensure_column(
        conn,
        "agent_instructions",
        "instruction_text",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "agent_instructions",
        "status",
        "TEXT NOT NULL DEFAULT 'pending'",
    )?;
    ensure_column(
        conn,
        "agent_instructions",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "agent_instructions",
        "updated_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(conn, "agent_instructions", "confirmed_at", "TEXT")?;
    ensure_column(conn, "agent_instructions", "canceled_at", "TEXT")?;

    ensure_column(
        conn,
        "agent_instruction_events",
        "project_id",
        "TEXT NOT NULL",
    )?;
    ensure_column(
        conn,
        "agent_instruction_events",
        "instruction_id",
        "TEXT NOT NULL",
    )?;
    ensure_column(
        conn,
        "agent_instruction_events",
        "event_type",
        "TEXT NOT NULL DEFAULT 'created'",
    )?;
    ensure_column(conn, "agent_instruction_events", "message", "TEXT")?;
    ensure_column(
        conn,
        "agent_instruction_events",
        "created_at",
        "TEXT NOT NULL DEFAULT ''",
    )?;

    Ok(())
}

fn row_to_chat_session_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatSessionSummary> {
    Ok(ChatSessionSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn fetch_chat_session_by_id(
    conn: &Connection,
    project_id: &str,
    session_id: &str,
) -> Result<Option<ChatSessionSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT id, project_id, title, status, created_at, updated_at
        FROM chat_sessions
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![session_id, project_id],
        row_to_chat_session_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_chat_message_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatMessageSummary> {
    Ok(ChatMessageSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        session_id: row.get("session_id")?,
        role: row.get("role")?,
        content: row.get("content")?,
        created_at: row.get("created_at")?,
    })
}

fn fetch_chat_message_by_id(
    conn: &Connection,
    project_id: &str,
    session_id: &str,
    message_id: &str,
) -> Result<Option<ChatMessageSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT id, project_id, session_id, role, content, created_at
        FROM chat_messages
        WHERE id = ?1 AND project_id = ?2 AND session_id = ?3
        LIMIT 1
    ",
        params![message_id, project_id, session_id],
        row_to_chat_message_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_agent_instruction_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AgentInstructionSummary> {
    Ok(AgentInstructionSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        instruction_text: row.get("instruction_text")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        confirmed_at: row
            .get::<_, Option<String>>("confirmed_at")?
            .unwrap_or_default(),
        canceled_at: row
            .get::<_, Option<String>>("canceled_at")?
            .unwrap_or_default(),
    })
}

fn fetch_agent_instruction_by_id(
    conn: &Connection,
    project_id: &str,
    instruction_id: &str,
) -> Result<Option<AgentInstructionSummary>, ProjectsRepoError> {
    conn.query_row(
        "
        SELECT
          id,
          project_id,
          instruction_text,
          status,
          created_at,
          updated_at,
          confirmed_at,
          canceled_at
        FROM agent_instructions
        WHERE id = ?1 AND project_id = ?2
        LIMIT 1
    ",
        params![instruction_id, project_id],
        row_to_agent_instruction_summary,
    )
    .optional()
    .map_err(ProjectsRepoError::from)
}

fn row_to_agent_instruction_event_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AgentInstructionEventSummary> {
    Ok(AgentInstructionEventSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        instruction_id: row.get("instruction_id")?,
        event_type: row.get("event_type")?,
        message: row.get::<_, Option<String>>("message")?.unwrap_or_default(),
        created_at: row.get("created_at")?,
    })
}

fn record_agent_instruction_event(
    conn: &Connection,
    project_id: &str,
    instruction_id: &str,
    event_type: &str,
    message: Option<String>,
) -> Result<(), ProjectsRepoError> {
    conn.execute(
        "
        INSERT INTO agent_instruction_events
          (id, project_id, instruction_id, event_type, message, created_at)
        VALUES
          (?1, ?2, ?3, ?4, ?5, ?6)
    ",
        params![
            Uuid::new_v4().to_string(),
            project_id,
            instruction_id,
            event_type,
            message,
            now_iso()
        ],
    )?;
    Ok(())
}
