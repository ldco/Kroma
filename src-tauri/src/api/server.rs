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
    pub projects_store: Arc<ProjectsStore>,
    pub pipeline_trigger: PipelineTriggerService,
}

impl AppState {
    pub fn new(route_count: usize, projects_store: Arc<ProjectsStore>) -> Self {
        let backend_ops: SharedPipelineBackendOps = Arc::new(
            default_backend_ops_with_native_ingest(projects_store.clone()),
        );
        let orchestrator: SharedPipelineOrchestrator =
            Arc::new(default_pipeline_orchestrator_with_rust_post_run_backend_ops(backend_ops));
        Self::new_with_pipeline_trigger(
            route_count,
            projects_store,
            PipelineTriggerService::new(orchestrator),
        )
    }

    pub fn new_with_pipeline_trigger(
        route_count: usize,
        projects_store: Arc<ProjectsStore>,
        pipeline_trigger: PipelineTriggerService,
    ) -> Self {
        Self {
            service_name: "kroma-backend-core",
            service_version: env!("CARGO_PKG_VERSION"),
            started_unix_ms: now_unix_ms(),
            route_count,
            projects_store,
            pipeline_trigger,
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

pub fn build_router_with_projects_store_and_pipeline_trigger(
    projects_store: Arc<ProjectsStore>,
    pipeline_trigger: PipelineTriggerService,
) -> Router {
    let catalog = route_catalog();
    let state =
        AppState::new_with_pipeline_trigger(catalog.len(), projects_store, pipeline_trigger);
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
