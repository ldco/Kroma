use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{
    ImportProjectBootstrapInput, ProjectBootstrapExport, ProjectBootstrapImportResult,
};

use super::handler_utils::{internal_error, into_json, map_repo_error, ApiObject};

#[derive(Debug, Clone, Serialize)]
struct BootstrapPromptResponse {
    ok: bool,
    bootstrap: ProjectBootstrapExport,
}

#[derive(Debug, Clone, Serialize)]
struct BootstrapImportResponse {
    ok: bool,
    bootstrap_import: ProjectBootstrapImportResult,
}

pub async fn get_bootstrap_prompt_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.export_project_bootstrap_prompt(slug.as_str()))
            .await;

    match result {
        Ok(Ok(bootstrap)) => (
            StatusCode::OK,
            into_json(BootstrapPromptResponse {
                ok: true,
                bootstrap,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("bootstrap prompt task failed: {join_error}")),
    }
}

pub async fn import_bootstrap_settings_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<ImportProjectBootstrapInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.import_project_bootstrap(slug.as_str(), payload))
            .await;

    match result {
        Ok(Ok(bootstrap_import)) => (
            StatusCode::OK,
            into_json(BootstrapImportResponse {
                ok: true,
                bootstrap_import,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("bootstrap import task failed: {join_error}")),
    }
}
