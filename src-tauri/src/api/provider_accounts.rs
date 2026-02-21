use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{
    ProjectsRepoError, ProviderAccountSummary, UpdateProviderAccountInput,
    UpsertProviderAccountInput,
};

type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Deserialize)]
pub struct SlugProviderPath {
    pub slug: String,
    #[serde(rename = "providerCode")]
    pub provider_code: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListProviderAccountsResponse {
    ok: bool,
    count: usize,
    provider_accounts: Vec<ProviderAccountSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderAccountResponse {
    ok: bool,
    provider_account: ProviderAccountSummary,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteProviderAccountResponse {
    ok: bool,
    deleted: bool,
    provider_code: String,
}

pub async fn list_provider_accounts_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.list_provider_accounts(slug.as_str())).await;

    match result {
        Ok(Ok(provider_accounts)) => (
            StatusCode::OK,
            into_json(ListProviderAccountsResponse {
                ok: true,
                count: provider_accounts.len(),
                provider_accounts,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!(
            "provider account listing task failed: {join_error}"
        )),
    }
}

pub async fn upsert_provider_account_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<UpsertProviderAccountInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.upsert_provider_account(slug.as_str(), payload))
            .await;

    match result {
        Ok(Ok(provider_account)) => (
            StatusCode::OK,
            into_json(ProviderAccountResponse {
                ok: true,
                provider_account,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => {
            internal_error(format!("provider account upsert task failed: {join_error}"))
        }
    }
}

pub async fn get_provider_account_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugProviderPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_provider_account_detail(path.slug.as_str(), path.provider_code.as_str())
    })
    .await;

    match result {
        Ok(Ok(provider_account)) => (
            StatusCode::OK,
            into_json(ProviderAccountResponse {
                ok: true,
                provider_account,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Provider account not found"),
        Err(join_error) => {
            internal_error(format!("provider account detail task failed: {join_error}"))
        }
    }
}

pub async fn update_provider_account_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugProviderPath>,
    Json(payload): Json<UpdateProviderAccountInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_provider_account(path.slug.as_str(), path.provider_code.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(provider_account)) => (
            StatusCode::OK,
            into_json(ProviderAccountResponse {
                ok: true,
                provider_account,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Provider account not found"),
        Err(join_error) => {
            internal_error(format!("provider account update task failed: {join_error}"))
        }
    }
}

pub async fn delete_provider_account_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugProviderPath>,
) -> ApiObject<Value> {
    let provider_code = path.provider_code.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_provider_account(path.slug.as_str(), path.provider_code.as_str())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeleteProviderAccountResponse {
                ok: true,
                deleted: true,
                provider_code,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Provider account not found"),
        Err(join_error) => {
            internal_error(format!("provider account delete task failed: {join_error}"))
        }
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
