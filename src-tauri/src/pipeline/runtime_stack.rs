use std::sync::Arc;

use crate::db::projects::ProjectsStore;
use crate::pipeline::backend_ops::{
    default_backend_ops_with_native_ingest, default_script_backend_ops, SharedPipelineBackendOps,
};
use crate::pipeline::post_run::PipelinePostRunService;
use crate::pipeline::runtime::{
    default_app_root_from_manifest_dir, PipelineOrchestrator, PipelineRunRequest,
    PipelineRuntimeError, RustDryRunPipelineOrchestrator, RustPostRunPipelineOrchestrator,
    RustRunModePipelineOrchestrator, SharedPipelineOrchestrator,
};
use crate::pipeline::tool_adapters::{default_native_tool_adapters, SharedPipelineToolAdapterOps};

#[derive(Debug, Default, Clone)]
struct RustOnlyUnsupportedPipelineOrchestrator;

impl PipelineOrchestrator for RustOnlyUnsupportedPipelineOrchestrator {
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<crate::pipeline::runtime::PipelineRunResult, PipelineRuntimeError> {
        Err(PipelineRuntimeError::PlanningPreflight(format!(
            "Rust-only pipeline runtime does not support this {} request shape yet. Provide preflight-supported inputs (manifest, jobs-file, scene_refs, or input path).",
            request.mode.as_str()
        )))
    }
}

pub fn default_pipeline_orchestrator_with_rust_post_run() -> RustPostRunPipelineOrchestrator {
    let backend_ops: SharedPipelineBackendOps = Arc::new(default_script_backend_ops());
    default_pipeline_orchestrator_with_rust_post_run_backend_ops(backend_ops)
}

pub fn default_pipeline_orchestrator_with_native_post_run(
    projects_store: Arc<ProjectsStore>,
) -> RustPostRunPipelineOrchestrator {
    let backend_ops: SharedPipelineBackendOps =
        Arc::new(default_backend_ops_with_native_ingest(projects_store));
    default_pipeline_orchestrator_with_rust_post_run_backend_ops(backend_ops)
}

pub fn default_pipeline_orchestrator_with_rust_post_run_backend_ops(
    backend_ops: SharedPipelineBackendOps,
) -> RustPostRunPipelineOrchestrator {
    let app_root = default_app_root_from_manifest_dir();
    let rust_only_inner: SharedPipelineOrchestrator =
        Arc::new(RustOnlyUnsupportedPipelineOrchestrator);
    let dry_inner: SharedPipelineOrchestrator = Arc::new(RustDryRunPipelineOrchestrator::new(
        rust_only_inner,
        app_root.clone(),
    ));
    let tool_adapters: SharedPipelineToolAdapterOps = Arc::new(default_native_tool_adapters());
    let inner: SharedPipelineOrchestrator = Arc::new(RustRunModePipelineOrchestrator::new(
        dry_inner,
        tool_adapters,
        app_root.clone(),
    ));
    let post_run = PipelinePostRunService::new(backend_ops);
    RustPostRunPipelineOrchestrator::new(inner, post_run)
}
