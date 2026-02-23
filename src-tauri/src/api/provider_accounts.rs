use axum::extract::{Path, State};
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
    ProviderAccountSummary, UpdateProviderAccountInput, UpsertProviderAccountInput,
};

#[derive(Debug, Clone, Deserialize)]
pub struct SlugProviderPath {
    pub slug: String,
    #[serde(rename = "providerCode")]
    pub provider_code: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteProviderAccountResponse {
    ok: bool,
    deleted: bool,
    provider_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_id: Option<String>,
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
    Extension(actor): Extension<AuthPrincipal>,
    Path(slug): Path<String>,
    Json(payload): Json<UpsertProviderAccountInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let slug_for_store = slug.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.upsert_provider_account(slug_for_store.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(provider_account)) => {
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                slug.as_str(),
                "provider_account.upsert",
                json!({"provider_code": provider_account.provider_code}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => {
                    return internal_error(format!("provider account audit failed: {message}"))
                }
            };
            (
                StatusCode::OK,
                into_json(ProviderAccountResponse {
                    ok: true,
                    provider_account,
                    audit_id,
                }),
            )
        }
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
                audit_id: None,
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
    Extension(actor): Extension<AuthPrincipal>,
    Path(path): Path<SlugProviderPath>,
    Json(payload): Json<UpdateProviderAccountInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let audit_slug = path.slug.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_provider_account(path.slug.as_str(), path.provider_code.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(provider_account)) => {
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                audit_slug.as_str(),
                "provider_account.update",
                json!({"provider_code": provider_account.provider_code}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => {
                    return internal_error(format!("provider account audit failed: {message}"))
                }
            };
            (
                StatusCode::OK,
                into_json(ProviderAccountResponse {
                    ok: true,
                    provider_account,
                    audit_id,
                }),
            )
        }
        Ok(Err(error)) => map_repo_error(error, "Provider account not found"),
        Err(join_error) => {
            internal_error(format!("provider account update task failed: {join_error}"))
        }
    }
}

pub async fn delete_provider_account_handler(
    State(state): State<AppState>,
    Extension(actor): Extension<AuthPrincipal>,
    Path(path): Path<SlugProviderPath>,
) -> ApiObject<Value> {
    let provider_code = path.provider_code.clone();
    let store = state.projects_store.clone();
    let audit_slug = path.slug.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_provider_account(path.slug.as_str(), path.provider_code.as_str())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                audit_slug.as_str(),
                "provider_account.delete",
                json!({"provider_code": provider_code.clone()}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => {
                    return internal_error(format!("provider account audit failed: {message}"))
                }
            };
            (
                StatusCode::OK,
                into_json(DeleteProviderAccountResponse {
                    ok: true,
                    deleted: true,
                    provider_code,
                    audit_id,
                }),
            )
        }
        Ok(Err(error)) => map_repo_error(error, "Provider account not found"),
        Err(join_error) => {
            internal_error(format!("provider account delete task failed: {join_error}"))
        }
    }
}
