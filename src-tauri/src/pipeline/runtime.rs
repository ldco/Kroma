use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde::Deserialize;
use thiserror::Error;

use crate::pipeline::backend_ops::default_script_backend_ops;
use crate::pipeline::post_run::{
    PipelinePostRunService, PostRunFinalizeParams, PostRunIngestParams,
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
}

fn append_pipeline_options_args(args: &mut Vec<String>, options: &PipelineRunOptions) {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PipelineScriptRunSummary {
    run_log_path: PathBuf,
    project_slug: Option<String>,
    project_root: Option<String>,
    jobs: Option<u64>,
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
        let spec = self.build_command(request)?;
        let output = self.runner.run(&spec)?;
        if output.status_code != 0 {
            return Err(PipelineRuntimeError::CommandFailed {
                program: spec.program,
                status_code: output.status_code,
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }

        Ok(PipelineRunResult {
            status_code: output.status_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
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
    let inner: SharedPipelineOrchestrator = Arc::new(default_script_pipeline_orchestrator());
    let backend_ops = Arc::new(default_script_backend_ops());
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
    use std::sync::{Arc, Mutex};

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
