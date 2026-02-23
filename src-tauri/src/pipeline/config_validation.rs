use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::pipeline::planning::load_planning_manifest_file;
use crate::pipeline::postprocess_planning::load_postprocess_planning_config;
use crate::pipeline::settings_layer::{
    load_app_pipeline_settings, load_project_pipeline_settings, merge_pipeline_settings_overlays,
    PipelineSettingsOverlay,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PipelineConfigValidationRequest {
    pub app_root: PathBuf,
    pub project_root: Option<PathBuf>,
    pub app_settings_path: Option<String>,
    pub project_settings_path: Option<String>,
    pub manifest_path_override: Option<String>,
    pub postprocess_config_path_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PipelineConfigValidationSummary {
    pub app_settings_loaded: bool,
    pub project_settings_loaded: bool,
    pub resolved_manifest_path: Option<String>,
    pub resolved_postprocess_config_path: Option<String>,
}

#[derive(Debug, Error)]
pub enum PipelineConfigValidationError {
    #[error("settings layer validation failed: {0}")]
    Settings(String),
    #[error("planning manifest validation failed ({path}): {message}")]
    Manifest { path: String, message: String },
    #[error("postprocess config validation failed ({path}): {message}")]
    PostprocessConfig { path: String, message: String },
}

pub fn validate_pipeline_config_stack(
    req: &PipelineConfigValidationRequest,
) -> Result<PipelineConfigValidationSummary, PipelineConfigValidationError> {
    let app_settings =
        load_app_pipeline_settings(req.app_root.as_path(), req.app_settings_path.as_deref())
            .map_err(|e| PipelineConfigValidationError::Settings(e.to_string()))?;
    let project_settings = load_project_pipeline_settings(
        req.project_root.as_deref(),
        req.project_settings_path.as_deref(),
    )
    .map_err(|e| PipelineConfigValidationError::Settings(e.to_string()))?;
    let merged = merge_pipeline_settings_overlays(
        &app_settings,
        &project_settings,
        &PipelineSettingsOverlay {
            manifest_path: req.manifest_path_override.clone(),
            postprocess_config_path: req.postprocess_config_path_override.clone(),
            ..PipelineSettingsOverlay::default()
        },
    );

    if let Some(manifest_path) = merged.manifest_path.as_deref() {
        let abs = resolve_path(req.app_root.as_path(), manifest_path);
        load_planning_manifest_file(abs.as_path()).map_err(|e| {
            PipelineConfigValidationError::Manifest {
                path: abs.display().to_string(),
                message: e.to_string(),
            }
        })?;
    }
    if let Some(postprocess_path) = merged.postprocess_config_path.as_deref() {
        load_postprocess_planning_config(req.app_root.as_path(), Some(postprocess_path)).map_err(
            |e| PipelineConfigValidationError::PostprocessConfig {
                path: resolve_path(req.app_root.as_path(), postprocess_path)
                    .display()
                    .to_string(),
                message: e.to_string(),
            },
        )?;
    }

    Ok(PipelineConfigValidationSummary {
        app_settings_loaded: overlay_present(&app_settings),
        project_settings_loaded: overlay_present(&project_settings),
        resolved_manifest_path: merged.manifest_path,
        resolved_postprocess_config_path: merged.postprocess_config_path,
    })
}

fn overlay_present(overlay: &PipelineSettingsOverlay) -> bool {
    overlay.manifest_path.is_some()
        || overlay.postprocess_config_path.is_some()
        || overlay.post_upscale.is_some()
        || overlay.upscale_backend.is_some()
        || overlay.post_color.is_some()
        || overlay.color_profile.is_some()
        || overlay.post_bg_remove.is_some()
        || overlay.bg_remove_backends.is_some()
        || overlay.bg_refine_openai.is_some()
        || overlay.bg_refine_openai_required.is_some()
}

fn resolve_path(app_root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        app_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_cfg_validate_{stamp}"));
        fs::create_dir_all(root.as_path()).expect("temp root");
        root
    }

    #[test]
    fn validates_layered_settings_and_referenced_files() {
        let root = temp_root();
        fs::create_dir_all(root.join("config")).expect("config dir");
        fs::create_dir_all(root.join("var/projects/demo/.kroma")).expect("project settings dir");
        fs::write(
            root.join("config/pipeline.settings.toml"),
            r#"[pipeline]
manifest_path = "config/manifest.json"
postprocess_config_path = "config/post.json"
"#,
        )
        .expect("app settings");
        fs::write(
            root.join("config/manifest.json"),
            r#"{"scene_refs":["a.png"],"prompts":{"style_base":"ok"}}"#,
        )
        .expect("manifest");
        fs::write(root.join("config/post.json"), r#"{}"#).expect("postprocess");

        let summary = validate_pipeline_config_stack(&PipelineConfigValidationRequest {
            app_root: root.clone(),
            project_root: Some(root.join("var/projects/demo")),
            app_settings_path: None,
            project_settings_path: None,
            manifest_path_override: None,
            postprocess_config_path_override: None,
        })
        .expect("validation should pass");

        assert!(summary.app_settings_loaded);
        assert!(!summary.project_settings_loaded);
        assert_eq!(
            summary.resolved_manifest_path.as_deref(),
            Some("config/manifest.json")
        );
        assert_eq!(
            summary.resolved_postprocess_config_path.as_deref(),
            Some("config/post.json")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn explicit_manifest_and_postprocess_overrides_validate_without_settings_files() {
        let root = temp_root();
        fs::create_dir_all(root.join("config")).expect("config dir");
        fs::write(
            root.join("config/manifest.json"),
            r#"{"scene_refs":["a.png"],"prompts":{"style_base":"ok"}}"#,
        )
        .expect("manifest");
        fs::write(root.join("config/post.json"), r#"{}"#).expect("postprocess");

        let summary = validate_pipeline_config_stack(&PipelineConfigValidationRequest {
            app_root: root.clone(),
            project_root: None,
            app_settings_path: None,
            project_settings_path: None,
            manifest_path_override: Some(String::from("config/manifest.json")),
            postprocess_config_path_override: Some(String::from("config/post.json")),
        })
        .expect("validation should pass");

        assert!(!summary.app_settings_loaded);
        assert!(!summary.project_settings_loaded);
        assert_eq!(
            summary.resolved_manifest_path.as_deref(),
            Some("config/manifest.json")
        );
        assert_eq!(
            summary.resolved_postprocess_config_path.as_deref(),
            Some("config/post.json")
        );

        let _ = fs::remove_dir_all(root);
    }
}
