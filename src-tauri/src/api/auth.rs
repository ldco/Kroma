use std::fmt;

use axum::extract::{Path, Request, State};
use axum::http::{header, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::server::AppState;
use crate::db::projects::{
    ApiTokenAuthContext, ApiTokenSummary, CreateApiTokenInput, CreateApiTokenResult,
};

use super::handler_utils::{internal_error, into_json, map_repo_error, ApiObject};

#[derive(Debug, Clone)]
pub enum AuthPrincipal {
    DevBypass,
    ApiToken {
        token_id: String,
        user_id: String,
        project_id: Option<String>,
    },
}

impl AuthPrincipal {
    pub fn actor_user_id(&self) -> Option<&str> {
        match self {
            Self::DevBypass => None,
            Self::ApiToken { user_id, .. } => Some(user_id.as_str()),
        }
    }
}

impl fmt::Display for AuthPrincipal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DevBypass => write!(f, "dev_bypass"),
            Self::ApiToken { token_id, .. } => write!(f, "api_token:{token_id}"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthTokenPath {
    #[serde(rename = "tokenId")]
    pub token_id: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CreateAuthTokenRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub project_slug: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateAuthTokenResponse {
    ok: bool,
    auth_token: CreateApiTokenResult,
}

#[derive(Debug, Clone, Serialize)]
struct ListAuthTokensResponse {
    ok: bool,
    count: usize,
    auth_tokens: Vec<ApiTokenSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteAuthTokenResponse {
    ok: bool,
    deleted: bool,
    token_id: String,
}

pub async fn create_auth_token_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateAuthTokenRequest>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.create_api_token_local(CreateApiTokenInput {
            label: payload.label,
            project_slug: payload.project_slug,
            expires_at: payload.expires_at,
        })
    })
    .await;

    match result {
        Ok(Ok(auth_token)) => (
            StatusCode::OK,
            into_json(CreateAuthTokenResponse {
                ok: true,
                auth_token,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("auth token create task failed: {join_error}")),
    }
}

pub async fn list_auth_tokens_handler(State(state): State<AppState>) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || store.list_api_tokens()).await;
    match result {
        Ok(Ok(auth_tokens)) => (
            StatusCode::OK,
            into_json(ListAuthTokensResponse {
                ok: true,
                count: auth_tokens.len(),
                auth_tokens,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Auth token not found"),
        Err(join_error) => internal_error(format!("auth token list task failed: {join_error}")),
    }
}

pub async fn revoke_auth_token_handler(
    State(state): State<AppState>,
    Path(path): Path<AuthTokenPath>,
) -> ApiObject<Value> {
    let token_id = path.token_id.clone();
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.revoke_api_token(token_id.as_str())).await;
    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeleteAuthTokenResponse {
                ok: true,
                deleted: true,
                token_id: path.token_id,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Auth token not found"),
        Err(join_error) => internal_error(format!("auth token revoke task failed: {join_error}")),
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    if state.auth_dev_bypass {
        request.extensions_mut().insert(AuthPrincipal::DevBypass);
        return next.run(request).await;
    }

    let is_safe_method = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let auth_path = path.starts_with("/auth/");
    let should_require_auth = auth_path || !is_safe_method;
    if !should_require_auth {
        request.extensions_mut().insert(AuthPrincipal::DevBypass);
        return next.run(request).await;
    }

    let Some(bearer) = extract_bearer_token(&request) else {
        return unauthorized("Missing Authorization: Bearer token");
    };

    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.validate_api_token(bearer.as_str())).await;
    match result {
        Ok(Ok(Some(ctx))) => {
            request
                .extensions_mut()
                .insert(auth_principal_from_token_ctx(ctx));
            next.run(request).await
        }
        Ok(Ok(None)) => unauthorized("Invalid or expired token"),
        Ok(Err(_)) | Err(_) => unauthorized("Token validation failed"),
    }
}

fn extract_bearer_token(request: &Request) -> Option<String> {
    let header_value = request.headers().get(header::AUTHORIZATION)?;
    let value = header_value.to_str().ok()?.trim();
    let mut parts = value.splitn(2, ' ');
    let scheme = parts.next()?.trim();
    let token = parts.next()?.trim();
    if !scheme.eq_ignore_ascii_case("Bearer") || token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

fn auth_principal_from_token_ctx(ctx: ApiTokenAuthContext) -> AuthPrincipal {
    AuthPrincipal::ApiToken {
        token_id: ctx.token_id,
        user_id: ctx.user_id,
        project_id: ctx.project_id,
    }
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"ok": false, "error": message})),
    )
        .into_response()
}
