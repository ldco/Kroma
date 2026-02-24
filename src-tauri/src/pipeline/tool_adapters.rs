use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use image::{DynamicImage, RgbImage};
use reqwest::blocking::{multipart, Client};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[cfg(test)]
use crate::pipeline::runtime::CommandOutput;
use crate::pipeline::runtime::{
    default_app_root_from_manifest_dir, CommandSpec, PipelineCommandRunner, PipelineRuntimeError,
    StdPipelineCommandRunner,
};

const REMBG_INLINE_PYTHON: &str = r#"
import argparse
from io import BytesIO
from pathlib import Path
import sys

def fail(message: str) -> None:
    raise SystemExit(message)

def normalize_format(value: str) -> str:
    fmt = value.lower().strip()
    if fmt == "jpeg":
        fmt = "jpg"
    if fmt not in {"png", "jpg", "webp"}:
        fail(f"Unsupported --format '{value}'. Expected png|jpg|webp")
    return fmt

def main():
    parser = argparse.ArgumentParser(description="Remove image background with rembg")
    parser.add_argument("--input", required=True, help="Input image file")
    parser.add_argument("--output", required=True, help="Output image file")
    parser.add_argument("--model", default="u2net", help="rembg model name")
    parser.add_argument("--format", default="png", help="Output format: png|jpg|webp")
    args = parser.parse_args()

    input_path = Path(args.input).resolve()
    output_path = Path(args.output).resolve()
    output_fmt = normalize_format(args.format)
    if not input_path.exists() or not input_path.is_file():
        fail(f"Input file not found: {input_path}")

    try:
        from rembg import remove, new_session
        from PIL import Image
    except Exception as exc:
        fail(f"rembg runtime is missing. Run: bash scripts/setup-rembg.sh\nOriginal error: {exc}")

    data = input_path.read_bytes()
    session = new_session(args.model)
    out_bytes = remove(data, session=session)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    if output_fmt == "png":
        output_path.write_bytes(out_bytes)
        return

    with Image.open(BytesIO(out_bytes)).convert("RGBA") as img:
        if output_fmt == "jpg":
            bg = Image.new("RGB", img.size, (255, 255, 255))
            bg.paste(img, mask=img.getchannel("A"))
            bg.save(output_path, format="JPEG", quality=95)
        elif output_fmt == "webp":
            img.save(output_path, format="WEBP", quality=95)

main()
"#;

const REALESRGAN_UPSCALE_INLINE_PYTHON: &str = r#"
import argparse
import sys
from pathlib import Path

def fail(message: str) -> None:
    raise SystemExit(message)

def add_local_source_paths() -> None:
    project_root = Path.cwd()
    src_root = project_root / "tools" / "realesrgan-python" / "src"
    for folder in ("Real-ESRGAN", "BasicSR"):
        candidate = src_root / folder
        if candidate.exists():
            sys.path.insert(0, str(candidate))

def clamp_extension(value: str) -> str:
    ext = value.lower().strip()
    if ext == "jpeg":
        ext = "jpg"
    allowed = {"auto", "jpg", "png", "webp"}
    if ext not in allowed:
        fail(f"Unsupported --ext '{value}'. Expected one of: {', '.join(sorted(allowed))}")
    return ext

def import_runtime_deps():
    add_local_source_paths()
    try:
        import cv2  # noqa: F401
        from basicsr.archs.rrdbnet_arch import RRDBNet  # noqa: F401
        from basicsr.utils.download_util import load_file_from_url  # noqa: F401
        from realesrgan import RealESRGANer  # noqa: F401
        from realesrgan.archs.srvgg_arch import SRVGGNetCompact  # noqa: F401
    except Exception as exc:
        fail(
            "Python Real-ESRGAN dependencies are missing. "
            "Run: bash scripts/setup-realesrgan-python.sh\n"
            f"Original error: {exc}"
        )

def build_model_spec(model_name: str):
    from basicsr.archs.rrdbnet_arch import RRDBNet
    from realesrgan.archs.srvgg_arch import SRVGGNetCompact

    if model_name == "RealESRGAN_x4plus":
        model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=4)
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth"
        ]
    if model_name == "RealESRNet_x4plus":
        model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=4)
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.1/RealESRNet_x4plus.pth"
        ]
    if model_name == "RealESRGAN_x4plus_anime_6B":
        model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=6, num_grow_ch=32, scale=4)
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.2.4/RealESRGAN_x4plus_anime_6B.pth"
        ]
    if model_name == "RealESRGAN_x2plus":
        model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=2)
        return 2, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.1/RealESRGAN_x2plus.pth"
        ]
    if model_name == "realesr-animevideov3":
        model = SRVGGNetCompact(num_in_ch=3, num_out_ch=3, num_feat=64, num_conv=16, upscale=4, act_type="prelu")
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-animevideov3.pth"
        ]
    if model_name == "realesr-general-x4v3":
        model = SRVGGNetCompact(num_in_ch=3, num_out_ch=3, num_feat=64, num_conv=32, upscale=4, act_type="prelu")
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-x4v3.pth"
        ]

    fail(
        "Unsupported --model-name. "
        "Use one of: RealESRGAN_x4plus, RealESRNet_x4plus, RealESRGAN_x4plus_anime_6B, "
        "RealESRGAN_x2plus, realesr-animevideov3, realesr-general-x4v3"
    )

def resolve_model_path(model_name: str, weights_dir: Path) -> str:
    from basicsr.utils.download_util import load_file_from_url

    weights_dir.mkdir(parents=True, exist_ok=True)
    candidate = weights_dir / f"{model_name}.pth"
    if candidate.exists():
        return str(candidate)

    _, _, urls = build_model_spec(model_name)
    resolved = None
    for url in urls:
        resolved = load_file_from_url(url=url, model_dir=str(weights_dir), progress=True, file_name=None)
    if not resolved:
        fail(f"Could not download model weights for {model_name}")
    return str(resolved)

def list_input_images(input_path: Path):
    if input_path.is_file():
        return [input_path]
    image_exts = {".jpg", ".jpeg", ".png", ".webp", ".bmp", ".tif", ".tiff"}
    files = []
    for item in sorted(input_path.rglob("*")):
        if item.is_file() and item.suffix.lower() in image_exts:
            files.append(item)
    return files

def output_path_for(src: Path, input_root: Path, output_path: Path, ext: str, input_is_dir: bool) -> Path:
    if not input_is_dir:
        if output_path.suffix.lower() in {".jpg", ".jpeg", ".png", ".webp"}:
            final = output_path
        else:
            output_path.mkdir(parents=True, exist_ok=True)
            final = output_path / src.name
    else:
        rel = src.relative_to(input_root)
        final = output_path / rel
        final.parent.mkdir(parents=True, exist_ok=True)
    if ext == "auto":
        normalized_ext = src.suffix.lower()
        if normalized_ext == ".jpeg":
            normalized_ext = ".jpg"
    else:
        normalized_ext = f".{ext}"
    return final.with_suffix(normalized_ext)

def main():
    parser = argparse.ArgumentParser(description="Real-ESRGAN python backend wrapper")
    parser.add_argument("--input", required=True, help="Input image file or directory")
    parser.add_argument("--output", required=True, help="Output image file or directory")
    parser.add_argument("--model-name", default="RealESRGAN_x4plus", help="Real-ESRGAN model name")
    parser.add_argument("--outscale", type=float, default=2.0, help="Final upscale ratio")
    parser.add_argument("--tile", type=int, default=0, help="Tile size, 0 for no tile")
    parser.add_argument("--tile-pad", type=int, default=10, help="Tile padding")
    parser.add_argument("--pre-pad", type=int, default=0, help="Pre padding")
    parser.add_argument("--ext", default="png", help="Output extension: auto|jpg|png|webp")
    parser.add_argument("--weights-dir", default="", help="Optional custom weights directory")
    parser.add_argument("--gpu-id", type=int, default=None, help="GPU id (optional)")
    parser.add_argument("--fp32", action="store_true", help="Force fp32")
    args = parser.parse_args()

    input_path = Path(args.input).resolve()
    output_path = Path(args.output).resolve()
    if not input_path.exists():
        fail(f"Input path not found: {input_path}")

    ext = clamp_extension(args.ext)
    import_runtime_deps()

    from realesrgan import RealESRGANer
    import cv2

    netscale, model, _ = build_model_spec(args.model_name)
    default_weights_dir = Path.cwd() / "tools" / "realesrgan-python" / "weights"
    weights_dir = Path(args.weights_dir).resolve() if args.weights_dir else default_weights_dir
    model_path = resolve_model_path(args.model_name, weights_dir)

    upsampler = RealESRGANer(
        scale=netscale,
        model_path=model_path,
        model=model,
        tile=args.tile,
        tile_pad=args.tile_pad,
        pre_pad=args.pre_pad,
        half=not args.fp32,
        gpu_id=args.gpu_id,
    )

    images = list_input_images(input_path)
    if not images:
        fail("No image files found in input path")
    input_is_dir = input_path.is_dir()
    if input_is_dir:
        output_path.mkdir(parents=True, exist_ok=True)
    else:
        output_path.parent.mkdir(parents=True, exist_ok=True)

    done = 0
    for src in images:
        dst = output_path_for(src, input_path, output_path, ext, input_is_dir)
        img = cv2.imread(str(src), cv2.IMREAD_UNCHANGED)
        if img is None:
            print(f"Skip unreadable image: {src}", file=sys.stderr)
            continue
        try:
            output, _ = upsampler.enhance(img, outscale=args.outscale)
        except RuntimeError as exc:
            fail(f"Upscale failed for {src}: {exc}. Try smaller --tile value.")
        dst.parent.mkdir(parents=True, exist_ok=True)
        ok = cv2.imwrite(str(dst), output)
        if not ok:
            fail(f"Failed to write output: {dst}")
        done += 1

    print(f"Processed {done} image(s) with model {args.model_name} (outscale x{args.outscale})")

main()
"#;

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
pub struct ScriptPipelineToolAdapters<R> {
    runner: R,
    app_root: PathBuf,
    node_binary: String,
    script_rel_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct NativeQaArchiveScriptToolAdapters<R> {
    runner: R,
    app_root: PathBuf,
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

impl<R> PipelineToolAdapterOps for ScriptPipelineToolAdapters<R>
where
    R: PipelineCommandRunner,
{
    fn generate_one(
        &self,
        request: &GenerateOneImageRequest,
    ) -> Result<GenerateOneImageResponse, ToolAdapterError> {
        self.generate_one_typed(request)
    }

    fn upscale(
        &self,
        request: &UpscalePassRequest,
    ) -> Result<UpscalePassResponse, ToolAdapterError> {
        self.upscale_typed(request)
    }

    fn color(&self, request: &ColorPassRequest) -> Result<ColorPassResponse, ToolAdapterError> {
        self.color_typed(request)
    }

    fn bgremove(
        &self,
        request: &BackgroundRemovePassRequest,
    ) -> Result<BackgroundRemovePassResponse, ToolAdapterError> {
        self.bgremove_typed(request)
    }

    fn qa(&self, request: &QaCheckRequest) -> Result<QaCheckResponse, ToolAdapterError> {
        self.qa_typed(request)
    }

    fn archive_bad(
        &self,
        request: &ArchiveBadRequest,
    ) -> Result<ArchiveBadResponse, ToolAdapterError> {
        self.archive_bad_typed(request)
    }
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
        let input_images_file_abs =
            resolve_under_root(self.app_root(), request.input_images_file.as_str());
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

        let output_abs = resolve_under_root(self.app_root(), request.output_path.as_str());
        if let Some(parent) = output_abs.parent() {
            fs::create_dir_all(parent).map_err(ToolAdapterError::Io)?;
        }
        let project_root = request
            .project_root
            .as_deref()
            .map(|v| resolve_under_root(self.app_root(), v))
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
            let abs = resolve_under_root(self.app_root(), rel.as_str());
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
        let input_abs = resolve_under_root(self.app_root(), request.input_path.as_str());
        if !input_abs.exists() {
            return Err(ToolAdapterError::Native(format!(
                "upscale input does not exist: {}",
                request.input_path
            )));
        }
        let output_abs = resolve_under_root(self.app_root(), request.output_path.as_str());
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
                .map(|v| resolve_under_root(self.app_root(), v))
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
        let mut args = vec![
            String::from("-c"),
            String::from(REALESRGAN_UPSCALE_INLINE_PYTHON),
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
        let input_abs = resolve_under_root(self.app_root(), request.input_path.as_str());
        if !input_abs.exists() {
            return Err(ToolAdapterError::Native(format!(
                "color-correction input does not exist: {}",
                request.input_path
            )));
        }
        let output_abs = resolve_under_root(self.app_root(), request.output_path.as_str());
        if let Some(parent) = output_abs.parent() {
            fs::create_dir_all(parent).map_err(ToolAdapterError::Io)?;
        }

        let project_root = request
            .project_root
            .as_deref()
            .map(|v| resolve_under_root(self.app_root(), v))
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

        let settings_path_abs = request
            .color_settings_path
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .map(|v| resolve_under_root(self.app_root(), v))
            .or_else(|| cfg_settings_file.map(|v| resolve_under_root(self.app_root(), v.as_str())));
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
        let input_abs = resolve_under_root(self.app_root(), request.input_path.as_str());
        if !input_abs.exists() {
            return Err(ToolAdapterError::Native(format!(
                "background-remove input does not exist: {}",
                request.input_path
            )));
        }
        let input_is_dir = fs::metadata(input_abs.as_path())
            .map_err(ToolAdapterError::Io)?
            .is_dir();
        let output_abs_root = resolve_under_root(self.app_root(), request.output_path.as_str());
        if input_is_dir {
            fs::create_dir_all(output_abs_root.as_path()).map_err(ToolAdapterError::Io)?;
        }
        let project_root = request
            .project_root
            .as_deref()
            .map(|v| resolve_under_root(self.app_root(), v))
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
            String::from("-c"),
            String::from(REMBG_INLINE_PYTHON),
            String::from("--input"),
            input_abs.to_string_lossy().to_string(),
            String::from("--output"),
            output_abs.to_string_lossy().to_string(),
            String::from("--model"),
            cfg.rembg_model.clone(),
            String::from("--format"),
            cfg.format.clone(),
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
        let input_abs = resolve_under_root(self.app_root(), request.input_path.as_str());
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
        let input_abs = resolve_under_root(self.app_root(), request.input_path.as_str());
        if !input_abs.exists() {
            return Err(ToolAdapterError::Native(format!(
                "archive-bad input not found: {}",
                request.input_path
            )));
        }

        let project_root = request
            .project_root
            .as_deref()
            .map(|v| resolve_under_root(self.app_root(), v))
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

fn resolve_under_root(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn path_for_output(app_root: &Path, path: &Path) -> String {
    let value = match path.strip_prefix(app_root) {
        Ok(rel) => rel.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    };
    value.replace('\\', "/")
}

fn is_image_path(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tif" | "tiff"
    )
}

fn list_image_files_recursive(input_abs: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let meta = fs::metadata(input_abs)?;
    if meta.is_file() {
        return Ok(if is_image_path(input_abs) {
            vec![input_abs.to_path_buf()]
        } else {
            Vec::new()
        });
    }

    let mut out = Vec::new();
    let mut entries = fs::read_dir(input_abs)?.collect::<Result<Vec<_>, std::io::Error>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            out.extend(list_image_files_recursive(path.as_path())?);
            continue;
        }
        if file_type.is_file() && is_image_path(path.as_path()) {
            out.push(path);
        }
    }
    Ok(out)
}

#[derive(Debug, Clone)]
struct ColorProfileConfig {
    brightness: f32,
    contrast: f32,
    saturation: f32,
    sharpness: f32,
    gamma: f32,
    autocontrast_cutoff: f32,
    red_multiplier: f32,
    green_multiplier: f32,
    blue_multiplier: f32,
}

impl Default for ColorProfileConfig {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            sharpness: 1.0,
            gamma: 1.0,
            autocontrast_cutoff: 0.0,
            red_multiplier: 1.0,
            green_multiplier: 1.0,
            blue_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
struct ColorSettingsConfig {
    profiles: std::collections::BTreeMap<String, ColorProfileConfig>,
}

fn load_color_settings_config(settings_path: Option<&Path>) -> Result<ColorSettingsConfig, String> {
    let Some(path) = settings_path else {
        return Ok(default_color_settings_config());
    };
    let raw =
        fs::read_to_string(path).map_err(|e| format!("read settings '{}': {e}", path.display()))?;
    let parsed: Value = serde_json::from_str(raw.as_str())
        .map_err(|e| format!("parse settings '{}': {e}", path.display()))?;
    let profiles_obj = parsed
        .get("profiles")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "settings file '{}' is missing object field 'profiles'",
                path.display()
            )
        })?;
    let mut profiles = std::collections::BTreeMap::new();
    for (name, value) in profiles_obj {
        let obj = value.as_object().ok_or_else(|| {
            format!(
                "profile '{}' in '{}' must be an object",
                name,
                path.display()
            )
        })?;
        profiles.insert(name.clone(), parse_color_profile_config(obj));
    }
    Ok(ColorSettingsConfig { profiles })
}

fn parse_color_profile_config(obj: &serde_json::Map<String, Value>) -> ColorProfileConfig {
    let mut cfg = ColorProfileConfig::default();
    cfg.brightness = obj
        .get("brightness")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.brightness);
    cfg.contrast = obj
        .get("contrast")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.contrast);
    cfg.saturation = obj
        .get("saturation")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.saturation);
    cfg.sharpness = obj
        .get("sharpness")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.sharpness);
    cfg.gamma = obj
        .get("gamma")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.gamma);
    cfg.autocontrast_cutoff = obj
        .get("autocontrast_cutoff")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.autocontrast_cutoff);
    cfg.red_multiplier = obj
        .get("red_multiplier")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.red_multiplier);
    cfg.green_multiplier = obj
        .get("green_multiplier")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.green_multiplier);
    cfg.blue_multiplier = obj
        .get("blue_multiplier")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.blue_multiplier);
    cfg
}

fn default_color_settings_config() -> ColorSettingsConfig {
    let mut profiles = std::collections::BTreeMap::new();
    profiles.insert(
        String::from("neutral"),
        ColorProfileConfig {
            brightness: 1.0,
            contrast: 1.02,
            saturation: 1.0,
            sharpness: 1.0,
            gamma: 1.0,
            autocontrast_cutoff: 0.0,
            red_multiplier: 1.0,
            green_multiplier: 1.0,
            blue_multiplier: 1.0,
        },
    );
    profiles.insert(
        String::from("cinematic_warm"),
        ColorProfileConfig {
            brightness: 0.98,
            contrast: 1.12,
            saturation: 1.08,
            sharpness: 1.04,
            gamma: 1.03,
            autocontrast_cutoff: 0.0,
            red_multiplier: 1.04,
            green_multiplier: 1.0,
            blue_multiplier: 0.96,
        },
    );
    profiles.insert(
        String::from("cold_rain"),
        ColorProfileConfig {
            brightness: 0.97,
            contrast: 1.1,
            saturation: 0.95,
            sharpness: 1.03,
            gamma: 1.01,
            autocontrast_cutoff: 0.0,
            red_multiplier: 0.97,
            green_multiplier: 1.0,
            blue_multiplier: 1.05,
        },
    );
    ColorSettingsConfig { profiles }
}

fn apply_color_profile_to_path(
    input_abs: &Path,
    output_abs: &Path,
    profile: &ColorProfileConfig,
) -> Result<(), String> {
    let image =
        image::open(input_abs).map_err(|e| format!("open '{}': {e}", input_abs.display()))?;
    let corrected = apply_color_profile_to_image(&image, profile);
    if let Some(parent) = output_abs.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir '{}': {e}", parent.display()))?;
    }
    corrected
        .save(output_abs)
        .map_err(|e| format!("save '{}': {e}", output_abs.display()))
}

fn apply_color_profile_to_image(
    image: &DynamicImage,
    profile: &ColorProfileConfig,
) -> DynamicImage {
    let mut out = image.to_rgb8();

    if profile.autocontrast_cutoff > 0.0 {
        apply_autocontrast_in_place(&mut out, profile.autocontrast_cutoff);
    }
    if (profile.brightness - 1.0).abs() > f32::EPSILON {
        apply_brightness_in_place(&mut out, profile.brightness);
    }
    if (profile.contrast - 1.0).abs() > f32::EPSILON {
        apply_contrast_in_place(&mut out, profile.contrast);
    }
    if (profile.saturation - 1.0).abs() > f32::EPSILON {
        apply_saturation_in_place(&mut out, profile.saturation);
    }
    if (profile.sharpness - 1.0).abs() > f32::EPSILON {
        out = apply_sharpness(&out, profile.sharpness);
    }
    if profile.gamma > 0.0 && (profile.gamma - 1.0).abs() > f32::EPSILON {
        apply_gamma_in_place(&mut out, profile.gamma);
    }
    if (profile.red_multiplier - 1.0).abs() > f32::EPSILON
        || (profile.green_multiplier - 1.0).abs() > f32::EPSILON
        || (profile.blue_multiplier - 1.0).abs() > f32::EPSILON
    {
        apply_rgb_multipliers_in_place(
            &mut out,
            profile.red_multiplier,
            profile.green_multiplier,
            profile.blue_multiplier,
        );
    }

    DynamicImage::ImageRgb8(out)
}

fn resolve_color_output_path(
    src_abs: &Path,
    input_root_abs: &Path,
    output_abs: &Path,
    input_is_dir: bool,
) -> Result<PathBuf, std::io::Error> {
    if !input_is_dir {
        if is_image_path(output_abs) {
            if let Some(parent) = output_abs.parent() {
                fs::create_dir_all(parent)?;
            }
            return Ok(output_abs.to_path_buf());
        }
        fs::create_dir_all(output_abs)?;
        return Ok(output_abs.join(
            src_abs
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("image.png"),
        ));
    }

    let rel = src_abs.strip_prefix(input_root_abs).unwrap_or(src_abs);
    let dst = output_abs.join(rel);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(dst)
}

fn apply_autocontrast_in_place(image: &mut RgbImage, _cutoff: f32) {
    let mut min_v = [u8::MAX; 3];
    let mut max_v = [u8::MIN; 3];
    for pixel in image.pixels() {
        for i in 0..3 {
            min_v[i] = min_v[i].min(pixel[i]);
            max_v[i] = max_v[i].max(pixel[i]);
        }
    }
    for pixel in image.pixels_mut() {
        for i in 0..3 {
            let minc = f32::from(min_v[i]);
            let maxc = f32::from(max_v[i]);
            if (maxc - minc).abs() < f32::EPSILON {
                continue;
            }
            let v = f32::from(pixel[i]);
            pixel[i] = clamp_u8(((v - minc) / (maxc - minc)) * 255.0);
        }
    }
}

fn apply_brightness_in_place(image: &mut RgbImage, factor: f32) {
    for pixel in image.pixels_mut() {
        for i in 0..3 {
            pixel[i] = clamp_u8(f32::from(pixel[i]) * factor);
        }
    }
}

fn apply_contrast_in_place(image: &mut RgbImage, factor: f32) {
    for pixel in image.pixels_mut() {
        for i in 0..3 {
            let centered = f32::from(pixel[i]) - 128.0;
            pixel[i] = clamp_u8(centered * factor + 128.0);
        }
    }
}

fn apply_saturation_in_place(image: &mut RgbImage, factor: f32) {
    for pixel in image.pixels_mut() {
        let r = f32::from(pixel[0]);
        let g = f32::from(pixel[1]);
        let b = f32::from(pixel[2]);
        let gray = 0.299 * r + 0.587 * g + 0.114 * b;
        pixel[0] = clamp_u8(gray + (r - gray) * factor);
        pixel[1] = clamp_u8(gray + (g - gray) * factor);
        pixel[2] = clamp_u8(gray + (b - gray) * factor);
    }
}

fn apply_sharpness(image: &RgbImage, factor: f32) -> RgbImage {
    if factor <= 1.0 + f32::EPSILON {
        return image.clone();
    }
    let sigma = 1.0_f32;
    let blurred = image::imageops::blur(image, sigma);
    let amount = factor - 1.0;
    let mut out = image.clone();
    for (dst, (orig, blur)) in out.pixels_mut().zip(image.pixels().zip(blurred.pixels())) {
        for i in 0..3 {
            let val = f32::from(orig[i]) + amount * (f32::from(orig[i]) - f32::from(blur[i]));
            dst[i] = clamp_u8(val);
        }
    }
    out
}

fn apply_gamma_in_place(image: &mut RgbImage, gamma: f32) {
    let inv_gamma = 1.0 / gamma.max(f32::EPSILON);
    let mut lut = [0_u8; 256];
    for (i, out) in lut.iter_mut().enumerate() {
        *out = clamp_u8((((i as f32) / 255.0).powf(inv_gamma)) * 255.0);
    }
    for pixel in image.pixels_mut() {
        pixel[0] = lut[pixel[0] as usize];
        pixel[1] = lut[pixel[1] as usize];
        pixel[2] = lut[pixel[2] as usize];
    }
}

fn apply_rgb_multipliers_in_place(image: &mut RgbImage, r_mul: f32, g_mul: f32, b_mul: f32) {
    for pixel in image.pixels_mut() {
        pixel[0] = clamp_u8(f32::from(pixel[0]) * r_mul);
        pixel[1] = clamp_u8(f32::from(pixel[1]) * g_mul);
        pixel[2] = clamp_u8(f32::from(pixel[2]) * b_mul);
    }
}

fn clamp_u8(value: f32) -> u8 {
    if !value.is_finite() {
        return 0;
    }
    value.clamp(0.0, 255.0).round() as u8
}

fn build_output_guard_report_value(
    app_root: &Path,
    input_abs: &Path,
    max_chroma_delta: f64,
    enforce_grayscale: bool,
    fail_on_chroma_exceed: bool,
) -> Value {
    let input_display = path_for_output(app_root, input_abs);
    let mut report = json!({
        "input": input_display,
        "settings": {
            "max_chroma_delta": max_chroma_delta.max(0.0),
            "enforce_grayscale": enforce_grayscale,
            "fail_on_chroma_exceed": fail_on_chroma_exceed,
        },
        "summary": {
            "total_files": 0_u64,
            "hard_failures": 0_u64,
            "soft_warnings": 0_u64,
        },
        "files": []
    });

    if !input_abs.exists() {
        report["summary"]["hard_failures"] = json!(1_u64);
        report["files"] = json!([{
            "file": path_for_output(app_root, input_abs),
            "error": "input_not_found",
            "hard_fail_reasons": ["input_not_found"],
            "soft_warnings": []
        }]);
        return report;
    }

    let images = match list_image_files_recursive(input_abs) {
        Ok(v) => v,
        Err(err) => {
            report["summary"]["hard_failures"] = json!(1_u64);
            report["files"] = json!([{
                "file": path_for_output(app_root, input_abs),
                "error": err.to_string(),
                "hard_fail_reasons": ["image_read_failed"],
                "soft_warnings": []
            }]);
            return report;
        }
    };
    report["summary"]["total_files"] = json!(images.len() as u64);
    if images.is_empty() {
        report["summary"]["hard_failures"] = json!(1_u64);
        report["files"] = json!([{
            "file": path_for_output(app_root, input_abs),
            "error": "no_images_found",
            "hard_fail_reasons": ["no_images_found"],
            "soft_warnings": []
        }]);
        return report;
    }

    let mut hard_failures = 0_u64;
    let mut soft_warnings = 0_u64;
    let mut file_entries = Vec::<Value>::with_capacity(images.len());

    for img_path in images {
        let file_display = path_for_output(app_root, img_path.as_path());
        match compute_chroma_delta_from_image_path(img_path.as_path()) {
            Ok(chroma) => {
                let grayscale_like = chroma <= max_chroma_delta;
                let mut hard = Vec::<&str>::new();
                let mut soft = Vec::<&str>::new();
                if enforce_grayscale && !grayscale_like {
                    hard.push("not_grayscale_like");
                }
                if chroma > max_chroma_delta {
                    if fail_on_chroma_exceed {
                        hard.push("chroma_exceeds_threshold");
                    } else {
                        soft.push("chroma_exceeds_threshold");
                    }
                }
                if !hard.is_empty() {
                    hard_failures += 1;
                }
                if !soft.is_empty() {
                    soft_warnings += 1;
                }
                file_entries.push(json!({
                    "file": file_display,
                    "chroma_delta": round_to_4(chroma),
                    "grayscale_like": grayscale_like,
                    "hard_fail_reasons": hard,
                    "soft_warnings": soft,
                }));
            }
            Err(err) => {
                hard_failures += 1;
                file_entries.push(json!({
                    "file": file_display,
                    "chroma_delta": Value::Null,
                    "grayscale_like": Value::Null,
                    "error": err.to_string(),
                    "hard_fail_reasons": ["image_read_failed"],
                    "soft_warnings": [],
                }));
            }
        }
    }

    report["summary"]["hard_failures"] = json!(hard_failures);
    report["summary"]["soft_warnings"] = json!(soft_warnings);
    report["files"] = Value::Array(file_entries);
    report
}

fn compute_chroma_delta_from_image_path(path: &Path) -> Result<f64, image::ImageError> {
    let image = image::open(path)?;
    Ok(compute_chroma_delta_for_image(&image))
}

fn compute_chroma_delta_for_image(image: &DynamicImage) -> f64 {
    let rgb = image.to_rgb8();
    let mut rg_sum = 0.0_f64;
    let mut rb_sum = 0.0_f64;
    let mut gb_sum = 0.0_f64;
    let mut count = 0_u64;
    for pixel in rgb.pixels() {
        let [r, g, b] = pixel.0;
        rg_sum += f64::from((i16::from(r) - i16::from(g)).abs());
        rb_sum += f64::from((i16::from(r) - i16::from(b)).abs());
        gb_sum += f64::from((i16::from(g) - i16::from(b)).abs());
        count += 1;
    }
    if count == 0 {
        return 0.0;
    }
    ((rg_sum / count as f64) + (rb_sum / count as f64) + (gb_sum / count as f64)) / 3.0
}

fn round_to_4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn resolve_bgremove_output_path(
    input_file_abs: &Path,
    input_root_abs: &Path,
    output_root_abs: &Path,
    input_is_dir: bool,
    format: &str,
) -> Result<PathBuf, std::io::Error> {
    if !input_is_dir {
        let out_meta = fs::metadata(output_root_abs).ok();
        let out_str = output_root_abs.to_string_lossy();
        let as_dir = out_str.ends_with('/')
            || out_str.ends_with('\\')
            || out_meta.as_ref().map(|m| m.is_dir()).unwrap_or(false)
            || !is_image_path(output_root_abs);
        if as_dir {
            fs::create_dir_all(output_root_abs)?;
            let base_raw = input_file_abs
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or("image");
            let base = {
                let s = sanitize_id(base_raw);
                if s.is_empty() {
                    String::from("image")
                } else {
                    s
                }
            };
            return Ok(output_root_abs.join(format!("{base}.{format}")));
        }
        if let Some(parent) = output_root_abs.parent() {
            fs::create_dir_all(parent)?;
        }
        return Ok(output_root_abs.to_path_buf());
    }

    let rel = input_file_abs
        .strip_prefix(input_root_abs)
        .unwrap_or(input_file_abs);
    let mut dst = output_root_abs.join(rel);
    dst.set_extension(format);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(dst)
}

fn sanitize_id(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_was_sep = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        let keep = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_';
        if keep {
            out.push(ch);
            last_was_sep = false;
            continue;
        }
        if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

fn make_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{:03}", now.as_secs(), now.subsec_millis())
}

fn archive_existing_target(
    target_abs: &Path,
    archive_dir_abs: &Path,
    tag: &str,
) -> Result<Option<PathBuf>, std::io::Error> {
    if !target_abs.exists() {
        return Ok(None);
    }
    if !fs::metadata(target_abs)?.is_file() {
        return Ok(None);
    }
    fs::create_dir_all(archive_dir_abs)?;
    let ext = target_abs
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let base_raw = target_abs
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or("file");
    let base = {
        let s = sanitize_id(base_raw);
        if s.is_empty() {
            String::from("file")
        } else {
            s
        }
    };
    let archived = archive_dir_abs.join(format!("{base}_{tag}_{}{}", make_stamp(), ext));
    fs::rename(target_abs, archived.as_path())?;
    Ok(Some(archived))
}

fn mime_for_path(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|v| v.to_str())
        .map(|v| v.trim().to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" => String::from("image/png"),
        "jpg" | "jpeg" => String::from("image/jpeg"),
        "webp" => String::from("image/webp"),
        "bmp" => String::from("image/bmp"),
        "tif" | "tiff" => String::from("image/tiff"),
        "gif" => String::from("image/gif"),
        _ => String::from("application/octet-stream"),
    }
}

fn load_dotenv_map(app_root: &Path) -> Result<HashMap<String, String>, std::io::Error> {
    let path = app_root.join(".env");
    if !path.is_file() {
        return Ok(HashMap::new());
    }
    let raw = fs::read_to_string(path)?;
    Ok(parse_dotenv_content(raw.as_str()))
}

fn parse_dotenv_content(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in raw.lines() {
        let mut value = line.trim();
        if value.is_empty() || value.starts_with('#') {
            continue;
        }
        if let Some(rest) = value.strip_prefix("export ") {
            value = rest.trim_start();
        }
        let Some((key_raw, value_raw)) = value.split_once('=') else {
            continue;
        };
        let key = key_raw.trim();
        if key.is_empty() {
            continue;
        }
        let mut parsed = value_raw.trim().to_string();
        if (parsed.starts_with('"') && parsed.ends_with('"'))
            || (parsed.starts_with('\'') && parsed.ends_with('\''))
        {
            if parsed.len() >= 2 {
                parsed = parsed[1..parsed.len() - 1].to_string();
            }
        } else if let Some((before_comment, _)) = parsed.split_once(" #") {
            parsed = before_comment.trim_end().to_string();
        }
        out.insert(key.to_string(), parsed);
    }
    out
}

fn load_color_adapter_config(
    app_root: &Path,
    config_path: Option<&str>,
) -> Result<(Option<String>, Option<String>), ToolAdapterError> {
    let Some(config_path) = config_path.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok((None, None));
    };
    let path = resolve_under_root(app_root, config_path);
    if !path.is_file() {
        return Err(ToolAdapterError::Native(format!(
            "postprocess config not found: {}",
            path_for_output(app_root, path.as_path())
        )));
    }
    let raw = fs::read_to_string(path.as_path()).map_err(ToolAdapterError::Io)?;
    let parsed: Value =
        serde_json::from_str(raw.as_str()).map_err(|source| ToolAdapterError::JsonDecode {
            source,
            stdout: raw,
        })?;
    let color = parsed.get("color").and_then(Value::as_object);
    let default_profile = color
        .and_then(|obj| obj.get("default_profile"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let settings_file = color
        .and_then(|obj| obj.get("settings_file"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    Ok((default_profile, settings_file))
}

#[derive(Debug, Clone)]
struct UpscaleAdapterConfig {
    backend: String,
    scale: u8,
    tile: u32,
    format: String,
    ncnn_binary: String,
    ncnn_model_dir: String,
    ncnn_model_name: String,
    python_bin: String,
    python_model_name: String,
    python_tile: u32,
    python_tile_pad: u32,
    python_pre_pad: u32,
    python_fp32: bool,
    python_gpu_id: Option<i64>,
}

impl Default for UpscaleAdapterConfig {
    fn default() -> Self {
        Self {
            backend: String::from("python"),
            scale: 2,
            tile: 0,
            format: String::from("png"),
            ncnn_binary: String::from("tools/realesrgan/realesrgan-ncnn-vulkan"),
            ncnn_model_dir: String::from("tools/realesrgan/models"),
            ncnn_model_name: String::from("realesrgan-x4plus"),
            python_bin: String::from("tools/realesrgan-python/.venv/bin/python"),
            python_model_name: String::from("RealESRGAN_x4plus"),
            python_tile: 0,
            python_tile_pad: 10,
            python_pre_pad: 0,
            python_fp32: false,
            python_gpu_id: None,
        }
    }
}

fn load_upscale_adapter_config(
    app_root: &Path,
    config_path: Option<&str>,
) -> Result<UpscaleAdapterConfig, ToolAdapterError> {
    let Some(config_path) = config_path.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(UpscaleAdapterConfig::default());
    };
    let path = resolve_under_root(app_root, config_path);
    if !path.is_file() {
        return Err(ToolAdapterError::Native(format!(
            "postprocess config not found: {}",
            path_for_output(app_root, path.as_path())
        )));
    }
    let raw = fs::read_to_string(path.as_path()).map_err(ToolAdapterError::Io)?;
    let parsed: Value =
        serde_json::from_str(raw.as_str()).map_err(|source| ToolAdapterError::JsonDecode {
            source,
            stdout: raw,
        })?;
    let mut cfg = UpscaleAdapterConfig::default();
    let Some(upscale) = parsed.get("upscale").and_then(Value::as_object) else {
        return Ok(cfg);
    };
    if let Some(v) = upscale.get("backend").and_then(Value::as_str) {
        let backend = v.trim().to_ascii_lowercase();
        if matches!(backend.as_str(), "ncnn" | "python") {
            cfg.backend = backend;
        }
    }
    if let Some(v) = upscale
        .get("binary")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        cfg.ncnn_binary = v.to_string();
    }
    if let Some(v) = upscale
        .get("model_dir")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        cfg.ncnn_model_dir = v.to_string();
    }
    if let Some(v) = upscale
        .get("model_name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        cfg.ncnn_model_name = v.to_string();
    }
    if let Some(v) = upscale
        .get("scale")
        .and_then(Value::as_u64)
        .and_then(|v| u8::try_from(v).ok())
        .filter(|v| *v >= 1)
    {
        cfg.scale = v;
    }
    if let Some(v) = upscale
        .get("tile")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
    {
        cfg.tile = v;
    }
    if let Some(v) = upscale
        .get("format")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        cfg.format = v.to_ascii_lowercase();
    }
    if let Some(py) = upscale.get("python").and_then(Value::as_object) {
        if let Some(v) = py
            .get("python_bin")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.python_bin = v.to_string();
        }
        if let Some(v) = py
            .get("model_name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.python_model_name = v.to_string();
        }
        if let Some(v) = py
            .get("tile")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
        {
            cfg.python_tile = v;
        } else {
            cfg.python_tile = cfg.tile;
        }
        if let Some(v) = py
            .get("tile_pad")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
        {
            cfg.python_tile_pad = v;
        }
        if let Some(v) = py
            .get("pre_pad")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
        {
            cfg.python_pre_pad = v;
        }
        if let Some(v) = py.get("fp32").and_then(Value::as_bool) {
            cfg.python_fp32 = v;
        }
        if let Some(v) = py.get("gpu_id") {
            cfg.python_gpu_id = v.as_i64();
        }
    }
    Ok(cfg)
}

#[derive(Debug, Clone)]
struct BgRemoveAdapterConfig {
    backends: Vec<String>,
    format: String,
    size: String,
    crop: bool,
    rembg_python_bin: String,
    rembg_model: String,
    photoroom_endpoint: String,
    photoroom_api_key_env: String,
    removebg_endpoint: String,
    removebg_api_key_env: String,
    openai_enabled: bool,
    openai_required: bool,
    openai_api_key_env: String,
    openai_model: Option<String>,
    openai_quality: Option<String>,
    openai_input_fidelity: String,
    openai_background: String,
    openai_prompt: String,
}

impl Default for BgRemoveAdapterConfig {
    fn default() -> Self {
        Self {
            backends: vec![String::from("rembg")],
            format: String::from("png"),
            size: String::from("auto"),
            crop: false,
            rembg_python_bin: String::from("tools/rembg/.venv/bin/python"),
            rembg_model: String::from("u2net"),
            photoroom_endpoint: String::from("https://sdk.photoroom.com/v1/segment"),
            photoroom_api_key_env: String::from("PHOTOROOM_API_KEY"),
            removebg_endpoint: String::from("https://api.remove.bg/v1.0/removebg"),
            removebg_api_key_env: String::from("REMOVE_BG_API_KEY"),
            openai_enabled: true,
            openai_required: true,
            openai_api_key_env: String::from("OPENAI_API_KEY"),
            openai_model: None,
            openai_quality: None,
            openai_input_fidelity: String::from("high"),
            openai_background: String::from("transparent"),
            openai_prompt: String::from(
                "Refine this subject cutout for production compositing. Keep identity and details unchanged. Clean edge artifacts and preserve transparency.",
            ),
        }
    }
}

fn load_bgremove_adapter_config(
    app_root: &Path,
    config_path: Option<&str>,
) -> Result<BgRemoveAdapterConfig, ToolAdapterError> {
    let Some(config_path) = config_path.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(BgRemoveAdapterConfig::default());
    };
    let path = resolve_under_root(app_root, config_path);
    if !path.is_file() {
        return Err(ToolAdapterError::Native(format!(
            "postprocess config not found: {}",
            path_for_output(app_root, path.as_path())
        )));
    }
    let raw = fs::read_to_string(path.as_path()).map_err(ToolAdapterError::Io)?;
    let parsed: Value =
        serde_json::from_str(raw.as_str()).map_err(|source| ToolAdapterError::JsonDecode {
            source,
            stdout: raw,
        })?;
    let mut cfg = BgRemoveAdapterConfig::default();
    let Some(bg) = parsed.get("bg_remove").and_then(Value::as_object) else {
        return Ok(cfg);
    };
    if let Some(fmt) = bg
        .get("format")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        cfg.format = match fmt.to_ascii_lowercase().as_str() {
            "jpeg" => String::from("jpg"),
            "png" | "jpg" | "webp" => fmt.to_ascii_lowercase(),
            _ => cfg.format,
        };
    }
    if let Some(v) = bg
        .get("size")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        cfg.size = v.to_string();
    }
    if let Some(v) = bg.get("crop").and_then(Value::as_bool) {
        cfg.crop = v;
    }
    if let Some(backends) = bg.get("backends").and_then(Value::as_array) {
        let parsed_backends = backends
            .iter()
            .filter_map(Value::as_str)
            .map(|v| v.trim().to_ascii_lowercase())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        if !parsed_backends.is_empty() {
            cfg.backends = parsed_backends;
        }
    }
    if let Some(rembg) = bg.get("rembg").and_then(Value::as_object) {
        if let Some(v) = rembg
            .get("python_bin")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.rembg_python_bin = v.to_string();
        }
        if let Some(v) = rembg
            .get("model")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.rembg_model = v.to_string();
        }
    }
    if let Some(photoroom) = bg.get("photoroom").and_then(Value::as_object) {
        if let Some(v) = photoroom
            .get("endpoint")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.photoroom_endpoint = v.to_string();
        }
        if let Some(v) = photoroom
            .get("api_key_env")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.photoroom_api_key_env = v.to_string();
        }
    }
    if let Some(removebg) = bg.get("removebg").and_then(Value::as_object) {
        if let Some(v) = removebg
            .get("endpoint")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.removebg_endpoint = v.to_string();
        }
        if let Some(v) = removebg
            .get("api_key_env")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.removebg_api_key_env = v.to_string();
        }
    }
    if let Some(openai) = bg.get("openai").and_then(Value::as_object) {
        if let Some(v) = openai.get("enabled").and_then(Value::as_bool) {
            cfg.openai_enabled = v;
        }
        if let Some(v) = openai.get("required").and_then(Value::as_bool) {
            cfg.openai_required = v;
        }
        if let Some(v) = openai
            .get("api_key_env")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.openai_api_key_env = v.to_string();
        }
        if let Some(v) = openai
            .get("model")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.openai_model = Some(v.to_string());
        }
        if let Some(v) = openai
            .get("quality")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.openai_quality = Some(v.to_string());
        }
        if let Some(v) = openai
            .get("input_fidelity")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.openai_input_fidelity = v.to_string();
        }
        if let Some(v) = openai
            .get("background")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.openai_background = v.to_string();
        }
        if let Some(v) = openai
            .get("prompt")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.openai_prompt = v.to_string();
        }
    }
    Ok(cfg)
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
    #[error("tool adapter filesystem error: {0}")]
    Io(#[source] std::io::Error),
    #[error("{0}")]
    Native(String),
}

pub fn default_native_qa_archive_script_tool_adapters(
) -> NativeQaArchiveScriptToolAdapters<StdPipelineCommandRunner> {
    NativeQaArchiveScriptToolAdapters::new(
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
        assert_eq!(seen[0].args.first().map(String::as_str), Some("-c"));
        assert!(seen[0].args.windows(2).any(|w| w == ["--outscale", "4"]));
        assert!(seen[0].args.windows(2).any(|w| w == ["--tile", "8"]));
        assert!(seen[0].args.iter().any(|v| v == "--fp32"));
        assert!(seen[0].args.windows(2).any(|w| w == ["--gpu-id", "0"]));

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
        assert_eq!(seen[0].args.first().map(String::as_str), Some("-c"));
        assert!(seen[0]
            .args
            .windows(2)
            .any(|w| w == ["--model", "u2net_human_seg"]));
        assert!(seen[0].args.windows(2).any(|w| w == ["--format", "webp"]));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_bgremove_rejects_unsupported_backend_without_script_fallback() {
        let app_root = temp_app_root_with_script();
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
        assert_eq!(seen[0].args.first().map(String::as_str), Some("-c"));

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
            .all(|cmd| cmd.args.first().map(String::as_str) == Some("-c")));
        assert!(!seen.iter().any(|cmd| cmd.program == "node"));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn native_bgremove_uses_rust_refine_path_without_script_fallback_when_optional_refine_fails() {
        let app_root = temp_app_root_with_script();
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
        assert_eq!(seen[0].args.first().map(String::as_str), Some("-c"));
        assert!(!seen.iter().any(|cmd| cmd.program == "node"));

        let _ = std::fs::remove_dir_all(app_root);
    }

    #[test]
    fn parse_dotenv_content_supports_export_quotes_and_comments() {
        let parsed = parse_dotenv_content(
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
