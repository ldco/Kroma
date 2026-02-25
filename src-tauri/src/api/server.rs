use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::middleware;
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
use crate::db::{resolve_backend_config, DatabaseBackendConfig};
use crate::pipeline::backend_ops::{
    default_backend_ops_with_native_ingest, SharedPipelineBackendOps,
};
use crate::pipeline::runtime::{
    default_pipeline_orchestrator_with_rust_post_run_backend_ops, SharedPipelineOrchestrator,
};
use crate::pipeline::trigger::PipelineTriggerService;

#[derive(Clone)]
pub struct AppState {
    pub service_name: &'static str,
    pub service_version: &'static str,
    pub started_unix_ms: u128,
    pub route_count: usize,
    pub auth_dev_bypass: bool,
    pub projects_store: Arc<ProjectsStore>,
    pub pipeline_trigger: PipelineTriggerService,
}

impl AppState {
    pub fn new(route_count: usize, projects_store: Arc<ProjectsStore>) -> Self {
        Self::new_with_pipeline_trigger_and_auth_dev_bypass(
            route_count,
            projects_store.clone(),
            default_pipeline_trigger(projects_store),
            auth_dev_bypass_enabled(),
        )
    }

    pub fn new_with_pipeline_trigger(
        route_count: usize,
        projects_store: Arc<ProjectsStore>,
        pipeline_trigger: PipelineTriggerService,
    ) -> Self {
        Self::new_with_pipeline_trigger_and_auth_dev_bypass(
            route_count,
            projects_store,
            pipeline_trigger,
            auth_dev_bypass_enabled(),
        )
    }

    fn new_with_pipeline_trigger_and_auth_dev_bypass(
        route_count: usize,
        projects_store: Arc<ProjectsStore>,
        pipeline_trigger: PipelineTriggerService,
        auth_dev_bypass: bool,
    ) -> Self {
        Self {
            service_name: "kroma-backend-core",
            service_version: env!("CARGO_PKG_VERSION"),
            started_unix_ms: now_unix_ms(),
            route_count,
            auth_dev_bypass,
            projects_store,
            pipeline_trigger,
        }
    }
}

pub fn build_router() -> Router {
    let repo_root = default_repo_root();
    let projects_store = match resolve_backend_config(repo_root.as_path()) {
        DatabaseBackendConfig::Sqlite(sqlite) => {
            Arc::new(ProjectsStore::new(sqlite.app_db_path, repo_root))
        }
        DatabaseBackendConfig::Postgres(pg) => {
            panic!(
                "KROMA_BACKEND_DB_URL is set ({}), but PostgreSQL backend wiring is not implemented yet. Unset KROMA_BACKEND_DB_URL to use SQLite.",
                pg.database_url
            );
        }
    };
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

pub fn build_router_with_projects_store_dev_bypass(projects_store: Arc<ProjectsStore>) -> Router {
    let catalog = route_catalog();
    let state = AppState::new_with_pipeline_trigger_and_auth_dev_bypass(
        catalog.len(),
        projects_store.clone(),
        default_pipeline_trigger(projects_store),
        true,
    );
    build_router_with_catalog(catalog, state)
}

pub fn build_router_with_projects_store_and_pipeline_trigger(
    projects_store: Arc<ProjectsStore>,
    pipeline_trigger: PipelineTriggerService,
) -> Router {
    let catalog = route_catalog();
    let state =
        AppState::new_with_pipeline_trigger(catalog.len(), projects_store, pipeline_trigger);
    build_router_with_catalog(catalog, state)
}

pub fn build_router_with_projects_store_and_pipeline_trigger_dev_bypass(
    projects_store: Arc<ProjectsStore>,
    pipeline_trigger: PipelineTriggerService,
) -> Router {
    let catalog = route_catalog();
    let state = AppState::new_with_pipeline_trigger_and_auth_dev_bypass(
        catalog.len(),
        projects_store,
        pipeline_trigger,
        true,
    );
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

    router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::api::auth::auth_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
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
        (HttpMethod::Post, "/auth/token") => post(crate::api::auth::create_auth_token_handler),
        (HttpMethod::Get, "/auth/tokens") => get(crate::api::auth::list_auth_tokens_handler),
        (HttpMethod::Delete, "/auth/tokens/{tokenId}") => {
            delete(crate::api::auth::revoke_auth_token_handler)
        }
        (HttpMethod::Get, "/api/projects") => get(crate::api::projects::list_projects_handler),
        (HttpMethod::Post, "/api/projects") => post(crate::api::projects::upsert_project_handler),
        (HttpMethod::Get, "/api/projects/{slug}") => get(crate::api::projects::get_project_handler),
        (HttpMethod::Get, "/api/projects/{slug}/bootstrap-prompt") => {
            get(crate::api::bootstrap::get_bootstrap_prompt_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/bootstrap-import") => {
            post(crate::api::bootstrap::import_bootstrap_settings_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/storage") => {
            get(crate::api::projects::get_project_storage_handler)
        }
        (HttpMethod::Put, "/api/projects/{slug}/storage/local") => {
            put(crate::api::projects::update_project_storage_local_handler)
        }
        (HttpMethod::Put, "/api/projects/{slug}/storage/s3") => {
            put(crate::api::projects::update_project_storage_s3_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/runs") => {
            get(crate::api::runs_assets::list_runs_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/runs/trigger") => {
            post(crate::api::runs_assets::trigger_run_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/runs/validate-config") => {
            post(crate::api::runs_assets::validate_pipeline_config_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/runs/{runId}") => {
            get(crate::api::runs_assets::get_run_detail_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/runs/{runId}/jobs") => {
            get(crate::api::runs_assets::list_run_jobs_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/assets") => {
            get(crate::api::runs_assets::list_assets_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/assets/{assetId}") => {
            get(crate::api::runs_assets::get_asset_detail_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/asset-links") => {
            get(crate::api::asset_links::list_asset_links_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/asset-links") => {
            post(crate::api::asset_links::create_asset_link_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/asset-links/{linkId}") => {
            get(crate::api::asset_links::get_asset_link_handler)
        }
        (HttpMethod::Put, "/api/projects/{slug}/asset-links/{linkId}") => {
            put(crate::api::asset_links::update_asset_link_handler)
        }
        (HttpMethod::Delete, "/api/projects/{slug}/asset-links/{linkId}") => {
            delete(crate::api::asset_links::delete_asset_link_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/quality-reports") => {
            get(crate::api::analytics::list_quality_reports_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/cost-events") => {
            get(crate::api::analytics::list_cost_events_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/exports") => {
            get(crate::api::exports::list_exports_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/exports/{exportId}") => {
            get(crate::api::exports::get_export_detail_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/prompt-templates") => {
            get(crate::api::prompt_templates::list_prompt_templates_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/prompt-templates") => {
            post(crate::api::prompt_templates::create_prompt_template_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/prompt-templates/{templateId}") => {
            get(crate::api::prompt_templates::get_prompt_template_handler)
        }
        (HttpMethod::Put, "/api/projects/{slug}/prompt-templates/{templateId}") => {
            put(crate::api::prompt_templates::update_prompt_template_handler)
        }
        (HttpMethod::Delete, "/api/projects/{slug}/prompt-templates/{templateId}") => {
            delete(crate::api::prompt_templates::delete_prompt_template_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/provider-accounts") => {
            get(crate::api::provider_accounts::list_provider_accounts_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/provider-accounts") => {
            post(crate::api::provider_accounts::upsert_provider_account_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/provider-accounts/{providerCode}") => {
            get(crate::api::provider_accounts::get_provider_account_handler)
        }
        (HttpMethod::Put, "/api/projects/{slug}/provider-accounts/{providerCode}") => {
            put(crate::api::provider_accounts::update_provider_account_handler)
        }
        (HttpMethod::Delete, "/api/projects/{slug}/provider-accounts/{providerCode}") => {
            delete(crate::api::provider_accounts::delete_provider_account_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/style-guides") => {
            get(crate::api::style_guides::list_style_guides_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/style-guides") => {
            post(crate::api::style_guides::create_style_guide_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/style-guides/{styleGuideId}") => {
            get(crate::api::style_guides::get_style_guide_handler)
        }
        (HttpMethod::Put, "/api/projects/{slug}/style-guides/{styleGuideId}") => {
            put(crate::api::style_guides::update_style_guide_handler)
        }
        (HttpMethod::Delete, "/api/projects/{slug}/style-guides/{styleGuideId}") => {
            delete(crate::api::style_guides::delete_style_guide_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/characters") => {
            get(crate::api::characters::list_characters_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/characters") => {
            post(crate::api::characters::create_character_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/characters/{characterId}") => {
            get(crate::api::characters::get_character_handler)
        }
        (HttpMethod::Put, "/api/projects/{slug}/characters/{characterId}") => {
            put(crate::api::characters::update_character_handler)
        }
        (HttpMethod::Delete, "/api/projects/{slug}/characters/{characterId}") => {
            delete(crate::api::characters::delete_character_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/reference-sets") => {
            get(crate::api::reference_sets::list_reference_sets_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/reference-sets") => {
            post(crate::api::reference_sets::create_reference_set_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/reference-sets/{referenceSetId}") => {
            get(crate::api::reference_sets::get_reference_set_handler)
        }
        (HttpMethod::Put, "/api/projects/{slug}/reference-sets/{referenceSetId}") => {
            put(crate::api::reference_sets::update_reference_set_handler)
        }
        (HttpMethod::Delete, "/api/projects/{slug}/reference-sets/{referenceSetId}") => {
            delete(crate::api::reference_sets::delete_reference_set_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/reference-sets/{referenceSetId}/items") => {
            get(crate::api::reference_sets::list_reference_set_items_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/reference-sets/{referenceSetId}/items") => {
            post(crate::api::reference_sets::create_reference_set_item_handler)
        }
        (
            HttpMethod::Get,
            "/api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}",
        ) => get(crate::api::reference_sets::get_reference_set_item_handler),
        (
            HttpMethod::Put,
            "/api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}",
        ) => put(crate::api::reference_sets::update_reference_set_item_handler),
        (
            HttpMethod::Delete,
            "/api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}",
        ) => delete(crate::api::reference_sets::delete_reference_set_item_handler),
        (HttpMethod::Get, "/api/projects/{slug}/chat/sessions") => {
            get(crate::api::chat::list_chat_sessions_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/chat/sessions") => {
            post(crate::api::chat::create_chat_session_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/chat/sessions/{sessionId}") => {
            get(crate::api::chat::get_chat_session_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/chat/sessions/{sessionId}/messages") => {
            get(crate::api::chat::list_chat_messages_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/chat/sessions/{sessionId}/messages") => {
            post(crate::api::chat::create_chat_message_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/agent/instructions") => {
            get(crate::api::agent_instructions::list_agent_instructions_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/agent/instructions") => {
            post(crate::api::agent_instructions::create_agent_instruction_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/agent/instructions/{instructionId}") => {
            get(crate::api::agent_instructions::get_agent_instruction_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/agent/instructions/{instructionId}/events") => {
            get(crate::api::agent_instructions::list_agent_instruction_events_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/agent/instructions/{instructionId}/confirm") => {
            post(crate::api::agent_instructions::confirm_agent_instruction_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/agent/instructions/{instructionId}/cancel") => {
            post(crate::api::agent_instructions::cancel_agent_instruction_handler)
        }
        (HttpMethod::Get, "/api/projects/{slug}/secrets") => {
            get(crate::api::secrets::list_secrets_handler)
        }
        (HttpMethod::Post, "/api/projects/{slug}/secrets") => {
            post(crate::api::secrets::upsert_secret_handler)
        }
        (HttpMethod::Delete, "/api/projects/{slug}/secrets/{providerCode}/{secretName}") => {
            delete(crate::api::secrets::delete_secret_handler)
        }
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

fn auth_dev_bypass_enabled() -> bool {
    std::env::var("KROMA_API_AUTH_DEV_BYPASS")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn default_pipeline_trigger(projects_store: Arc<ProjectsStore>) -> PipelineTriggerService {
    let backend_ops: SharedPipelineBackendOps =
        Arc::new(default_backend_ops_with_native_ingest(projects_store));
    let orchestrator: SharedPipelineOrchestrator =
        Arc::new(default_pipeline_orchestrator_with_rust_post_run_backend_ops(backend_ops));
    PipelineTriggerService::new(orchestrator)
}

fn default_repo_root() -> PathBuf {
    let fallback = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    fallback.canonicalize().unwrap_or(fallback)
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
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_auth_bypass_env(value: Option<&str>, run: impl FnOnce()) {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let key = "KROMA_API_AUTH_DEV_BYPASS";
        let original = std::env::var(key).ok();
        match value {
            Some(v) => unsafe { std::env::set_var(key, v) },
            None => unsafe { std::env::remove_var(key) },
        }
        run();
        if let Some(v) = original {
            unsafe { std::env::set_var(key, v) };
        } else {
            unsafe { std::env::remove_var(key) };
        }
    }

    #[test]
    fn openapi_path_is_compatible_with_router_syntax() {
        assert_eq!(
            openapi_path_to_router("/api/projects/{slug}/runs/{runId}"),
            "/api/projects/{slug}/runs/{runId}"
        );
        assert_eq!(openapi_path_to_router("/health"), "/health");
    }

    #[test]
    fn auth_dev_bypass_defaults_off_when_env_missing() {
        with_auth_bypass_env(None, || {
            assert!(!auth_dev_bypass_enabled());
        });
    }

    #[test]
    fn auth_dev_bypass_parses_truthy_values() {
        with_auth_bypass_env(Some("true"), || {
            assert!(auth_dev_bypass_enabled());
        });
        with_auth_bypass_env(Some("1"), || {
            assert!(auth_dev_bypass_enabled());
        });
    }
}
