use std::path::PathBuf;

use crate::pipeline::dry_run_execution::execute_rust_dry_run_with_preflight;
use crate::pipeline::planning_preflight::build_rust_planning_preflight_summary;
use crate::pipeline::post_run::PipelinePostRunService;
use crate::pipeline::post_run_execution::run_post_run_finalize_best_effort;
use crate::pipeline::request_settings::effective_pipeline_request_with_layered_settings;
use crate::pipeline::run_mode_execution::execute_rust_run_mode_with_tool_adapters;
use crate::pipeline::runtime::{
    default_app_root_from_manifest_dir, validate_project_slug, PipelineOrchestrator,
    PipelineRunMode, PipelineRunRequest, PipelineRunResult, PipelineRuntimeError,
    SharedPipelineOrchestrator,
};
use crate::pipeline::tool_adapters::SharedPipelineToolAdapterOps;

#[derive(Clone)]
pub struct RustPostRunPipelineOrchestrator {
    inner: SharedPipelineOrchestrator,
    post_run: PipelinePostRunService,
    app_root: PathBuf,
}

impl RustPostRunPipelineOrchestrator {
    pub fn new(inner: SharedPipelineOrchestrator, post_run: PipelinePostRunService) -> Self {
        Self {
            inner,
            post_run,
            app_root: default_app_root_from_manifest_dir(),
        }
    }

    pub fn with_app_root(mut self, app_root: PathBuf) -> Self {
        self.app_root = app_root;
        self
    }
}

#[derive(Clone)]
pub struct RustDryRunPipelineOrchestrator {
    inner: SharedPipelineOrchestrator,
    app_root: PathBuf,
}

impl RustDryRunPipelineOrchestrator {
    pub fn new(inner: SharedPipelineOrchestrator, app_root: PathBuf) -> Self {
        Self { inner, app_root }
    }
}

#[derive(Clone)]
pub struct RustRunModePipelineOrchestrator {
    inner: SharedPipelineOrchestrator,
    tools: SharedPipelineToolAdapterOps,
    app_root: PathBuf,
}

impl RustRunModePipelineOrchestrator {
    pub fn new(
        inner: SharedPipelineOrchestrator,
        tools: SharedPipelineToolAdapterOps,
        app_root: PathBuf,
    ) -> Self {
        Self {
            inner,
            tools,
            app_root,
        }
    }
}

impl PipelineOrchestrator for RustPostRunPipelineOrchestrator {
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<PipelineRunResult, PipelineRuntimeError> {
        match self.inner.execute(request) {
            Ok(mut result) => {
                run_post_run_finalize_best_effort(
                    self.app_root.as_path(),
                    &self.post_run,
                    request,
                    result.stdout.as_str(),
                    &mut result.stderr,
                );
                Ok(result)
            }
            Err(PipelineRuntimeError::CommandFailed {
                program,
                status_code,
                stdout,
                mut stderr,
            }) => {
                run_post_run_finalize_best_effort(
                    self.app_root.as_path(),
                    &self.post_run,
                    request,
                    stdout.as_str(),
                    &mut stderr,
                );
                Err(PipelineRuntimeError::CommandFailed {
                    program,
                    status_code,
                    stdout,
                    stderr,
                })
            }
            Err(other) => Err(other),
        }
    }
}

impl PipelineOrchestrator for RustDryRunPipelineOrchestrator {
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<PipelineRunResult, PipelineRuntimeError> {
        if !matches!(request.mode, PipelineRunMode::Dry) {
            return self.inner.execute(request);
        }
        validate_project_slug(request.project_slug.as_str())?;
        let request =
            effective_pipeline_request_with_layered_settings(self.app_root.as_path(), request)?;
        let Some(planned) =
            build_rust_planning_preflight_summary(self.app_root.as_path(), &request)?
        else {
            return self.inner.execute(&request);
        };
        execute_rust_dry_run_with_preflight(self.app_root.as_path(), &request, &planned)
    }
}

impl PipelineOrchestrator for RustRunModePipelineOrchestrator {
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<PipelineRunResult, PipelineRuntimeError> {
        if !matches!(request.mode, PipelineRunMode::Run) {
            return self.inner.execute(request);
        }
        validate_project_slug(request.project_slug.as_str())?;
        let request =
            effective_pipeline_request_with_layered_settings(self.app_root.as_path(), request)?;
        let Some(planned) =
            build_rust_planning_preflight_summary(self.app_root.as_path(), &request)?
        else {
            return self.inner.execute(&request);
        };
        if !request.confirm_spend {
            return Err(PipelineRuntimeError::PlanningPreflight(String::from(
                "Spending is locked. Add --confirm-spend for paid calls.",
            )));
        }

        execute_rust_run_mode_with_tool_adapters(
            self.app_root.as_path(),
            self.tools.as_ref(),
            &request,
            &planned,
        )
    }
}
