use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{json, Value};
use thiserror::Error;
use uuid::Uuid;

use crate::db::projects::ProjectsStore;
use crate::pipeline::backend_ops::{
    default_backend_ops_with_native_ingest, default_script_backend_ops, SharedPipelineBackendOps,
};
use crate::pipeline::execution::{
    build_planned_run_log_record, build_run_log_output_guard_record, ensure_generation_mode_dirs,
    execution_project_dirs, finalize_job_from_candidates, plan_job_candidate_output_paths,
    summarize_output_guard_report, ExecutionCandidateJobOutputs, ExecutionCandidateJobResult,
    ExecutionCandidateRank, ExecutionCandidateResult, ExecutionCandidateStatus,
    ExecutionOutputGuardReport, ExecutionOutputGuardReportFile, ExecutionOutputGuardReportSummary,
    ExecutionPlannedJob, ExecutionPlannedOutputGuardRecord, ExecutionPlannedPostprocessRecord,
    ExecutionPlannedRunLogContext, ExecutionPostprocessPathConfig, ExecutionUpscalePathConfig,
};
use crate::pipeline::pathing::{
    list_image_files_recursive as list_image_files_recursive_shared,
    path_for_output as path_for_output_shared, resolve_under_root,
};
use crate::pipeline::planning::{
    build_generation_jobs, default_planning_manifest, load_planned_jobs_file,
    load_planning_manifest_file, PipelinePlanningOutputGuard, PlannedGenerationJob,
};
use crate::pipeline::post_run::{
    PipelinePostRunService, PostRunFinalizeParams, PostRunIngestParams, PostRunSyncS3Params,
};
use crate::pipeline::postprocess_planning::{
    load_postprocess_planning_config, resolve_planned_postprocess_record,
    PostprocessPlanningConfig, PostprocessPlanningOverrides,
};
use crate::pipeline::runlog::{
    format_summary_marker, write_pretty_json_with_newline, PipelineRunSummaryMarkerPayload,
};
use crate::pipeline::runlog_enrich::{build_planned_template, RunLogPlannedTemplateInput};
use crate::pipeline::runlog_patch::{
    normalize_script_run_log_job_finalization, normalize_script_run_log_job_finalizations_file,
    patch_script_run_log_planned_metadata_file,
};
use crate::pipeline::settings_layer::{
    load_app_pipeline_settings, load_project_pipeline_settings, merge_pipeline_settings_overlays,
    PipelineSettingsLayerPaths, PipelineSettingsOverlay,
};
use crate::pipeline::tool_adapters::{
    default_native_tool_adapters, ArchiveBadRequest, BackgroundRemovePassRequest, ColorPassRequest,
    GenerateOneImageRequest, PipelineToolAdapterOps, QaCheckRequest, SharedPipelineToolAdapterOps,
    ToolAdapterError, UpscalePassRequest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineRunMode {
    Dry,
    Run,
}

impl PipelineRunMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dry => "dry",
            Self::Run => "run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineInputSource {
    InputPath(String),
    SceneRefs(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStageFilter {
    Style,
    Time,
    Weather,
}

impl PipelineStageFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Style => "style",
            Self::Time => "time",
            Self::Weather => "weather",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineTimeFilter {
    Day,
    Night,
}

impl PipelineTimeFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Night => "night",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineWeatherFilter {
    Clear,
    Rain,
}

impl PipelineWeatherFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Rain => "rain",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineUpscaleBackend {
    Ncnn,
    Python,
}

impl PipelineUpscaleBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ncnn => "ncnn",
            Self::Python => "python",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PipelinePostprocessOptions {
    pub config_path: Option<String>,
    pub upscale: bool,
    pub upscale_backend: Option<PipelineUpscaleBackend>,
    pub color: bool,
    pub color_profile: Option<String>,
    pub bg_remove: bool,
    pub bg_remove_backends: Vec<String>,
    pub bg_refine_openai: Option<bool>,
    pub bg_refine_openai_required: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PipelineRunOptions {
    pub app_settings_path: Option<String>,
    pub project_settings_path: Option<String>,
    pub manifest_path: Option<String>,
    pub jobs_file: Option<String>,
    pub project_root: Option<String>,
    pub input_source: Option<PipelineInputSource>,
    pub style_refs: Vec<String>,
    pub stage: Option<PipelineStageFilter>,
    pub time: Option<PipelineTimeFilter>,
    pub weather: Option<PipelineWeatherFilter>,
    pub candidates: Option<u8>,
    pub postprocess: PipelinePostprocessOptions,
    pub backend_db_ingest: Option<bool>,
    pub storage_sync_s3: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineRunRequest {
    pub project_slug: String,
    pub mode: PipelineRunMode,
    pub confirm_spend: bool,
    pub options: PipelineRunOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineRunResult {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait PipelineCommandRunner: Send + Sync + 'static {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, PipelineRuntimeError>;
}

#[derive(Debug, Default, Clone)]
pub struct StdPipelineCommandRunner;

impl PipelineCommandRunner for StdPipelineCommandRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, PipelineRuntimeError> {
        let output = Command::new(spec.program.as_str())
            .args(spec.args.iter().map(String::as_str))
            .current_dir(spec.cwd.as_path())
            .output()
            .map_err(PipelineRuntimeError::Io)?;

        Ok(CommandOutput {
            status_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(output.stdout.as_slice()).to_string(),
            stderr: String::from_utf8_lossy(output.stderr.as_slice()).to_string(),
        })
    }
}

pub trait PipelineOrchestrator: Send + Sync + 'static {
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<PipelineRunResult, PipelineRuntimeError>;
}

pub type SharedPipelineOrchestrator = Arc<dyn PipelineOrchestrator>;

#[derive(Debug, Default, Clone)]
struct RustOnlyUnsupportedPipelineOrchestrator;

impl PipelineOrchestrator for RustOnlyUnsupportedPipelineOrchestrator {
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<PipelineRunResult, PipelineRuntimeError> {
        Err(PipelineRuntimeError::PlanningPreflight(format!(
            "Rust-only pipeline runtime does not support this {} request shape yet. Provide preflight-supported inputs (manifest, jobs-file, scene_refs, or input path).",
            request.mode.as_str()
        )))
    }
}

fn build_rust_planning_preflight_summary(
    app_root: &Path,
    request: &PipelineRunRequest,
) -> Result<Option<RustPlanningPreflightSummary>, PipelineRuntimeError> {
    let has_manifest = request.options.manifest_path.is_some();
    let has_jobs_file = request.options.jobs_file.is_some();
    let has_scene_refs = matches!(
        request.options.input_source,
        Some(PipelineInputSource::SceneRefs(_))
    );
    let has_input_path = matches!(
        request.options.input_source,
        Some(PipelineInputSource::InputPath(_))
    );
    if !has_manifest && !has_jobs_file && !has_scene_refs && !has_input_path {
        return Ok(None);
    }

    let mut manifest = if let Some(manifest_path_raw) = request.options.manifest_path.as_deref() {
        let manifest_path = resolve_under_app_root(app_root, manifest_path_raw);
        load_planning_manifest_file(manifest_path.as_path()).map_err(|error| {
            PipelineRuntimeError::PlanningPreflight(format!(
                "manifest parse failed ({}): {error}",
                manifest_path.display()
            ))
        })?
    } else {
        default_planning_manifest()
    };

    match request.options.input_source.as_ref() {
        Some(PipelineInputSource::SceneRefs(values)) => {
            manifest.scene_refs = values.clone();
        }
        Some(PipelineInputSource::InputPath(path)) => {
            let input_abs = resolve_under_app_root(app_root, path);
            if !input_abs.exists() {
                return Err(PipelineRuntimeError::PlanningPreflight(format!(
                    "input not found: {}",
                    path
                )));
            }
            manifest.scene_refs = list_image_files_recursive(input_abs.as_path())
                .map_err(PipelineRuntimeError::Io)?
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
        }
        None => {}
    }

    if !has_jobs_file && !request.options.style_refs.is_empty() {
        manifest.style_refs = request.options.style_refs.clone();
    }

    let stage = request.options.stage.unwrap_or(PipelineStageFilter::Style);
    let time = request.options.time.unwrap_or(PipelineTimeFilter::Day);
    let weather = request
        .options
        .weather
        .unwrap_or(PipelineWeatherFilter::Clear);
    let jobs = if let Some(jobs_file_raw) = request.options.jobs_file.as_deref() {
        let jobs_path = resolve_under_app_root(app_root, jobs_file_raw);
        load_planned_jobs_file(jobs_path.as_path())
            .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?
    } else {
        build_generation_jobs(&manifest, stage, time, weather).map_err(|error| {
            PipelineRuntimeError::PlanningPreflight(format!(
                "manifest planning preflight failed: {error}"
            ))
        })?
    };
    if (jobs.len() as u64) > manifest.safe_batch_limit {
        return Err(PipelineRuntimeError::PlanningPreflight(format!(
            "Batch exceeds safety limit ({}). Use --allow-large-batch to override.",
            manifest.safe_batch_limit
        )));
    }

    let postprocess_cfg = load_postprocess_planning_config(
        app_root,
        request.options.postprocess.config_path.as_deref(),
    )
    .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?;
    let planned_postprocess = resolve_planned_postprocess_record(
        &postprocess_cfg,
        &PostprocessPlanningOverrides {
            post_upscale: request.options.postprocess.upscale,
            post_color: request.options.postprocess.color,
            post_bg_remove: request.options.postprocess.bg_remove,
            upscale_backend: request
                .options
                .postprocess
                .upscale_backend
                .map(|v| v.as_str().to_string()),
            color_profile: request.options.postprocess.color_profile.clone(),
            bg_remove_backends: request.options.postprocess.bg_remove_backends.clone(),
            bg_refine_openai: request.options.postprocess.bg_refine_openai,
            bg_refine_openai_required: request.options.postprocess.bg_refine_openai_required,
        },
    )
    .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?;

    Ok(Some(RustPlanningPreflightSummary {
        job_ids: jobs.iter().map(|job| job.id.clone()).collect(),
        manifest_output_guard: manifest.output_guard.clone(),
        postprocess_path_config: execution_postprocess_path_config_from_planning(
            &postprocess_cfg,
            &planned_postprocess,
        ),
        planned_postprocess,
        manifest_candidate_count: manifest.generation.candidates,
        manifest_max_candidates: manifest.generation.max_candidates,
        jobs,
    }))
}

fn list_image_files_recursive(input_abs: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    list_image_files_recursive_shared(input_abs)
}

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

    fn build_script_request(&self, request: &PipelineRunRequest) -> PipelineRunRequest {
        let mut script_request = request.clone();
        // Rust owns backend ingest for the typed HTTP trigger path; prevent duplicate script ingest.
        script_request.options.backend_db_ingest = Some(false);
        // Keep S3 sync disabled until the Rust path owns sync policy/options end-to-end.
        script_request.options.storage_sync_s3 = Some(false);
        script_request
    }

    fn build_post_run_sync_s3_params(request: &PipelineRunRequest) -> Option<PostRunSyncS3Params> {
        if !matches!(request.mode, PipelineRunMode::Run) {
            return None;
        }
        if !request.options.storage_sync_s3.unwrap_or(false) {
            return None;
        }
        Some(PostRunSyncS3Params {
            project_slug: request.project_slug.clone(),
            dry_run: false,
            delete: false,
            allow_missing_local: false,
        })
    }

    fn run_post_run_finalize_best_effort(
        &self,
        request: &PipelineRunRequest,
        stdout: &str,
        stderr: &mut String,
    ) {
        let Some(summary) = parse_script_run_summary_from_stdout(stdout) else {
            append_stderr_line(
                stderr,
                "Rust post-run finalize skipped: missing summary marker or 'Run log:' line in pipeline stdout",
            );
            return;
        };
        if let Some(project_slug) = summary.project_slug.as_deref() {
            if project_slug != request.project_slug {
                append_stderr_line(
                    stderr,
                    format!(
                        "Rust post-run ingest warning: script stdout project '{}' does not match request '{}'",
                        project_slug, request.project_slug
                    ),
                );
            }
        }
        self.normalize_script_run_log_best_effort(summary.run_log_path.as_path(), stderr);
        self.enrich_script_run_log_planned_metadata_best_effort(
            request,
            summary.run_log_path.as_path(),
            stderr,
        );

        let finalize = self.post_run.finalize_run(PostRunFinalizeParams {
            ingest: PostRunIngestParams {
                run_log_path: summary.run_log_path,
                project_slug: request.project_slug.clone(),
                project_name: request.project_slug.clone(),
                create_project_if_missing: true,
                compute_hashes: false,
            },
            sync_s3: Self::build_post_run_sync_s3_params(request),
        });

        if let Err(error) = finalize {
            append_stderr_line(stderr, format!("Rust post-run finalize skipped: {error}"));
        }
    }

    fn normalize_script_run_log_best_effort(&self, run_log_path: &Path, stderr: &mut String) {
        if let Err(error) =
            normalize_script_run_log_job_finalizations_file(self.app_root.as_path(), run_log_path)
        {
            append_stderr_line(
                stderr,
                format!("Rust run-log normalization skipped: {error}"),
            );
        }
    }

    fn enrich_script_run_log_planned_metadata_best_effort(
        &self,
        request: &PipelineRunRequest,
        run_log_path: &Path,
        stderr: &mut String,
    ) {
        if let Err(error) = enrich_script_run_log_planned_metadata_file(
            self.app_root.as_path(),
            request,
            run_log_path,
        ) {
            append_stderr_line(
                stderr,
                format!("Rust planned-metadata run-log patch skipped: {error}"),
            );
        }
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
        match self.inner.execute(&self.build_script_request(request)) {
            Ok(mut result) => {
                self.run_post_run_finalize_best_effort(
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
                self.run_post_run_finalize_best_effort(request, stdout.as_str(), &mut stderr);
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

        let project_root_abs = request
            .options
            .project_root
            .as_deref()
            .map(|v| resolve_under_app_root(self.app_root.as_path(), v))
            .unwrap_or_else(|| {
                self.app_root
                    .join("var/projects")
                    .join(request.project_slug.as_str())
            });
        let project_dirs = execution_project_dirs(project_root_abs.as_path());
        ensure_generation_mode_dirs(&project_dirs).map_err(PipelineRuntimeError::Io)?;
        let run_log_path_abs = project_dirs
            .runs
            .join(format!("run_{}.json", make_run_log_stamp()));

        let stage = request.options.stage.unwrap_or(PipelineStageFilter::Style);
        let time = request.options.time.unwrap_or(PipelineTimeFilter::Day);
        let weather = request
            .options
            .weather
            .unwrap_or(PipelineWeatherFilter::Clear);
        let candidate_count = request
            .options
            .candidates
            .map(u64::from)
            .unwrap_or(planned.manifest_candidate_count);
        let project_root_display =
            path_for_output(self.app_root.as_path(), project_dirs.root.as_path());
        let run_log_display = path_for_output(self.app_root.as_path(), run_log_path_abs.as_path());
        let timestamp = iso_like_timestamp();

        let execution_jobs = planned
            .jobs
            .iter()
            .cloned()
            .map(ExecutionPlannedJob::from)
            .collect::<Vec<_>>();
        let run_meta = build_planned_run_log_record(
            ExecutionPlannedRunLogContext {
                timestamp,
                project_slug: request.project_slug.clone(),
                stage: stage.as_str().to_string(),
                time: time.as_str().to_string(),
                weather: weather.as_str().to_string(),
                project_root: project_root_display.clone(),
                resolved_from_backend: request.options.project_root.is_some(),
                candidate_count,
                max_candidate_count: planned.manifest_max_candidates,
                planned_postprocess: planned.planned_postprocess.clone(),
                planned_output_guard: map_manifest_output_guard_to_planned_record(
                    &planned.manifest_output_guard,
                ),
            },
            execution_jobs.as_slice(),
        );

        write_pretty_json_with_newline(run_log_path_abs.as_path(), &run_meta)
            .map_err(|e| PipelineRuntimeError::Io(std::io::Error::other(e.to_string())))?;
        let marker = format_summary_marker(&PipelineRunSummaryMarkerPayload {
            run_log_path: run_log_display.clone(),
            project_slug: request.project_slug.clone(),
            project_root: project_root_display.clone(),
            jobs: planned.job_count(),
            mode: String::from("dry"),
        })
        .map_err(|e| PipelineRuntimeError::Io(std::io::Error::other(e.to_string())))?;

        let stdout = [
            format!("Run log: {run_log_display}"),
            format!("Project: {}", request.project_slug),
            format!("Project root: {project_root_display}"),
            format!("Jobs: {} (dry/planned)", planned.job_count()),
            marker,
        ]
        .join("\n");

        Ok(PipelineRunResult {
            status_code: 0,
            stdout,
            stderr: String::new(),
        })
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

fn execute_rust_run_mode_with_tool_adapters(
    app_root: &Path,
    tools: &dyn PipelineToolAdapterOps,
    request: &PipelineRunRequest,
    planned: &RustPlanningPreflightSummary,
) -> Result<PipelineRunResult, PipelineRuntimeError> {
    let stage = request.options.stage.unwrap_or(PipelineStageFilter::Style);
    let time = request.options.time.unwrap_or(PipelineTimeFilter::Day);
    let weather = request
        .options
        .weather
        .unwrap_or(PipelineWeatherFilter::Clear);

    let candidate_count = request
        .options
        .candidates
        .map(u64::from)
        .unwrap_or(planned.manifest_candidate_count);
    if candidate_count < 1 {
        return Err(PipelineRuntimeError::PlanningPreflight(String::from(
            "Invalid candidate count: expected >= 1",
        )));
    }
    if candidate_count > planned.manifest_max_candidates {
        return Err(PipelineRuntimeError::PlanningPreflight(format!(
            "Candidate count {} exceeds limit {}.",
            candidate_count, planned.manifest_max_candidates
        )));
    }
    let candidate_count_u8 = u8::try_from(candidate_count).map_err(|_| {
        PipelineRuntimeError::PlanningPreflight(format!(
            "Candidate count {} exceeds Rust execution limit 255.",
            candidate_count
        ))
    })?;

    let project_root_abs = default_project_root_for_request(app_root, request);
    let project_dirs = execution_project_dirs(project_root_abs.as_path());
    ensure_generation_mode_dirs(&project_dirs).map_err(PipelineRuntimeError::Io)?;
    let run_log_path_abs = project_dirs
        .runs
        .join(format!("run_{}.json", make_run_log_stamp()));

    let project_root_display = path_for_output(app_root, project_dirs.root.as_path());
    let run_log_display = path_for_output(app_root, run_log_path_abs.as_path());
    let timestamp = iso_like_timestamp();
    let output_guard_cfg = &planned.manifest_output_guard;
    let planned_output_guard = map_manifest_output_guard_to_planned_record(output_guard_cfg);
    let mut failed_output_guard_jobs = 0_u64;
    let mut jobs_json = Vec::<Value>::with_capacity(planned.jobs.len());

    let mut model_name = String::new();
    let mut image_size = String::new();
    let mut image_quality = String::new();

    let project_root_arg = path_for_output(app_root, project_dirs.root.as_path());
    let manifest_path_for_qa = request.options.manifest_path.clone();

    for job in planned.jobs.iter() {
        let candidate_plans = plan_job_candidate_output_paths(
            &project_dirs,
            job.id.as_str(),
            candidate_count_u8,
            &planned.postprocess_path_config,
        )
        .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;

        let input_images_file_path = write_json_temp_file(
            app_root,
            "pipeline_input_images",
            &serde_json::to_value(&job.input_images)
                .map_err(|e| PipelineRuntimeError::PlannedJobsTempFile(e.to_string()))?,
        )?;
        let input_images_file_arg = path_for_output(app_root, input_images_file_path.as_path());

        let mut candidate_records = Vec::<Value>::with_capacity(candidate_plans.len());
        let mut execution_candidates =
            Vec::<ExecutionCandidateJobResult>::with_capacity(candidate_plans.len());

        for plan in candidate_plans {
            let generated_rel = path_for_output(app_root, plan.generated.as_path());
            let generate_resp = tools
                .generate_one(&GenerateOneImageRequest {
                    project_slug: request.project_slug.clone(),
                    project_root: Some(project_root_arg.clone()),
                    prompt: job.prompt.clone(),
                    input_images_file: input_images_file_arg.clone(),
                    output_path: generated_rel.clone(),
                    model: None,
                    size: None,
                    quality: None,
                })
                .map_err(tool_adapter_error_to_runtime)?;
            if model_name.is_empty() {
                model_name = generate_resp.model.clone();
                image_size = generate_resp.size.clone();
                image_quality = generate_resp.quality.clone();
            }

            let mut current_path = PathBuf::from(generate_resp.output.clone());
            let mut candidate_json = json!({
                "candidate_index": plan.candidate_index,
                "output": generate_resp.output,
                "status": "generated",
                "rank": {
                    "hard_failures": 0,
                    "soft_warnings": 0,
                    "avg_chroma_exceed": 0.0
                }
            });
            let mut bg_remove_output = None::<PathBuf>;
            let mut upscale_output = None::<PathBuf>;
            let mut color_output = None::<PathBuf>;

            if let Some(bg_remove_path) = plan.bg_remove.as_ref() {
                let bg_resp = tools
                    .bgremove(&BackgroundRemovePassRequest {
                        project_slug: request.project_slug.clone(),
                        project_root: Some(project_root_arg.clone()),
                        input_path: path_for_output(app_root, current_path.as_path()),
                        output_path: path_for_output(app_root, bg_remove_path.as_path()),
                        postprocess_config_path: request.options.postprocess.config_path.clone(),
                        backends: planned.planned_postprocess.bg_remove_backends.clone(),
                        bg_refine_openai: Some(planned.planned_postprocess.bg_refine_openai),
                        bg_refine_openai_required: Some(
                            planned.planned_postprocess.bg_refine_openai_required,
                        ),
                    })
                    .map_err(tool_adapter_error_to_runtime)?;
                let single = bg_resp.results.first().ok_or_else(|| {
                    PipelineRuntimeError::PlanningPreflight(String::from(
                        "bgremove adapter returned no per-file result",
                    ))
                })?;
                let bg_meta = json!({
                    "input": single.input,
                    "output": single.output,
                    "backend": single.backend,
                    "backends_tried": bg_resp.backends,
                    "refine_openai": single.refine_openai,
                    "refine_error": single.refine_error
                });
                candidate_json["bg_remove"] = bg_meta.clone();
                let next = bg_meta
                    .get("output")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        PipelineRuntimeError::PlanningPreflight(String::from(
                            "bgremove adapter JSON missing output",
                        ))
                    })?;
                current_path = PathBuf::from(next);
                bg_remove_output = Some(current_path.clone());
            }

            if let Some(upscale_path) = plan.upscale.as_ref() {
                let upscale_resp = tools
                    .upscale(&UpscalePassRequest {
                        project_slug: request.project_slug.clone(),
                        project_root: Some(project_root_arg.clone()),
                        input_path: path_for_output(app_root, current_path.as_path()),
                        output_path: path_for_output(app_root, upscale_path.as_path()),
                        postprocess_config_path: request.options.postprocess.config_path.clone(),
                        upscale_backend: planned.planned_postprocess.upscale_backend.clone(),
                        upscale_scale: planned
                            .postprocess_path_config
                            .upscale
                            .as_ref()
                            .map(|cfg| cfg.scale),
                        upscale_format: planned
                            .postprocess_path_config
                            .upscale
                            .as_ref()
                            .map(|cfg| cfg.format.clone()),
                    })
                    .map_err(tool_adapter_error_to_runtime)?;
                candidate_json["upscale"] = serde_json::to_value(&upscale_resp)
                    .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;
                current_path = PathBuf::from(upscale_resp.output);
                upscale_output = Some(current_path.clone());
            }

            if let Some(color_path) = plan.color.as_ref() {
                let color_resp = tools
                    .color(&ColorPassRequest {
                        project_slug: request.project_slug.clone(),
                        project_root: Some(project_root_arg.clone()),
                        input_path: path_for_output(app_root, current_path.as_path()),
                        output_path: path_for_output(app_root, color_path.as_path()),
                        postprocess_config_path: request.options.postprocess.config_path.clone(),
                        profile: planned.planned_postprocess.color_profile.clone(),
                        color_settings_path: None,
                    })
                    .map_err(tool_adapter_error_to_runtime)?;
                candidate_json["color"] = serde_json::to_value(&color_resp)
                    .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;
                current_path = PathBuf::from(color_resp.output);
                color_output = Some(current_path.clone());
            }

            let mut status = ExecutionCandidateStatus::Done;
            let mut final_output = Some(current_path.clone());
            let mut rank = ExecutionCandidateRank {
                hard_failures: 0,
                soft_warnings: 0,
                avg_chroma_exceed: 0.0,
            };

            if planned_output_guard.enabled {
                let qa_resp = tools
                    .qa(&QaCheckRequest {
                        project_slug: request.project_slug.clone(),
                        project_root: Some(project_root_arg.clone()),
                        input_path: path_for_output(app_root, current_path.as_path()),
                        manifest_path: manifest_path_for_qa.clone(),
                        output_guard_enabled: Some(true),
                        enforce_grayscale: Some(output_guard_cfg.enforce_grayscale),
                        max_chroma_delta: Some(output_guard_cfg.max_chroma_delta),
                        fail_on_chroma_exceed: Some(output_guard_cfg.fail_on_chroma_exceed),
                    })
                    .map_err(tool_adapter_error_to_runtime)?;
                let guard_report_value = qa_resp.report.as_ref().ok_or_else(|| {
                    PipelineRuntimeError::PlanningPreflight(String::from(
                        "qa adapter response missing report payload",
                    ))
                })?;
                let guard_report = parse_execution_output_guard_report(guard_report_value);
                rank =
                    summarize_output_guard_report(&guard_report, output_guard_cfg.max_chroma_delta);
                candidate_json["rank"] = serde_json::to_value(&rank)
                    .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;

                let mut bad_archive = None::<PathBuf>;
                if rank.hard_failures > 0 {
                    let archive_resp = tools
                        .archive_bad(&ArchiveBadRequest {
                            project_slug: request.project_slug.clone(),
                            project_root: Some(project_root_arg.clone()),
                            input_path: path_for_output(app_root, current_path.as_path()),
                        })
                        .map_err(tool_adapter_error_to_runtime)?;
                    bad_archive = archive_resp
                        .moved
                        .first()
                        .map(|m| PathBuf::from(m.to.clone()));
                    status = ExecutionCandidateStatus::FailedOutputGuard;
                    final_output = None;
                }

                let guard_record = build_run_log_output_guard_record(
                    &guard_report,
                    current_path.as_path(),
                    bad_archive.as_deref(),
                    |p| path_for_output(app_root, p),
                );
                candidate_json["output_guard"] = serde_json::to_value(&guard_record)
                    .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;
            }

            candidate_json["status"] = json!(status.as_str());
            if let Some(final_output_path) = final_output.as_ref() {
                candidate_json["final_output"] =
                    json!(path_for_output(app_root, final_output_path.as_path()));
            }

            execution_candidates.push(ExecutionCandidateJobResult {
                candidate: ExecutionCandidateResult {
                    candidate_index: plan.candidate_index,
                    status,
                    rank: rank.clone(),
                },
                outputs: ExecutionCandidateJobOutputs {
                    output: Some(PathBuf::from(
                        candidate_json
                            .get("output")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                    )),
                    final_output,
                    bg_remove: bg_remove_output,
                    upscale: upscale_output,
                    color: color_output,
                },
            });
            candidate_records.push(candidate_json);
        }

        let _ = fs::remove_file(input_images_file_path.as_path());

        let finalized = finalize_job_from_candidates(execution_candidates.as_slice())
            .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;
        failed_output_guard_jobs += finalized.failed_output_guard_jobs_increment;

        let mut job_json = json!({
            "id": job.id,
            "mode": job.mode,
            "time": job.time,
            "weather": job.weather,
            "input_images": job.input_images,
            "prompt": job.prompt,
            "status": "running",
            "selected_candidate": Value::Null,
            "final_output": Value::Null,
            "candidates": candidate_records,
            "planned_generation": { "candidates": candidate_count },
            "planned_postprocess": planned.planned_postprocess,
            "planned_output_guard": planned_output_guard
        });
        normalize_script_run_log_job_finalization(&mut job_json)
            .map_err(PipelineRuntimeError::PlanningPreflight)?;
        jobs_json.push(job_json);
    }

    let mut run_meta = json!({
        "timestamp": timestamp,
        "project": request.project_slug,
        "mode": "run",
        "stage": stage.as_str(),
        "time": time.as_str(),
        "weather": weather.as_str(),
        "model": model_name,
        "size": image_size,
        "quality": image_quality,
        "generation": {
            "candidates": candidate_count,
            "max_candidates": planned.manifest_max_candidates
        },
        "postprocess": planned.planned_postprocess,
        "output_guard": planned_output_guard,
        "storage": {
            "project_root": project_root_display,
            "resolved_from_backend": request.options.project_root.is_some()
        },
        "jobs": jobs_json
    });

    write_pretty_json_with_newline(run_log_path_abs.as_path(), &run_meta)
        .map_err(|e| PipelineRuntimeError::Io(std::io::Error::other(e.to_string())))?;
    let marker = format_summary_marker(&PipelineRunSummaryMarkerPayload {
        run_log_path: run_log_display.clone(),
        project_slug: request.project_slug.clone(),
        project_root: project_root_display.clone(),
        jobs: planned.job_count(),
        mode: String::from("run"),
    })
    .map_err(|e| PipelineRuntimeError::Io(std::io::Error::other(e.to_string())))?;
    let stdout = [
        format!("Run log: {run_log_display}"),
        format!("Project: {}", request.project_slug),
        format!("Project root: {project_root_display}"),
        format!("Jobs: {} (run/completed)", planned.job_count()),
        marker,
    ]
    .join("\n");

    if failed_output_guard_jobs > 0 {
        return Err(PipelineRuntimeError::CommandFailed {
            program: String::from("rust-pipeline"),
            status_code: 1,
            stdout,
            stderr: format!(
                "Output guard failed for {} job(s). Bad outputs moved to {}",
                failed_output_guard_jobs,
                path_for_output(app_root, project_dirs.archive_bad.as_path())
            ),
        });
    }

    // Keep post-run wrapper path compatible by returning script-like summary lines.
    // Also ensure planned metadata normalization wrapper stays idempotent on Rust-owned logs.
    let _ = run_meta.as_object_mut();
    Ok(PipelineRunResult {
        status_code: 0,
        stdout,
        stderr: String::new(),
    })
}

fn tool_adapter_error_to_runtime(error: ToolAdapterError) -> PipelineRuntimeError {
    match error {
        ToolAdapterError::CommandRunner(source) => source,
        ToolAdapterError::CommandFailed {
            program,
            status_code,
            stdout,
            stderr,
        } => PipelineRuntimeError::CommandFailed {
            program,
            status_code,
            stdout,
            stderr,
        },
        other => PipelineRuntimeError::PlanningPreflight(other.to_string()),
    }
}

fn write_json_temp_file(
    app_root: &Path,
    prefix: &str,
    value: &Value,
) -> Result<PathBuf, PipelineRuntimeError> {
    let dir = app_root.join("var/tmp");
    fs::create_dir_all(dir.as_path()).map_err(|error| {
        PipelineRuntimeError::PlannedJobsTempFile(format!(
            "create dir '{}': {error}",
            dir.display()
        ))
    })?;
    let path = dir.join(format!("{prefix}_{}.json", Uuid::new_v4()));
    write_pretty_json_with_newline(path.as_path(), value)
        .map_err(|e| PipelineRuntimeError::PlannedJobsTempFile(e.to_string()))?;
    Ok(path)
}

fn parse_execution_output_guard_report(value: &Value) -> ExecutionOutputGuardReport {
    let summary_obj = value.get("summary").and_then(Value::as_object);
    let summary = summary_obj.map(|obj| ExecutionOutputGuardReportSummary {
        total_files: obj.get("total_files").and_then(Value::as_u64).unwrap_or(0),
        hard_failures: obj
            .get("hard_failures")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        soft_warnings: obj
            .get("soft_warnings")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    });
    let files = value
        .get("files")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|file| {
            let obj = file.as_object().cloned().unwrap_or_default();
            ExecutionOutputGuardReportFile {
                file: obj.get("file").and_then(Value::as_str).map(PathBuf::from),
                chroma_delta: obj.get("chroma_delta").and_then(Value::as_f64),
            }
        })
        .collect::<Vec<_>>();
    ExecutionOutputGuardReport { summary, files }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PipelineScriptRunSummary {
    run_log_path: PathBuf,
    project_slug: Option<String>,
    project_root: Option<String>,
    jobs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
struct RustPlanningPreflightSummary {
    job_ids: Vec<String>,
    manifest_output_guard: PipelinePlanningOutputGuard,
    planned_postprocess: ExecutionPlannedPostprocessRecord,
    postprocess_path_config: ExecutionPostprocessPathConfig,
    manifest_candidate_count: u64,
    manifest_max_candidates: u64,
    jobs: Vec<PlannedGenerationJob>,
}

impl RustPlanningPreflightSummary {
    fn job_count(&self) -> u64 {
        self.job_ids.len() as u64
    }
}

fn execution_postprocess_path_config_from_planning(
    cfg: &PostprocessPlanningConfig,
    planned: &ExecutionPlannedPostprocessRecord,
) -> ExecutionPostprocessPathConfig {
    ExecutionPostprocessPathConfig {
        bg_remove_format: planned.bg_remove.then(|| cfg.bg_remove_format.clone()),
        upscale: planned.upscale.then(|| ExecutionUpscalePathConfig {
            scale: cfg.upscale_scale,
            format: cfg.upscale_format.clone(),
        }),
        color_profile: planned.color.then(|| {
            planned
                .color_profile
                .clone()
                .unwrap_or_else(|| cfg.color_default_profile.clone())
        }),
    }
}

fn map_manifest_output_guard_to_planned_record(
    cfg: &PipelinePlanningOutputGuard,
) -> ExecutionPlannedOutputGuardRecord {
    ExecutionPlannedOutputGuardRecord {
        enabled: true,
        enforce_grayscale: cfg.enforce_grayscale,
        max_chroma_delta: cfg.max_chroma_delta,
        fail_on_chroma_exceed: cfg.fail_on_chroma_exceed,
    }
}

fn parse_script_run_summary_from_stdout(stdout: &str) -> Option<PipelineScriptRunSummary> {
    const MARKER: &str = "KROMA_PIPELINE_SUMMARY_JSON:";
    if let Some(marker_line) = stdout
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with(MARKER))
    {
        let payload = marker_line.trim_start_matches(MARKER).trim();
        if !payload.is_empty() {
            if let Ok(parsed) = serde_json::from_str::<PipelineScriptRunSummaryMarker>(payload) {
                return Some(PipelineScriptRunSummary {
                    run_log_path: PathBuf::from(parsed.run_log_path),
                    project_slug: parsed.project_slug.filter(|v| !v.trim().is_empty()),
                    project_root: parsed.project_root.filter(|v| !v.trim().is_empty()),
                    jobs: parsed.jobs,
                });
            }
        }
    }

    let mut run_log_path = None;
    let mut project_slug = None;
    let mut project_root = None;
    let mut jobs = None;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("Run log:") {
            let value = value.trim();
            if !value.is_empty() {
                run_log_path = Some(PathBuf::from(value));
            }
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Project:") {
            let value = value.trim();
            if !value.is_empty() {
                project_slug = Some(value.to_string());
            }
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Project root:") {
            let value = value.trim();
            if !value.is_empty() {
                project_root = Some(value.to_string());
            }
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Jobs:") {
            let count_token = value.split_whitespace().next().unwrap_or_default();
            if let Ok(parsed) = count_token.parse::<u64>() {
                jobs = Some(parsed);
            }
        }
    }

    Some(PipelineScriptRunSummary {
        run_log_path: run_log_path?,
        project_slug,
        project_root,
        jobs,
    })
}

#[derive(Debug, Deserialize)]
struct PipelineScriptRunSummaryMarker {
    run_log_path: String,
    #[serde(default)]
    project_slug: Option<String>,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    jobs: Option<u64>,
}

fn append_stderr_line(stderr: &mut String, line: impl AsRef<str>) {
    if !stderr.trim().is_empty() {
        stderr.push('\n');
    }
    stderr.push_str(line.as_ref());
}

fn enrich_script_run_log_planned_metadata_file(
    app_root: &Path,
    request: &PipelineRunRequest,
    run_log_path: &Path,
) -> Result<(), String> {
    let effective = effective_pipeline_request_with_layered_settings(app_root, request)
        .map_err(|e| format!("resolve layered settings: {e}"))?;
    let Some(planned) = build_rust_planning_preflight_summary(app_root, &effective)
        .map_err(|e| format!("build planning preflight summary: {e}"))?
    else {
        return Ok(());
    };
    if planned.jobs.is_empty() {
        return Ok(());
    }

    let project_root_abs = effective
        .options
        .project_root
        .as_deref()
        .map(|v| resolve_under_app_root(app_root, v))
        .unwrap_or_else(|| {
            app_root
                .join("var/projects")
                .join(effective.project_slug.as_str())
        });
    let stage = effective
        .options
        .stage
        .unwrap_or(PipelineStageFilter::Style);
    let time = effective.options.time.unwrap_or(PipelineTimeFilter::Day);
    let weather = effective
        .options
        .weather
        .unwrap_or(PipelineWeatherFilter::Clear);
    let candidate_count = effective
        .options
        .candidates
        .map(u64::from)
        .unwrap_or(planned.manifest_candidate_count);

    let planned_template = build_planned_template(RunLogPlannedTemplateInput {
        project_slug: effective.project_slug.clone(),
        stage: stage.as_str().to_string(),
        time: time.as_str().to_string(),
        weather: weather.as_str().to_string(),
        project_root: path_for_output(app_root, project_root_abs.as_path()),
        resolved_from_backend: effective.options.project_root.is_some(),
        candidate_count,
        max_candidate_count: planned.manifest_max_candidates,
        planned_postprocess: planned.planned_postprocess.clone(),
        planned_output_guard: map_manifest_output_guard_to_planned_record(
            &planned.manifest_output_guard,
        ),
        jobs: planned.jobs.clone(),
    });

    patch_script_run_log_planned_metadata_file(app_root, run_log_path, &planned_template)
}

#[derive(Debug, Error)]
pub enum PipelineRuntimeError {
    #[error("invalid project slug for pipeline run")]
    InvalidProjectSlug,
    #[error("command execution failed: {0}")]
    Io(std::io::Error),
    #[error("pipeline command failed ({program}) with exit code {status_code}: {stderr}")]
    CommandFailed {
        program: String,
        status_code: i32,
        stdout: String,
        stderr: String,
    },
    #[error("{0}")]
    PlanningPreflight(String),
    #[error("failed to write planned jobs temp file: {0}")]
    PlannedJobsTempFile(String),
}

fn validate_project_slug(value: &str) -> Result<(), PipelineRuntimeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(PipelineRuntimeError::InvalidProjectSlug);
    }
    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        Ok(())
    } else {
        Err(PipelineRuntimeError::InvalidProjectSlug)
    }
}

fn resolve_under_app_root(app_root: &Path, value: &str) -> PathBuf {
    resolve_under_root(app_root, value)
}

fn default_project_root_for_request(app_root: &Path, request: &PipelineRunRequest) -> PathBuf {
    request
        .options
        .project_root
        .as_deref()
        .map(|v| resolve_under_app_root(app_root, v))
        .unwrap_or_else(|| {
            app_root
                .join("var/projects")
                .join(request.project_slug.as_str())
        })
}

fn effective_pipeline_request_with_layered_settings(
    app_root: &Path,
    request: &PipelineRunRequest,
) -> Result<PipelineRunRequest, PipelineRuntimeError> {
    let layer_paths = PipelineSettingsLayerPaths {
        app_settings_path: request.options.app_settings_path.clone(),
        project_settings_path: request.options.project_settings_path.clone(),
    };
    let app_settings =
        load_app_pipeline_settings(app_root, layer_paths.app_settings_path.as_deref())
            .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?;
    let project_root = default_project_root_for_request(app_root, request);
    let project_settings = load_project_pipeline_settings(
        Some(project_root.as_path()),
        layer_paths.project_settings_path.as_deref(),
    )
    .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?;
    let explicit = request_options_to_settings_overlay(&request.options);
    let merged = merge_pipeline_settings_overlays(&app_settings, &project_settings, &explicit);
    Ok(apply_settings_overlay_to_request(request, &merged))
}

fn request_options_to_settings_overlay(options: &PipelineRunOptions) -> PipelineSettingsOverlay {
    PipelineSettingsOverlay {
        manifest_path: options.manifest_path.clone(),
        postprocess_config_path: options.postprocess.config_path.clone(),
        post_upscale: options.postprocess.upscale.then_some(true),
        upscale_backend: options
            .postprocess
            .upscale_backend
            .map(|v| v.as_str().to_string()),
        post_color: options.postprocess.color.then_some(true),
        color_profile: options.postprocess.color_profile.clone(),
        post_bg_remove: options.postprocess.bg_remove.then_some(true),
        bg_remove_backends: (!options.postprocess.bg_remove_backends.is_empty())
            .then(|| options.postprocess.bg_remove_backends.clone()),
        bg_refine_openai: options.postprocess.bg_refine_openai,
        bg_refine_openai_required: options.postprocess.bg_refine_openai_required,
    }
}

fn apply_settings_overlay_to_request(
    request: &PipelineRunRequest,
    overlay: &PipelineSettingsOverlay,
) -> PipelineRunRequest {
    let mut out = request.clone();
    if out.options.manifest_path.is_none() {
        out.options.manifest_path = overlay.manifest_path.clone();
    }
    if out.options.postprocess.config_path.is_none() {
        out.options.postprocess.config_path = overlay.postprocess_config_path.clone();
    }
    if !out.options.postprocess.upscale {
        out.options.postprocess.upscale = overlay.post_upscale.unwrap_or(false);
    }
    if out.options.postprocess.upscale_backend.is_none() {
        out.options.postprocess.upscale_backend = overlay
            .upscale_backend
            .as_deref()
            .and_then(parse_pipeline_upscale_backend);
    }
    if !out.options.postprocess.color {
        out.options.postprocess.color = overlay.post_color.unwrap_or(false);
    }
    if out.options.postprocess.color_profile.is_none() {
        out.options.postprocess.color_profile = overlay.color_profile.clone();
    }
    if !out.options.postprocess.bg_remove {
        out.options.postprocess.bg_remove = overlay.post_bg_remove.unwrap_or(false);
    }
    if out.options.postprocess.bg_remove_backends.is_empty() {
        out.options.postprocess.bg_remove_backends =
            overlay.bg_remove_backends.clone().unwrap_or_default();
    }
    if out.options.postprocess.bg_refine_openai.is_none() {
        out.options.postprocess.bg_refine_openai = overlay.bg_refine_openai;
    }
    if out.options.postprocess.bg_refine_openai_required.is_none() {
        out.options.postprocess.bg_refine_openai_required = overlay.bg_refine_openai_required;
    }
    out
}

fn parse_pipeline_upscale_backend(value: &str) -> Option<PipelineUpscaleBackend> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ncnn" => Some(PipelineUpscaleBackend::Ncnn),
        "python" => Some(PipelineUpscaleBackend::Python),
        _ => None,
    }
}

fn path_for_output(app_root: &Path, path: &Path) -> String {
    path_for_output_shared(app_root, path)
}

fn iso_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", now.as_secs(), now.subsec_millis())
}

fn make_run_log_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{:03}", now.as_secs(), now.subsec_millis())
}

pub fn default_app_root_from_manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .to_path_buf()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::ProjectsStore;
    use crate::pipeline::backend_ops::{
        BackendCommandResult, BackendIngestRunRequest, BackendOpsError,
        BackendSyncProjectS3Request, PipelineBackendOps,
    };
    use serde_json::json;
    use std::fs;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_app_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("kroma_runtime_app_root_{stamp}"));
        fs::create_dir_all(path.as_path()).expect("temp app root should exist");
        path
    }

    #[derive(Default)]
    struct FakePostRunBackendOps {
        seen_ingest: Mutex<Vec<BackendIngestRunRequest>>,
        seen_sync: Mutex<Vec<BackendSyncProjectS3Request>>,
    }

    impl PipelineBackendOps for FakePostRunBackendOps {
        fn ingest_run(
            &self,
            request: &BackendIngestRunRequest,
        ) -> Result<BackendCommandResult, BackendOpsError> {
            self.seen_ingest
                .lock()
                .expect("fake post-run ingest mutex poisoned")
                .push(request.clone());
            Ok(BackendCommandResult {
                stdout: String::from(
                    "{\"ok\":true,\"project_slug\":\"demo\",\"run_id\":\"r1\",\"run_log_path\":\"var/projects/demo/runs/run_1.json\",\"jobs\":1,\"candidates\":1,\"assets_upserted\":1,\"quality_reports_written\":1,\"cost_events_written\":0,\"status\":\"ok\"}",
                ),
                stderr: String::new(),
                json: Some(json!({
                    "ok": true,
                    "project_slug": "demo",
                    "run_id": "r1",
                    "run_log_path": "var/projects/demo/runs/run_1.json",
                    "jobs": 1,
                    "candidates": 1,
                    "assets_upserted": 1,
                    "quality_reports_written": 1,
                    "cost_events_written": 0,
                    "status": "ok"
                })),
            })
        }

        fn sync_project_s3(
            &self,
            request: &BackendSyncProjectS3Request,
        ) -> Result<BackendCommandResult, BackendOpsError> {
            self.seen_sync
                .lock()
                .expect("fake post-run sync mutex poisoned")
                .push(request.clone());
            Ok(BackendCommandResult {
                stdout: String::from(
                    "{\"ok\":true,\"project_slug\":\"demo\",\"project_root\":\"/tmp/demo\",\"destination\":\"s3://bucket/demo/\",\"dry_run\":false,\"delete\":false}",
                ),
                stderr: String::new(),
                json: Some(json!({
                    "ok": true,
                    "project_slug": "demo",
                    "project_root": "/tmp/demo",
                    "destination": "s3://bucket/demo/",
                    "dry_run": false,
                    "delete": false
                })),
            })
        }
    }

    #[derive(Default)]
    struct FakeInnerOrchestrator {
        seen: Mutex<Vec<PipelineRunRequest>>,
        next: Mutex<Option<Result<PipelineRunResult, PipelineRuntimeError>>>,
    }

    impl FakeInnerOrchestrator {
        fn with_success_stdout(stdout: &str) -> Self {
            Self {
                seen: Mutex::new(Vec::new()),
                next: Mutex::new(Some(Ok(PipelineRunResult {
                    status_code: 0,
                    stdout: String::from(stdout),
                    stderr: String::new(),
                }))),
            }
        }

        fn with_command_failed(stdout: &str, stderr: &str) -> Self {
            Self {
                seen: Mutex::new(Vec::new()),
                next: Mutex::new(Some(Err(PipelineRuntimeError::CommandFailed {
                    program: String::from("node"),
                    status_code: 1,
                    stdout: String::from(stdout),
                    stderr: String::from(stderr),
                }))),
            }
        }
    }

    impl PipelineOrchestrator for FakeInnerOrchestrator {
        fn execute(
            &self,
            request: &PipelineRunRequest,
        ) -> Result<PipelineRunResult, PipelineRuntimeError> {
            self.seen
                .lock()
                .expect("fake inner orchestrator mutex poisoned")
                .push(request.clone());
            self.next
                .lock()
                .expect("fake inner orchestrator next mutex poisoned")
                .take()
                .unwrap_or_else(|| {
                    Ok(PipelineRunResult {
                        status_code: 0,
                        stdout: String::new(),
                        stderr: String::new(),
                    })
                })
        }
    }

    struct FakeToolAdapters {
        seen_generate: Mutex<Vec<GenerateOneImageRequest>>,
        seen_upscale: Mutex<Vec<UpscalePassRequest>>,
        seen_color: Mutex<Vec<ColorPassRequest>>,
        seen_bgremove: Mutex<Vec<BackgroundRemovePassRequest>>,
        seen_qa: Mutex<Vec<QaCheckRequest>>,
        seen_archive: Mutex<Vec<ArchiveBadRequest>>,
        qa_hard_failures: bool,
        qa_soft_warnings: u64,
    }

    impl Default for FakeToolAdapters {
        fn default() -> Self {
            Self {
                seen_generate: Mutex::new(Vec::new()),
                seen_upscale: Mutex::new(Vec::new()),
                seen_color: Mutex::new(Vec::new()),
                seen_bgremove: Mutex::new(Vec::new()),
                seen_qa: Mutex::new(Vec::new()),
                seen_archive: Mutex::new(Vec::new()),
                qa_hard_failures: false,
                qa_soft_warnings: 0,
            }
        }
    }

    impl FakeToolAdapters {
        fn with_qa_hard_failures() -> Self {
            Self {
                qa_hard_failures: true,
                ..Self::default()
            }
        }
    }

    impl PipelineToolAdapterOps for FakeToolAdapters {
        fn generate_one(
            &self,
            request: &GenerateOneImageRequest,
        ) -> Result<crate::pipeline::tool_adapters::GenerateOneImageResponse, ToolAdapterError>
        {
            self.seen_generate
                .lock()
                .expect("fake tool adapters generate mutex poisoned")
                .push(request.clone());
            Ok(crate::pipeline::tool_adapters::GenerateOneImageResponse {
                ok: true,
                project: request.project_slug.clone(),
                output: request.output_path.clone(),
                input_images: vec![String::from("var/projects/demo/scenes/a.png")],
                model: String::from("gpt-image-1"),
                size: String::from("1024x1536"),
                quality: String::from("high"),
                bytes_written: 1234,
            })
        }

        fn upscale(
            &self,
            request: &UpscalePassRequest,
        ) -> Result<crate::pipeline::tool_adapters::UpscalePassResponse, ToolAdapterError> {
            self.seen_upscale
                .lock()
                .expect("fake tool adapters upscale mutex poisoned")
                .push(request.clone());
            Ok(crate::pipeline::tool_adapters::UpscalePassResponse {
                backend: request
                    .upscale_backend
                    .clone()
                    .unwrap_or_else(|| String::from("python")),
                input: request.input_path.clone(),
                output: request.output_path.clone(),
                scale: u64::from(request.upscale_scale.unwrap_or(2)),
                model: String::from("fake-upscale-model"),
            })
        }

        fn color(
            &self,
            request: &ColorPassRequest,
        ) -> Result<crate::pipeline::tool_adapters::ColorPassResponse, ToolAdapterError> {
            self.seen_color
                .lock()
                .expect("fake tool adapters color mutex poisoned")
                .push(request.clone());
            Ok(crate::pipeline::tool_adapters::ColorPassResponse {
                input: request.input_path.clone(),
                output: request.output_path.clone(),
                profile: request
                    .profile
                    .clone()
                    .unwrap_or_else(|| String::from("neutral")),
                settings: String::from("builtin"),
            })
        }

        fn bgremove(
            &self,
            request: &BackgroundRemovePassRequest,
        ) -> Result<crate::pipeline::tool_adapters::BackgroundRemovePassResponse, ToolAdapterError>
        {
            self.seen_bgremove
                .lock()
                .expect("fake tool adapters bgremove mutex poisoned")
                .push(request.clone());
            Ok(
                crate::pipeline::tool_adapters::BackgroundRemovePassResponse {
                    input: request.input_path.clone(),
                    output: request.output_path.clone(),
                    backends: if request.backends.is_empty() {
                        vec![String::from("rembg")]
                    } else {
                        request.backends.clone()
                    },
                    refine_openai: request.bg_refine_openai.unwrap_or(false),
                    refine_openai_required: request.bg_refine_openai_required.unwrap_or(false),
                    format: String::from("png"),
                    processed: 1,
                    results: vec![
                        crate::pipeline::tool_adapters::BackgroundRemovePassFileResult {
                            input: request.input_path.clone(),
                            output: request.output_path.clone(),
                            backend: request
                                .backends
                                .first()
                                .cloned()
                                .unwrap_or_else(|| String::from("rembg")),
                            refine_openai: request.bg_refine_openai.unwrap_or(false),
                            refine_error: None,
                        },
                    ],
                },
            )
        }

        fn qa(
            &self,
            request: &QaCheckRequest,
        ) -> Result<crate::pipeline::tool_adapters::QaCheckResponse, ToolAdapterError> {
            self.seen_qa
                .lock()
                .expect("fake tool adapters qa mutex poisoned")
                .push(request.clone());
            let hard_failures = if self.qa_hard_failures { 1 } else { 0 };
            Ok(crate::pipeline::tool_adapters::QaCheckResponse {
                ok: hard_failures == 0,
                skipped: None,
                enabled: true,
                reason: None,
                has_hard_failures: Some(hard_failures > 0),
                input: Some(request.input_path.clone()),
                summary: Some(crate::pipeline::tool_adapters::QaCheckSummary {
                    total_files: 1,
                    hard_failures,
                    soft_warnings: self.qa_soft_warnings,
                }),
                report: Some(json!({
                    "summary": { "total_files": 1, "hard_failures": hard_failures, "soft_warnings": self.qa_soft_warnings },
                    "files": [{ "file": request.input_path, "chroma_delta": 0.5 }]
                })),
            })
        }

        fn archive_bad(
            &self,
            request: &ArchiveBadRequest,
        ) -> Result<crate::pipeline::tool_adapters::ArchiveBadResponse, ToolAdapterError> {
            self.seen_archive
                .lock()
                .expect("fake tool adapters archive mutex poisoned")
                .push(request.clone());
            Ok(crate::pipeline::tool_adapters::ArchiveBadResponse {
                ok: true,
                archive_dir: String::from("var/projects/demo/archive/bad"),
                moved_count: 1,
                moved: vec![crate::pipeline::tool_adapters::ArchiveBadMovedFile {
                    from: request.input_path.clone(),
                    to: String::from("var/projects/demo/archive/bad/out.png"),
                }],
            })
        }
    }

    #[test]
    fn rust_post_run_wrapper_disables_script_ingest_and_runs_backend_ingest() {
        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout(
            "Run log: var/projects/demo/runs/run_1.json\nProject: demo",
        ));
        let backend_ops = Arc::new(FakePostRunBackendOps::default());
        let post_run = PipelinePostRunService::new(backend_ops.clone());
        let wrapper = RustPostRunPipelineOrchestrator::new(inner.clone(), post_run);

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("wrapper execution should succeed");

        assert_eq!(result.status_code, 0);
        assert!(result.stderr.is_empty());

        let seen_request = inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .first()
            .cloned()
            .expect("inner request should be recorded");
        assert_eq!(seen_request.options.backend_db_ingest, Some(false));
        assert_eq!(seen_request.options.storage_sync_s3, Some(false));

        let seen_ingest = backend_ops
            .seen_ingest
            .lock()
            .expect("fake post-run ingest mutex poisoned");
        assert_eq!(seen_ingest.len(), 1);
        assert_eq!(
            seen_ingest[0].run_log_path,
            PathBuf::from("var/projects/demo/runs/run_1.json")
        );
    }

    #[test]
    fn rust_post_run_wrapper_warns_when_run_log_line_is_missing() {
        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("Project: demo"));
        let backend_ops = Arc::new(FakePostRunBackendOps::default());
        let post_run = PipelinePostRunService::new(backend_ops.clone());
        let wrapper = RustPostRunPipelineOrchestrator::new(inner, post_run);

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("wrapper execution should stay successful");

        assert!(result
            .stderr
            .contains("Rust post-run finalize skipped: missing summary marker or 'Run log:' line"));
        assert_eq!(
            backend_ops
                .seen_ingest
                .lock()
                .expect("fake post-run ingest mutex poisoned")
                .len(),
            0
        );
        assert_eq!(
            backend_ops
                .seen_sync
                .lock()
                .expect("fake post-run sync mutex poisoned")
                .len(),
            0
        );
    }

    #[test]
    fn rust_post_run_wrapper_runs_backend_sync_when_requested() {
        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout(
            "KROMA_PIPELINE_SUMMARY_JSON: {\"run_log_path\":\"var/projects/demo/runs/run_1.json\",\"project_slug\":\"demo\",\"project_root\":\"var/projects/demo\",\"jobs\":1,\"mode\":\"run\"}",
        ));
        let backend_ops = Arc::new(FakePostRunBackendOps::default());
        let post_run = PipelinePostRunService::new(backend_ops.clone());
        let wrapper = RustPostRunPipelineOrchestrator::new(inner.clone(), post_run);

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    storage_sync_s3: Some(true),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("wrapper execution should succeed");

        assert_eq!(result.status_code, 0);
        assert!(result.stderr.is_empty());

        let seen_request = inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .first()
            .cloned()
            .expect("inner request should be recorded");
        assert_eq!(seen_request.options.backend_db_ingest, Some(false));
        assert_eq!(seen_request.options.storage_sync_s3, Some(false));

        assert_eq!(
            backend_ops
                .seen_ingest
                .lock()
                .expect("fake post-run ingest mutex poisoned")
                .len(),
            1
        );
        let seen_sync = backend_ops
            .seen_sync
            .lock()
            .expect("fake post-run sync mutex poisoned");
        assert_eq!(seen_sync.len(), 1);
        assert_eq!(seen_sync[0].project_slug, "demo");
        assert!(!seen_sync[0].dry_run);
        assert!(!seen_sync[0].delete);
        assert!(!seen_sync[0].allow_missing_local);
    }

    #[test]
    fn rust_post_run_wrapper_ingests_when_inner_command_fails_after_run_log() {
        let inner = Arc::new(FakeInnerOrchestrator::with_command_failed(
            "Run log: var/projects/demo/runs/run_1.json\nProject: demo",
            "output guard failed",
        ));
        let backend_ops = Arc::new(FakePostRunBackendOps::default());
        let post_run = PipelinePostRunService::new(backend_ops.clone());
        let wrapper = RustPostRunPipelineOrchestrator::new(inner, post_run);

        let err = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect_err("wrapper should preserve command failure");

        match err {
            PipelineRuntimeError::CommandFailed { stdout, stderr, .. } => {
                assert!(stdout.contains("Run log:"));
                assert!(stderr.contains("output guard failed"));
            }
            other => panic!("unexpected error: {other:?}"),
        }

        assert_eq!(
            backend_ops
                .seen_ingest
                .lock()
                .expect("fake post-run ingest mutex poisoned")
                .len(),
            1
        );
    }

    #[test]
    fn rust_post_run_wrapper_normalizes_run_log_job_finalization_before_ingest() {
        let app_root = temp_app_root();
        let run_log_rel = PathBuf::from("var/projects/demo/runs/run_1.json");
        let run_log_abs = app_root.join(run_log_rel.as_path());
        fs::create_dir_all(
            run_log_abs
                .parent()
                .expect("run log parent should have a parent"),
        )
        .expect("run log dir should exist");
        fs::write(
            run_log_abs.as_path(),
            serde_json::to_vec_pretty(&json!({
                    "project": "demo",
                    "mode": "run",
                    "jobs": [
                        {
                        "id": "style_1_a",
                        "status": "done",
                        "selected_candidate": 2,
                        "final_output": "var/projects/demo/outputs/wrong.png",
                        "output": "var/projects/demo/outputs/wrong.png",
                        "failure_reason": "stale",
                        "candidates": [
                            {
                                "candidate_index": 1,
                                "status": "done",
                                "output": "var/projects/demo/outputs/a.png",
                                "final_output": "var/projects/demo/color_corrected/a_profile.png",
                                "rank": {
                                    "hard_failures": 0,
                                    "soft_warnings": 0,
                                    "avg_chroma_exceed": 0.0
                                },
                                "bg_remove": { "output": "var/projects/demo/background_removed/a_nobg.png" },
                                "output_guard": { "summary": { "hard_failures": 0 } }
                            },
                            {
                                "candidate_index": 2,
                                "status": "done",
                                "output": "var/projects/demo/outputs/b.png",
                                "final_output": "var/projects/demo/color_corrected/b_profile.png",
                                "rank": {
                                    "hard_failures": 1,
                                    "soft_warnings": 0,
                                    "avg_chroma_exceed": 0.0
                                }
                            }
                        ]
                    }
                ]
            }))
            .expect("run log json should serialize"),
        )
        .expect("run log should be written");

        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout(
            "KROMA_PIPELINE_SUMMARY_JSON: {\"run_log_path\":\"var/projects/demo/runs/run_1.json\",\"project_slug\":\"demo\",\"project_root\":\"var/projects/demo\",\"jobs\":1,\"mode\":\"run\"}",
        ));
        let backend_ops = Arc::new(FakePostRunBackendOps::default());
        let post_run = PipelinePostRunService::new(backend_ops.clone());
        let wrapper =
            RustPostRunPipelineOrchestrator::new(inner, post_run).with_app_root(app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("wrapper execution should succeed");

        assert!(result.stderr.is_empty());
        assert_eq!(
            backend_ops
                .seen_ingest
                .lock()
                .expect("fake post-run ingest mutex poisoned")
                .len(),
            1
        );

        let raw = fs::read_to_string(run_log_abs.as_path()).expect("run log should be readable");
        let run_log: serde_json::Value =
            serde_json::from_str(raw.as_str()).expect("run log should parse");
        let job = run_log
            .get("jobs")
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .expect("first job should exist");

        assert_eq!(job.get("status").and_then(|v| v.as_str()), Some("done"));
        assert_eq!(
            job.get("selected_candidate").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            job.get("final_output").and_then(|v| v.as_str()),
            Some("var/projects/demo/color_corrected/a_profile.png")
        );
        assert_eq!(
            job.get("output").and_then(|v| v.as_str()),
            Some("var/projects/demo/color_corrected/a_profile.png")
        );
        assert!(job.get("failure_reason").is_none());
        assert_eq!(
            job.get("bg_remove")
                .and_then(|v| v.get("output"))
                .and_then(|v| v.as_str()),
            Some("var/projects/demo/background_removed/a_nobg.png")
        );
        assert!(job.get("output_guard").is_some());
        assert_eq!(
            job.get("planned_generation")
                .and_then(|v| v.get("candidates"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            job.get("planned_postprocess")
                .and_then(|v| v.get("pipeline_order"))
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str()),
            Some("generate")
        );
        assert_eq!(
            job.get("planned_output_guard")
                .and_then(|v| v.get("max_chroma_delta"))
                .and_then(|v| v.as_f64()),
            Some(2.0)
        );
        assert_eq!(
            run_log
                .get("generation")
                .and_then(|v| v.get("candidates"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            run_log
                .get("generation")
                .and_then(|v| v.get("max_candidates"))
                .and_then(|v| v.as_u64()),
            Some(6)
        );
        assert_eq!(
            run_log
                .get("storage")
                .and_then(|v| v.get("project_root"))
                .and_then(|v| v.as_str()),
            Some("var/projects/demo")
        );
        assert_eq!(
            run_log
                .get("postprocess")
                .and_then(|v| v.get("pipeline_order"))
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str()),
            Some("generate")
        );

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn default_rust_pipeline_stack_rejects_unsupported_request_shape_without_script_fallback() {
        let backend_ops = Arc::new(FakePostRunBackendOps::default());
        let orchestrator =
            default_pipeline_orchestrator_with_rust_post_run_backend_ops(backend_ops.clone());

        let err = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions::default(),
            })
            .expect_err("unsupported rust-only request shape should fail");

        match err {
            PipelineRuntimeError::PlanningPreflight(message) => {
                assert!(message.contains("Rust-only pipeline runtime"));
                assert!(message.contains("preflight-supported inputs"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert_eq!(
            backend_ops
                .seen_ingest
                .lock()
                .expect("fake post-run ingest mutex poisoned")
                .len(),
            0
        );
    }

    #[test]
    fn default_native_post_run_stack_rejects_unsupported_request_shape_without_script_fallback() {
        let root = temp_app_root();
        let db = root.join("var/backend/app.db");
        let store = Arc::new(ProjectsStore::new(db, root));
        let orchestrator = default_pipeline_orchestrator_with_native_post_run(store);

        let err = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions::default(),
            })
            .expect_err("unsupported rust-only request shape should fail");

        match err {
            PipelineRuntimeError::PlanningPreflight(message) => {
                assert!(message.contains("Rust-only pipeline runtime"));
                assert!(message.contains("preflight-supported inputs"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rust_dry_run_wrapper_handles_scene_refs_without_inner_script_call() {
        let app_root = temp_app_root();
        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let wrapper = RustDryRunPipelineOrchestrator::new(inner.clone(), app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from(
                        "var/projects/demo/scenes/a.png",
                    )])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("rust dry run should succeed");

        assert!(result.stdout.contains("Run log: "));
        assert!(result.stdout.contains("Project: demo"));
        assert!(result.stdout.contains("KROMA_PIPELINE_SUMMARY_JSON:"));
        assert!(inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let summary = parse_script_run_summary_from_stdout(result.stdout.as_str())
            .expect("summary should parse");
        let run_log_abs = app_root.join(summary.run_log_path);
        assert!(run_log_abs.is_file());

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_dry_run_wrapper_writes_planned_job_fields_from_typed_builder() {
        let app_root = temp_app_root();
        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let wrapper = RustDryRunPipelineOrchestrator::new(inner, app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    candidates: Some(2),
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from(
                        "var/projects/demo/scenes/a.png",
                    )])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("rust dry run should succeed");

        let summary = parse_script_run_summary_from_stdout(result.stdout.as_str())
            .expect("summary should parse");
        let run_log_abs = app_root.join(summary.run_log_path);
        let raw = fs::read_to_string(run_log_abs.as_path()).expect("run log should be readable");
        let run_log: serde_json::Value =
            serde_json::from_str(raw.as_str()).expect("run log should parse as json");
        let jobs = run_log
            .get("jobs")
            .and_then(|v| v.as_array())
            .expect("jobs should be an array");
        assert_eq!(
            run_log
                .get("generation")
                .and_then(|v| v.get("candidates"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            run_log
                .get("generation")
                .and_then(|v| v.get("max_candidates"))
                .and_then(|v| v.as_u64()),
            Some(6)
        );
        assert_eq!(jobs.len(), 1);
        assert_eq!(
            jobs[0].get("status").and_then(|v| v.as_str()),
            Some("planned")
        );
        assert_eq!(
            jobs[0]
                .get("planned_generation")
                .and_then(|v| v.get("candidates"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            jobs[0]
                .get("planned_postprocess")
                .and_then(|v| v.get("pipeline_order"))
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str()),
            Some("generate")
        );
        assert_eq!(
            jobs[0]
                .get("planned_output_guard")
                .and_then(|v| v.get("max_chroma_delta"))
                .and_then(|v| v.as_f64()),
            Some(2.0)
        );

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_dry_run_wrapper_uses_manifest_generation_defaults_when_candidates_omitted() {
        let app_root = temp_app_root();
        let manifest_path = app_root.join("manifest.json");
        let postprocess_path = app_root.join("postprocess.json");
        fs::write(
            manifest_path.as_path(),
            r#"{
  "scene_refs": ["var/projects/demo/scenes/a.png"],
  "generation": { "candidates": 3, "max_candidates": 9 },
  "output_guard": {
    "enforce_grayscale": true,
    "max_chroma_delta": 1.5,
    "fail_on_chroma_exceed": true
  }
}
"#,
        )
        .expect("manifest should be written");
        fs::write(
            postprocess_path.as_path(),
            r#"{
  "upscale": { "backend": "ncnn" },
  "color": { "default_profile": "studio" },
  "bg_remove": {
    "backends": ["photoroom"],
    "openai": { "enabled": false, "required": false }
  }
}
"#,
        )
        .expect("postprocess config should be written");
        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let wrapper = RustDryRunPipelineOrchestrator::new(inner.clone(), app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    manifest_path: Some(String::from("manifest.json")),
                    postprocess: PipelinePostprocessOptions {
                        config_path: Some(String::from("postprocess.json")),
                        upscale: true,
                        color: true,
                        bg_remove: true,
                        ..PipelinePostprocessOptions::default()
                    },
                    ..PipelineRunOptions::default()
                },
            })
            .expect("rust dry run should succeed");

        let summary = parse_script_run_summary_from_stdout(result.stdout.as_str())
            .expect("summary should parse");
        let run_log_abs = app_root.join(summary.run_log_path);
        let raw = fs::read_to_string(run_log_abs.as_path()).expect("run log should be readable");
        let run_log: serde_json::Value =
            serde_json::from_str(raw.as_str()).expect("run log should parse as json");
        let jobs = run_log
            .get("jobs")
            .and_then(|v| v.as_array())
            .expect("jobs should be an array");

        assert_eq!(
            run_log
                .get("generation")
                .and_then(|v| v.get("candidates"))
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            run_log
                .get("generation")
                .and_then(|v| v.get("max_candidates"))
                .and_then(|v| v.as_u64()),
            Some(9)
        );
        assert_eq!(
            run_log
                .get("output_guard")
                .and_then(|v| v.get("enforce_grayscale"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            run_log
                .get("output_guard")
                .and_then(|v| v.get("max_chroma_delta"))
                .and_then(|v| v.as_f64()),
            Some(1.5)
        );
        assert_eq!(
            run_log
                .get("output_guard")
                .and_then(|v| v.get("fail_on_chroma_exceed"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            run_log
                .get("postprocess")
                .and_then(|v| v.get("upscale"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            run_log
                .get("postprocess")
                .and_then(|v| v.get("upscale_backend"))
                .and_then(|v| v.as_str()),
            Some("ncnn")
        );
        assert_eq!(
            run_log
                .get("postprocess")
                .and_then(|v| v.get("color_profile"))
                .and_then(|v| v.as_str()),
            Some("studio")
        );
        assert_eq!(
            run_log
                .get("postprocess")
                .and_then(|v| v.get("bg_remove_backends"))
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str()),
            Some("photoroom")
        );
        assert_eq!(
            run_log
                .get("postprocess")
                .and_then(|v| v.get("bg_refine_openai"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            jobs[0]
                .get("planned_generation")
                .and_then(|v| v.get("candidates"))
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            jobs[0]
                .get("planned_output_guard")
                .and_then(|v| v.get("max_chroma_delta"))
                .and_then(|v| v.as_f64()),
            Some(1.5)
        );
        assert_eq!(
            jobs[0]
                .get("planned_postprocess")
                .and_then(|v| v.get("upscale_backend"))
                .and_then(|v| v.as_str()),
            Some("ncnn")
        );
        assert!(inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_dry_run_wrapper_handles_input_path_without_inner_script_call() {
        let app_root = temp_app_root();
        let input_dir = app_root.join("inputs");
        fs::create_dir_all(input_dir.join("nested")).expect("input dir should exist");
        fs::write(input_dir.join("a.png"), b"a").expect("image a should exist");
        fs::write(input_dir.join("nested/b.jpg"), b"b").expect("image b should exist");
        fs::write(input_dir.join("nested/readme.txt"), b"x").expect("non-image file should exist");
        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("ok"));
        let wrapper = RustDryRunPipelineOrchestrator::new(inner.clone(), app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::InputPath(String::from("inputs"))),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("rust dry input-path run should succeed");

        assert!(result.stdout.contains("Jobs: 2 (dry/planned)"));
        assert!(app_root.join("var/projects/demo/outputs").is_dir());
        assert!(app_root.join("var/projects/demo/archive/bad").is_dir());
        assert!(app_root.join("var/projects/demo/archive/replaced").is_dir());
        assert!(inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_dry_run_wrapper_handles_jobs_file_without_inner_script_call() {
        let app_root = temp_app_root();
        fs::create_dir_all(app_root.join("var/tmp")).expect("tmp dir should exist");
        fs::write(
            app_root.join("var/tmp/jobs.json"),
            r#"[{
  "id":"manual_job_1",
  "prompt":"prompt",
  "mode":"style",
  "time":"day",
  "weather":"clear",
  "input_images":["var/projects/demo/scenes/a.png"]
}]"#,
        )
        .expect("jobs file should be written");

        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let wrapper = RustDryRunPipelineOrchestrator::new(inner.clone(), app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    jobs_file: Some(String::from("var/tmp/jobs.json")),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("rust dry jobs-file run should succeed");

        assert!(result.stdout.contains("Jobs: 1 (dry/planned)"));
        assert!(inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let summary = parse_script_run_summary_from_stdout(result.stdout.as_str())
            .expect("summary should parse");
        let run_log_abs = app_root.join(summary.run_log_path);
        let raw = fs::read_to_string(run_log_abs.as_path()).expect("run log should be readable");
        let run_log: serde_json::Value =
            serde_json::from_str(raw.as_str()).expect("run log should parse");
        let job = run_log
            .get("jobs")
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .expect("job should exist");
        assert_eq!(job.get("id").and_then(|v| v.as_str()), Some("manual_job_1"));
        assert_eq!(
            job.get("planned_generation")
                .and_then(|v| v.get("candidates"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_dry_run_wrapper_applies_project_settings_layer_for_postprocess_defaults() {
        let app_root = temp_app_root();
        let project_settings_dir = app_root.join("var/projects/demo/.kroma");
        fs::create_dir_all(project_settings_dir.as_path())
            .expect("project settings dir should exist");
        fs::create_dir_all(app_root.join("config")).expect("config dir should exist");
        fs::write(
            project_settings_dir.join("pipeline.settings.json"),
            r#"{
  "pipeline": {
    "postprocess_config_path": "config/post.json",
    "postprocess": {
      "upscale": true,
      "upscale_backend": "ncnn",
      "color": true,
      "color_profile": "project-profile",
      "bg_remove": true,
      "bg_remove_backends": ["photoroom"],
      "bg_refine_openai": false,
      "bg_refine_openai_required": false
    }
  }
}"#,
        )
        .expect("project settings should be written");
        fs::write(app_root.join("config/post.json"), r#"{}"#)
            .expect("postprocess config should be written");

        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let wrapper = RustDryRunPipelineOrchestrator::new(inner.clone(), app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from(
                        "var/projects/demo/scenes/a.png",
                    )])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("rust dry run should succeed");

        let summary = parse_script_run_summary_from_stdout(result.stdout.as_str())
            .expect("summary should parse");
        let run_log_abs = app_root.join(summary.run_log_path);
        let raw = fs::read_to_string(run_log_abs.as_path()).expect("run log should be readable");
        let run_log: serde_json::Value =
            serde_json::from_str(raw.as_str()).expect("run log should parse");
        let post = run_log
            .get("postprocess")
            .expect("postprocess should exist");

        assert_eq!(post.get("upscale").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            post.get("upscale_backend").and_then(|v| v.as_str()),
            Some("ncnn")
        );
        assert_eq!(post.get("color").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            post.get("color_profile").and_then(|v| v.as_str()),
            Some("project-profile")
        );
        assert_eq!(post.get("bg_remove").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            post.get("bg_remove_backends")
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str()),
            Some("photoroom")
        );
        assert_eq!(
            post.get("bg_refine_openai").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert!(inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_run_mode_wrapper_handles_jobs_file_without_inner_script_call() {
        let app_root = temp_app_root();
        fs::create_dir_all(app_root.join("var/tmp")).expect("tmp dir should exist");
        fs::write(
            app_root.join("var/tmp/jobs.json"),
            r#"[{
  "id":"manual_job_1",
  "prompt":"prompt",
  "mode":"style",
  "time":"day",
  "weather":"clear",
  "input_images":["var/projects/demo/scenes/a.png"]
}]"#,
        )
        .expect("jobs file should be written");

        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let tools_impl = Arc::new(FakeToolAdapters::default());
        let tools: SharedPipelineToolAdapterOps = tools_impl.clone();
        let wrapper = RustRunModePipelineOrchestrator::new(inner.clone(), tools, app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    jobs_file: Some(String::from("var/tmp/jobs.json")),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("rust run-mode wrapper should succeed");

        assert!(result.stdout.contains("Jobs: 1 (run/completed)"));
        assert!(result.stdout.contains("KROMA_PIPELINE_SUMMARY_JSON:"));
        assert!(inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let summary = parse_script_run_summary_from_stdout(result.stdout.as_str())
            .expect("summary should parse");
        let run_log_abs = app_root.join(summary.run_log_path);
        let raw = fs::read_to_string(run_log_abs.as_path()).expect("run log should be readable");
        let run_log: serde_json::Value =
            serde_json::from_str(raw.as_str()).expect("run log should parse");
        let job = run_log
            .get("jobs")
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .expect("job should exist");
        assert_eq!(job.get("status").and_then(|v| v.as_str()), Some("done"));
        assert_eq!(
            job.get("selected_candidate").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            job.get("final_output").and_then(|v| v.as_str()),
            Some("var/projects/demo/outputs/manual_job_1.png")
        );
        assert_eq!(
            job.get("output_guard")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("hard_failures"))
                .and_then(|v| v.as_u64()),
            Some(0)
        );
        assert_eq!(
            run_log
                .get("postprocess")
                .and_then(|v| v.get("pipeline_order"))
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str()),
            Some("generate")
        );
        assert_eq!(
            tools_impl
                .seen_generate
                .lock()
                .expect("fake tool adapters generate mutex poisoned")
                .len(),
            1
        );
        assert_eq!(
            tools_impl
                .seen_qa
                .lock()
                .expect("fake tool adapters qa mutex poisoned")
                .len(),
            1
        );
        assert_eq!(
            tools_impl
                .seen_archive
                .lock()
                .expect("fake tool adapters archive mutex poisoned")
                .len(),
            0
        );

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_run_mode_wrapper_executes_optional_passes_with_script_parity_paths() {
        let app_root = temp_app_root();
        fs::create_dir_all(app_root.join("var/tmp")).expect("tmp dir should exist");
        fs::write(
            app_root.join("var/tmp/jobs.json"),
            r#"[{
  "id":"manual_job_1",
  "prompt":"prompt",
  "mode":"style",
  "time":"day",
  "weather":"clear",
  "input_images":["var/projects/demo/scenes/a.png"]
}]"#,
        )
        .expect("jobs file should be written");
        fs::write(
            app_root.join("postprocess.json"),
            r#"{
  "upscale": { "backend": "ncnn", "scale": 4, "format": "png" },
  "color": { "default_profile": "cinematic" },
  "bg_remove": { "format": "webp", "backends": ["rembg"], "openai": { "enabled": false, "required": false } }
}"#,
        )
        .expect("postprocess config should be written");

        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let tools_impl = Arc::new(FakeToolAdapters::default());
        let tools: SharedPipelineToolAdapterOps = tools_impl.clone();
        let wrapper = RustRunModePipelineOrchestrator::new(inner.clone(), tools, app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    jobs_file: Some(String::from("var/tmp/jobs.json")),
                    postprocess: PipelinePostprocessOptions {
                        config_path: Some(String::from("postprocess.json")),
                        upscale: true,
                        color: true,
                        bg_remove: true,
                        ..PipelinePostprocessOptions::default()
                    },
                    ..PipelineRunOptions::default()
                },
            })
            .expect("rust run-mode wrapper with optional passes should succeed");

        assert!(result.stdout.contains("Jobs: 1 (run/completed)"));
        assert!(inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let summary = parse_script_run_summary_from_stdout(result.stdout.as_str())
            .expect("summary should parse");
        let run_log_abs = app_root.join(summary.run_log_path);
        let raw = fs::read_to_string(run_log_abs.as_path()).expect("run log should be readable");
        let run_log: serde_json::Value =
            serde_json::from_str(raw.as_str()).expect("run log should parse");
        let job = run_log
            .get("jobs")
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .expect("job should exist");
        assert_eq!(job.get("status").and_then(|v| v.as_str()), Some("done"));
        assert_eq!(
            job.get("final_output").and_then(|v| v.as_str()),
            Some("var/projects/demo/color_corrected/manual_job_1_nobg_x4_cinematic.png")
        );
        assert_eq!(
            job.get("bg_remove")
                .and_then(|v| v.get("output"))
                .and_then(|v| v.as_str()),
            Some("var/projects/demo/background_removed/manual_job_1_nobg.webp")
        );
        assert_eq!(
            job.get("upscale")
                .and_then(|v| v.get("output"))
                .and_then(|v| v.as_str()),
            Some("var/projects/demo/upscaled/manual_job_1_nobg_x4.png")
        );
        assert_eq!(
            job.get("color")
                .and_then(|v| v.get("output"))
                .and_then(|v| v.as_str()),
            Some("var/projects/demo/color_corrected/manual_job_1_nobg_x4_cinematic.png")
        );

        let bgremove_seen = tools_impl
            .seen_bgremove
            .lock()
            .expect("fake tool adapters bgremove mutex poisoned");
        assert_eq!(bgremove_seen.len(), 1);
        assert_eq!(
            bgremove_seen[0].output_path,
            "var/projects/demo/background_removed/manual_job_1_nobg.webp"
        );
        drop(bgremove_seen);

        let upscale_seen = tools_impl
            .seen_upscale
            .lock()
            .expect("fake tool adapters upscale mutex poisoned");
        assert_eq!(upscale_seen.len(), 1);
        assert_eq!(
            upscale_seen[0].output_path,
            "var/projects/demo/upscaled/manual_job_1_nobg_x4.png"
        );
        assert_eq!(upscale_seen[0].upscale_scale, Some(4));
        drop(upscale_seen);

        let color_seen = tools_impl
            .seen_color
            .lock()
            .expect("fake tool adapters color mutex poisoned");
        assert_eq!(color_seen.len(), 1);
        assert_eq!(
            color_seen[0].output_path,
            "var/projects/demo/color_corrected/manual_job_1_nobg_x4_cinematic.png"
        );
        assert_eq!(color_seen[0].profile.as_deref(), Some("cinematic"));

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_run_mode_wrapper_returns_failure_and_archives_bad_outputs_on_output_guard_fail() {
        let app_root = temp_app_root();
        fs::create_dir_all(app_root.join("var/tmp")).expect("tmp dir should exist");
        fs::write(
            app_root.join("var/tmp/jobs.json"),
            r#"[{
  "id":"manual_job_1",
  "prompt":"prompt",
  "mode":"style",
  "time":"day",
  "weather":"clear",
  "input_images":["var/projects/demo/scenes/a.png"]
}]"#,
        )
        .expect("jobs file should be written");

        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let tools_impl = Arc::new(FakeToolAdapters::with_qa_hard_failures());
        let tools: SharedPipelineToolAdapterOps = tools_impl.clone();
        let wrapper = RustRunModePipelineOrchestrator::new(inner.clone(), tools, app_root.clone());

        let err = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    jobs_file: Some(String::from("var/tmp/jobs.json")),
                    ..PipelineRunOptions::default()
                },
            })
            .expect_err("output-guard fail should return command failure");

        match err {
            PipelineRuntimeError::CommandFailed { stdout, stderr, .. } => {
                assert!(stdout.contains("Run log: "));
                assert!(stderr.contains("Output guard failed for 1 job(s)"));

                let summary = parse_script_run_summary_from_stdout(stdout.as_str())
                    .expect("summary should parse");
                let run_log_abs = app_root.join(summary.run_log_path);
                let raw =
                    fs::read_to_string(run_log_abs.as_path()).expect("run log should be readable");
                let run_log: serde_json::Value =
                    serde_json::from_str(raw.as_str()).expect("run log should parse");
                let job = run_log
                    .get("jobs")
                    .and_then(|v| v.as_array())
                    .and_then(|v| v.first())
                    .expect("job should exist");
                assert_eq!(
                    job.get("status").and_then(|v| v.as_str()),
                    Some("failed_output_guard")
                );
                assert_eq!(
                    job.get("failure_reason").and_then(|v| v.as_str()),
                    Some("all_candidates_failed_output_guard")
                );
                assert_eq!(job.get("final_output").and_then(|v| v.as_str()), None);
                let candidate = job
                    .get("candidates")
                    .and_then(|v| v.as_array())
                    .and_then(|v| v.first())
                    .expect("candidate should exist");
                assert_eq!(
                    candidate.get("status").and_then(|v| v.as_str()),
                    Some("failed_output_guard")
                );
                assert_eq!(
                    candidate
                        .get("output_guard")
                        .and_then(|v| v.get("bad_archive"))
                        .and_then(|v| v.as_str()),
                    Some("var/projects/demo/archive/bad/out.png")
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }

        assert_eq!(
            tools_impl
                .seen_archive
                .lock()
                .expect("fake tool adapters archive mutex poisoned")
                .len(),
            1
        );
        assert!(inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn rust_post_run_wrapper_ingests_rust_run_mode_failure_with_run_log() {
        let app_root = temp_app_root();
        fs::create_dir_all(app_root.join("var/tmp")).expect("tmp dir should exist");
        fs::write(
            app_root.join("var/tmp/jobs.json"),
            r#"[{
  "id":"manual_job_1",
  "prompt":"prompt",
  "mode":"style",
  "time":"day",
  "weather":"clear",
  "input_images":["var/projects/demo/scenes/a.png"]
}]"#,
        )
        .expect("jobs file should be written");

        let fallback_inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("should-not-run"));
        let tools_impl = Arc::new(FakeToolAdapters::with_qa_hard_failures());
        let tools: SharedPipelineToolAdapterOps = tools_impl.clone();
        let run_inner: SharedPipelineOrchestrator = Arc::new(RustRunModePipelineOrchestrator::new(
            fallback_inner.clone(),
            tools,
            app_root.clone(),
        ));
        let backend_ops = Arc::new(FakePostRunBackendOps::default());
        let post_run = PipelinePostRunService::new(backend_ops.clone());
        let wrapper = RustPostRunPipelineOrchestrator::new(run_inner, post_run)
            .with_app_root(app_root.clone());

        let err = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    jobs_file: Some(String::from("var/tmp/jobs.json")),
                    ..PipelineRunOptions::default()
                },
            })
            .expect_err("output-guard failure should be preserved");

        let (stdout, stderr) = match err {
            PipelineRuntimeError::CommandFailed { stdout, stderr, .. } => (stdout, stderr),
            other => panic!("unexpected error: {other:?}"),
        };
        assert!(stdout.contains("Run log: "));
        assert!(stdout.contains("KROMA_PIPELINE_SUMMARY_JSON:"));
        assert!(stderr.contains("Output guard failed for 1 job(s)"));
        assert!(!stderr.contains("Rust post-run ingest skipped"));

        let summary = parse_script_run_summary_from_stdout(stdout.as_str())
            .expect("summary should parse from rust run-mode stdout");
        let seen_ingest = backend_ops
            .seen_ingest
            .lock()
            .expect("fake post-run ingest mutex poisoned");
        assert_eq!(seen_ingest.len(), 1);
        assert_eq!(seen_ingest[0].project_slug, "demo");
        assert_eq!(seen_ingest[0].run_log_path, summary.run_log_path);
        drop(seen_ingest);

        assert!(app_root.join(summary.run_log_path).is_file());
        assert_eq!(
            tools_impl
                .seen_archive
                .lock()
                .expect("fake tool adapters archive mutex poisoned")
                .len(),
            1
        );
        assert!(fallback_inner
            .seen
            .lock()
            .expect("fake inner orchestrator mutex poisoned")
            .is_empty());

        let _ = fs::remove_dir_all(app_root);
    }

    #[test]
    fn list_image_files_recursive_returns_sorted_deterministic_order() {
        let root = temp_app_root();
        let input = root.join("inputs");
        fs::create_dir_all(input.join("z-dir")).expect("z-dir should exist");
        fs::create_dir_all(input.join("a-dir")).expect("a-dir should exist");
        fs::write(input.join("b.png"), b"b").expect("b should exist");
        fs::write(input.join("a.png"), b"a").expect("a should exist");
        fs::write(input.join("z-dir/2.jpg"), b"z2").expect("z2 should exist");
        fs::write(input.join("a-dir/1.jpg"), b"a1").expect("a1 should exist");

        let files =
            list_image_files_recursive(input.as_path()).expect("image file listing should work");
        let rels = files
            .iter()
            .map(|p| {
                p.strip_prefix(root.as_path())
                    .expect("path should be under root")
            })
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>();

        assert_eq!(
            rels,
            vec![
                String::from("inputs/a-dir/1.jpg"),
                String::from("inputs/a.png"),
                String::from("inputs/b.png"),
                String::from("inputs/z-dir/2.jpg")
            ]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn parses_script_run_summary_from_stdout_lines() {
        let parsed = parse_script_run_summary_from_stdout(
            "Run log: var/projects/demo/runs/run_1.json\nProject: demo\nProject root: var/projects/demo\nJobs: 3 (run/completed)\n",
        )
        .expect("summary should parse");

        assert_eq!(
            parsed.run_log_path,
            PathBuf::from("var/projects/demo/runs/run_1.json")
        );
        assert_eq!(parsed.project_slug.as_deref(), Some("demo"));
        assert_eq!(parsed.project_root.as_deref(), Some("var/projects/demo"));
        assert_eq!(parsed.jobs, Some(3));
    }

    #[test]
    fn parses_script_run_summary_from_marker_line() {
        let parsed = parse_script_run_summary_from_stdout(
            "KROMA_PIPELINE_SUMMARY_JSON: {\"run_log_path\":\"var/projects/demo/runs/run_1.json\",\"project_slug\":\"demo\",\"project_root\":\"var/projects/demo\",\"jobs\":3,\"mode\":\"run\"}\n",
        )
        .expect("marker summary should parse");

        assert_eq!(
            parsed.run_log_path,
            PathBuf::from("var/projects/demo/runs/run_1.json")
        );
        assert_eq!(parsed.project_slug.as_deref(), Some("demo"));
        assert_eq!(parsed.project_root.as_deref(), Some("var/projects/demo"));
        assert_eq!(parsed.jobs, Some(3));
    }
}
