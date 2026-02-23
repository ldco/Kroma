use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PipelineSettingsOverlay {
    pub manifest_path: Option<String>,
    pub postprocess_config_path: Option<String>,
    pub post_upscale: Option<bool>,
    pub upscale_backend: Option<String>,
    pub post_color: Option<bool>,
    pub color_profile: Option<String>,
    pub post_bg_remove: Option<bool>,
    pub bg_remove_backends: Option<Vec<String>>,
    pub bg_refine_openai: Option<bool>,
    pub bg_refine_openai_required: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PipelineSettingsLayerPaths {
    pub app_settings_path: Option<String>,
    pub project_settings_path: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PipelineSettingsLayerError {
    #[error("failed to read pipeline settings '{path}': {message}")]
    ReadFile { path: String, message: String },
    #[error("failed to parse pipeline settings JSON '{path}': {message}")]
    ParseJson { path: String, message: String },
    #[error("failed to parse pipeline settings TOML '{path}': {message}")]
    ParseToml { path: String, message: String },
    #[error("pipeline settings root must be a JSON object")]
    RootMustBeObject,
    #[error("pipeline settings field '{field}' has invalid type")]
    InvalidFieldType { field: String },
}

pub fn load_app_pipeline_settings(
    app_root: &Path,
    explicit_path: Option<&str>,
) -> Result<PipelineSettingsOverlay, PipelineSettingsLayerError> {
    if let Some(path) = explicit_path
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .map(|p| if p.is_absolute() { p } else { app_root.join(p) })
    {
        return load_optional_overlay_by_extension(path.as_path());
    }

    let toml_path = app_root.join("config/pipeline.settings.toml");
    if toml_path.exists() {
        return load_optional_overlay_from_toml_path(toml_path.as_path());
    }

    // Back-compat during migration of local developer setups.
    load_optional_overlay_from_json_path(app_root.join("config/pipeline.settings.json").as_path())
}

pub fn load_project_pipeline_settings(
    project_root: Option<&Path>,
    explicit_path: Option<&str>,
) -> Result<PipelineSettingsOverlay, PipelineSettingsLayerError> {
    let Some(path) = explicit_path
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .map(|p| {
            if p.is_absolute() {
                p
            } else if let Some(root) = project_root {
                root.join(p)
            } else {
                p
            }
        })
        .or_else(|| project_root.map(|root| root.join(".kroma").join("pipeline.settings.json")))
    else {
        return Ok(PipelineSettingsOverlay::default());
    };
    load_optional_overlay_from_json_path(path.as_path())
}

pub fn merge_pipeline_settings_overlays(
    app: &PipelineSettingsOverlay,
    project: &PipelineSettingsOverlay,
    overrides: &PipelineSettingsOverlay,
) -> PipelineSettingsOverlay {
    PipelineSettingsOverlay {
        manifest_path: choose_string(
            overrides.manifest_path.as_deref(),
            project.manifest_path.as_deref(),
            app.manifest_path.as_deref(),
        ),
        postprocess_config_path: choose_string(
            overrides.postprocess_config_path.as_deref(),
            project.postprocess_config_path.as_deref(),
            app.postprocess_config_path.as_deref(),
        ),
        post_upscale: overrides
            .post_upscale
            .or(project.post_upscale)
            .or(app.post_upscale),
        upscale_backend: choose_string(
            overrides.upscale_backend.as_deref(),
            project.upscale_backend.as_deref(),
            app.upscale_backend.as_deref(),
        ),
        post_color: overrides
            .post_color
            .or(project.post_color)
            .or(app.post_color),
        color_profile: choose_string(
            overrides.color_profile.as_deref(),
            project.color_profile.as_deref(),
            app.color_profile.as_deref(),
        ),
        post_bg_remove: overrides
            .post_bg_remove
            .or(project.post_bg_remove)
            .or(app.post_bg_remove),
        bg_remove_backends: overrides
            .bg_remove_backends
            .clone()
            .or_else(|| project.bg_remove_backends.clone())
            .or_else(|| app.bg_remove_backends.clone()),
        bg_refine_openai: overrides
            .bg_refine_openai
            .or(project.bg_refine_openai)
            .or(app.bg_refine_openai),
        bg_refine_openai_required: overrides
            .bg_refine_openai_required
            .or(project.bg_refine_openai_required)
            .or(app.bg_refine_openai_required),
    }
}

pub fn parse_pipeline_settings_overlay_json(
    value: &Value,
) -> Result<PipelineSettingsOverlay, PipelineSettingsLayerError> {
    let root = value
        .as_object()
        .ok_or(PipelineSettingsLayerError::RootMustBeObject)?;
    let pipeline_value = root.get("pipeline").unwrap_or(value);
    let pipeline = pipeline_value
        .as_object()
        .ok_or(PipelineSettingsLayerError::RootMustBeObject)?;

    let mut out = PipelineSettingsOverlay::default();
    if let Some(v) = pipeline.get("manifest_path") {
        out.manifest_path = Some(parse_string(v, "manifest_path")?);
    }
    if let Some(v) = pipeline.get("postprocess_config_path") {
        out.postprocess_config_path = Some(parse_string(v, "postprocess_config_path")?);
    }
    if let Some(post) = pipeline.get("postprocess") {
        let post =
            post.as_object()
                .ok_or_else(|| PipelineSettingsLayerError::InvalidFieldType {
                    field: String::from("postprocess"),
                })?;
        if let Some(v) = post.get("upscale") {
            out.post_upscale = Some(parse_bool(v, "postprocess.upscale")?);
        }
        if let Some(v) = post.get("upscale_backend") {
            out.upscale_backend = Some(parse_upscale_backend_string(
                v,
                "postprocess.upscale_backend",
            )?);
        }
        if let Some(v) = post.get("color") {
            out.post_color = Some(parse_bool(v, "postprocess.color")?);
        }
        if let Some(v) = post.get("color_profile") {
            out.color_profile = Some(parse_string(v, "postprocess.color_profile")?);
        }
        if let Some(v) = post.get("bg_remove") {
            out.post_bg_remove = Some(parse_bool(v, "postprocess.bg_remove")?);
        }
        if let Some(v) = post.get("bg_remove_backends") {
            out.bg_remove_backends = Some(parse_string_array(v, "postprocess.bg_remove_backends")?);
        }
        if let Some(v) = post.get("bg_refine_openai") {
            out.bg_refine_openai = Some(parse_bool(v, "postprocess.bg_refine_openai")?);
        }
        if let Some(v) = post.get("bg_refine_openai_required") {
            out.bg_refine_openai_required =
                Some(parse_bool(v, "postprocess.bg_refine_openai_required")?);
        }
    }
    Ok(out)
}

fn load_optional_overlay_by_extension(
    path: &Path,
) -> Result<PipelineSettingsOverlay, PipelineSettingsLayerError> {
    match path
        .extension()
        .and_then(|v| v.to_str())
        .map(|v| v.to_ascii_lowercase())
    {
        Some(ext) if ext == "toml" => load_optional_overlay_from_toml_path(path),
        _ => load_optional_overlay_from_json_path(path),
    }
}

fn load_optional_overlay_from_json_path(
    path: &Path,
) -> Result<PipelineSettingsOverlay, PipelineSettingsLayerError> {
    if !path.exists() {
        return Ok(PipelineSettingsOverlay::default());
    }
    let raw = fs::read_to_string(path).map_err(|error| PipelineSettingsLayerError::ReadFile {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    let parsed = serde_json::from_str::<Value>(raw.as_str()).map_err(|error| {
        PipelineSettingsLayerError::ParseJson {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    parse_pipeline_settings_overlay_json(&parsed)
}

fn load_optional_overlay_from_toml_path(
    path: &Path,
) -> Result<PipelineSettingsOverlay, PipelineSettingsLayerError> {
    if !path.exists() {
        return Ok(PipelineSettingsOverlay::default());
    }
    let raw = fs::read_to_string(path).map_err(|error| PipelineSettingsLayerError::ReadFile {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    let parsed = toml::from_str::<toml::Value>(raw.as_str()).map_err(|error| {
        PipelineSettingsLayerError::ParseToml {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    let json_value =
        serde_json::to_value(parsed).map_err(|error| PipelineSettingsLayerError::ParseToml {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
    parse_pipeline_settings_overlay_json(&json_value)
}

fn choose_string(a: Option<&str>, b: Option<&str>, c: Option<&str>) -> Option<String> {
    a.or(b).or(c).map(str::to_string)
}

fn parse_string(value: &Value, field: &str) -> Result<String, PipelineSettingsLayerError> {
    let parsed = value
        .as_str()
        .map(str::trim)
        .ok_or_else(|| PipelineSettingsLayerError::InvalidFieldType {
            field: field.to_string(),
        })?;
    if parsed.is_empty() {
        return Err(PipelineSettingsLayerError::InvalidFieldType {
            field: field.to_string(),
        });
    }
    Ok(parsed.to_string())
}

fn parse_bool(value: &Value, field: &str) -> Result<bool, PipelineSettingsLayerError> {
    value
        .as_bool()
        .ok_or_else(|| PipelineSettingsLayerError::InvalidFieldType {
            field: field.to_string(),
        })
}

fn parse_string_array(
    value: &Value,
    field: &str,
) -> Result<Vec<String>, PipelineSettingsLayerError> {
    let arr = value
        .as_array()
        .ok_or_else(|| PipelineSettingsLayerError::InvalidFieldType {
            field: field.to_string(),
        })?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        out.push(parse_string(item, field)?);
    }
    Ok(out)
}

fn parse_upscale_backend_string(
    value: &Value,
    field: &str,
) -> Result<String, PipelineSettingsLayerError> {
    let parsed = parse_string(value, field)?.to_ascii_lowercase();
    if matches!(parsed.as_str(), "ncnn" | "python") {
        Ok(parsed)
    } else {
        Err(PipelineSettingsLayerError::InvalidFieldType {
            field: field.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_nested_pipeline_settings_overlay() {
        let overlay = parse_pipeline_settings_overlay_json(&serde_json::json!({
            "pipeline": {
                "manifest_path": "config/manifest.json",
                "postprocess_config_path": "config/post.json",
                "postprocess": {
                    "upscale": true,
                    "upscale_backend": "ncnn",
                    "color": true,
                    "color_profile": "studio",
                    "bg_remove": true,
                    "bg_remove_backends": ["rembg", "photoroom"],
                    "bg_refine_openai": false,
                    "bg_refine_openai_required": false
                }
            }
        }))
        .expect("settings overlay should parse");

        assert_eq!(
            overlay.manifest_path.as_deref(),
            Some("config/manifest.json")
        );
        assert_eq!(
            overlay.postprocess_config_path.as_deref(),
            Some("config/post.json")
        );
        assert_eq!(overlay.post_upscale, Some(true));
        assert_eq!(overlay.upscale_backend.as_deref(), Some("ncnn"));
        assert_eq!(overlay.post_color, Some(true));
        assert_eq!(overlay.color_profile.as_deref(), Some("studio"));
        assert_eq!(overlay.post_bg_remove, Some(true));
        assert_eq!(
            overlay.bg_remove_backends.as_deref(),
            Some(&[String::from("rembg"), String::from("photoroom")][..])
        );
        assert_eq!(overlay.bg_refine_openai, Some(false));
        assert_eq!(overlay.bg_refine_openai_required, Some(false));
    }

    #[test]
    fn merges_layers_with_override_precedence() {
        let app = PipelineSettingsOverlay {
            manifest_path: Some(String::from("app.json")),
            post_upscale: Some(false),
            ..PipelineSettingsOverlay::default()
        };
        let project = PipelineSettingsOverlay {
            manifest_path: Some(String::from("project.json")),
            post_upscale: Some(true),
            color_profile: Some(String::from("proj")),
            ..PipelineSettingsOverlay::default()
        };
        let overrides = PipelineSettingsOverlay {
            color_profile: Some(String::from("cli")),
            ..PipelineSettingsOverlay::default()
        };

        let merged = merge_pipeline_settings_overlays(&app, &project, &overrides);
        assert_eq!(merged.manifest_path.as_deref(), Some("project.json"));
        assert_eq!(merged.post_upscale, Some(true));
        assert_eq!(merged.color_profile.as_deref(), Some("cli"));
    }

    #[test]
    fn loads_optional_app_toml_and_project_json_files() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_settings_layer_{stamp}"));
        let app_dir = root.join("app/config");
        let project_dir = root.join("project/.kroma");
        fs::create_dir_all(app_dir.as_path()).expect("app dir");
        fs::create_dir_all(project_dir.as_path()).expect("project dir");
        fs::write(
            app_dir.join("pipeline.settings.toml"),
            r#"[pipeline]
manifest_path = "app.toml.json"

[pipeline.postprocess]
upscale = true
upscale_backend = "ncnn"
"#,
        )
        .expect("app settings write");
        fs::write(
            project_dir.join("pipeline.settings.json"),
            r#"{"pipeline":{"manifest_path":"project.json"}}"#,
        )
        .expect("project settings write");

        let app_overlay =
            load_app_pipeline_settings(root.join("app").as_path(), None).expect("app load");
        let project_overlay =
            load_project_pipeline_settings(Some(root.join("project").as_path()), None)
                .expect("project load");

        assert_eq!(app_overlay.manifest_path.as_deref(), Some("app.toml.json"));
        assert_eq!(app_overlay.post_upscale, Some(true));
        assert_eq!(app_overlay.upscale_backend.as_deref(), Some("ncnn"));
        assert_eq!(
            project_overlay.manifest_path.as_deref(),
            Some("project.json")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn app_loader_falls_back_to_json_when_toml_missing() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_settings_layer_json_fallback_{stamp}"));
        let app_dir = root.join("app/config");
        fs::create_dir_all(app_dir.as_path()).expect("app dir");
        fs::write(
            app_dir.join("pipeline.settings.json"),
            r#"{"pipeline":{"manifest_path":"legacy.json"}}"#,
        )
        .expect("json app settings write");

        let app_overlay =
            load_app_pipeline_settings(root.join("app").as_path(), None).expect("app load");
        assert_eq!(app_overlay.manifest_path.as_deref(), Some("legacy.json"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_empty_string_settings_values() {
        let err = parse_pipeline_settings_overlay_json(&serde_json::json!({
            "pipeline": {
                "manifest_path": "   "
            }
        }))
        .expect_err("empty manifest_path should fail");

        assert_eq!(
            err,
            PipelineSettingsLayerError::InvalidFieldType {
                field: String::from("manifest_path")
            }
        );
    }

    #[test]
    fn explicit_project_settings_relative_path_resolves_from_project_root() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_project_settings_rel_{stamp}"));
        let project_root = root.join("project");
        let nested = project_root.join(".kroma");
        fs::create_dir_all(nested.as_path()).expect("project settings dir");
        fs::write(
            nested.join("custom.json"),
            r#"{"pipeline":{"manifest_path":"from-project-root.json"}}"#,
        )
        .expect("project settings write");

        let overlay =
            load_project_pipeline_settings(Some(project_root.as_path()), Some(".kroma/custom.json"))
                .expect("project settings should load");
        assert_eq!(
            overlay.manifest_path.as_deref(),
            Some("from-project-root.json")
        );

        let _ = fs::remove_dir_all(root);
    }
}
