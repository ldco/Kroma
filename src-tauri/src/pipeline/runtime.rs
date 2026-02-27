use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use thiserror::Error;

#[cfg(test)]
use crate::pipeline::runlog_parse::parse_script_run_summary_from_stdout;
pub use crate::pipeline::runtime_orchestrators::{
    RustDryRunPipelineOrchestrator, RustPostRunPipelineOrchestrator,
    RustRunModePipelineOrchestrator,
};
pub use crate::pipeline::runtime_stack::{
    default_pipeline_orchestrator_with_native_post_run,
    default_pipeline_orchestrator_with_rust_post_run,
    default_pipeline_orchestrator_with_rust_post_run_backend_ops,
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

#[cfg(test)]
fn list_image_files_recursive(input_abs: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    crate::pipeline::pathing::list_image_files_recursive(input_abs)
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

pub(crate) fn validate_project_slug(value: &str) -> Result<(), PipelineRuntimeError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::ProjectsStore;
    use crate::pipeline::backend_ops::{
        BackendCommandResult, BackendIngestRunRequest, BackendOpsError,
        BackendSyncProjectS3Request, PipelineBackendOps,
    };
    use crate::pipeline::post_run::PipelinePostRunService;
    use crate::pipeline::tool_adapters::{
        ArchiveBadRequest, BackgroundRemovePassRequest, ColorPassRequest, GenerateOneImageRequest,
        PipelineToolAdapterOps, QaCheckRequest, SharedPipelineToolAdapterOps, ToolAdapterError,
        UpscalePassRequest,
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
    fn rust_post_run_wrapper_preserves_request_options_and_runs_backend_ingest() {
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
        assert_eq!(seen_request.options.backend_db_ingest, None);
        assert_eq!(seen_request.options.storage_sync_s3, None);

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
        assert_eq!(seen_request.options.backend_db_ingest, None);
        assert_eq!(seen_request.options.storage_sync_s3, Some(true));

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
