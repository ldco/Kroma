use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{
    ensure_column, fetch_project_by_slug, normalize_chat_role, normalize_optional_storage_field,
    normalize_required_text, normalize_slug, now_iso, AgentInstructionWorkerLease,
    ProjectsRepoError, ProjectsStore,
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
                    next_attempt_at = NULL,
                    last_error = NULL,
                    locked_by = NULL,
                    locked_at = NULL,
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
                    locked_by = NULL,
                    locked_at = NULL,
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

    pub(crate) fn reserve_next_agent_instruction(
        &self,
        worker_id: &str,
        max_locked_seconds: i64,
        default_max_attempts: i64,
    ) -> Result<Option<AgentInstructionWorkerLease>, ProjectsRepoError> {
        let safe_worker_id = normalize_required_text(worker_id, "worker_id")?;
        let max_locked_seconds = max_locked_seconds.max(1);
        let default_max_attempts = default_max_attempts.max(1);
        self.with_connection_mut(|conn| {
            let now = Utc::now();
            let now_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let lock_cutoff = (now - Duration::seconds(max_locked_seconds))
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string();

            let tx = conn.transaction()?;
            let lease = tx
                .query_row(
                    "
                    SELECT
                      ai.id AS instruction_id,
                      ai.project_id AS project_id,
                      p.slug AS project_slug,
                      ai.instruction_text AS instruction_text,
                      COALESCE(ai.attempts, 0) AS attempts,
                      COALESCE(NULLIF(ai.max_attempts, 0), ?3) AS max_attempts
                    FROM agent_instructions ai
                    JOIN projects p ON p.id = ai.project_id
                    WHERE ai.status = 'confirmed'
                      AND (ai.next_attempt_at IS NULL OR ai.next_attempt_at <= ?1)
                      AND (ai.locked_at IS NULL OR ai.locked_at <= ?2)
                    ORDER BY COALESCE(ai.updated_at, ai.created_at) ASC, ai.id ASC
                    LIMIT 1
                    ",
                    params![now_iso, lock_cutoff, default_max_attempts],
                    row_to_agent_instruction_worker_lease,
                )
                .optional()?;
            let Some(lease) = lease else {
                tx.commit()?;
                return Ok(None);
            };

            let updated = tx.execute(
                "
                UPDATE agent_instructions
                SET status = 'running',
                    locked_by = ?1,
                    locked_at = ?2,
                    max_attempts = COALESCE(NULLIF(max_attempts, 0), ?3),
                    updated_at = ?2
                WHERE id = ?4 AND status = 'confirmed'
                ",
                params![
                    safe_worker_id,
                    now_iso,
                    default_max_attempts,
                    lease.instruction_id
                ],
            )?;
            if updated != 1 {
                tx.commit()?;
                return Ok(None);
            }

            tx.commit()?;
            Ok(Some(lease))
        })
    }

    pub(crate) fn complete_agent_instruction_success(
        &self,
        lease: &AgentInstructionWorkerLease,
        attempts: i64,
        max_attempts: i64,
        remote_status: &str,
        response_json: &Value,
        http_status: Option<u16>,
    ) -> Result<(), ProjectsRepoError> {
        let remote_status = match remote_status.trim().to_ascii_lowercase().as_str() {
            "done" | "failed" | "running" => remote_status.trim().to_ascii_lowercase(),
            _ => String::from("done"),
        };
        let now = now_iso();
        let response_serialized = serde_json::to_string(response_json)
            .unwrap_or_else(|_| String::from("{\"ok\":false,\"error\":\"invalid_response\"}"));
        self.with_connection_mut(|conn| {
            let tx = conn.transaction()?;
            tx.execute(
                "
                UPDATE agent_instructions
                SET status = ?1,
                    attempts = ?2,
                    max_attempts = ?3,
                    next_attempt_at = NULL,
                    last_error = NULL,
                    locked_by = NULL,
                    locked_at = NULL,
                    agent_response_json = ?4,
                    updated_at = ?5
                WHERE id = ?6 AND project_id = ?7
                ",
                params![
                    remote_status,
                    attempts,
                    max_attempts,
                    response_serialized,
                    now,
                    lease.instruction_id,
                    lease.project_id
                ],
            )?;
            record_agent_instruction_event(
                &tx,
                lease.project_id.as_str(),
                lease.instruction_id.as_str(),
                "result",
                Some(format!(
                    "remote_status={}, attempts={}, http_status={}",
                    remote_status,
                    attempts,
                    http_status
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| String::from("none"))
                )),
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    pub(crate) fn complete_agent_instruction_retry_or_fail(
        &self,
        lease: &AgentInstructionWorkerLease,
        attempts: i64,
        max_attempts: i64,
        retry_backoff_seconds: i64,
        error: &str,
    ) -> Result<(), ProjectsRepoError> {
        let retryable = attempts < max_attempts;
        let next_attempt_at = if retryable {
            Some(
                (Utc::now() + Duration::seconds(retry_backoff_seconds.max(1) * attempts))
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            )
        } else {
            None
        };
        let status = if retryable { "confirmed" } else { "failed" };
        let event_type = if retryable {
            "retry_scheduled"
        } else {
            "error"
        };
        let now = now_iso();
        let clean_error = error.trim();
        let error_value = if clean_error.is_empty() {
            String::from("dispatch_failed")
        } else {
            clean_error.to_string()
        };
        self.with_connection_mut(|conn| {
            let tx = conn.transaction()?;
            tx.execute(
                "
                UPDATE agent_instructions
                SET status = ?1,
                    attempts = ?2,
                    max_attempts = ?3,
                    next_attempt_at = ?4,
                    last_error = ?5,
                    locked_by = NULL,
                    locked_at = NULL,
                    updated_at = ?6
                WHERE id = ?7 AND project_id = ?8
                ",
                params![
                    status,
                    attempts,
                    max_attempts,
                    next_attempt_at,
                    error_value,
                    now,
                    lease.instruction_id,
                    lease.project_id
                ],
            )?;
            record_agent_instruction_event(
                &tx,
                lease.project_id.as_str(),
                lease.instruction_id.as_str(),
                event_type,
                Some(format!(
                    "error={}, attempts={}, max_attempts={}, next_attempt_at={}",
                    error_value,
                    attempts,
                    max_attempts,
                    next_attempt_at.unwrap_or_else(|| String::from("none"))
                )),
            )?;
            tx.commit()?;
            Ok(())
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
        "agent_instructions",
        "attempts",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "agent_instructions",
        "max_attempts",
        "INTEGER NOT NULL DEFAULT 3",
    )?;
    ensure_column(conn, "agent_instructions", "next_attempt_at", "TEXT")?;
    ensure_column(conn, "agent_instructions", "last_error", "TEXT")?;
    ensure_column(conn, "agent_instructions", "locked_by", "TEXT")?;
    ensure_column(conn, "agent_instructions", "locked_at", "TEXT")?;
    ensure_column(conn, "agent_instructions", "agent_response_json", "TEXT")?;

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

fn row_to_agent_instruction_worker_lease(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AgentInstructionWorkerLease> {
    Ok(AgentInstructionWorkerLease {
        instruction_id: row.get("instruction_id")?,
        project_id: row.get("project_id")?,
        project_slug: row.get("project_slug")?,
        instruction_text: row.get("instruction_text")?,
        attempts: row.get::<_, Option<i64>>("attempts")?.unwrap_or(0),
        max_attempts: row.get::<_, Option<i64>>("max_attempts")?.unwrap_or(3),
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
