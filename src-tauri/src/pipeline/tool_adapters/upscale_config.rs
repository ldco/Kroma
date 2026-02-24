use std::fs;
use std::path::Path;

use serde_json::Value;

use super::pathing::{path_for_output, resolve_request_path_under_root};
use super::ToolAdapterError;

#[derive(Debug, Clone)]
pub(super) struct UpscaleAdapterConfig {
    pub(super) backend: String,
    pub(super) scale: u8,
    pub(super) tile: u32,
    pub(super) format: String,
    pub(super) ncnn_binary: String,
    pub(super) ncnn_model_dir: String,
    pub(super) ncnn_model_name: String,
    pub(super) python_bin: String,
    pub(super) python_inference_script: String,
    pub(super) python_model_name: String,
    pub(super) python_tile: u32,
    pub(super) python_tile_pad: u32,
    pub(super) python_pre_pad: u32,
    pub(super) python_fp32: bool,
    pub(super) python_gpu_id: Option<i64>,
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
            python_inference_script: String::from(
                "tools/realesrgan-python/src/Real-ESRGAN/inference_realesrgan.py",
            ),
            python_model_name: String::from("RealESRGAN_x4plus"),
            python_tile: 0,
            python_tile_pad: 10,
            python_pre_pad: 0,
            python_fp32: false,
            python_gpu_id: None,
        }
    }
}

pub(super) fn load_upscale_adapter_config(
    app_root: &Path,
    config_path: Option<&str>,
) -> Result<UpscaleAdapterConfig, ToolAdapterError> {
    let Some(config_path) = config_path.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(UpscaleAdapterConfig::default());
    };
    let path = resolve_request_path_under_root(app_root, config_path, "postprocess_config_path")?;
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
            .get("inference_script")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            cfg.python_inference_script = v.to_string();
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
