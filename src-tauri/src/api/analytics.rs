use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{CostEventSummary, ProjectsRepoError, QualityReportSummary};

type ApiObject<T> = (StatusCode, Json<T>);

const DEFAULT_LIMIT: i64 = 500;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListAnalyticsQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListQualityReportsResponse {
    ok: bool,
    count: usize,
    quality_reports: Vec<QualityReportSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ListCostEventsResponse {
    ok: bool,
    count: usize,
    cost_events: Vec<CostEventSummary>,
}

pub async fn list_quality_reports_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListAnalyticsQuery>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    let result =
        tokio::task::spawn_blocking(move || store.list_quality_reports(slug.as_str(), limit)).await;

    match result {
        Ok(Ok(quality_reports)) => (
            StatusCode::OK,
            into_json(ListQualityReportsResponse {
                ok: true,
                count: quality_reports.len(),
                quality_reports,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error),
        Err(join_error) => {
            internal_error(format!("quality report listing task failed: {join_error}"))
        }
    }
}

pub async fn list_cost_events_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListAnalyticsQuery>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    let result =
        tokio::task::spawn_blocking(move || store.list_cost_events(slug.as_str(), limit)).await;

    match result {
        Ok(Ok(cost_events)) => (
            StatusCode::OK,
            into_json(ListCostEventsResponse {
                ok: true,
                count: cost_events.len(),
                cost_events,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error),
        Err(join_error) => internal_error(format!("cost event listing task failed: {join_error}")),
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
