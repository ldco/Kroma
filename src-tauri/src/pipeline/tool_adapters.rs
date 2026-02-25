use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use reqwest::blocking::{multipart, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[cfg(test)]
use crate::pipeline::runtime::CommandOutput;
use crate::pipeline::runtime::{
    default_app_root_from_manifest_dir, CommandSpec, PipelineCommandRunner, PipelineRuntimeError,
    StdPipelineCommandRunner,
};
use bgremove_config::{load_bgremove_adapter_config, BgRemoveAdapterConfig};
use color_config::load_color_adapter_config;
use color_ops::{
    apply_color_profile_to_path, load_color_settings_config, resolve_color_output_path,
};
use dotenv_utils::load_dotenv_map;
use file_ops::{archive_existing_target, make_stamp, mime_for_path, resolve_bgremove_output_path};
use pathing::{
    list_image_files_recursive, path_for_output, resolve_request_path_under_root,
    resolve_under_root,
};
use qa_report::build_output_guard_report_value;
use upscale_config::load_upscale_adapter_config;

mod bgremove_config;
mod color_config;
mod color_ops;
mod dotenv_utils;
mod file_ops;
mod pathing;
mod qa_report;
mod upscale_config;

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

#[derive(Debug, Deserialize)]
struct OpenAiImagesEditsResponse {
    #[serde(default)]
    data: Vec<OpenAiImagesEditsResponseItem>,
}

#[derive(Debug, Deserialize)]
struct OpenAiImagesEditsResponseItem {
    #[serde(default)]
    b64_json: Option<String>,
}

pub trait PipelineToolAdapterOps: Send + Sync + 'static {
    fn generate_one(
        &self,
        request: &GenerateOneImageRequest,
    ) -> Result<GenerateOneImageResponse, ToolAdapterError>;
    fn upscale(
        &self,
        request: &UpscalePassRequest,
    ) -> Result<UpscalePassResponse, ToolAdapterError>;
    fn color(&self, request: &ColorPassRequest) -> Result<ColorPassResponse, ToolAdapterError>;
    fn bgremove(
        &self,
        request: &BackgroundRemovePassRequest,
    ) -> Result<BackgroundRemovePassResponse, ToolAdapterError>;
    fn qa(&self, request: &QaCheckRequest) -> Result<QaCheckResponse, ToolAdapterError>;
    fn archive_bad(
        &self,
        request: &ArchiveBadRequest,
    ) -> Result<ArchiveBadResponse, ToolAdapterError>;
}

pub type SharedPipelineToolAdapterOps = Arc<dyn PipelineToolAdapterOps>;

#[derive(Debug, Clone)]
pub struct NativeQaArchiveScriptToolAdapters<R> {
    runner: R,
    app_root: PathBuf,
}

impl<R> NativeQaArchiveScriptToolAdapters<R>
where
    R: PipelineCommandRunner,
{
    pub fn new(app_root: PathBuf, runner: R) -> Self {
        Self { runner, app_root }
    }

    fn app_root(&self) -> &Path {
        self.app_root.as_path()
    }

    fn generate_one_native(
        &self,
        request: &GenerateOneImageRequest,
    ) -> Result<GenerateOneImageResponse, ToolAdapterError> {
        validate_project_slug(request.project_slug.as_str())?;
        let input_images_file_abs = resolve_request_path_under_root(
            self.app_root(),
            request.input_images_file.as_str(),
            "input_images_file",
        )?;
        if !input_images_file_abs.is_file() {
            return Err(ToolAdapterError::Native(format!(
                "generate-one input-images file not found: {}",
                request.input_images_file
            )));
        }
        let raw =
            fs::read_to_string(input_images_file_abs.as_path()).map_err(ToolAdapterError::Io)?;
        let parsed: Value =
            serde_json::from_str(raw.as_str()).map_err(|source| ToolAdapterError::JsonDecode {
                source,
                stdout: raw,
            })?;
        let input_images = parsed
            .as_array()
            .ok_or_else(|| {
                ToolAdapterError::Native(String::from(
                    "generate-one --input-images-file must contain a JSON array of paths",
                ))
            })?
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if input_images.is_empty() {
            return Err(ToolAdapterError::Native(String::from(
                "generate-one --input-images-file contains no image paths",
            )));
        }
        let prompt = request.prompt.trim();
        if prompt.is_empty() {
            return Err(ToolAdapterError::Native(String::from(
                "generate-one requires non-empty prompt",
            )));
        }

        let dotenv = load_dotenv_map(self.app_root()).unwrap_or_default();
        let api_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .or_else(|| dotenv.get("OPENAI_API_KEY").cloned())
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                ToolAdapterError::Native(String::from(
                    "Missing OPENAI_API_KEY in environment or .env",
                ))
            })?;
        let model = request
            .model
            .clone()
            .or_else(|| std::env::var("OPENAI_IMAGE_MODEL").ok())
            .or_else(|| dotenv.get("OPENAI_IMAGE_MODEL").cloned())
            .unwrap_or_else(|| String::from("gpt-image-1"));
        let size = request
            .size
            .clone()
            .or_else(|| std::env::var("OPENAI_IMAGE_SIZE").ok())
            .or_else(|| dotenv.get("OPENAI_IMAGE_SIZE").cloned())
            .unwrap_or_else(|| String::from("1024x1536"));
        let quality = request
            .quality
            .clone()
            .or_else(|| std::env::var("OPENAI_IMAGE_QUALITY").ok())
            .or_else(|| dotenv.get("OPENAI_IMAGE_QUALITY").cloned())
            .unwrap_or_else(|| String::from("high"));

        let output_abs = resolve_request_path_under_root(
            self.app_root(),
            request.output_path.as_str(),
            "output_path",
        )?;
        if let Some(parent) = output_abs.parent() {
            fs::create_dir_all(parent).map_err(ToolAdapterError::Io)?;
        }
        let project_root = request
            .project_root
            .as_deref()
            .map(|v| resolve_request_path_under_root(self.app_root(), v, "project_root"))
            .transpose()?
            .unwrap_or_else(|| {
                self.app_root()
                    .join("var/projects")
                    .join(request.project_slug.as_str())
            });
        let archive_replaced = project_root.join("archive").join("replaced");
        let _ =
            archive_existing_target(output_abs.as_path(), archive_replaced.as_path(), "replaced")
                .map_err(ToolAdapterError::Io)?;

        let mut form = multipart::Form::new()
            .text("model", model.clone())
            .text("size", size.clone())
            .text("quality", quality.clone())
            .text("prompt", prompt.to_string())
            .text("output_format", "png")
            .text("input_fidelity", "high");

        for rel in input_images.iter() {
            let abs =
                resolve_request_path_under_root(self.app_root(), rel.as_str(), "input_image")?;
            if !abs.is_file() {
                return Err(ToolAdapterError::Native(format!(
                    "generate-one input image not found: {rel}"
                )));
            }
            let bytes = fs::read(abs.as_path()).map_err(ToolAdapterError::Io)?;
            let file_name = abs
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("image.png")
                .to_string();
            let mime = mime_for_path(abs.as_path());
            let part = multipart::Part::bytes(bytes)
                .file_name(file_name)
                .mime_str(mime.as_str())
                .map_err(|e| ToolAdapterError::Native(format!("invalid mime '{mime}': {e}")))?;
            form = form.part("image[]", part);
        }

        let client = Client::builder()
            .build()
            .map_err(|e| ToolAdapterError::Native(format!("http client init failed: {e}")))?;
        let resp = client
            .post("https://api.openai.com/v1/images/edits")
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .map_err(|e| ToolAdapterError::Native(format!("OpenAI request failed: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ToolAdapterError::Native(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }
        let payload: OpenAiImagesEditsResponse = resp
            .json()
            .map_err(|e| ToolAdapterError::Native(format!("OpenAI JSON decode failed: {e}")))?;
        let b64 = payload
            .data
            .first()
            .and_then(|d| d.b64_json.as_deref())
            .ok_or_else(|| {
                ToolAdapterError::Native(String::from("API returned no image payload"))
            })?;
        let bytes = BASE64_STANDARD
            .decode(b64.as_bytes())
            .map_err(|e| ToolAdapterError::Native(format!("image base64 decode failed: {e}")))?;
        fs::write(output_abs.as_path(), bytes.as_slice()).map_err(ToolAdapterError::Io)?;

        Ok(GenerateOneImageResponse {
            ok: true,
            project: request.project_slug.clone(),
            output: path_for_output(self.app_root(), output_abs.as_path()),
            input_images,
            model,
            size,
            quality,
            bytes_written: bytes.len(),
        })
    }

    fn upscale_native(
        &self,
        request: &UpscalePassRequest,
    ) -> Result<UpscalePassResponse, ToolAdapterError> {
        validate_project_slug(request.project_slug.as_str())?;
        let input_abs = resolve_request_path_under_root(
            self.app_root(),
            request.input_path.as_str(),
            "input_path",
        )?;
        if !input_abs.exists() {
            return Err(ToolAdapterError::Native(format!(
                "upscale input does not exist: {}",
                request.input_path
            )));
        }
        let output_abs = resolve_request_path_under_root(
            self.app_root(),
            request.output_path.as_str(),
            "output_path",
        )?;
        let input_meta = fs::metadata(input_abs.as_path()).map_err(ToolAdapterError::Io)?;
        if input_meta.is_dir() {
            fs::create_dir_all(output_abs.as_path()).map_err(ToolAdapterError::Io)?;
        } else {
            if let Some(parent) = output_abs.parent() {
                fs::create_dir_all(parent).map_err(ToolAdapterError::Io)?;
            }
            let project_root = request
                .project_root
                .as_deref()
                .map(|v| resolve_request_path_under_root(self.app_root(), v, "project_root"))
                .transpose()?
                .unwrap_or_else(|| {
                    self.app_root()
                        .join("var/projects")
                        .join(request.project_slug.as_str())
                });
            let archive_replaced = project_root.join("archive").join("replaced");
            let _ = archive_existing_target(
                output_abs.as_path(),
                archive_replaced.as_path(),
                "replaced",
            )
            .map_err(ToolAdapterError::Io)?;
        }

        let cfg = load_upscale_adapter_config(
            self.app_root(),
            request.postprocess_config_path.as_deref(),
        )?;
        let backend = request
            .upscale_backend
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(cfg.backend.as_str())
            .to_ascii_lowercase();
        if !matches!(backend.as_str(), "ncnn" | "python") {
            return Err(ToolAdapterError::Native(format!(
                "invalid upscale backend '{}'. Expected ncnn|python.",
                backend
            )));
        }
        let scale = request.upscale_scale.unwrap_or(cfg.scale);
        let format = request
            .upscale_format
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(cfg.format.as_str())
            .to_ascii_lowercase();

        if backend == "ncnn" {
            let bin_path = resolve_under_root(self.app_root(), cfg.ncnn_binary.as_str());
            let model_dir = resolve_under_root(self.app_root(), cfg.ncnn_model_dir.as_str());
            if !bin_path.exists() {
                return Err(ToolAdapterError::Native(format!(
                    "Real-ESRGAN binary not found: {}",
                    path_for_output(self.app_root(), bin_path.as_path())
                )));
            }
            if !model_dir.exists() {
                return Err(ToolAdapterError::Native(format!(
                    "Real-ESRGAN models dir not found: {}",
                    path_for_output(self.app_root(), model_dir.as_path())
                )));
            }
            let mut args = vec![
                String::from("-i"),
                input_abs.to_string_lossy().to_string(),
                String::from("-o"),
                output_abs.to_string_lossy().to_string(),
                String::from("-s"),
                scale.to_string(),
                String::from("-m"),
                model_dir.to_string_lossy().to_string(),
                String::from("-n"),
                cfg.ncnn_model_name.clone(),
                String::from("-f"),
                format.clone(),
            ];
            args.push(String::from("-t"));
            args.push(cfg.tile.to_string());

            let output = self
                .runner
                .run(&CommandSpec {
                    program: bin_path.to_string_lossy().to_string(),
                    args,
                    cwd: self.app_root.clone(),
                })
                .map_err(ToolAdapterError::CommandRunner)?;
            if output.status_code != 0 {
                return Err(ToolAdapterError::CommandFailed {
                    program: path_for_output(self.app_root(), bin_path.as_path()),
                    status_code: output.status_code,
                    stdout: output.stdout,
                    stderr: output.stderr,
                });
            }

            return Ok(UpscalePassResponse {
                backend,
                input: path_for_output(self.app_root(), input_abs.as_path()),
                output: path_for_output(self.app_root(), output_abs.as_path()),
                scale: u64::from(scale),
                model: cfg.ncnn_model_name,
            });
        }

        let python_program = if cfg.python_bin.contains('/') || cfg.python_bin.starts_with('.') {
            resolve_under_root(self.app_root(), cfg.python_bin.as_str())
                .to_string_lossy()
                .to_string()
        } else {
            cfg.python_bin.clone()
        };
        let python_script = resolve_request_path_under_root(
            self.app_root(),
            cfg.python_inference_script.as_str(),
            "upscale.python.inference_script",
        )?;
        if !python_script.is_file() {
            return Err(ToolAdapterError::Native(format!(
                "Real-ESRGAN python inference script not found: {}. Run: bash scripts/setup-realesrgan-python.sh",
                path_for_output(self.app_root(), python_script.as_path())
            )));
        }
        let mut args = vec![
            python_script.to_string_lossy().to_string(),
            String::from("--input"),
            input_abs.to_string_lossy().to_string(),
            String::from("--output"),
            output_abs.to_string_lossy().to_string(),
            String::from("--model-name"),
            cfg.python_model_name.clone(),
            String::from("--outscale"),
            scale.to_string(),
            String::from("--tile"),
            cfg.python_tile.to_string(),
            String::from("--tile-pad"),
            cfg.python_tile_pad.to_string(),
            String::from("--pre-pad"),
            cfg.python_pre_pad.to_string(),
            String::from("--ext"),
            format.clone(),
        ];
        if cfg.python_fp32 {
            args.push(String::from("--fp32"));
        }
        if let Some(gpu_id) = cfg.python_gpu_id {
            args.push(String::from("--gpu-id"));
            args.push(gpu_id.to_string());
        }

        let output = self
            .runner
            .run(&CommandSpec {
                program: python_program.clone(),
                args,
                cwd: self.app_root.clone(),
            })
            .map_err(ToolAdapterError::CommandRunner)?;
        if output.status_code != 0 {
            return Err(ToolAdapterError::CommandFailed {
                program: python_program,
                status_code: output.status_code,
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }

        Ok(UpscalePassResponse {
            backend,
            input: path_for_output(self.app_root(), input_abs.as_path()),
            output: path_for_output(self.app_root(), output_abs.as_path()),
            scale: u64::from(scale),
            model: cfg.python_model_name,
        })
    }

    fn color_native(
        &self,
        request: &ColorPassRequest,
    ) -> Result<ColorPassResponse, ToolAdapterError> {
        validate_project_slug(request.project_slug.as_str())?;
        let input_abs = resolve_request_path_under_root(
            self.app_root(),
            request.input_path.as_str(),
            "input_path",
        )?;
        if !input_abs.exists() {
            return Err(ToolAdapterError::Native(format!(
                "color-correction input does not exist: {}",
                request.input_path
            )));
        }
        let output_abs = resolve_request_path_under_root(
            self.app_root(),
            request.output_path.as_str(),
            "output_path",
        )?;
        if let Some(parent) = output_abs.parent() {
            fs::create_dir_all(parent).map_err(ToolAdapterError::Io)?;
        }

        let project_root = request
            .project_root
            .as_deref()
            .map(|v| resolve_request_path_under_root(self.app_root(), v, "project_root"))
            .transpose()?
            .unwrap_or_else(|| {
                self.app_root()
                    .join("var/projects")
                    .join(request.project_slug.as_str())
            });
        let archive_replaced = project_root.join("archive").join("replaced");
        let _ =
            archive_existing_target(output_abs.as_path(), archive_replaced.as_path(), "replaced")
                .map_err(ToolAdapterError::Io)?;

        let (cfg_default_profile, cfg_settings_file) =
            load_color_adapter_config(self.app_root(), request.postprocess_config_path.as_deref())?;
        let chosen_profile = request
            .profile
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(cfg_default_profile.as_deref().unwrap_or("neutral"))
            .to_string();

        let settings_path_abs = if let Some(v) = request
            .color_settings_path
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
            Some(resolve_request_path_under_root(
                self.app_root(),
                v,
                "color_settings_path",
            )?)
        } else if let Some(v) = cfg_settings_file
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
            Some(resolve_request_path_under_root(
                self.app_root(),
                v,
                "color.settings_file",
            )?)
        } else {
            None
        };
        if let Some(path) = settings_path_abs.as_ref() {
            if !path.is_file() {
                return Err(ToolAdapterError::Native(format!(
                    "missing color settings file: {}",
                    path_for_output(self.app_root(), path.as_path())
                )));
            }
        }
        let color_settings = load_color_settings_config(settings_path_abs.as_deref())
            .map_err(ToolAdapterError::Native)?;
        let profile = color_settings
            .profiles
            .get(chosen_profile.as_str())
            .ok_or_else(|| {
                ToolAdapterError::Native(format!(
                    "Profile '{}' not found in {}",
                    chosen_profile,
                    settings_path_abs
                        .as_ref()
                        .map(|v| path_for_output(self.app_root(), v.as_path()))
                        .unwrap_or_else(|| String::from("builtin"))
                ))
            })?
            .clone();

        let input_meta = fs::metadata(input_abs.as_path()).map_err(ToolAdapterError::Io)?;
        let input_is_dir = input_meta.is_dir();
        let images =
            list_image_files_recursive(input_abs.as_path()).map_err(ToolAdapterError::Io)?;
        if images.is_empty() {
            return Err(ToolAdapterError::Native(String::from(
                "No image files found for correction",
            )));
        }
        for src in images {
            let dst = resolve_color_output_path(
                src.as_path(),
                input_abs.as_path(),
                output_abs.as_path(),
                input_is_dir,
            )
            .map_err(ToolAdapterError::Io)?;
            apply_color_profile_to_path(src.as_path(), dst.as_path(), &profile)
                .map_err(|e| ToolAdapterError::Native(format!("color correction failed: {e}")))?;
        }

        Ok(ColorPassResponse {
            input: path_for_output(self.app_root(), input_abs.as_path()),
            output: path_for_output(self.app_root(), output_abs.as_path()),
            profile: chosen_profile,
            settings: settings_path_abs
                .as_ref()
                .map(|v| path_for_output(self.app_root(), v.as_path()))
                .unwrap_or_else(|| String::from("builtin")),
        })
    }

    fn bgremove_native_simple_rembg(
        &self,
        request: &BackgroundRemovePassRequest,
    ) -> Result<BackgroundRemovePassResponse, ToolAdapterError> {
        validate_project_slug(request.project_slug.as_str())?;
        let input_abs = resolve_request_path_under_root(
            self.app_root(),
            request.input_path.as_str(),
            "input_path",
        )?;
        if !input_abs.exists() {
            return Err(ToolAdapterError::Native(format!(
                "background-remove input does not exist: {}",
                request.input_path
            )));
        }
        let input_is_dir = fs::metadata(input_abs.as_path())
            .map_err(ToolAdapterError::Io)?
            .is_dir();
        let output_abs_root = resolve_request_path_under_root(
            self.app_root(),
            request.output_path.as_str(),
            "output_path",
        )?;
        if input_is_dir {
            fs::create_dir_all(output_abs_root.as_path()).map_err(ToolAdapterError::Io)?;
        }
        let project_root = request
            .project_root
            .as_deref()
            .map(|v| resolve_request_path_under_root(self.app_root(), v, "project_root"))
            .transpose()?
            .unwrap_or_else(|| {
                self.app_root()
                    .join("var/projects")
                    .join(request.project_slug.as_str())
            });
        let archive_replaced = project_root.join("archive").join("replaced");

        let cfg = load_bgremove_adapter_config(
            self.app_root(),
            request.postprocess_config_path.as_deref(),
        )?;
        let backends = if request.backends.is_empty() {
            cfg.backends.clone()
        } else {
            request
                .backends
                .iter()
                .map(|v| v.trim().to_ascii_lowercase())
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>()
        };
        let refine_enabled = request.bg_refine_openai.unwrap_or(cfg.openai_enabled);
        let refine_required = request
            .bg_refine_openai_required
            .unwrap_or(cfg.openai_required);

        if backends.is_empty() {
            return Err(ToolAdapterError::Native(String::from(
                "background remove requires at least one backend",
            )));
        }
        let unsupported = backends
            .iter()
            .filter(|v| !matches!(v.as_str(), "rembg" | "photoroom" | "removebg"))
            .cloned()
            .collect::<Vec<_>>();
        if !unsupported.is_empty() {
            return Err(ToolAdapterError::Native(format!(
                "Unsupported background-remove backend(s): {}. Expected rembg|photoroom|removebg.",
                unsupported.join(", ")
            )));
        }

        let rembg_python =
            if cfg.rembg_python_bin.contains('/') || cfg.rembg_python_bin.starts_with('.') {
                resolve_under_root(self.app_root(), cfg.rembg_python_bin.as_str())
                    .to_string_lossy()
                    .to_string()
            } else {
                cfg.rembg_python_bin.clone()
            };
        let files =
            list_image_files_recursive(input_abs.as_path()).map_err(ToolAdapterError::Io)?;
        if files.is_empty() {
            return Err(ToolAdapterError::Native(String::from(
                "No image files found for background remove",
            )));
        }
        let dotenv = if refine_enabled || backends.iter().any(|b| b != "rembg") {
            load_dotenv_map(self.app_root()).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let mut results = Vec::with_capacity(files.len());
        for file_abs in files {
            let output_abs = resolve_bgremove_output_path(
                file_abs.as_path(),
                input_abs.as_path(),
                output_abs_root.as_path(),
                input_is_dir,
                cfg.format.as_str(),
            )
            .map_err(ToolAdapterError::Io)?;
            let _ = archive_existing_target(
                output_abs.as_path(),
                archive_replaced.as_path(),
                "replaced",
            )
            .map_err(ToolAdapterError::Io)?;

            let backend_used = self.run_bgremove_backend_fallback(
                file_abs.as_path(),
                output_abs.as_path(),
                &backends,
                &cfg,
                rembg_python.as_str(),
                &dotenv,
            )?;

            let mut refine_applied = false;
            let mut refine_error = None;
            if refine_enabled {
                let ext = output_abs
                    .extension()
                    .and_then(|v| v.to_str())
                    .unwrap_or("png")
                    .to_string();
                let stem = output_abs
                    .file_stem()
                    .and_then(|v| v.to_str())
                    .unwrap_or("image")
                    .to_string();
                let tmp_abs = output_abs.with_file_name(format!(
                    "{stem}_openai_tmp_{}.{}",
                    make_stamp(),
                    ext
                ));
                match self.run_bg_refine_openai(
                    output_abs.as_path(),
                    tmp_abs.as_path(),
                    cfg.format.as_str(),
                    &cfg,
                    &dotenv,
                ) {
                    Ok(()) => {
                        let _ = archive_existing_target(
                            output_abs.as_path(),
                            archive_replaced.as_path(),
                            "replaced",
                        )
                        .map_err(ToolAdapterError::Io)?;
                        fs::rename(tmp_abs.as_path(), output_abs.as_path())
                            .map_err(ToolAdapterError::Io)?;
                        refine_applied = true;
                    }
                    Err(err) => {
                        let _ = fs::remove_file(tmp_abs.as_path());
                        let message = err.to_string();
                        if refine_required {
                            return Err(ToolAdapterError::Native(format!(
                                "OpenAI refine failed for {}: {}",
                                path_for_output(self.app_root(), file_abs.as_path()),
                                message
                            )));
                        }
                        refine_error = Some(message);
                    }
                }
            }

            results.push(BackgroundRemovePassFileResult {
                input: path_for_output(self.app_root(), file_abs.as_path()),
                output: path_for_output(self.app_root(), output_abs.as_path()),
                backend: backend_used,
                refine_openai: refine_applied,
                refine_error,
            });
        }

        Ok(BackgroundRemovePassResponse {
            input: path_for_output(self.app_root(), input_abs.as_path()),
            output: path_for_output(self.app_root(), output_abs_root.as_path()),
            backends,
            refine_openai: refine_enabled,
            refine_openai_required: refine_required,
            format: cfg.format,
            processed: results.len() as u64,
            results,
        })
    }

    fn run_bgremove_backend_fallback(
        &self,
        input_abs: &Path,
        output_abs: &Path,
        backends: &[String],
        cfg: &BgRemoveAdapterConfig,
        rembg_python: &str,
        dotenv: &HashMap<String, String>,
    ) -> Result<String, ToolAdapterError> {
        let mut failures = Vec::new();
        for backend in backends {
            let result = match backend.as_str() {
                "rembg" => self.run_bgremove_rembg(input_abs, output_abs, rembg_python, cfg),
                "photoroom" => self.run_bgremove_http_backend(
                    input_abs,
                    output_abs,
                    cfg.photoroom_endpoint.as_str(),
                    "x-api-key",
                    cfg.photoroom_api_key_env.as_str(),
                    cfg,
                    dotenv,
                ),
                "removebg" => self.run_bgremove_http_backend(
                    input_abs,
                    output_abs,
                    cfg.removebg_endpoint.as_str(),
                    "X-Api-Key",
                    cfg.removebg_api_key_env.as_str(),
                    cfg,
                    dotenv,
                ),
                _ => Err(ToolAdapterError::Native(format!(
                    "Unsupported backend '{}'",
                    backend
                ))),
            };
            match result {
                Ok(()) => return Ok(backend.clone()),
                Err(err) => failures.push(format!("{backend}: {}", err)),
            }
        }

        Err(ToolAdapterError::Native(format!(
            "Background remove failed for {}. {}",
            path_for_output(self.app_root(), input_abs),
            failures.join(" | ")
        )))
    }

    fn run_bgremove_rembg(
        &self,
        input_abs: &Path,
        output_abs: &Path,
        rembg_python: &str,
        cfg: &BgRemoveAdapterConfig,
    ) -> Result<(), ToolAdapterError> {
        let args = vec![
            String::from("-m"),
            String::from("rembg"),
            String::from("i"),
            String::from("-m"),
            cfg.rembg_model.clone(),
            input_abs.to_string_lossy().to_string(),
            output_abs.to_string_lossy().to_string(),
        ];
        let output = self
            .runner
            .run(&CommandSpec {
                program: rembg_python.to_string(),
                args,
                cwd: self.app_root.clone(),
            })
            .map_err(ToolAdapterError::CommandRunner)?;
        if output.status_code != 0 {
            return Err(ToolAdapterError::CommandFailed {
                program: rembg_python.to_string(),
                status_code: output.status_code,
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }
        Ok(())
    }

    fn run_bgremove_http_backend(
        &self,
        input_abs: &Path,
        output_abs: &Path,
        endpoint: &str,
        auth_header_name: &str,
        api_key_env_name: &str,
        cfg: &BgRemoveAdapterConfig,
        dotenv: &HashMap<String, String>,
    ) -> Result<(), ToolAdapterError> {
        let api_key = std::env::var(api_key_env_name)
            .ok()
            .or_else(|| dotenv.get(api_key_env_name).cloned())
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                ToolAdapterError::Native(format!(
                    "Missing {api_key_env_name} for background remove"
                ))
            })?;

        let bytes = fs::read(input_abs).map_err(ToolAdapterError::Io)?;
        let file_name = input_abs
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("image.png")
            .to_string();
        let mime = mime_for_path(input_abs);
        let part = multipart::Part::bytes(bytes)
            .file_name(file_name)
            .mime_str(mime.as_str())
            .map_err(|e| ToolAdapterError::Native(format!("invalid mime '{mime}': {e}")))?;
        let mut form = multipart::Form::new().part("image_file", part);
        if !cfg.size.trim().is_empty() && !cfg.size.eq_ignore_ascii_case("auto") {
            form = form.text("size", cfg.size.clone());
        }
        if !cfg.format.trim().is_empty() {
            form = form.text("format", cfg.format.clone());
        }
        if cfg.crop {
            form = form.text("crop", String::from("true"));
        }

        let client = Client::builder()
            .build()
            .map_err(|e| ToolAdapterError::Native(format!("http client init failed: {e}")))?;
        let resp = client
            .post(endpoint)
            .header(auth_header_name, api_key)
            .multipart(form)
            .send()
            .map_err(|e| ToolAdapterError::Native(format!("HTTP request failed: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ToolAdapterError::Native(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }
        let out_bytes = resp
            .bytes()
            .map_err(|e| ToolAdapterError::Native(format!("HTTP body read failed: {e}")))?;
        if let Some(parent) = output_abs.parent() {
            fs::create_dir_all(parent).map_err(ToolAdapterError::Io)?;
        }
        fs::write(output_abs, out_bytes.as_ref()).map_err(ToolAdapterError::Io)?;
        Ok(())
    }

    fn run_bg_refine_openai(
        &self,
        input_abs: &Path,
        output_abs: &Path,
        format: &str,
        cfg: &BgRemoveAdapterConfig,
        dotenv: &HashMap<String, String>,
    ) -> Result<(), ToolAdapterError> {
        let api_key = std::env::var(cfg.openai_api_key_env.as_str())
            .ok()
            .or_else(|| dotenv.get(cfg.openai_api_key_env.as_str()).cloned())
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                ToolAdapterError::Native(format!(
                    "Missing {} for OpenAI background refinement",
                    cfg.openai_api_key_env
                ))
            })?;
        let model = cfg
            .openai_model
            .clone()
            .or_else(|| std::env::var("OPENAI_IMAGE_MODEL").ok())
            .or_else(|| dotenv.get("OPENAI_IMAGE_MODEL").cloned())
            .unwrap_or_else(|| String::from("gpt-image-1"));
        let quality = cfg
            .openai_quality
            .clone()
            .or_else(|| std::env::var("OPENAI_IMAGE_QUALITY").ok())
            .or_else(|| dotenv.get("OPENAI_IMAGE_QUALITY").cloned())
            .unwrap_or_else(|| String::from("high"));

        let buffer = fs::read(input_abs).map_err(ToolAdapterError::Io)?;
        let file_name = input_abs
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("image.png")
            .to_string();
        let mime = mime_for_path(input_abs);

        let form = multipart::Form::new()
            .text("model", model)
            .text("prompt", cfg.openai_prompt.clone())
            .text("quality", quality)
            .text("input_fidelity", cfg.openai_input_fidelity.clone())
            .text("output_format", format.to_string())
            .text("background", cfg.openai_background.clone())
            .part(
                "image[]",
                multipart::Part::bytes(buffer)
                    .file_name(file_name)
                    .mime_str(mime.as_str())
                    .map_err(|e| ToolAdapterError::Native(format!("invalid mime '{mime}': {e}")))?,
            );

        let client = Client::builder()
            .build()
            .map_err(|e| ToolAdapterError::Native(format!("http client init failed: {e}")))?;
        let resp = client
            .post("https://api.openai.com/v1/images/edits")
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .map_err(|e| ToolAdapterError::Native(format!("OpenAI request failed: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ToolAdapterError::Native(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }
        let payload: OpenAiImagesEditsResponse = resp
            .json()
            .map_err(|e| ToolAdapterError::Native(format!("OpenAI JSON decode failed: {e}")))?;
        let b64 = payload
            .data
            .first()
            .and_then(|d| d.b64_json.as_deref())
            .ok_or_else(|| {
                ToolAdapterError::Native(String::from("OpenAI refine returned no image payload"))
            })?;
        let bytes = BASE64_STANDARD
            .decode(b64.as_bytes())
            .map_err(|e| ToolAdapterError::Native(format!("image base64 decode failed: {e}")))?;
        if let Some(parent) = output_abs.parent() {
            fs::create_dir_all(parent).map_err(ToolAdapterError::Io)?;
        }
        fs::write(output_abs, bytes).map_err(ToolAdapterError::Io)?;
        Ok(())
    }

    fn qa_native(&self, request: &QaCheckRequest) -> Result<QaCheckResponse, ToolAdapterError> {
        validate_project_slug(request.project_slug.as_str())?;
        let enabled = request.output_guard_enabled.unwrap_or(true);
        if !enabled {
            return Ok(QaCheckResponse {
                ok: true,
                skipped: Some(true),
                enabled: false,
                reason: Some(String::from("output_guard_disabled")),
                has_hard_failures: None,
                input: None,
                summary: None,
                report: None,
            });
        }

        let max_chroma_delta = request.max_chroma_delta.unwrap_or(2.0);
        if !max_chroma_delta.is_finite() || max_chroma_delta < 0.0 {
            return Err(ToolAdapterError::Native(format!(
                "invalid max_chroma_delta '{}'",
                max_chroma_delta
            )));
        }
        let enforce_grayscale = request.enforce_grayscale.unwrap_or(false);
        let fail_on_chroma_exceed = request.fail_on_chroma_exceed.unwrap_or(false);
        let input_abs = resolve_request_path_under_root(
            self.app_root(),
            request.input_path.as_str(),
            "input_path",
        )?;
        let report = build_output_guard_report_value(
            self.app_root(),
            input_abs.as_path(),
            max_chroma_delta,
            enforce_grayscale,
            fail_on_chroma_exceed,
        );
        let summary_obj = report.get("summary").and_then(Value::as_object);
        let summary = QaCheckSummary {
            total_files: summary_obj
                .and_then(|v| v.get("total_files"))
                .and_then(Value::as_u64)
                .unwrap_or(0),
            hard_failures: summary_obj
                .and_then(|v| v.get("hard_failures"))
                .and_then(Value::as_u64)
                .unwrap_or(0),
            soft_warnings: summary_obj
                .and_then(|v| v.get("soft_warnings"))
                .and_then(Value::as_u64)
                .unwrap_or(0),
        };
        Ok(QaCheckResponse {
            ok: summary.hard_failures == 0,
            skipped: None,
            enabled: true,
            reason: None,
            has_hard_failures: Some(summary.hard_failures > 0),
            input: Some(path_for_output(self.app_root(), input_abs.as_path())),
            summary: Some(summary),
            report: Some(report),
        })
    }

    fn archive_bad_native(
        &self,
        request: &ArchiveBadRequest,
    ) -> Result<ArchiveBadResponse, ToolAdapterError> {
        validate_project_slug(request.project_slug.as_str())?;
        let input_abs = resolve_request_path_under_root(
            self.app_root(),
            request.input_path.as_str(),
            "input_path",
        )?;
        if !input_abs.exists() {
            return Err(ToolAdapterError::Native(format!(
                "archive-bad input not found: {}",
                request.input_path
            )));
        }

        let project_root = request
            .project_root
            .as_deref()
            .map(|v| resolve_request_path_under_root(self.app_root(), v, "project_root"))
            .transpose()?
            .unwrap_or_else(|| {
                self.app_root()
                    .join("var/projects")
                    .join(request.project_slug.as_str())
            });
        let archive_dir = project_root.join("archive").join("bad");
        fs::create_dir_all(archive_dir.as_path()).map_err(ToolAdapterError::Io)?;

        let files =
            list_image_files_recursive(input_abs.as_path()).map_err(ToolAdapterError::Io)?;
        if files.is_empty() {
            return Err(ToolAdapterError::Native(String::from(
                "archive-bad found no image files to move",
            )));
        }

        let mut moved = Vec::new();
        for file in files {
            if let Some(to) = archive_existing_target(file.as_path(), archive_dir.as_path(), "bad")
                .map_err(ToolAdapterError::Io)?
            {
                moved.push(ArchiveBadMovedFile {
                    from: path_for_output(self.app_root(), file.as_path()),
                    to: path_for_output(self.app_root(), to.as_path()),
                });
            }
        }

        Ok(ArchiveBadResponse {
            ok: true,
            archive_dir: path_for_output(self.app_root(), archive_dir.as_path()),
            moved_count: moved.len() as u64,
            moved,
        })
    }
}

impl<R> PipelineToolAdapterOps for NativeQaArchiveScriptToolAdapters<R>
where
    R: PipelineCommandRunner,
{
    fn generate_one(
        &self,
        request: &GenerateOneImageRequest,
    ) -> Result<GenerateOneImageResponse, ToolAdapterError> {
        self.generate_one_native(request)
    }

    fn upscale(
        &self,
        request: &UpscalePassRequest,
    ) -> Result<UpscalePassResponse, ToolAdapterError> {
        self.upscale_native(request)
    }

    fn color(&self, request: &ColorPassRequest) -> Result<ColorPassResponse, ToolAdapterError> {
        self.color_native(request)
    }

    fn bgremove(
        &self,
        request: &BackgroundRemovePassRequest,
    ) -> Result<BackgroundRemovePassResponse, ToolAdapterError> {
        self.bgremove_native_simple_rembg(request)
    }

    fn qa(&self, request: &QaCheckRequest) -> Result<QaCheckResponse, ToolAdapterError> {
        self.qa_native(request)
    }

    fn archive_bad(
        &self,
        request: &ArchiveBadRequest,
    ) -> Result<ArchiveBadResponse, ToolAdapterError> {
        self.archive_bad_native(request)
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
    #[error("tool adapter filesystem error: {0}")]
    Io(#[source] std::io::Error),
    #[error("{0}")]
    Native(String),
}

pub fn default_native_tool_adapters() -> NativeQaArchiveScriptToolAdapters<StdPipelineCommandRunner>
{
    NativeQaArchiveScriptToolAdapters::new(
        default_app_root_from_manifest_dir(),
        StdPipelineCommandRunner,
    )
}

pub fn default_native_qa_archive_script_tool_adapters(
) -> NativeQaArchiveScriptToolAdapters<StdPipelineCommandRunner> {
    default_native_tool_adapters()
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

    fn temp_app_root() -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_tool_adapters_native_{stamp}"));
        std::fs::create_dir_all(&root).expect("temp root should exist");
        root
    }

    #[test]
    fn resolve_request_path_under_root_rejects_escaping_paths() {
        let app_root = temp_app_root();

        let abs_path =
            resolve_request_path_under_root(app_root.as_path(), "/tmp/outside.png", "input_path")
                .expect_err("absolute path should be rejected");
        assert!(abs_path.to_string().contains("relative path"));

        let parent_path =
            resolve_request_path_under_root(app_root.as_path(), "../outside.png", "input_path")
                .expect_err("parent traversal should be rejected");
        assert!(parent_path.to_string().contains("within app root"));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_qa_rejects_parent_path_traversal() {
        let app_root = temp_app_root();
        let runner = FakeRunner::default();
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner);

        let err = adapters
            .qa(&QaCheckRequest {
                project_slug: String::from("demo"),
                project_root: None,
                input_path: String::from("../etc/passwd"),
                manifest_path: None,
                output_guard_enabled: Some(true),
                enforce_grayscale: Some(true),
                max_chroma_delta: Some(1.5),
                fail_on_chroma_exceed: Some(false),
            })
            .expect_err("parent traversal should be rejected");

        assert!(err.to_string().contains("within app root"));
        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_qa_computes_output_guard_report_in_rust_without_running_script() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs"))
            .expect("outputs dir should exist");
        let img = image::RgbImage::from_pixel(2, 2, image::Rgb([32, 32, 32]));
        img.save(app_root.join("var/projects/demo/outputs/a.png"))
            .expect("image should be written");

        let runner = FakeRunner::default();
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());

        let parsed = adapters
            .qa(&QaCheckRequest {
                project_slug: String::from("demo"),
                project_root: None,
                input_path: String::from("var/projects/demo/outputs/a.png"),
                manifest_path: None,
                output_guard_enabled: Some(true),
                enforce_grayscale: Some(true),
                max_chroma_delta: Some(1.5),
                fail_on_chroma_exceed: Some(false),
            })
            .expect("native qa should compute report");

        assert!(parsed.ok);
        assert_eq!(parsed.enabled, true);
        assert_eq!(parsed.summary.as_ref().map(|s| s.total_files), Some(1));
        assert_eq!(parsed.summary.as_ref().map(|s| s.hard_failures), Some(0));
        assert_eq!(parsed.summary.as_ref().map(|s| s.soft_warnings), Some(0));
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 0);

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_archive_bad_moves_images_without_running_script() {
        let app_root = temp_app_root();
        let project_root = app_root.join("var/projects/demo");
        let outputs = project_root.join("outputs");
        std::fs::create_dir_all(outputs.as_path()).expect("outputs dir should exist");
        std::fs::write(outputs.join("A Final!!.png"), b"png").expect("image should exist");
        std::fs::write(outputs.join("note.txt"), b"txt").expect("non-image should exist");

        let runner = FakeRunner::default();
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());
        let result = adapters
            .archive_bad(&ArchiveBadRequest {
                project_slug: String::from("demo"),
                project_root: None,
                input_path: String::from("var/projects/demo/outputs"),
            })
            .expect("native archive-bad should succeed");

        assert!(result.ok);
        assert_eq!(result.moved_count, 1);
        assert_eq!(result.archive_dir, "var/projects/demo/archive/bad");
        assert_eq!(runner.take_seen().len(), 0);
        assert!(project_root.join("outputs/A Final!!.png").exists() == false);
        assert!(project_root.join("outputs/note.txt").exists());
        assert_eq!(result.moved.len(), 1);
        assert_eq!(
            result.moved[0].from,
            "var/projects/demo/outputs/A Final!!.png"
        );
        assert!(result.moved[0]
            .to
            .starts_with("var/projects/demo/archive/bad/a_final_bad_"));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_color_applies_profile_in_rust_and_uses_postprocess_config_defaults() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs"))
            .expect("outputs dir should exist");
        std::fs::create_dir_all(app_root.join("config")).expect("config dir should exist");
        let img = image::RgbImage::from_pixel(2, 2, image::Rgb([100, 90, 80]));
        img.save(app_root.join("var/projects/demo/outputs/in.png"))
            .expect("input image should exist");
        std::fs::write(
            app_root.join("config/color-settings.json"),
            r#"{"profiles":{"cinematic-v2":{"brightness":1.0,"contrast":1.0,"saturation":1.0,"sharpness":1.0,"gamma":1.0,"autocontrast_cutoff":0,"red_multiplier":1.2,"green_multiplier":1.0,"blue_multiplier":0.8}}}"#,
        )
            .expect("settings file should exist");
        std::fs::write(
            app_root.join("config/postprocess.json"),
            r#"{"color":{"default_profile":"cinematic-v2","settings_file":"config/color-settings.json"}}"#,
        )
            .expect("postprocess config should exist");

        let runner = FakeRunner::default();
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());

        let resp = adapters
            .color(&ColorPassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                input_path: String::from("var/projects/demo/outputs/in.png"),
                output_path: String::from("var/projects/demo/color_corrected/out.png"),
                postprocess_config_path: Some(String::from("config/postprocess.json")),
                profile: None,
                color_settings_path: None,
            })
            .expect("native color should succeed");

        assert_eq!(resp.profile, "cinematic-v2");
        assert_eq!(resp.settings, "config/color-settings.json");
        let out_path = app_root.join("var/projects/demo/color_corrected/out.png");
        assert!(out_path.is_file());
        let out_img = image::open(out_path)
            .expect("output image should load")
            .to_rgb8();
        assert_eq!(out_img.get_pixel(0, 0)[0], 120);
        assert_eq!(out_img.get_pixel(0, 0)[1], 90);
        assert_eq!(out_img.get_pixel(0, 0)[2], 64);
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 0);

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_upscale_runs_python_backend_directly_with_config_defaults() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs"))
            .expect("outputs dir should exist");
        std::fs::create_dir_all(app_root.join("config")).expect("config dir should exist");
        let inference_script = app_root.join("tools/realesrgan-python/src/Real-ESRGAN");
        std::fs::create_dir_all(inference_script.as_path())
            .expect("inference script dir should exist");
        std::fs::write(
            inference_script.join("inference_realesrgan.py"),
            b"#!/usr/bin/env python3",
        )
        .expect("inference script should exist");
        std::fs::write(app_root.join("var/projects/demo/outputs/in.png"), b"png")
            .expect("input should exist");
        std::fs::write(
            app_root.join("config/postprocess.json"),
            r#"{"upscale":{"backend":"python","scale":4,"format":"png","python":{"python_bin":"python3","model_name":"RealESRGAN_x4plus","tile":8,"tile_pad":12,"pre_pad":2,"fp32":true,"gpu_id":0}}}"#,
        )
        .expect("postprocess config should exist");

        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }));
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());

        let resp = adapters
            .upscale(&UpscalePassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                input_path: String::from("var/projects/demo/outputs/in.png"),
                output_path: String::from("var/projects/demo/upscaled/out.png"),
                postprocess_config_path: Some(String::from("config/postprocess.json")),
                upscale_backend: None,
                upscale_scale: None,
                upscale_format: None,
            })
            .expect("native upscale should succeed");

        assert_eq!(resp.backend, "python");
        assert_eq!(resp.scale, 4);
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].program, "python3");
        assert!(seen[0]
            .args
            .first()
            .map(|v| v.ends_with("tools/realesrgan-python/src/Real-ESRGAN/inference_realesrgan.py"))
            .unwrap_or(false));
        assert!(seen[0].args.windows(2).any(|w| w == ["--outscale", "4"]));
        assert!(seen[0].args.windows(2).any(|w| w == ["--tile", "8"]));
        assert!(seen[0].args.iter().any(|v| v == "--fp32"));
        assert!(seen[0].args.windows(2).any(|w| w == ["--gpu-id", "0"]));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_upscale_errors_when_python_inference_script_is_missing() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs"))
            .expect("outputs dir should exist");
        std::fs::create_dir_all(app_root.join("config")).expect("config dir should exist");
        std::fs::write(app_root.join("var/projects/demo/outputs/in.png"), b"png")
            .expect("input should exist");
        std::fs::write(
            app_root.join("config/postprocess.json"),
            r#"{"upscale":{"backend":"python","scale":4,"format":"png","python":{"python_bin":"python3"}}}"#,
        )
        .expect("postprocess config should exist");

        let runner = FakeRunner::default();
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner);

        let err = adapters
            .upscale(&UpscalePassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                input_path: String::from("var/projects/demo/outputs/in.png"),
                output_path: String::from("var/projects/demo/upscaled/out.png"),
                postprocess_config_path: Some(String::from("config/postprocess.json")),
                upscale_backend: None,
                upscale_scale: None,
                upscale_format: None,
            })
            .expect_err("missing script should error");
        assert!(err.to_string().contains("inference script not found"));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_bgremove_runs_rembg_directly_when_simple_case() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs"))
            .expect("outputs dir should exist");
        std::fs::create_dir_all(app_root.join("config")).expect("config dir should exist");
        std::fs::write(app_root.join("var/projects/demo/outputs/in.png"), b"png")
            .expect("input should exist");
        std::fs::write(
            app_root.join("config/postprocess.json"),
            r#"{"bg_remove":{"backends":["rembg"],"format":"webp","rembg":{"python_bin":"python3","model":"u2net_human_seg"},"openai":{"enabled":false,"required":false}}}"#,
        )
        .expect("postprocess config should exist");

        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }));
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());

        let resp = adapters
            .bgremove(&BackgroundRemovePassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                input_path: String::from("var/projects/demo/outputs/in.png"),
                output_path: String::from("var/projects/demo/background_removed/out.webp"),
                postprocess_config_path: Some(String::from("config/postprocess.json")),
                backends: vec![String::from("rembg")],
                bg_refine_openai: Some(false),
                bg_refine_openai_required: Some(false),
            })
            .expect("native bgremove should succeed");

        assert_eq!(resp.processed, 1);
        assert_eq!(resp.format, "webp");
        assert_eq!(resp.backends, vec!["rembg"]);
        assert_eq!(resp.results[0].backend, "rembg");
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].program, "python3");
        assert_eq!(seen[0].args.first().map(String::as_str), Some("-m"));
        assert!(seen[0].args.windows(2).any(|w| w == ["-m", "rembg"]));
        assert!(seen[0].args.iter().any(|v| v == "i"));
        assert!(seen[0]
            .args
            .windows(2)
            .any(|w| w == ["-m", "u2net_human_seg"]));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_bgremove_rejects_unsupported_backend_without_script_fallback() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs"))
            .expect("outputs dir should exist");
        std::fs::write(app_root.join("var/projects/demo/outputs/in.png"), b"png")
            .expect("input should exist");

        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from(
                "{\"input\":\"var/projects/demo/outputs/in.png\",\"output\":\"var/projects/demo/background_removed/out.png\",\"backends\":[\"removebg\"],\"refine_openai\":true,\"refine_openai_required\":true,\"format\":\"png\",\"processed\":1,\"results\":[{\"input\":\"var/projects/demo/outputs/in.png\",\"output\":\"var/projects/demo/background_removed/out.png\",\"backend\":\"removebg\",\"refine_openai\":true,\"refine_error\":null}]}",
            ),
            stderr: String::new(),
        }));
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());

        let err = adapters
            .bgremove(&BackgroundRemovePassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                input_path: String::from("var/projects/demo/outputs/in.png"),
                output_path: String::from("var/projects/demo/background_removed/out.png"),
                postprocess_config_path: None,
                backends: vec![String::from("unsupportedbg")],
                bg_refine_openai: Some(true),
                bg_refine_openai_required: Some(true),
            })
            .expect_err("unsupported backend should error");

        match err {
            ToolAdapterError::Native(message) => {
                assert!(message.contains("Unsupported background-remove backend"));
                assert!(message.contains("unsupportedbg"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 0);

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_bgremove_tries_photoroom_then_falls_back_to_rembg_without_script() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs"))
            .expect("outputs dir should exist");
        std::fs::create_dir_all(app_root.join("config")).expect("config dir should exist");
        std::fs::write(app_root.join("var/projects/demo/outputs/in.png"), b"png")
            .expect("input should exist");
        std::fs::write(
            app_root.join("config/postprocess.json"),
            r#"{"bg_remove":{"backends":["photoroom","rembg"],"format":"png","rembg":{"python_bin":"python3","model":"u2net"},"photoroom":{"api_key_env":"PHOTOROOM_TEST_API_KEY_MISSING","endpoint":"https://example.invalid/photoroom"}}}"#,
        )
        .expect("postprocess config should exist");

        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }));
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());

        let resp = adapters
            .bgremove(&BackgroundRemovePassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                input_path: String::from("var/projects/demo/outputs/in.png"),
                output_path: String::from("var/projects/demo/background_removed/out.png"),
                postprocess_config_path: Some(String::from("config/postprocess.json")),
                backends: vec![String::from("photoroom"), String::from("rembg")],
                bg_refine_openai: Some(false),
                bg_refine_openai_required: Some(false),
            })
            .expect("native backend fallback should succeed via rembg");

        assert_eq!(resp.processed, 1);
        assert_eq!(resp.results[0].backend, "rembg");
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].program, "python3");
        assert_eq!(seen[0].args.first().map(String::as_str), Some("-m"));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_bgremove_runs_rembg_directly_for_directory_input() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs/sub"))
            .expect("input dir should exist");
        std::fs::create_dir_all(app_root.join("config")).expect("config dir should exist");
        std::fs::write(app_root.join("var/projects/demo/outputs/a.png"), b"png")
            .expect("input should exist");
        std::fs::write(app_root.join("var/projects/demo/outputs/sub/b.jpg"), b"jpg")
            .expect("nested input should exist");
        std::fs::write(
            app_root.join("config/postprocess.json"),
            r#"{"bg_remove":{"backends":["rembg"],"format":"webp","rembg":{"python_bin":"python3","model":"u2net"}}}"#,
        )
        .expect("postprocess config should exist");

        let runner = FakeRunner::default();
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());

        let resp = adapters
            .bgremove(&BackgroundRemovePassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                input_path: String::from("var/projects/demo/outputs"),
                output_path: String::from("var/projects/demo/background_removed"),
                postprocess_config_path: Some(String::from("config/postprocess.json")),
                backends: vec![String::from("rembg")],
                bg_refine_openai: Some(false),
                bg_refine_openai_required: Some(false),
            })
            .expect("native directory bgremove should succeed");

        assert_eq!(resp.processed, 2);
        assert_eq!(resp.format, "webp");
        assert_eq!(resp.results.len(), 2);
        assert!(resp
            .results
            .iter()
            .any(|r| r.output == "var/projects/demo/background_removed/a.webp"));
        assert!(resp
            .results
            .iter()
            .any(|r| r.output == "var/projects/demo/background_removed/sub/b.webp"));
        let seen = runner.take_seen();
        assert_eq!(seen.len(), 2);
        assert!(seen.iter().all(|cmd| cmd.program == "python3"));
        assert!(seen
            .iter()
            .all(|cmd| cmd.args.first().map(String::as_str) == Some("-m")));
        assert!(!seen.iter().any(|cmd| cmd.program == "node"));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_bgremove_uses_rust_refine_path_without_script_fallback_when_optional_refine_fails() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/projects/demo/outputs"))
            .expect("outputs dir should exist");
        std::fs::create_dir_all(app_root.join("config")).expect("config dir should exist");
        std::fs::write(app_root.join("var/projects/demo/outputs/in.png"), b"png")
            .expect("input should exist");
        std::fs::write(
            app_root.join("config/postprocess.json"),
            r#"{"bg_remove":{"backends":["rembg"],"format":"png","rembg":{"python_bin":"python3","model":"u2net"},"openai":{"enabled":true,"required":false,"api_key_env":"BG_REFINE_TEST_API_KEY_MISSING"}}}"#,
        )
        .expect("postprocess config should exist");

        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }));
        let adapters = NativeQaArchiveScriptToolAdapters::new(app_root.clone(), runner.clone());

        let resp = adapters
            .bgremove(&BackgroundRemovePassRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                input_path: String::from("var/projects/demo/outputs/in.png"),
                output_path: String::from("var/projects/demo/background_removed/out.png"),
                postprocess_config_path: Some(String::from("config/postprocess.json")),
                backends: vec![String::from("rembg")],
                bg_refine_openai: Some(true),
                bg_refine_openai_required: Some(false),
            })
            .expect("optional refine failure should still succeed");

        assert!(resp.refine_openai);
        assert!(!resp.refine_openai_required);
        assert_eq!(resp.processed, 1);
        assert_eq!(resp.results[0].backend, "rembg");
        assert!(!resp.results[0].refine_openai);
        let refine_error = resp.results[0]
            .refine_error
            .as_deref()
            .expect("refine error should be captured");
        assert!(refine_error.contains("BG_REFINE_TEST_API_KEY_MISSING"));

        let seen = runner.take_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].program, "python3");
        assert_eq!(seen[0].args.first().map(String::as_str), Some("-m"));
        assert!(!seen.iter().any(|cmd| cmd.program == "node"));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn parse_dotenv_content_supports_export_quotes_and_comments() {
        let parsed = dotenv_utils::parse_dotenv_content(
            r#"
                # comment
                OPENAI_API_KEY = sk-test
                export OPENAI_IMAGE_MODEL="gpt-image-1"
                OPENAI_IMAGE_SIZE=1024x1536 # inline comment
                OPENAI_IMAGE_QUALITY='high'
                INVALID_LINE
            "#,
        );

        assert_eq!(
            parsed.get("OPENAI_API_KEY").map(String::as_str),
            Some("sk-test")
        );
        assert_eq!(
            parsed.get("OPENAI_IMAGE_MODEL").map(String::as_str),
            Some("gpt-image-1")
        );
        assert_eq!(
            parsed.get("OPENAI_IMAGE_SIZE").map(String::as_str),
            Some("1024x1536")
        );
        assert_eq!(
            parsed.get("OPENAI_IMAGE_QUALITY").map(String::as_str),
            Some("high")
        );
        assert!(!parsed.contains_key("INVALID_LINE"));
    }

    #[test]
    fn native_generate_one_errors_before_network_on_invalid_input_images_json() {
        let app_root = temp_app_root();
        std::fs::create_dir_all(app_root.join("var/tmp")).expect("temp dir should exist");
        std::fs::write(app_root.join("var/tmp/input-images.json"), b"{not-json")
            .expect("input images file should exist");

        let adapters =
            NativeQaArchiveScriptToolAdapters::new(app_root.clone(), FakeRunner::default());
        let err = adapters
            .generate_one(&GenerateOneImageRequest {
                project_slug: String::from("demo"),
                project_root: Some(String::from("var/projects/demo")),
                prompt: String::from("test prompt"),
                input_images_file: String::from("var/tmp/input-images.json"),
                output_path: String::from("var/projects/demo/outputs/out.png"),
                model: None,
                size: None,
                quality: None,
            })
            .expect_err("invalid json should fail");

        match err {
            ToolAdapterError::JsonDecode { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }

        let _ = std::fs::remove_dir_all(app_root);
    }
}
