use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{
    CreateVoiceSttInput, CreateVoiceTtsInput, ProjectsRepoError, VoiceRequestSummary,
};

type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Deserialize)]
pub struct SlugRequestPath {
    pub slug: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct VoiceRequestResponse {
    ok: bool,
    request: VoiceRequestSummary,
}

pub async fn voice_stt_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateVoiceSttInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.create_voice_stt_request(slug.as_str(), payload))
            .await;

    match result {
        Ok(Ok(request)) => (
            StatusCode::OK,
            into_json(VoiceRequestResponse { ok: true, request }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("voice stt task failed: {join_error}")),
    }
}

pub async fn voice_tts_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateVoiceTtsInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.create_voice_tts_request(slug.as_str(), payload))
            .await;

    match result {
        Ok(Ok(request)) => (
            StatusCode::OK,
            into_json(VoiceRequestResponse { ok: true, request }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("voice tts task failed: {join_error}")),
    }
}

pub async fn get_voice_request_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugRequestPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_voice_request_detail(path.slug.as_str(), path.request_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(request)) => (
            StatusCode::OK,
            into_json(VoiceRequestResponse { ok: true, request }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Voice request not found"),
        Err(join_error) => {
            internal_error(format!("voice request detail task failed: {join_error}"))
        }
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
