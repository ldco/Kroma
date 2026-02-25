use std::fs;
use std::path::Path;

use serde_json::Value;

use super::pathing::{path_for_output, resolve_request_path_under_root};
use super::ToolAdapterError;

#[derive(Debug, Clone)]
pub(super) struct BgRemoveAdapterConfig {
    pub(super) backends: Vec<String>,
    pub(super) format: String,
    pub(super) size: String,
    pub(super) crop: bool,
    pub(super) rembg_python_bin: String,
    pub(super) rembg_model: String,
    pub(super) photoroom_endpoint: String,
    pub(super) photoroom_api_key_env: String,
    pub(super) removebg_endpoint: String,
    pub(super) removebg_api_key_env: String,
    pub(super) openai_enabled: bool,
    pub(super) openai_required: bool,
    pub(super) openai_api_key_env: String,
    pub(super) openai_model: Option<String>,
    pub(super) openai_quality: Option<String>,
    pub(super) openai_input_fidelity: String,
    pub(super) openai_background: String,
    pub(super) openai_prompt: String,
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

pub(super) fn load_bgremove_adapter_config(
    app_root: &Path,
    config_path: Option<&str>,
) -> Result<BgRemoveAdapterConfig, ToolAdapterError> {
    let Some(config_path) = config_path.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(BgRemoveAdapterConfig::default());
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
