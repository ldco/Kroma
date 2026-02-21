use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{ProjectExportSummary, ProjectsRepoError};

type ApiObject<T> = (StatusCode, Json<T>);

const DEFAULT_LIMIT: i64 = 500;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListExportsQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlugExportPath {
    pub slug: String,
    #[serde(rename = "exportId")]
    pub export_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListExportsResponse {
    ok: bool,
    count: usize,
    exports: Vec<ProjectExportSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ExportDetailResponse {
    ok: bool,
    export: ProjectExportSummary,
}

pub async fn list_exports_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListExportsQuery>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    let result =
        tokio::task::spawn_blocking(move || store.list_project_exports(slug.as_str(), limit)).await;

    match result {
        Ok(Ok(exports)) => (
            StatusCode::OK,
            into_json(ListExportsResponse {
                ok: true,
                count: exports.len(),
                exports,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("export listing task failed: {join_error}")),
    }
}

pub async fn get_export_detail_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugExportPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_project_export_detail(path.slug.as_str(), path.export_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(export)) => (
            StatusCode::OK,
            into_json(ExportDetailResponse { ok: true, export }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project export not found"),
        Err(join_error) => internal_error(format!("export detail task failed: {join_error}")),
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
