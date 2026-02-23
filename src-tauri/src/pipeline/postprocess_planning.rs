use std::fs;
use std::path::Path;

use serde_json::Value;
use thiserror::Error;

use crate::pipeline::execution::ExecutionPlannedPostprocessRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostprocessPlanningConfig {
    pub upscale_backend: String,
    pub color_default_profile: String,
    pub bg_remove_backends: Vec<String>,
    pub bg_refine_openai_enabled: bool,
    pub bg_refine_openai_required: bool,
}

impl Default for PostprocessPlanningConfig {
    fn default() -> Self {
        Self {
            upscale_backend: String::from("python"),
            color_default_profile: String::from("neutral"),
            bg_remove_backends: vec![String::from("rembg")],
            bg_refine_openai_enabled: true,
            bg_refine_openai_required: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PostprocessPlanningOverrides {
    pub post_upscale: bool,
    pub post_color: bool,
    pub post_bg_remove: bool,
    pub upscale_backend: Option<String>,
    pub color_profile: Option<String>,
    pub bg_remove_backends: Vec<String>,
    pub bg_refine_openai: Option<bool>,
    pub bg_refine_openai_required: Option<bool>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PostprocessPlanningError {
    #[error("failed to read postprocess config '{path}': {message}")]
    ReadFile { path: String, message: String },
    #[error("failed to parse postprocess config JSON '{path}': {message}")]
    ParseJson { path: String, message: String },
    #[error("postprocess config root must be a JSON object")]
    RootMustBeObject,
    #[error("postprocess config field '{field}' has invalid type")]
    InvalidFieldType { field: String },
    #[error("invalid upscale backend '{0}'. Expected ncnn|python.")]
    InvalidUpscaleBackend(String),
    #[error("invalid background-remove backend '{0}'. Expected rembg|photoroom|removebg.")]
    InvalidBgRemoveBackend(String),
}

pub fn load_postprocess_planning_config(
    app_root: &Path,
    config_path: Option<&str>,
) -> Result<PostprocessPlanningConfig, PostprocessPlanningError> {
    let Some(config_path) = config_path.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(PostprocessPlanningConfig::default());
    };
    let path = if Path::new(config_path).is_absolute() {
        Path::new(config_path).to_path_buf()
    } else {
        app_root.join(config_path)
    };
    let raw =
        fs::read_to_string(path.as_path()).map_err(|error| PostprocessPlanningError::ReadFile {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
    let parsed = serde_json::from_str::<Value>(raw.as_str()).map_err(|error| {
        PostprocessPlanningError::ParseJson {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    parse_postprocess_planning_config_json(&parsed)
}

pub fn parse_postprocess_planning_config_json(
    value: &Value,
) -> Result<PostprocessPlanningConfig, PostprocessPlanningError> {
    let root = value
        .as_object()
        .ok_or(PostprocessPlanningError::RootMustBeObject)?;
    let mut cfg = PostprocessPlanningConfig::default();

    if let Some(upscale) = root.get("upscale") {
        let upscale_obj =
            upscale
                .as_object()
                .ok_or_else(|| PostprocessPlanningError::InvalidFieldType {
                    field: String::from("upscale"),
                })?;
        if let Some(backend) = upscale_obj.get("backend") {
            let raw =
                backend
                    .as_str()
                    .ok_or_else(|| PostprocessPlanningError::InvalidFieldType {
                        field: String::from("upscale.backend"),
                    })?;
            cfg.upscale_backend = parse_upscale_backend(raw)?;
        }
    }

    if let Some(color) = root.get("color") {
        let color_obj =
            color
                .as_object()
                .ok_or_else(|| PostprocessPlanningError::InvalidFieldType {
                    field: String::from("color"),
                })?;
        if let Some(default_profile) = color_obj.get("default_profile") {
            let raw = default_profile.as_str().ok_or_else(|| {
                PostprocessPlanningError::InvalidFieldType {
                    field: String::from("color.default_profile"),
                }
            })?;
            cfg.color_default_profile = raw.trim().to_string();
        }
    }

    if let Some(bg_remove) = root.get("bg_remove") {
        let bg_remove_obj =
            bg_remove
                .as_object()
                .ok_or_else(|| PostprocessPlanningError::InvalidFieldType {
                    field: String::from("bg_remove"),
                })?;
        if let Some(backends) = bg_remove_obj.get("backends") {
            cfg.bg_remove_backends =
                parse_bg_remove_backends_value(backends, "bg_remove.backends")?;
        }
        if let Some(openai) = bg_remove_obj.get("openai") {
            let openai_obj =
                openai
                    .as_object()
                    .ok_or_else(|| PostprocessPlanningError::InvalidFieldType {
                        field: String::from("bg_remove.openai"),
                    })?;
            if let Some(enabled) = openai_obj.get("enabled") {
                cfg.bg_refine_openai_enabled = enabled.as_bool().ok_or_else(|| {
                    PostprocessPlanningError::InvalidFieldType {
                        field: String::from("bg_remove.openai.enabled"),
                    }
                })?;
            }
            if let Some(required) = openai_obj.get("required") {
                cfg.bg_refine_openai_required = required.as_bool().ok_or_else(|| {
                    PostprocessPlanningError::InvalidFieldType {
                        field: String::from("bg_remove.openai.required"),
                    }
                })?;
            }
        }
    }

    Ok(cfg)
}

pub fn resolve_planned_postprocess_record(
    cfg: &PostprocessPlanningConfig,
    overrides: &PostprocessPlanningOverrides,
) -> Result<ExecutionPlannedPostprocessRecord, PostprocessPlanningError> {
    let post_upscale = overrides.post_upscale;
    let post_color = overrides.post_color;
    let post_bg_remove = overrides.post_bg_remove;

    let upscale_backend = if post_upscale {
        let resolved = overrides
            .upscale_backend
            .as_deref()
            .unwrap_or(cfg.upscale_backend.as_str());
        Some(parse_upscale_backend(resolved)?)
    } else {
        None
    };

    let color_profile = if post_color {
        Some(
            overrides
                .color_profile
                .as_deref()
                .map(str::trim)
                .unwrap_or(cfg.color_default_profile.as_str())
                .to_string(),
        )
    } else {
        None
    };

    let bg_remove_backends = if post_bg_remove {
        if overrides.bg_remove_backends.is_empty() {
            cfg.bg_remove_backends.clone()
        } else {
            validate_bg_remove_backends(
                overrides
                    .bg_remove_backends
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_slice(),
            )?
        }
    } else {
        Vec::new()
    };

    let bg_refine_openai = if post_bg_remove {
        overrides
            .bg_refine_openai
            .unwrap_or(cfg.bg_refine_openai_enabled)
    } else {
        false
    };
    let bg_refine_openai_required = if post_bg_remove {
        overrides
            .bg_refine_openai_required
            .unwrap_or(cfg.bg_refine_openai_required)
    } else {
        false
    };

    let mut pipeline_order = vec![String::from("generate")];
    if post_bg_remove {
        pipeline_order.push(String::from("bg_remove"));
        if bg_refine_openai {
            pipeline_order.push(String::from("bg_refine_openai"));
        }
    }
    if post_upscale {
        pipeline_order.push(String::from("upscale"));
    }
    if post_color {
        pipeline_order.push(String::from("color"));
    }

    Ok(ExecutionPlannedPostprocessRecord {
        upscale: post_upscale,
        upscale_backend,
        color: post_color,
        color_profile,
        bg_remove: post_bg_remove,
        bg_remove_backends,
        bg_refine_openai,
        bg_refine_openai_required,
        pipeline_order,
    })
}

fn parse_upscale_backend(value: &str) -> Result<String, PostprocessPlanningError> {
    let backend = value.trim().to_ascii_lowercase();
    if matches!(backend.as_str(), "ncnn" | "python") {
        Ok(backend)
    } else {
        Err(PostprocessPlanningError::InvalidUpscaleBackend(backend))
    }
}

fn parse_bg_remove_backends_value(
    value: &Value,
    field: &str,
) -> Result<Vec<String>, PostprocessPlanningError> {
    let arr = value
        .as_array()
        .ok_or_else(|| PostprocessPlanningError::InvalidFieldType {
            field: field.to_string(),
        })?;
    let mut values = Vec::with_capacity(arr.len());
    for item in arr {
        let item = item
            .as_str()
            .ok_or_else(|| PostprocessPlanningError::InvalidFieldType {
                field: field.to_string(),
            })?;
        values.push(item);
    }
    validate_bg_remove_backends(values.as_slice())
}

fn validate_bg_remove_backends(values: &[&str]) -> Result<Vec<String>, PostprocessPlanningError> {
    let mut out = Vec::with_capacity(values.len());
    for item in values {
        let normalized = item.trim().to_ascii_lowercase();
        if !matches!(normalized.as_str(), "rembg" | "photoroom" | "removebg") {
            return Err(PostprocessPlanningError::InvalidBgRemoveBackend(normalized));
        }
        out.push(normalized);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_config_json_uses_script_parity_defaults_for_planning_subset() {
        let cfg = parse_postprocess_planning_config_json(&serde_json::json!({}))
            .expect("empty config should use defaults");

        assert_eq!(cfg.upscale_backend, "python");
        assert_eq!(cfg.color_default_profile, "neutral");
        assert_eq!(cfg.bg_remove_backends, vec!["rembg"]);
        assert!(cfg.bg_refine_openai_enabled);
        assert!(cfg.bg_refine_openai_required);
    }

    #[test]
    fn parse_config_json_merges_nested_overrides_for_planning_subset() {
        let cfg = parse_postprocess_planning_config_json(&serde_json::json!({
            "upscale": { "backend": "ncnn" },
            "color": { "default_profile": "cinematic-v2" },
            "bg_remove": {
                "backends": ["PhotoRoom", "removebg"],
                "openai": { "enabled": false, "required": false }
            }
        }))
        .expect("config should parse");

        assert_eq!(cfg.upscale_backend, "ncnn");
        assert_eq!(cfg.color_default_profile, "cinematic-v2");
        assert_eq!(cfg.bg_remove_backends, vec!["photoroom", "removebg"]);
        assert!(!cfg.bg_refine_openai_enabled);
        assert!(!cfg.bg_refine_openai_required);
    }

    #[test]
    fn resolve_planned_postprocess_record_matches_script_ordering_and_toggles() {
        let cfg = PostprocessPlanningConfig {
            upscale_backend: String::from("python"),
            color_default_profile: String::from("neutral"),
            bg_remove_backends: vec![String::from("rembg")],
            bg_refine_openai_enabled: true,
            bg_refine_openai_required: true,
        };
        let record = resolve_planned_postprocess_record(
            &cfg,
            &PostprocessPlanningOverrides {
                post_upscale: true,
                post_color: true,
                post_bg_remove: true,
                upscale_backend: Some(String::from("ncnn")),
                color_profile: Some(String::from("cinematic-v2")),
                bg_remove_backends: vec![String::from("removebg")],
                bg_refine_openai: Some(false),
                bg_refine_openai_required: Some(false),
            },
        )
        .expect("planned postprocess should resolve");

        assert!(record.upscale);
        assert_eq!(record.upscale_backend.as_deref(), Some("ncnn"));
        assert!(record.color);
        assert_eq!(record.color_profile.as_deref(), Some("cinematic-v2"));
        assert!(record.bg_remove);
        assert_eq!(record.bg_remove_backends, vec!["removebg"]);
        assert!(!record.bg_refine_openai);
        assert!(!record.bg_refine_openai_required);
        assert_eq!(
            record.pipeline_order,
            vec!["generate", "bg_remove", "upscale", "color"]
        );
    }

    #[test]
    fn load_config_reads_file_and_validates_values() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_post_cfg_{stamp}"));
        fs::create_dir_all(root.as_path()).expect("temp root should exist");
        let cfg_path = root.join("post.json");
        fs::write(
            cfg_path.as_path(),
            r#"{"upscale":{"backend":"ncnn"},"bg_remove":{"backends":["rembg"]}}"#,
        )
        .expect("config file should be written");

        let cfg = load_postprocess_planning_config(root.as_path(), Some("post.json"))
            .expect("config file should load");
        assert_eq!(cfg.upscale_backend, "ncnn");

        let _ = fs::remove_dir_all(root);
    }
}
