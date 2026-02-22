use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::server::AppState;
use crate::pipeline::runtime::PipelineRuntimeError;
use crate::pipeline::trigger::{
    PipelineTriggerError, TriggerMode, TriggerPipelineInput, TriggerRunParams, TriggerStage,
    TriggerTime, TriggerWeather,
};

use super::handler_utils::{internal_error, into_json, map_repo_error, ApiObject};
use crate::db::projects::{AssetSummary, RunJobSummary, RunSummary};

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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TriggerRunInput {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub confirm_spend: Option<bool>,
    #[serde(default)]
    pub project_root: Option<String>,
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default)]
    pub scene_refs: Option<Vec<String>>,
    #[serde(default)]
    pub style_refs: Option<Vec<String>>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub weather: Option<String>,
    #[serde(default)]
    pub candidates: Option<i64>,
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

#[derive(Debug, Clone, Serialize)]
struct TriggerRunResponse {
    ok: bool,
    pipeline_trigger: TriggerRunResultPayload,
}

#[derive(Debug, Clone, Serialize)]
struct TriggerRunResultPayload {
    mode: String,
    status_code: i32,
    stdout: String,
    stderr: String,
    adapter: &'static str,
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

pub async fn trigger_run_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<TriggerRunInput>,
) -> ApiObject<Value> {
    let mode = match parse_trigger_mode(payload.mode.as_deref()) {
        Ok(mode) => mode,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                into_json(json!({"ok": false, "error": message})),
            );
        }
    };

    let params = match build_trigger_params(&payload) {
        Ok(params) => params,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                into_json(json!({"ok": false, "error": message})),
            );
        }
    };

    let store = state.projects_store.clone();
    let slug_for_lookup = slug.clone();
    let project_check =
        tokio::task::spawn_blocking(move || store.get_project_detail(slug_for_lookup.as_str()))
            .await;
    match project_check {
        Ok(Ok(_project)) => {}
        Ok(Err(error)) => return map_repo_error(error, "Project not found"),
        Err(join_error) => {
            return internal_error(format!(
                "pipeline trigger project lookup task failed: {join_error}"
            ));
        }
    }

    let trigger = state.pipeline_trigger.clone();
    let confirm_spend = payload.confirm_spend.unwrap_or(false);
    let mode_label = trigger_mode_label(mode);
    let result = tokio::task::spawn_blocking(move || {
        trigger.trigger(TriggerPipelineInput {
            project_slug: slug,
            mode,
            confirm_spend,
            params,
        })
    })
    .await;

    match result {
        Ok(Ok(output)) => (
            StatusCode::OK,
            into_json(TriggerRunResponse {
                ok: true,
                pipeline_trigger: TriggerRunResultPayload {
                    mode: mode_label.to_string(),
                    status_code: output.status_code,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    adapter: "script_fallback",
                },
            }),
        ),
        Ok(Err(error)) => map_pipeline_trigger_error(error),
        Err(join_error) => internal_error(format!("pipeline trigger task failed: {join_error}")),
    }
}

fn build_trigger_params(payload: &TriggerRunInput) -> Result<TriggerRunParams, String> {
    let project_root = payload
        .project_root
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned);

    let input_path = payload
        .input
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned);
    let scene_refs = normalize_string_list(payload.scene_refs.as_ref(), "scene_refs")?;

    if input_path.is_some() && scene_refs.is_some() {
        return Err(String::from("Provide only one of: input, scene_refs"));
    }
    if input_path.is_none() && scene_refs.is_none() {
        return Err(String::from("Provide one of: input, scene_refs"));
    }

    let style_refs =
        normalize_string_list(payload.style_refs.as_ref(), "style_refs")?.unwrap_or_default();

    let stage = if let Some(stage) = payload
        .stage
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(match stage.to_ascii_lowercase().as_str() {
            "style" => TriggerStage::Style,
            "time" => TriggerStage::Time,
            "weather" => TriggerStage::Weather,
            _ => {
                return Err(String::from(
                    "Field 'stage' must be one of: style, time, weather",
                ))
            }
        })
    } else {
        None
    };

    let time = if let Some(time) = payload
        .time
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(match time.to_ascii_lowercase().as_str() {
            "day" => TriggerTime::Day,
            "night" => TriggerTime::Night,
            _ => return Err(String::from("Field 'time' must be one of: day, night")),
        })
    } else {
        None
    };

    let weather = if let Some(weather) = payload
        .weather
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(match weather.to_ascii_lowercase().as_str() {
            "clear" => TriggerWeather::Clear,
            "rain" => TriggerWeather::Rain,
            _ => return Err(String::from("Field 'weather' must be one of: clear, rain")),
        })
    } else {
        None
    };

    let mut candidates_u8 = None;
    if let Some(candidates) = payload.candidates {
        if !(1..=6).contains(&candidates) {
            return Err(String::from("Field 'candidates' must be between 1 and 6"));
        }
        candidates_u8 = Some(candidates as u8);
    }

    Ok(TriggerRunParams {
        project_root,
        input: input_path,
        scene_refs,
        style_refs,
        stage,
        time,
        weather,
        candidates: candidates_u8,
    })
}

fn normalize_string_list(
    values: Option<&Vec<String>>,
    field_name: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(values) = values else {
        return Ok(None);
    };
    if values.is_empty() {
        return Err(format!("Field '{field_name}' must not be empty"));
    }

    let mut out = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(format!(
                "Field '{field_name}' must not contain empty values"
            ));
        }
        out.push(trimmed.to_string());
    }
    Ok(Some(out))
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

fn parse_trigger_mode(raw: Option<&str>) -> Result<TriggerMode, String> {
    match raw.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(TriggerMode::Dry),
        Some(value) if value.eq_ignore_ascii_case("dry") => Ok(TriggerMode::Dry),
        Some(value) if value.eq_ignore_ascii_case("run") => Ok(TriggerMode::Run),
        Some(_) => Err(String::from("Field 'mode' must be one of: dry, run")),
    }
}

fn trigger_mode_label(mode: TriggerMode) -> &'static str {
    match mode {
        TriggerMode::Dry => "dry",
        TriggerMode::Run => "run",
    }
}

fn map_pipeline_trigger_error(error: PipelineTriggerError) -> ApiObject<Value> {
    match error {
        PipelineTriggerError::MissingSpendConfirmation => (
            StatusCode::BAD_REQUEST,
            into_json(json!({
                "ok": false,
                "error": "Run mode requires explicit spend confirmation"
            })),
        ),
        PipelineTriggerError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            into_json(json!({
                "ok": false,
                "error": message
            })),
        ),
        PipelineTriggerError::Runtime(PipelineRuntimeError::InvalidProjectSlug) => (
            StatusCode::BAD_REQUEST,
            into_json(json!({
                "ok": false,
                "error": "Invalid project slug for pipeline run"
            })),
        ),
        PipelineTriggerError::Runtime(PipelineRuntimeError::CommandFailed { stderr, .. }) => (
            StatusCode::BAD_REQUEST,
            into_json(json!({
                "ok": false,
                "error": summarize_pipeline_command_failure(stderr.as_str())
            })),
        ),
        PipelineTriggerError::Runtime(PipelineRuntimeError::ScriptNotFound(path)) => {
            internal_error(format!(
                "pipeline script fallback missing: {}",
                path.display()
            ))
        }
        PipelineTriggerError::Runtime(PipelineRuntimeError::Io(source)) => {
            internal_error(format!("pipeline command execution error: {source}"))
        }
    }
}

fn summarize_pipeline_command_failure(stderr: &str) -> String {
    let first_line = stderr
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("Pipeline command failed");
    if first_line == "Pipeline command failed" {
        String::from(first_line)
    } else {
        format!("Pipeline command failed: {first_line}")
    }
}
