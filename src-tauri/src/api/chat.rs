use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{
    ChatMessageSummary, ChatSessionSummary, CreateChatMessageInput, CreateChatSessionInput,
    ProjectsRepoError,
};

type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Deserialize)]
pub struct SlugSessionPath {
    pub slug: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListSessionsResponse {
    ok: bool,
    count: usize,
    sessions: Vec<ChatSessionSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionResponse {
    ok: bool,
    session: ChatSessionSummary,
}

#[derive(Debug, Clone, Serialize)]
struct ListMessagesResponse {
    ok: bool,
    session_id: String,
    count: usize,
    messages: Vec<ChatMessageSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct MessageResponse {
    ok: bool,
    message: ChatMessageSummary,
}

pub async fn list_chat_sessions_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || store.list_chat_sessions(slug.as_str())).await;

    match result {
        Ok(Ok(sessions)) => (
            StatusCode::OK,
            into_json(ListSessionsResponse {
                ok: true,
                count: sessions.len(),
                sessions,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => {
            internal_error(format!("chat session listing task failed: {join_error}"))
        }
    }
}

pub async fn create_chat_session_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateChatSessionInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.create_chat_session(slug.as_str(), payload))
            .await;

    match result {
        Ok(Ok(session)) => (
            StatusCode::OK,
            into_json(SessionResponse { ok: true, session }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("chat session create task failed: {join_error}")),
    }
}

pub async fn get_chat_session_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugSessionPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_chat_session_detail(path.slug.as_str(), path.session_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(session)) => (
            StatusCode::OK,
            into_json(SessionResponse { ok: true, session }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Chat session not found"),
        Err(join_error) => internal_error(format!("chat session detail task failed: {join_error}")),
    }
}

pub async fn list_chat_messages_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugSessionPath>,
) -> ApiObject<Value> {
    let session_id = path.session_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.list_chat_messages(path.slug.as_str(), path.session_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(messages)) => (
            StatusCode::OK,
            into_json(ListMessagesResponse {
                ok: true,
                session_id,
                count: messages.len(),
                messages,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Chat session not found"),
        Err(join_error) => {
            internal_error(format!("chat message listing task failed: {join_error}"))
        }
    }
}

pub async fn create_chat_message_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugSessionPath>,
    Json(payload): Json<CreateChatMessageInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.create_chat_message(path.slug.as_str(), path.session_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(message)) => (
            StatusCode::OK,
            into_json(MessageResponse { ok: true, message }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Chat session not found"),
        Err(join_error) => internal_error(format!("chat message create task failed: {join_error}")),
    }
}

fn map_repo_error(error: ProjectsRepoError, not_found_message: &str) -> ApiObject<Value> {
    match error {
        ProjectsRepoError::NotFound => (
            StatusCode::NOT_FOUND,
            into_json(ErrorResponse {
                ok: false,
                error: String::from(not_found_message),
            }),
        ),
        ProjectsRepoError::Validation(message) => (
            StatusCode::BAD_REQUEST,
            into_json(ErrorResponse {
                ok: false,
                error: message,
            }),
        ),
        ProjectsRepoError::Sqlite(source) => internal_error(format!("database error: {source}")),
    }
}

fn internal_error(message: String) -> ApiObject<Value> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        into_json(ErrorResponse {
            ok: false,
            error: message,
        }),
    )
}

fn into_json(payload: impl Serialize) -> Json<Value> {
    Json(serde_json::to_value(payload).expect("api payload should serialize"))
}
