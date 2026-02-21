use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{AssetSummary, ProjectsRepoError, RunJobSummary, RunSummary};

type ApiObject<T> = (StatusCode, Json<T>);

const DEFAULT_RUNS_LIMIT: i64 = 200;
const DEFAULT_ASSETS_LIMIT: i64 = 500;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListRunsQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListAssetsQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlugRunPath {
    pub slug: String,
    #[serde(rename = "runId")]
    pub run_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlugAssetPath {
    pub slug: String,
    #[serde(rename = "assetId")]
    pub asset_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListRunsResponse {
    ok: bool,
    count: usize,
    runs: Vec<RunSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct RunDetailResponse {
    ok: bool,
    run: RunSummary,
    jobs: Vec<RunJobSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct RunJobsResponse {
    ok: bool,
    run_id: String,
    count: usize,
    jobs: Vec<RunJobSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ListAssetsResponse {
    ok: bool,
    count: usize,
    assets: Vec<AssetSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct AssetDetailResponse {
    ok: bool,
    asset: AssetSummary,
}

pub async fn list_runs_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListRunsQuery>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let limit = query.limit.unwrap_or(DEFAULT_RUNS_LIMIT);
    let result = tokio::task::spawn_blocking(move || store.list_runs(slug.as_str(), limit)).await;

    match result {
        Ok(Ok(runs)) => (
            StatusCode::OK,
            into_json(ListRunsResponse {
                ok: true,
                count: runs.len(),
                runs,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("run listing task failed: {join_error}")),
    }
}

pub async fn get_run_detail_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugRunPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_run_detail(path.slug.as_str(), path.run_id.as_str())
    })
    .await;

    match result {
        Ok(Ok((run, jobs))) => (
            StatusCode::OK,
            into_json(RunDetailResponse {
                ok: true,
                run,
                jobs,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Run not found"),
        Err(join_error) => internal_error(format!("run detail task failed: {join_error}")),
    }
}

pub async fn list_run_jobs_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugRunPath>,
) -> ApiObject<Value> {
    let run_id = path.run_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.list_run_jobs(path.slug.as_str(), path.run_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(jobs)) => (
            StatusCode::OK,
            into_json(RunJobsResponse {
                ok: true,
                run_id,
                count: jobs.len(),
                jobs,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Run not found"),
        Err(join_error) => internal_error(format!("run jobs task failed: {join_error}")),
    }
}

pub async fn list_assets_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListAssetsQuery>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let limit = query.limit.unwrap_or(DEFAULT_ASSETS_LIMIT);
    let result = tokio::task::spawn_blocking(move || store.list_assets(slug.as_str(), limit)).await;

    match result {
        Ok(Ok(assets)) => (
            StatusCode::OK,
            into_json(ListAssetsResponse {
                ok: true,
                count: assets.len(),
                assets,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("asset listing task failed: {join_error}")),
    }
}

pub async fn get_asset_detail_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugAssetPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_asset_detail(path.slug.as_str(), path.asset_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(asset)) => (
            StatusCode::OK,
            into_json(AssetDetailResponse { ok: true, asset }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Asset not found"),
        Err(join_error) => internal_error(format!("asset detail task failed: {join_error}")),
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
