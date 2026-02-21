use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{ProjectsRepoError, SecretSummary, UpsertSecretInput};

type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Deserialize)]
pub struct SlugSecretPath {
    pub slug: String,
    #[serde(rename = "providerCode")]
    pub provider_code: String,
    #[serde(rename = "secretName")]
    pub secret_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListSecretsResponse {
    ok: bool,
    count: usize,
    secrets: Vec<SecretSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct SecretResponse {
    ok: bool,
    secret: SecretSummary,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteSecretResponse {
    ok: bool,
    deleted: bool,
}

pub async fn list_secrets_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.list_project_secrets(slug.as_str())).await;

    match result {
        Ok(Ok(secrets)) => (
            StatusCode::OK,
            into_json(ListSecretsResponse {
                ok: true,
                count: secrets.len(),
                secrets,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("secret listing task failed: {join_error}")),
    }
}

pub async fn upsert_secret_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<UpsertSecretInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.upsert_project_secret(slug.as_str(), payload))
            .await;

    match result {
        Ok(Ok(secret)) => (
            StatusCode::OK,
            into_json(SecretResponse { ok: true, secret }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("secret upsert task failed: {join_error}")),
    }
}

pub async fn delete_secret_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugSecretPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_project_secret(
            path.slug.as_str(),
            path.provider_code.as_str(),
            path.secret_name.as_str(),
        )
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeleteSecretResponse {
                ok: true,
                deleted: true,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Secret not found"),
        Err(join_error) => internal_error(format!("secret delete task failed: {join_error}")),
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
