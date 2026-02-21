use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{
    CreateStyleGuideInput, ProjectsRepoError, StyleGuideSummary, UpdateStyleGuideInput,
};

type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Deserialize)]
pub struct SlugStyleGuidePath {
    pub slug: String,
    #[serde(rename = "styleGuideId")]
    pub style_guide_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListStyleGuidesResponse {
    ok: bool,
    count: usize,
    style_guides: Vec<StyleGuideSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct StyleGuideResponse {
    ok: bool,
    style_guide: StyleGuideSummary,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteStyleGuideResponse {
    ok: bool,
    deleted: bool,
    id: String,
}

pub async fn list_style_guides_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || store.list_style_guides(slug.as_str())).await;

    match result {
        Ok(Ok(style_guides)) => (
            StatusCode::OK,
            into_json(ListStyleGuidesResponse {
                ok: true,
                count: style_guides.len(),
                style_guides,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("style guide listing task failed: {join_error}")),
    }
}

pub async fn create_style_guide_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateStyleGuideInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.create_style_guide(slug.as_str(), payload)).await;

    match result {
        Ok(Ok(style_guide)) => (
            StatusCode::OK,
            into_json(StyleGuideResponse {
                ok: true,
                style_guide,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("style guide create task failed: {join_error}")),
    }
}

pub async fn get_style_guide_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugStyleGuidePath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_style_guide_detail(path.slug.as_str(), path.style_guide_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(style_guide)) => (
            StatusCode::OK,
            into_json(StyleGuideResponse {
                ok: true,
                style_guide,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Style guide not found"),
        Err(join_error) => internal_error(format!("style guide detail task failed: {join_error}")),
    }
}

pub async fn update_style_guide_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugStyleGuidePath>,
    Json(payload): Json<UpdateStyleGuideInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_style_guide(path.slug.as_str(), path.style_guide_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(style_guide)) => (
            StatusCode::OK,
            into_json(StyleGuideResponse {
                ok: true,
                style_guide,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Style guide not found"),
        Err(join_error) => internal_error(format!("style guide update task failed: {join_error}")),
    }
}

pub async fn delete_style_guide_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugStyleGuidePath>,
) -> ApiObject<Value> {
    let style_guide_id = path.style_guide_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_style_guide(path.slug.as_str(), path.style_guide_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeleteStyleGuideResponse {
                ok: true,
                deleted: true,
                id: style_guide_id,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Style guide not found"),
        Err(join_error) => internal_error(format!("style guide delete task failed: {join_error}")),
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
