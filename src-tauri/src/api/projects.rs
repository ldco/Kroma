use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::audit::write_project_audit_event;
use crate::api::auth::AuthPrincipal;
use crate::api::server::AppState;

use super::handler_utils::{internal_error, into_json, map_repo_error, ApiObject};
use crate::db::projects::{
    ProjectCounts, ProjectInfo, ProjectStoragePayload, ProjectStorageProject, ProjectSummary,
    StorageConfig, UpdateStorageLocalInput, UpdateStorageS3Input, UpsertProjectInput,
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListProjectsQuery {
    pub username: Option<String>,
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
struct ProjectStorageResponse {
    ok: bool,
    project: ProjectStorageProject,
    storage: StorageConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_id: Option<String>,
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
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
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
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("project detail task failed: {join_error}")),
    }
}

pub async fn upsert_project_handler(
    State(state): State<AppState>,
    Extension(actor): Extension<AuthPrincipal>,
    Json(payload): Json<UpsertProjectInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || store.upsert_project(payload)).await;

    match result {
        Ok(Ok(project_storage)) => {
            let project_slug = project_storage.project.slug.clone();
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                project_slug.as_str(),
                "project.upsert",
                json!({"project_id": project_storage.project.id, "slug": project_slug}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => return internal_error(format!("project audit failed: {message}")),
            };
            (
                StatusCode::OK,
                into_json(storage_response(project_storage, None, audit_id)),
            )
        }
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("project upsert task failed: {join_error}")),
    }
}

pub async fn get_project_storage_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.get_project_storage(slug.as_str())).await;

    match result {
        Ok(Ok(payload)) => (
            StatusCode::OK,
            into_json(storage_response(payload, None, None)),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("project storage task failed: {join_error}")),
    }
}

pub async fn update_project_storage_local_handler(
    State(state): State<AppState>,
    Extension(actor): Extension<AuthPrincipal>,
    Path(slug): Path<String>,
    Json(payload): Json<UpdateStorageLocalInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let slug_for_store = slug.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_project_storage_local(slug_for_store.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(updated)) => {
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                slug.as_str(),
                "storage.local.update",
                json!({"updated": "local"}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => return internal_error(format!("storage audit failed: {message}")),
            };
            (
                StatusCode::OK,
                into_json(storage_response(
                    updated,
                    Some(String::from("local")),
                    audit_id,
                )),
            )
        }
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => {
            internal_error(format!("storage local update task failed: {join_error}"))
        }
    }
}

pub async fn update_project_storage_s3_handler(
    State(state): State<AppState>,
    Extension(actor): Extension<AuthPrincipal>,
    Path(slug): Path<String>,
    Json(payload): Json<UpdateStorageS3Input>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let slug_for_store = slug.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_project_storage_s3(slug_for_store.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(updated)) => {
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                slug.as_str(),
                "storage.s3.update",
                json!({"updated": "s3"}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => return internal_error(format!("storage audit failed: {message}")),
            };
            (
                StatusCode::OK,
                into_json(storage_response(
                    updated,
                    Some(String::from("s3")),
                    audit_id,
                )),
            )
        }
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("storage s3 update task failed: {join_error}")),
    }
}

fn storage_response(
    payload: ProjectStoragePayload,
    updated: Option<String>,
    audit_id: Option<String>,
) -> ProjectStorageResponse {
    ProjectStorageResponse {
        ok: true,
        project: payload.project,
        storage: payload.storage,
        updated,
        audit_id,
    }
}
