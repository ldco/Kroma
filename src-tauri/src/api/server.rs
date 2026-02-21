use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, head, options, patch, post, put, MethodRouter};
use axum::{Extension, Json, Router};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::api::error::ErrorKind;
use crate::api::response::{failure, ApiJson};
use crate::api::routes::{route_catalog, RouteDefinition};
use crate::contract::HttpMethod;
use crate::db::projects::ProjectsStore;

#[derive(Clone)]
pub struct AppState {
    pub service_name: &'static str,
    pub service_version: &'static str,
    pub started_unix_ms: u128,
    pub route_count: usize,
    pub projects_store: Arc<ProjectsStore>,
}

impl AppState {
    pub fn new(route_count: usize, projects_store: Arc<ProjectsStore>) -> Self {
        Self {
            service_name: "kroma-backend-core",
            service_version: env!("CARGO_PKG_VERSION"),
            started_unix_ms: now_unix_ms(),
            route_count,
            projects_store,
        }
    }
}

pub fn build_router() -> Router {
    let repo_root = default_repo_root();
    let db_path = resolve_db_path(repo_root.as_path());
    let projects_store = Arc::new(ProjectsStore::new(db_path, repo_root));
    projects_store
        .initialize()
        .expect("projects store should initialize schema");
    build_router_with_projects_store(projects_store)
}

pub fn build_router_with_projects_store(projects_store: Arc<ProjectsStore>) -> Router {
    let catalog = route_catalog();
    let state = AppState::new(catalog.len(), projects_store);
    build_router_with_catalog(catalog, state)
}

fn build_router_with_catalog(catalog: Vec<RouteDefinition>, state: AppState) -> Router {
    let mut router = Router::new().route("/health", get(health_handler));

    for route in catalog {
        if route.spec.method == HttpMethod::Get && route.spec.path == "/health" {
            continue;
        }

        let axum_path = openapi_path_to_router(route.spec.path.as_str());
        let method_router = method_router_for(route);
        router = router.route(axum_path.as_str(), method_router);
    }

    router.layer(TraceLayer::new_for_http()).with_state(state)
}

pub async fn serve(addr: SocketAddr) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let app = build_router();
    info!(bind = %addr, "starting kroma-backend-core HTTP surface");
    axum::serve(listener, app).await
}

fn method_router_for(route: RouteDefinition) -> MethodRouter<AppState> {
    let method = route.spec.method;
    let path = route.spec.path.clone();

    match (method, path.as_str()) {
        (HttpMethod::Get, "/api/projects") => get(crate::api::projects::list_projects_handler),
        (HttpMethod::Post, "/api/projects") => post(crate::api::projects::upsert_project_handler),
        (HttpMethod::Get, "/api/projects/{slug}") => get(crate::api::projects::get_project_handler),
        _ => {
            let extension = Extension(route);
            match method {
                HttpMethod::Get => get(contract_stub_handler).layer(extension),
                HttpMethod::Post => post(contract_stub_handler).layer(extension),
                HttpMethod::Put => put(contract_stub_handler).layer(extension),
                HttpMethod::Delete => delete(contract_stub_handler).layer(extension),
                HttpMethod::Patch => patch(contract_stub_handler).layer(extension),
                HttpMethod::Options => options(contract_stub_handler).layer(extension),
                HttpMethod::Head => head(contract_stub_handler).layer(extension),
            }
        }
    }
}

fn default_repo_root() -> PathBuf {
    let fallback = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    fallback.canonicalize().unwrap_or(fallback)
}

fn resolve_db_path(repo_root: &Path) -> PathBuf {
    let raw =
        std::env::var("KROMA_BACKEND_DB").unwrap_or_else(|_| String::from("var/backend/app.db"));
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        candidate
    } else {
        repo_root.join(candidate)
    }
}

async fn health_handler(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "status": "ok",
            "service": state.service_name,
            "version": state.service_version,
            "started_unix_ms": state.started_unix_ms,
            "route_count": state.route_count,
        })),
    )
}

async fn contract_stub_handler(
    State(_state): State<AppState>,
    Extension(route): Extension<RouteDefinition>,
    OriginalUri(uri): OriginalUri,
) -> ApiJson<serde_json::Value> {
    let details = json!({
        "domain": format!("{:?}", route.domain),
        "handler_id": route.handler_id,
        "contract": {
            "method": route.spec.method.as_str(),
            "path": route.spec.path,
        },
        "request_uri": uri.to_string(),
    });

    failure(
        StatusCode::NOT_IMPLEMENTED,
        ErrorKind::Unknown,
        "not_implemented",
        "Endpoint contract is mounted but implementation is not attached yet.",
        Some(details),
    )
}

fn openapi_path_to_router(path: &str) -> String {
    path.to_string()
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_path_is_compatible_with_router_syntax() {
        assert_eq!(
            openapi_path_to_router("/api/projects/{slug}/runs/{runId}"),
            "/api/projects/{slug}/runs/{runId}"
        );
        assert_eq!(openapi_path_to_router("/health"), "/health");
    }
}
