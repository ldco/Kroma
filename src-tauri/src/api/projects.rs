use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{
    ProjectCounts, ProjectInfo, ProjectStorageProject, ProjectSummary, ProjectsRepoError,
    StorageConfig, UpsertProjectInput,
};

type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListProjectsQuery {
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListProjectsResponse {
    ok: bool,
    count: usize,
    projects: Vec<ProjectSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ProjectDetailResponse {
    ok: bool,
    project: ProjectInfo,
    counts: ProjectCounts,
    storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize)]
struct ProjectUpsertResponse {
    ok: bool,
    project: ProjectStorageProject,
    storage: StorageConfig,
}

pub async fn list_projects_handler(
    State(state): State<AppState>,
    Query(query): Query<ListProjectsQuery>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let username = query.username.clone();
    let result =
        tokio::task::spawn_blocking(move || store.list_projects(username.as_deref())).await;

    match result {
        Ok(Ok(projects)) => (
            StatusCode::OK,
            into_json(ListProjectsResponse {
                ok: true,
                count: projects.len(),
                projects,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error),
        Err(join_error) => internal_error(format!("project listing task failed: {join_error}")),
    }
}

pub async fn get_project_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || store.get_project_detail(slug.as_str())).await;

    match result {
        Ok(Ok(detail)) => (
            StatusCode::OK,
            into_json(ProjectDetailResponse {
                ok: true,
                project: detail.project,
                counts: detail.counts,
                storage: detail.storage,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error),
        Err(join_error) => internal_error(format!("project detail task failed: {join_error}")),
    }
}

pub async fn upsert_project_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpsertProjectInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || store.upsert_project(payload)).await;

    match result {
        Ok(Ok(project_storage)) => (
            StatusCode::OK,
            into_json(ProjectUpsertResponse {
                ok: true,
                project: project_storage.project,
                storage: project_storage.storage,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error),
        Err(join_error) => internal_error(format!("project upsert task failed: {join_error}")),
    }
}

fn map_repo_error(error: ProjectsRepoError) -> ApiObject<Value> {
    match error {
        ProjectsRepoError::NotFound => (
            StatusCode::NOT_FOUND,
            into_json(ErrorResponse {
                ok: false,
                error: String::from("Project not found"),
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
