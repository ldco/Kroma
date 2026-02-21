use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;

use super::handler_utils::{internal_error, into_json, map_repo_error, ApiObject};
use crate::db::projects::{CostEventSummary, QualityReportSummary};

const DEFAULT_LIMIT: i64 = 500;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListAnalyticsQuery {
    pub limit: Option<i64>,
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
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
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
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("cost event listing task failed: {join_error}")),
    }
}
