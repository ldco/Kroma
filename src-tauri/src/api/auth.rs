use std::fmt;

use axum::extract::{Path, Request, State};
use axum::http::{header, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Extension;
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
    BootstrapFirstToken,
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
            Self::BootstrapFirstToken => None,
            Self::ApiToken { user_id, .. } => Some(user_id.as_str()),
        }
    }
}

impl fmt::Display for AuthPrincipal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DevBypass => write!(f, "dev_bypass"),
            Self::BootstrapFirstToken => write!(f, "bootstrap_first_token"),
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
    Extension(actor): Extension<AuthPrincipal>,
    Json(payload): Json<CreateAuthTokenRequest>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let bootstrap_first_token_only = matches!(actor, AuthPrincipal::BootstrapFirstToken);
    let result = tokio::task::spawn_blocking(move || {
        let input = CreateApiTokenInput {
            label: payload.label,
            project_slug: payload.project_slug,
            expires_at: payload.expires_at,
        };
        if bootstrap_first_token_only {
            store.create_first_api_token_local(input)
        } else {
            store.create_api_token_local(input)
        }
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
    let project_slug = extract_project_slug(path.as_str());
    if state.auth_dev_bypass {
        request.extensions_mut().insert(AuthPrincipal::DevBypass);
        return next.run(request).await;
    }

    let is_safe_method = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let auth_path = path.starts_with("/auth/");
    let should_require_auth = auth_path || !is_safe_method || project_slug.is_some();
    if !should_require_auth {
        request.extensions_mut().insert(AuthPrincipal::DevBypass);
        return next.run(request).await;
    }

    if is_bootstrap_auth_token_create_route(request.method(), path.as_str())
        && state.auth_bootstrap_allow_unauth_token_create
        && extract_bearer_token(&request).is_none()
    {
        let store = state.projects_store.clone();
        let has_active_tokens =
            tokio::task::spawn_blocking(move || store.has_active_api_tokens()).await;
        match has_active_tokens {
            Ok(Ok(false)) => {
                request
                    .extensions_mut()
                    .insert(AuthPrincipal::BootstrapFirstToken);
                return next.run(request).await;
            }
            Ok(Ok(true)) => {}
            Ok(Err(_)) | Err(_) => {
                return unauthorized("Token bootstrap check failed");
            }
        }
    }

    let Some(bearer) = extract_bearer_token(&request) else {
        return unauthorized("Missing Authorization: Bearer token");
    };

    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.validate_api_token(bearer.as_str())).await;
    match result {
        Ok(Ok(Some(ctx))) => {
            if let Some(slug) = project_slug.as_deref() {
                let store = state.projects_store.clone();
                let slug = slug.to_string();
                let user_id = ctx.user_id.clone();
                let project_id = ctx.project_id.clone();
                let access = tokio::task::spawn_blocking(move || {
                    store.authorize_project_slug_access(
                        slug.as_str(),
                        user_id.as_str(),
                        project_id.as_deref(),
                    )
                })
                .await;

                match access {
                    Ok(Ok(true)) => {}
                    Ok(Ok(false)) => {
                        return forbidden("Token is not authorized for this project");
                    }
                    Ok(Err(_)) | Err(_) => {
                        return forbidden("Project access check failed");
                    }
                }
            }

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

fn is_bootstrap_auth_token_create_route(method: &Method, path: &str) -> bool {
    *method == Method::POST && path == "/auth/token"
}

fn extract_project_slug(path: &str) -> Option<String> {
    let mut segments = path.trim_matches('/').split('/');
    if segments.next()? != "api" {
        return None;
    }
    if segments.next()? != "projects" {
        return None;
    }
    let slug = segments.next()?;
    if slug.trim().is_empty() {
        return None;
    }
    Some(slug.to_string())
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

fn forbidden(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"ok": false, "error": message})),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use axum::http::Method;

    use super::{extract_project_slug, is_bootstrap_auth_token_create_route};

    #[test]
    fn extract_project_slug_from_project_routes() {
        assert_eq!(
            extract_project_slug("/api/projects/demo/runs").as_deref(),
            Some("demo")
        );
        assert_eq!(
            extract_project_slug("/api/projects/demo").as_deref(),
            Some("demo")
        );
    }

    #[test]
    fn extract_project_slug_ignores_non_project_routes() {
        assert!(extract_project_slug("/api/projects").is_none());
        assert!(extract_project_slug("/api/other/demo").is_none());
        assert!(extract_project_slug("/health").is_none());
    }

    #[test]
    fn bootstrap_route_detection_requires_post_auth_token_path() {
        assert!(is_bootstrap_auth_token_create_route(
            &Method::POST,
            "/auth/token"
        ));
        assert!(!is_bootstrap_auth_token_create_route(
            &Method::GET,
            "/auth/token"
        ));
        assert!(!is_bootstrap_auth_token_create_route(
            &Method::POST,
            "/auth/tokens"
        ));
    }
}
