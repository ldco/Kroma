use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use thiserror::Error;

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

#[cfg(test)]
mod tests {
    use super::*;
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
                stderr,
                ..
            } => {
                assert_eq!(status_code, 2);
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
}
