use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::pipeline::backend_ops::{default_script_backend_ops, SharedPipelineBackendOps};
use crate::pipeline::planning::{
    build_generation_jobs, default_planning_manifest, load_planning_manifest_file,
    PlannedGenerationJob,
};
use crate::pipeline::post_run::{
    PipelinePostRunService, PostRunFinalizeParams, PostRunIngestParams,
};
use crate::pipeline::runlog::{
    format_summary_marker, write_pretty_json_with_newline, PipelineRunSummaryMarkerPayload,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PipelineRunOptions {
    pub manifest_path: Option<String>,
    pub jobs_file: Option<String>,
    pub project_root: Option<String>,
    pub input_source: Option<PipelineInputSource>,
    pub style_refs: Vec<String>,
    pub stage: Option<PipelineStageFilter>,
    pub time: Option<PipelineTimeFilter>,
    pub weather: Option<PipelineWeatherFilter>,
    pub candidates: Option<u8>,
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

#[derive(Debug, Clone)]
pub struct ScriptPipelineOrchestrator<R> {
    runner: R,
    app_root: PathBuf,
    node_binary: String,
    script_rel_path: PathBuf,
}

impl<R> ScriptPipelineOrchestrator<R>
where
    R: PipelineCommandRunner,
{
    pub fn new(app_root: PathBuf, runner: R) -> Self {
        Self {
            runner,
            app_root,
            node_binary: String::from("node"),
            script_rel_path: PathBuf::from("scripts/image-lab.mjs"),
        }
    }

    pub fn with_node_binary(mut self, node_binary: impl Into<String>) -> Self {
        self.node_binary = node_binary.into();
        self
    }

    pub fn with_script_rel_path(mut self, script_rel_path: impl Into<PathBuf>) -> Self {
        self.script_rel_path = script_rel_path.into();
        self
    }

    pub fn build_command(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<CommandSpec, PipelineRuntimeError> {
        validate_project_slug(request.project_slug.as_str())?;

        let script_path = self.app_root.join(self.script_rel_path.as_path());
        if !script_path.is_file() {
            return Err(PipelineRuntimeError::ScriptNotFound(script_path));
        }

        let mut args = vec![
            script_path.to_string_lossy().to_string(),
            request.mode.as_str().to_string(),
            String::from("--project"),
            request.project_slug.clone(),
        ];
        if request.confirm_spend {
            args.push(String::from("--confirm-spend"));
        }
        append_pipeline_options_args(&mut args, &request.options);

        Ok(CommandSpec {
            program: self.node_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    fn run_rust_planning_preflight(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<Option<RustPlanningPreflightSummary>, PipelineRuntimeError> {
        build_rust_planning_preflight_summary(self.app_root.as_path(), request)
    }
}

fn build_rust_planning_preflight_summary(
    app_root: &Path,
    request: &PipelineRunRequest,
) -> Result<Option<RustPlanningPreflightSummary>, PipelineRuntimeError> {
    let has_manifest = request.options.manifest_path.is_some();
    let has_scene_refs = matches!(
        request.options.input_source,
        Some(PipelineInputSource::SceneRefs(_))
    );
    if !has_manifest && !has_scene_refs {
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
        Some(PipelineInputSource::InputPath(_)) => {
            // File/dir expansion is still script-owned.
            return Ok(None);
        }
        None => {}
    }

    if !request.options.style_refs.is_empty() {
        manifest.style_refs = request.options.style_refs.clone();
    }

    let stage = request.options.stage.unwrap_or(PipelineStageFilter::Style);
    let time = request.options.time.unwrap_or(PipelineTimeFilter::Day);
    let weather = request
        .options
        .weather
        .unwrap_or(PipelineWeatherFilter::Clear);
    let jobs = build_generation_jobs(&manifest, stage, time, weather).map_err(|error| {
        PipelineRuntimeError::PlanningPreflight(format!(
            "manifest planning preflight failed: {error}"
        ))
    })?;

    Ok(Some(RustPlanningPreflightSummary {
        job_ids: jobs.iter().map(|job| job.id.clone()).collect(),
        jobs,
    }))
}

fn append_pipeline_options_args(args: &mut Vec<String>, options: &PipelineRunOptions) {
    if let Some(manifest_path) = options.manifest_path.as_ref() {
        args.push(String::from("--manifest"));
        args.push(manifest_path.clone());
    }
    if let Some(jobs_file) = options.jobs_file.as_ref() {
        args.push(String::from("--jobs-file"));
        args.push(jobs_file.clone());
    }

    if let Some(project_root) = options.project_root.as_ref() {
        args.push(String::from("--project-root"));
        args.push(project_root.clone());
    }

    if let Some(input_source) = options.input_source.as_ref() {
        match input_source {
            PipelineInputSource::InputPath(path) => {
                args.push(String::from("--input"));
                args.push(path.clone());
            }
            PipelineInputSource::SceneRefs(values) => {
                args.push(String::from("--scene-refs"));
                args.push(values.join(","));
            }
        }
    }

    if !options.style_refs.is_empty() {
        args.push(String::from("--style-refs"));
        args.push(options.style_refs.join(","));
    }

    if let Some(stage) = options.stage {
        args.push(String::from("--stage"));
        args.push(stage.as_str().to_string());
    }

    if let Some(time) = options.time {
        args.push(String::from("--time"));
        args.push(time.as_str().to_string());
    }

    if let Some(weather) = options.weather {
        args.push(String::from("--weather"));
        args.push(weather.as_str().to_string());
    }

    if let Some(candidates) = options.candidates {
        args.push(String::from("--candidates"));
        args.push(candidates.to_string());
    }

    if let Some(enabled) = options.backend_db_ingest {
        args.push(String::from("--backend-db-ingest"));
        args.push(enabled.to_string());
    }

    if let Some(enabled) = options.storage_sync_s3 {
        args.push(String::from("--storage-sync-s3"));
        args.push(enabled.to_string());
    }
}

#[derive(Clone)]
pub struct RustPostRunPipelineOrchestrator {
    inner: SharedPipelineOrchestrator,
    post_run: PipelinePostRunService,
}

impl RustPostRunPipelineOrchestrator {
    pub fn new(inner: SharedPipelineOrchestrator, post_run: PipelinePostRunService) -> Self {
        Self { inner, post_run }
    }

    fn build_script_request(&self, request: &PipelineRunRequest) -> PipelineRunRequest {
        let mut script_request = request.clone();
        // Rust owns backend ingest for the typed HTTP trigger path; prevent duplicate script ingest.
        script_request.options.backend_db_ingest = Some(false);
        // Keep S3 sync disabled until the Rust path owns sync policy/options end-to-end.
        script_request.options.storage_sync_s3 = Some(false);
        script_request
    }

    fn run_post_run_ingest_best_effort(
        &self,
        request: &PipelineRunRequest,
        stdout: &str,
        stderr: &mut String,
    ) {
        let Some(summary) = parse_script_run_summary_from_stdout(stdout) else {
            append_stderr_line(
                stderr,
                "Rust post-run ingest skipped: missing 'Run log:' line in pipeline stdout",
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

        let finalize = self.post_run.finalize_run(PostRunFinalizeParams {
            ingest: PostRunIngestParams {
                run_log_path: summary.run_log_path,
                project_slug: request.project_slug.clone(),
                project_name: request.project_slug.clone(),
                create_project_if_missing: true,
                compute_hashes: false,
            },
            sync_s3: None,
        });

        if let Err(error) = finalize {
            append_stderr_line(stderr, format!("Rust post-run ingest skipped: {error}"));
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

impl PipelineOrchestrator for RustPostRunPipelineOrchestrator {
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<PipelineRunResult, PipelineRuntimeError> {
        match self.inner.execute(&self.build_script_request(request)) {
            Ok(mut result) => {
                self.run_post_run_ingest_best_effort(
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
                self.run_post_run_ingest_best_effort(request, stdout.as_str(), &mut stderr);
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
        let Some(planned) =
            build_rust_planning_preflight_summary(self.app_root.as_path(), request)?
        else {
            return self.inner.execute(request);
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
        let runs_dir = project_root_abs.join("runs");
        fs::create_dir_all(runs_dir.as_path()).map_err(PipelineRuntimeError::Io)?;
        let run_log_path_abs = runs_dir.join(format!("run_{}.json", make_run_log_stamp()));

        let stage = request.options.stage.unwrap_or(PipelineStageFilter::Style);
        let time = request.options.time.unwrap_or(PipelineTimeFilter::Day);
        let weather = request
            .options
            .weather
            .unwrap_or(PipelineWeatherFilter::Clear);
        let candidate_count = u64::from(request.options.candidates.unwrap_or(1));
        let project_root_display =
            path_for_output(self.app_root.as_path(), project_root_abs.as_path());
        let run_log_display = path_for_output(self.app_root.as_path(), run_log_path_abs.as_path());
        let timestamp = iso_like_timestamp();

        let jobs_json = planned
            .jobs
            .iter()
            .map(|job| {
                serde_json::json!({
                    "id": job.id,
                    "mode": job.mode,
                    "time": job.time,
                    "weather": job.weather,
                    "input_images": job.input_images,
                    "prompt": job.prompt,
                    "status": "planned",
                    "planned_generation": { "candidates": candidate_count },
                    "planned_postprocess": {
                        "upscale": false,
                        "upscale_backend": serde_json::Value::Null,
                        "color": false,
                        "color_profile": serde_json::Value::Null,
                        "bg_remove": false,
                        "bg_remove_backends": [],
                        "bg_refine_openai": false,
                        "bg_refine_openai_required": false,
                        "pipeline_order": ["generate"]
                    },
                    "planned_output_guard": {
                        "enabled": true,
                        "enforce_grayscale": false,
                        "max_chroma_delta": 2,
                        "fail_on_chroma_exceed": false
                    }
                })
            })
            .collect::<Vec<_>>();

        let run_meta = serde_json::json!({
            "timestamp": timestamp,
            "project": request.project_slug,
            "mode": "dry",
            "stage": stage.as_str(),
            "time": time.as_str(),
            "weather": weather.as_str(),
            "model": "",
            "size": "",
            "quality": "",
            "generation": {
                "candidates": candidate_count,
                "max_candidates": candidate_count
            },
            "postprocess": {
                "upscale": false,
                "upscale_backend": serde_json::Value::Null,
                "color": false,
                "color_profile": serde_json::Value::Null,
                "bg_remove": false,
                "bg_remove_backends": [],
                "bg_refine_openai": false,
                "bg_refine_openai_required": false,
                "pipeline_order": ["generate"]
            },
            "output_guard": {
                "enabled": true,
                "enforce_grayscale": false,
                "max_chroma_delta": 2,
                "fail_on_chroma_exceed": false
            },
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PipelineScriptRunSummary {
    run_log_path: PathBuf,
    project_slug: Option<String>,
    project_root: Option<String>,
    jobs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustPlanningPreflightSummary {
    job_ids: Vec<String>,
    jobs: Vec<PlannedGenerationJob>,
}

impl RustPlanningPreflightSummary {
    fn job_count(&self) -> u64 {
        self.job_ids.len() as u64
    }

    fn ids_preview(&self, limit: usize) -> String {
        self.job_ids
            .iter()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
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

impl<R> PipelineOrchestrator for ScriptPipelineOrchestrator<R>
where
    R: PipelineCommandRunner,
{
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<PipelineRunResult, PipelineRuntimeError> {
        let mut script_request = request.clone();
        let mut temp_jobs_file = None::<PathBuf>;
        let mut spec = self.build_command(request)?;
        let planned_jobs = self.run_rust_planning_preflight(request)?;
        if let Some(planned) = planned_jobs.as_ref() {
            if request.options.manifest_path.is_some() && !planned.jobs.is_empty() {
                let jobs_file =
                    write_planned_jobs_temp_file(self.app_root.as_path(), &planned.jobs)?;
                script_request.options.jobs_file = Some(jobs_file.to_string_lossy().to_string());
                spec = self.build_command(&script_request)?;
                temp_jobs_file = Some(jobs_file);
            }
        }
        let output = match self.runner.run(&spec) {
            Ok(output) => output,
            Err(error) => {
                if let Some(path) = temp_jobs_file.take() {
                    let _ = fs::remove_file(path);
                }
                return Err(error);
            }
        };
        if let Some(path) = temp_jobs_file.take() {
            let _ = fs::remove_file(path);
        }
        if output.status_code != 0 {
            return Err(PipelineRuntimeError::CommandFailed {
                program: spec.program,
                status_code: output.status_code,
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }

        let mut result = PipelineRunResult {
            status_code: output.status_code,
            stdout: output.stdout,
            stderr: output.stderr,
        };
        if let Some(planned) = planned_jobs {
            if let Some(summary) = parse_script_run_summary_from_stdout(result.stdout.as_str()) {
                if let Some(actual_jobs) = summary.jobs {
                    let expected_jobs = planned.job_count();
                    if actual_jobs != expected_jobs {
                        let ids_preview = planned.ids_preview(3);
                        append_stderr_line(
                            &mut result.stderr,
                            format!(
                                "Rust planning preflight warning: planned {expected_jobs} jobs but script reported {actual_jobs}{}",
                                if ids_preview.is_empty() {
                                    String::new()
                                } else {
                                    format!(" (planned ids: {ids_preview})")
                                }
                            ),
                        );
                    }
                }
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum PipelineRuntimeError {
    #[error("invalid project slug for pipeline run")]
    InvalidProjectSlug,
    #[error("pipeline script not found: {0}")]
    ScriptNotFound(PathBuf),
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
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        app_root.join(path)
    }
}

fn write_planned_jobs_temp_file(
    app_root: &Path,
    jobs: &[PlannedGenerationJob],
) -> Result<PathBuf, PipelineRuntimeError> {
    let dir = app_root.join("var/tmp");
    fs::create_dir_all(dir.as_path()).map_err(|error| {
        PipelineRuntimeError::PlannedJobsTempFile(format!(
            "create dir '{}': {error}",
            dir.display()
        ))
    })?;
    let path = dir.join(format!("pipeline_jobs_{}.json", Uuid::new_v4()));
    let payload = serde_json::to_vec_pretty(jobs).map_err(|error| {
        PipelineRuntimeError::PlannedJobsTempFile(format!(
            "serialize planned jobs '{}': {error}",
            path.display()
        ))
    })?;
    fs::write(path.as_path(), payload).map_err(|error| {
        PipelineRuntimeError::PlannedJobsTempFile(format!(
            "write file '{}': {error}",
            path.display()
        ))
    })?;
    Ok(path)
}

fn normalize_relish_path(value: &str) -> String {
    value.replace('\\', "/")
}

fn path_for_output(app_root: &Path, path: &Path) -> String {
    match path.strip_prefix(app_root) {
        Ok(rel) => normalize_relish_path(rel.to_string_lossy().as_ref()),
        Err(_) => normalize_relish_path(path.to_string_lossy().as_ref()),
    }
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

pub fn default_script_pipeline_orchestrator() -> ScriptPipelineOrchestrator<StdPipelineCommandRunner>
{
    ScriptPipelineOrchestrator::new(
        default_app_root_from_manifest_dir(),
        StdPipelineCommandRunner,
    )
}

pub fn default_pipeline_orchestrator_with_rust_post_run() -> RustPostRunPipelineOrchestrator {
    let backend_ops: SharedPipelineBackendOps = Arc::new(default_script_backend_ops());
    default_pipeline_orchestrator_with_rust_post_run_backend_ops(backend_ops)
}

pub fn default_pipeline_orchestrator_with_rust_post_run_backend_ops(
    backend_ops: SharedPipelineBackendOps,
) -> RustPostRunPipelineOrchestrator {
    let app_root = default_app_root_from_manifest_dir();
    let script_inner: SharedPipelineOrchestrator = Arc::new(default_script_pipeline_orchestrator());
    let inner: SharedPipelineOrchestrator =
        Arc::new(RustDryRunPipelineOrchestrator::new(script_inner, app_root));
    let post_run = PipelinePostRunService::new(backend_ops);
    RustPostRunPipelineOrchestrator::new(inner, post_run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::backend_ops::{
        BackendCommandResult, BackendIngestRunRequest, BackendOpsError,
        BackendSyncProjectS3Request, PipelineBackendOps,
    };
    use serde_json::json;
    use std::fs;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone, Default)]
    struct FakeRunner {
        seen: Arc<Mutex<Vec<CommandSpec>>>,
        next: Arc<Mutex<Option<Result<CommandOutput, PipelineRuntimeError>>>>,
    }

    impl FakeRunner {
        fn with_next(result: Result<CommandOutput, PipelineRuntimeError>) -> Self {
            Self {
                seen: Arc::new(Mutex::new(Vec::new())),
                next: Arc::new(Mutex::new(Some(result))),
            }
        }

        fn take_seen(&self) -> Vec<CommandSpec> {
            std::mem::take(&mut *self.seen.lock().expect("fake runner mutex poisoned"))
        }
    }

    impl PipelineCommandRunner for FakeRunner {
        fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, PipelineRuntimeError> {
            self.seen
                .lock()
                .expect("fake runner mutex poisoned")
                .push(spec.clone());
            self.next
                .lock()
                .expect("fake runner mutex poisoned")
                .take()
                .unwrap_or_else(|| {
                    Ok(CommandOutput {
                        status_code: 0,
                        stdout: String::new(),
                        stderr: String::new(),
                    })
                })
        }
    }

    fn test_orchestrator(runner: FakeRunner) -> ScriptPipelineOrchestrator<FakeRunner> {
        let app_root = default_app_root_from_manifest_dir();
        ScriptPipelineOrchestrator::new(app_root, runner)
    }

    fn temp_manifest_file(contents: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("kroma_runtime_manifest_{stamp}.json"));
        fs::write(path.as_path(), contents).expect("temp manifest should be written");
        path
    }

    fn temp_app_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("kroma_runtime_app_root_{stamp}"));
        fs::create_dir_all(path.as_path()).expect("temp app root should exist");
        path
    }

    #[test]
    fn builds_script_command_for_dry_mode() {
        let orchestrator = test_orchestrator(FakeRunner::default());
        let cmd = orchestrator
            .build_command(&PipelineRunRequest {
                project_slug: String::from("demo-project"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    project_root: Some(String::from("/tmp/demo")),
                    input_source: Some(PipelineInputSource::SceneRefs(vec![
                        String::from("a.png"),
                        String::from("b.png"),
                    ])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("command should build");

        assert_eq!(cmd.program, "node");
        assert_eq!(cmd.args[1], "dry");
        assert_eq!(cmd.args[2], "--project");
        assert_eq!(cmd.args[3], "demo-project");
        assert_eq!(cmd.args[4], "--project-root");
    }

    #[test]
    fn rejects_invalid_project_slug() {
        let orchestrator = test_orchestrator(FakeRunner::default());
        let err = orchestrator
            .build_command(&PipelineRunRequest {
                project_slug: String::from("bad slug"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions::default(),
            })
            .expect_err("invalid slug should be rejected");
        assert!(matches!(err, PipelineRuntimeError::InvalidProjectSlug));
    }

    #[test]
    fn executes_and_returns_output_on_success() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from("ok"),
            stderr: String::new(),
        }));
        let orchestrator = test_orchestrator(runner.clone());

        let result = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("execution should succeed");

        assert_eq!(result.status_code, 0);
        assert_eq!(result.stdout, "ok");
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].args[1], "dry");
    }

    #[test]
    fn reports_non_zero_exit_as_error() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 2,
            stdout: String::new(),
            stderr: String::from("usage error"),
        }));
        let orchestrator = test_orchestrator(runner);

        let err = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions::default(),
            })
            .expect_err("non-zero exit should fail");

        match err {
            PipelineRuntimeError::CommandFailed {
                status_code,
                stdout,
                stderr,
                ..
            } => {
                assert_eq!(status_code, 2);
                assert_eq!(stdout, "");
                assert_eq!(stderr, "usage error");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn build_command_includes_confirm_spend_for_run_mode_request() {
        let orchestrator = test_orchestrator(FakeRunner::default());
        let cmd = orchestrator
            .build_command(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions::default(),
            })
            .expect("command should build");

        assert!(cmd.args.iter().any(|arg| arg == "--confirm-spend"));
    }

    #[test]
    fn build_command_includes_internal_backend_flags_when_set() {
        let orchestrator = test_orchestrator(FakeRunner::default());
        let cmd = orchestrator
            .build_command(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Run,
                confirm_spend: true,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    backend_db_ingest: Some(false),
                    storage_sync_s3: Some(false),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("command should build");

        assert!(cmd
            .args
            .windows(2)
            .any(|w| w == ["--backend-db-ingest", "false"]));
        assert!(cmd
            .args
            .windows(2)
            .any(|w| w == ["--storage-sync-s3", "false"]));
    }

    #[test]
    fn build_command_includes_manifest_path_when_set() {
        let orchestrator = test_orchestrator(FakeRunner::default());
        let cmd = orchestrator
            .build_command(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    manifest_path: Some(String::from("config/manifest.json")),
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("command should build");

        assert!(cmd
            .args
            .windows(2)
            .any(|w| w == ["--manifest", "config/manifest.json"]));
    }

    #[test]
    fn execute_rejects_invalid_manifest_preflight_before_runner() {
        let runner = FakeRunner::default();
        let orchestrator = test_orchestrator(runner.clone());
        let manifest_path =
            temp_manifest_file(r#"{"prompts":{"style_base":123},"scene_refs":["a.png"]}"#);

        let err = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect_err("invalid manifest preflight should fail");

        match err {
            PipelineRuntimeError::PlanningPreflight(message) => {
                assert!(message.contains("manifest parse failed"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(runner.take_seen().is_empty());

        let _ = fs::remove_file(manifest_path);
    }

    #[test]
    fn execute_preserves_invalid_project_slug_precedence_over_manifest_preflight() {
        let runner = FakeRunner::default();
        let orchestrator = test_orchestrator(runner.clone());
        let manifest_path = temp_manifest_file(r#"{"prompts":{"style_base":123}}"#);

        let err = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("bad slug"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect_err("invalid slug should be rejected before preflight");

        assert!(matches!(err, PipelineRuntimeError::InvalidProjectSlug));
        assert!(runner.take_seen().is_empty());

        let _ = fs::remove_file(manifest_path);
    }

    #[test]
    fn execute_runs_when_manifest_preflight_succeeds() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from("ok"),
            stderr: String::new(),
        }));
        let orchestrator = test_orchestrator(runner.clone());
        let manifest_path =
            temp_manifest_file(r#"{"prompts":{"style_base":"ok"},"scene_refs":["a.png"]}"#);

        let result = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                    input_source: Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")])),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("execution should succeed");

        assert_eq!(result.status_code, 0);
        assert_eq!(runner.take_seen().len(), 1);

        let _ = fs::remove_file(manifest_path);
    }

    #[test]
    fn execute_injects_jobs_file_when_rust_planning_produces_jobs() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }));
        let orchestrator = test_orchestrator(runner.clone());
        let manifest_path =
            temp_manifest_file(r#"{"scene_refs":["a.png"],"prompts":{"style_base":"ok"}}"#);

        let _ = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("execution should succeed");

        let seen = runner.take_seen();
        assert_eq!(seen.len(), 1);
        assert!(seen[0].args.windows(2).any(|w| w[0] == "--jobs-file"));

        let _ = fs::remove_file(manifest_path);
    }

    #[test]
    fn execute_warns_when_manifest_planned_jobs_mismatch_script_summary() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from(
                "KROMA_PIPELINE_SUMMARY_JSON: {\"run_log_path\":\"var/projects/demo/runs/run_1.json\",\"project_slug\":\"demo\",\"jobs\":2}\n",
            ),
            stderr: String::new(),
        }));
        let orchestrator = test_orchestrator(runner.clone());
        let manifest_path =
            temp_manifest_file(r#"{"scene_refs":["a.png"],"prompts":{"style_base":"ok"}}"#);

        let result = orchestrator
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("execution should succeed with warning");

        assert!(result
            .stderr
            .contains("Rust planning preflight warning: planned 1 jobs but script reported 2"));
        assert_eq!(runner.take_seen().len(), 1);

        let _ = fs::remove_file(manifest_path);
    }

    #[derive(Default)]
    struct FakePostRunBackendOps {
        seen_ingest: Mutex<Vec<BackendIngestRunRequest>>,
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
            _request: &BackendSyncProjectS3Request,
        ) -> Result<BackendCommandResult, BackendOpsError> {
            panic!("sync_project_s3 should not be called by wrapper yet");
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
            .contains("Rust post-run ingest skipped: missing 'Run log:' line"));
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
    fn rust_dry_run_wrapper_delegates_input_path_mode_to_inner() {
        let app_root = temp_app_root();
        let inner = Arc::new(FakeInnerOrchestrator::with_success_stdout("ok"));
        let wrapper = RustDryRunPipelineOrchestrator::new(inner.clone(), app_root.clone());

        let result = wrapper
            .execute(&PipelineRunRequest {
                project_slug: String::from("demo"),
                mode: PipelineRunMode::Dry,
                confirm_spend: false,
                options: PipelineRunOptions {
                    input_source: Some(PipelineInputSource::InputPath(String::from("input-dir"))),
                    ..PipelineRunOptions::default()
                },
            })
            .expect("wrapper should delegate and succeed");

        assert_eq!(result.stdout, "ok");
        assert_eq!(
            inner
                .seen
                .lock()
                .expect("fake inner orchestrator mutex poisoned")
                .len(),
            1
        );

        let _ = fs::remove_dir_all(app_root);
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
