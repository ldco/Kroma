use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::pipeline::runtime::{
    default_app_root_from_manifest_dir, CommandOutput, CommandSpec, PipelineCommandRunner,
    PipelineRuntimeError, StdPipelineCommandRunner,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateOneImageRequest {
    pub project_slug: String,
    pub project_root: Option<String>,
    pub prompt: String,
    pub input_images_file: String,
    pub output_path: String,
    pub model: Option<String>,
    pub size: Option<String>,
    pub quality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerateOneImageResponse {
    pub ok: bool,
    pub project: String,
    pub output: String,
    pub input_images: Vec<String>,
    pub model: String,
    pub size: String,
    pub quality: String,
    pub bytes_written: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpscalePassRequest {
    pub project_slug: String,
    pub project_root: Option<String>,
    pub input_path: String,
    pub output_path: String,
    pub postprocess_config_path: Option<String>,
    pub upscale_backend: Option<String>,
    pub upscale_scale: Option<u8>,
    pub upscale_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpscalePassResponse {
    pub backend: String,
    pub input: String,
    pub output: String,
    pub scale: u64,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorPassRequest {
    pub project_slug: String,
    pub project_root: Option<String>,
    pub input_path: String,
    pub output_path: String,
    pub postprocess_config_path: Option<String>,
    pub profile: Option<String>,
    pub color_settings_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorPassResponse {
    pub input: String,
    pub output: String,
    pub profile: String,
    pub settings: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundRemovePassRequest {
    pub project_slug: String,
    pub project_root: Option<String>,
    pub input_path: String,
    pub output_path: String,
    pub postprocess_config_path: Option<String>,
    pub backends: Vec<String>,
    pub bg_refine_openai: Option<bool>,
    pub bg_refine_openai_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackgroundRemovePassFileResult {
    pub input: String,
    pub output: String,
    pub backend: String,
    pub refine_openai: bool,
    pub refine_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackgroundRemovePassResponse {
    pub input: String,
    pub output: String,
    pub backends: Vec<String>,
    pub refine_openai: bool,
    pub refine_openai_required: bool,
    pub format: String,
    pub processed: u64,
    pub results: Vec<BackgroundRemovePassFileResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QaCheckRequest {
    pub project_slug: String,
    pub project_root: Option<String>,
    pub input_path: String,
    pub manifest_path: Option<String>,
    pub output_guard_enabled: Option<bool>,
    pub enforce_grayscale: Option<bool>,
    pub max_chroma_delta: Option<f64>,
    pub fail_on_chroma_exceed: Option<bool>,
    pub qa_python_bin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QaCheckSummary {
    pub total_files: u64,
    pub hard_failures: u64,
    pub soft_warnings: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QaCheckResponse {
    pub ok: bool,
    #[serde(default)]
    pub skipped: Option<bool>,
    pub enabled: bool,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub has_hard_failures: Option<bool>,
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default)]
    pub summary: Option<QaCheckSummary>,
    #[serde(default)]
    pub report: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveBadRequest {
    pub project_slug: String,
    pub project_root: Option<String>,
    pub input_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchiveBadMovedFile {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchiveBadResponse {
    pub ok: bool,
    pub archive_dir: String,
    pub moved_count: u64,
    pub moved: Vec<ArchiveBadMovedFile>,
}

#[derive(Debug, Clone)]
pub struct ScriptPipelineToolAdapters<R> {
    runner: R,
    app_root: PathBuf,
    node_binary: String,
    script_rel_path: PathBuf,
}

impl<R> ScriptPipelineToolAdapters<R>
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

    pub fn build_generate_one_command(
        &self,
        request: &GenerateOneImageRequest,
    ) -> Result<CommandSpec, ToolAdapterError> {
        let mut args = self.base_args(
            "generate-one",
            request.project_slug.as_str(),
            request.project_root.as_deref(),
        )?;
        args.push(String::from("--prompt"));
        args.push(request.prompt.clone());
        args.push(String::from("--input-images-file"));
        args.push(request.input_images_file.clone());
        args.push(String::from("--output"));
        args.push(request.output_path.clone());
        if let Some(model) = request.model.as_ref() {
            args.push(String::from("--model"));
            args.push(model.clone());
        }
        if let Some(size) = request.size.as_ref() {
            args.push(String::from("--size"));
            args.push(size.clone());
        }
        if let Some(quality) = request.quality.as_ref() {
            args.push(String::from("--quality"));
            args.push(quality.clone());
        }
        args.push(String::from("--json"));
        Ok(CommandSpec {
            program: self.node_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    pub fn build_upscale_command(
        &self,
        request: &UpscalePassRequest,
    ) -> Result<CommandSpec, ToolAdapterError> {
        let mut args = self.base_args(
            "upscale",
            request.project_slug.as_str(),
            request.project_root.as_deref(),
        )?;
        args.push(String::from("--input"));
        args.push(request.input_path.clone());
        args.push(String::from("--output"));
        args.push(request.output_path.clone());
        if let Some(cfg) = request.postprocess_config_path.as_ref() {
            args.push(String::from("--postprocess-config"));
            args.push(cfg.clone());
        }
        if let Some(backend) = request.upscale_backend.as_ref() {
            args.push(String::from("--upscale-backend"));
            args.push(backend.clone());
        }
        if let Some(scale) = request.upscale_scale {
            args.push(String::from("--upscale-scale"));
            args.push(scale.to_string());
        }
        if let Some(format) = request.upscale_format.as_ref() {
            args.push(String::from("--upscale-format"));
            args.push(format.clone());
        }
        args.push(String::from("--json"));
        Ok(CommandSpec {
            program: self.node_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    pub fn build_color_command(
        &self,
        request: &ColorPassRequest,
    ) -> Result<CommandSpec, ToolAdapterError> {
        let mut args = self.base_args(
            "color",
            request.project_slug.as_str(),
            request.project_root.as_deref(),
        )?;
        args.push(String::from("--input"));
        args.push(request.input_path.clone());
        args.push(String::from("--output"));
        args.push(request.output_path.clone());
        if let Some(cfg) = request.postprocess_config_path.as_ref() {
            args.push(String::from("--postprocess-config"));
            args.push(cfg.clone());
        }
        if let Some(profile) = request.profile.as_ref() {
            args.push(String::from("--profile"));
            args.push(profile.clone());
        }
        if let Some(settings) = request.color_settings_path.as_ref() {
            args.push(String::from("--color-settings"));
            args.push(settings.clone());
        }
        args.push(String::from("--json"));
        Ok(CommandSpec {
            program: self.node_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    pub fn build_bgremove_command(
        &self,
        request: &BackgroundRemovePassRequest,
    ) -> Result<CommandSpec, ToolAdapterError> {
        let mut args = self.base_args(
            "bgremove",
            request.project_slug.as_str(),
            request.project_root.as_deref(),
        )?;
        args.push(String::from("--input"));
        args.push(request.input_path.clone());
        args.push(String::from("--output"));
        args.push(request.output_path.clone());
        if let Some(cfg) = request.postprocess_config_path.as_ref() {
            args.push(String::from("--postprocess-config"));
            args.push(cfg.clone());
        }
        if !request.backends.is_empty() {
            args.push(String::from("--bg-remove-backends"));
            args.push(request.backends.join(","));
        }
        if let Some(enabled) = request.bg_refine_openai {
            args.push(String::from("--bg-refine-openai"));
            args.push(enabled.to_string());
        }
        if let Some(enabled) = request.bg_refine_openai_required {
            args.push(String::from("--bg-refine-openai-required"));
            args.push(enabled.to_string());
        }
        args.push(String::from("--json"));
        Ok(CommandSpec {
            program: self.node_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    pub fn build_qa_command(
        &self,
        request: &QaCheckRequest,
    ) -> Result<CommandSpec, ToolAdapterError> {
        let mut args = self.base_args(
            "qa",
            request.project_slug.as_str(),
            request.project_root.as_deref(),
        )?;
        args.push(String::from("--input"));
        args.push(request.input_path.clone());
        if let Some(manifest) = request.manifest_path.as_ref() {
            args.push(String::from("--manifest"));
            args.push(manifest.clone());
        }
        if let Some(enabled) = request.output_guard_enabled {
            args.push(String::from("--output-guard-enabled"));
            args.push(enabled.to_string());
        }
        if let Some(enabled) = request.enforce_grayscale {
            args.push(String::from("--enforce-grayscale"));
            args.push(enabled.to_string());
        }
        if let Some(value) = request.max_chroma_delta {
            args.push(String::from("--max-chroma-delta"));
            args.push(value.to_string());
        }
        if let Some(enabled) = request.fail_on_chroma_exceed {
            args.push(String::from("--fail-on-chroma-exceed"));
            args.push(enabled.to_string());
        }
        if let Some(bin) = request.qa_python_bin.as_ref() {
            args.push(String::from("--qa-python-bin"));
            args.push(bin.clone());
        }
        args.push(String::from("--json"));
        Ok(CommandSpec {
            program: self.node_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    pub fn build_archive_bad_command(
        &self,
        request: &ArchiveBadRequest,
    ) -> Result<CommandSpec, ToolAdapterError> {
        let mut args = self.base_args(
            "archive-bad",
            request.project_slug.as_str(),
            request.project_root.as_deref(),
        )?;
        args.push(String::from("--input"));
        args.push(request.input_path.clone());
        args.push(String::from("--json"));
        Ok(CommandSpec {
            program: self.node_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    pub fn generate_one_typed(
        &self,
        request: &GenerateOneImageRequest,
    ) -> Result<GenerateOneImageResponse, ToolAdapterError> {
        let spec = self.build_generate_one_command(request)?;
        self.run_json_command(spec)
    }

    pub fn upscale_typed(
        &self,
        request: &UpscalePassRequest,
    ) -> Result<UpscalePassResponse, ToolAdapterError> {
        let spec = self.build_upscale_command(request)?;
        self.run_json_command(spec)
    }

    pub fn color_typed(
        &self,
        request: &ColorPassRequest,
    ) -> Result<ColorPassResponse, ToolAdapterError> {
        let spec = self.build_color_command(request)?;
        self.run_json_command(spec)
    }

    pub fn bgremove_typed(
        &self,
        request: &BackgroundRemovePassRequest,
    ) -> Result<BackgroundRemovePassResponse, ToolAdapterError> {
        let spec = self.build_bgremove_command(request)?;
        self.run_json_command(spec)
    }

    pub fn qa_typed(&self, request: &QaCheckRequest) -> Result<QaCheckResponse, ToolAdapterError> {
        let spec = self.build_qa_command(request)?;
        self.run_json_command(spec)
    }

    pub fn archive_bad_typed(
        &self,
        request: &ArchiveBadRequest,
    ) -> Result<ArchiveBadResponse, ToolAdapterError> {
        let spec = self.build_archive_bad_command(request)?;
        self.run_json_command(spec)
    }

    fn base_args(
        &self,
        mode: &str,
        project_slug: &str,
        project_root: Option<&str>,
    ) -> Result<Vec<String>, ToolAdapterError> {
        validate_project_slug(project_slug)?;
        let script_path = self.script_abs_path()?;
        let mut args = vec![
            script_path.to_string_lossy().to_string(),
            mode.to_string(),
            String::from("--project"),
            project_slug.to_string(),
        ];
        if let Some(project_root) = project_root {
            args.push(String::from("--project-root"));
            args.push(project_root.to_string());
        }
        Ok(args)
    }

    fn script_abs_path(&self) -> Result<PathBuf, ToolAdapterError> {
        let script_path = self.app_root.join(self.script_rel_path.as_path());
        if !script_path.is_file() {
            return Err(ToolAdapterError::ScriptNotFound(script_path));
        }
        Ok(script_path)
    }

    fn run_json_command<T>(&self, spec: CommandSpec) -> Result<T, ToolAdapterError>
    where
        T: DeserializeOwned,
    {
        let output = self
            .runner
            .run(&spec)
            .map_err(ToolAdapterError::CommandRunner)?;
        if output.status_code != 0 {
            return Err(ToolAdapterError::CommandFailed {
                program: spec.program,
                status_code: output.status_code,
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }

        serde_json::from_str(output.stdout.as_str()).map_err(|source| {
            ToolAdapterError::JsonDecode {
                source,
                stdout: output.stdout,
            }
        })
    }
}

fn validate_project_slug(value: &str) -> Result<(), ToolAdapterError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ToolAdapterError::InvalidProjectSlug);
    }
    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        Ok(())
    } else {
        Err(ToolAdapterError::InvalidProjectSlug)
    }
}

#[derive(Debug, Error)]
pub enum ToolAdapterError {
    #[error("invalid project slug for pipeline tool adapter")]
    InvalidProjectSlug,
    #[error("image-lab script not found: {0}")]
    ScriptNotFound(PathBuf),
    #[error("tool adapter command runner error: {0}")]
    CommandRunner(#[source] PipelineRuntimeError),
    #[error("tool adapter command failed ({program}) with exit code {status_code}: {stderr}")]
    CommandFailed {
        program: String,
        status_code: i32,
        stdout: String,
        stderr: String,
    },
    #[error("tool adapter JSON decode failed: {source}")]
    JsonDecode {
        #[source]
        source: serde_json::Error,
        stdout: String,
    },
}

pub fn default_script_pipeline_tool_adapters(
) -> ScriptPipelineToolAdapters<StdPipelineCommandRunner> {
    ScriptPipelineToolAdapters::new(
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

    fn temp_app_root_with_script() -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_tool_adapters_{stamp}"));
        let scripts = root.join("scripts");
        std::fs::create_dir_all(&scripts).expect("scripts dir should exist");
        std::fs::write(scripts.join("image-lab.mjs"), b"// test")
            .expect("script file should exist");
        root
    }

    #[test]
    fn builds_generate_one_command_with_json_output() {
        let app_root = temp_app_root_with_script();
        let adapters = ScriptPipelineToolAdapters::new(app_root.clone(), FakeRunner::default());
        let cmd = adapters
            .build_generate_one_command(&GenerateOneImageRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                prompt: String::from("hello"),
                input_images_file: String::from("var/tmp/in.json"),
                output_path: String::from("var/projects/demo/outputs/a.png"),
                model: Some(String::from("gpt-image-1")),
                size: Some(String::from("1024x1024")),
                quality: Some(String::from("high")),
            })
            .expect("generate-one command should build");

        assert_eq!(cmd.program, "node");
        assert_eq!(cmd.cwd, app_root);
        assert!(cmd.args.iter().any(|v| v == "generate-one"));
        assert!(cmd.args.iter().any(|v| v == "--input-images-file"));
        assert!(cmd.args.iter().any(|v| v == "--output"));
        assert!(cmd.args.iter().any(|v| v == "--json"));
    }

    #[test]
    fn parses_qa_json_response() {
        let app_root = temp_app_root_with_script();
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from(
                "{\"ok\":false,\"enabled\":true,\"has_hard_failures\":true,\"input\":\"var/projects/demo/outputs/a.png\",\"summary\":{\"total_files\":1,\"hard_failures\":1,\"soft_warnings\":0},\"report\":{\"summary\":{\"total_files\":1,\"hard_failures\":1,\"soft_warnings\":0},\"files\":[]}}",
            ),
            stderr: String::new(),
        }));
        let adapters = ScriptPipelineToolAdapters::new(app_root, runner.clone());

        let parsed = adapters
            .qa_typed(&QaCheckRequest {
                project_slug: String::from("demo"),
                project_root: None,
                input_path: String::from("var/projects/demo/outputs/a.png"),
                manifest_path: None,
                output_guard_enabled: None,
                enforce_grayscale: None,
                max_chroma_delta: None,
                fail_on_chroma_exceed: None,
                qa_python_bin: None,
            })
            .expect("qa json should parse");

        assert!(!parsed.ok);
        assert_eq!(parsed.has_hard_failures, Some(true));
        assert_eq!(parsed.summary.as_ref().map(|s| s.hard_failures), Some(1));
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 1);
        assert!(seen[0].args.iter().any(|v| v == "qa"));
        assert!(seen[0].args.iter().any(|v| v == "--json"));
    }

    #[test]
    fn build_bgremove_command_includes_override_flags() {
        let app_root = temp_app_root_with_script();
        let adapters = ScriptPipelineToolAdapters::new(app_root, FakeRunner::default());
        let cmd = adapters
            .build_bgremove_command(&BackgroundRemovePassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("/tmp/demo")),
                input_path: String::from("a.png"),
                output_path: String::from("b.png"),
                postprocess_config_path: Some(String::from("config/postprocess.json")),
                backends: vec![String::from("rembg"), String::from("removebg")],
                bg_refine_openai: Some(true),
                bg_refine_openai_required: Some(false),
            })
            .expect("bgremove command should build");

        assert!(cmd.args.iter().any(|v| v == "bgremove"));
        assert!(cmd.args.iter().any(|v| v == "--bg-remove-backends"));
        assert!(cmd
            .args
            .windows(2)
            .any(|w| w[0] == "--bg-remove-backends" && w[1] == "rembg,removebg"));
        assert!(cmd
            .args
            .windows(2)
            .any(|w| w[0] == "--bg-refine-openai" && w[1] == "true"));
        assert!(cmd
            .args
            .windows(2)
            .any(|w| w[0] == "--bg-refine-openai-required" && w[1] == "false"));
    }

    #[test]
    fn returns_command_failed_error_with_stdout_and_stderr() {
        let app_root = temp_app_root_with_script();
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 7,
            stdout: String::from("some-stdout"),
            stderr: String::from("some-stderr"),
        }));
        let adapters = ScriptPipelineToolAdapters::new(app_root, runner);

        let err = adapters
            .archive_bad_typed(&ArchiveBadRequest {
                project_slug: String::from("demo"),
                project_root: None,
                input_path: String::from("var/projects/demo/outputs"),
            })
            .expect_err("non-zero exit should error");

        match err {
            ToolAdapterError::CommandFailed {
                status_code,
                stdout,
                stderr,
                ..
            } => {
                assert_eq!(status_code, 7);
                assert_eq!(stdout, "some-stdout");
                assert_eq!(stderr, "some-stderr");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
