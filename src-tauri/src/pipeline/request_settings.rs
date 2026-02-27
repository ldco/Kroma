use std::path::{Path, PathBuf};

use crate::pipeline::pathing::resolve_under_root;
use crate::pipeline::runtime::{
    PipelineRunOptions, PipelineRunRequest, PipelineRuntimeError, PipelineUpscaleBackend,
};
use crate::pipeline::settings_layer::{
    load_app_pipeline_settings, load_project_pipeline_settings, merge_pipeline_settings_overlays,
    PipelineSettingsLayerPaths, PipelineSettingsOverlay,
};

pub(crate) fn default_project_root_for_request(
    app_root: &Path,
    request: &PipelineRunRequest,
) -> PathBuf {
    request
        .options
        .project_root
        .as_deref()
        .map(|v| resolve_under_app_root(app_root, v))
        .unwrap_or_else(|| {
            app_root
                .join("var/projects")
                .join(request.project_slug.as_str())
        })
}

pub(crate) fn effective_pipeline_request_with_layered_settings(
    app_root: &Path,
    request: &PipelineRunRequest,
) -> Result<PipelineRunRequest, PipelineRuntimeError> {
    let layer_paths = PipelineSettingsLayerPaths {
        app_settings_path: request.options.app_settings_path.clone(),
        project_settings_path: request.options.project_settings_path.clone(),
    };
    let app_settings =
        load_app_pipeline_settings(app_root, layer_paths.app_settings_path.as_deref())
            .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?;
    let project_root = default_project_root_for_request(app_root, request);
    let project_settings = load_project_pipeline_settings(
        Some(project_root.as_path()),
        layer_paths.project_settings_path.as_deref(),
    )
    .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?;
    let explicit = request_options_to_settings_overlay(&request.options);
    let merged = merge_pipeline_settings_overlays(&app_settings, &project_settings, &explicit);
    Ok(apply_settings_overlay_to_request(request, &merged))
}

fn resolve_under_app_root(app_root: &Path, value: &str) -> PathBuf {
    resolve_under_root(app_root, value)
}

fn request_options_to_settings_overlay(options: &PipelineRunOptions) -> PipelineSettingsOverlay {
    PipelineSettingsOverlay {
        manifest_path: options.manifest_path.clone(),
        postprocess_config_path: options.postprocess.config_path.clone(),
        post_upscale: options.postprocess.upscale.then_some(true),
        upscale_backend: options
            .postprocess
            .upscale_backend
            .map(|v| v.as_str().to_string()),
        post_color: options.postprocess.color.then_some(true),
        color_profile: options.postprocess.color_profile.clone(),
        post_bg_remove: options.postprocess.bg_remove.then_some(true),
        bg_remove_backends: (!options.postprocess.bg_remove_backends.is_empty())
            .then(|| options.postprocess.bg_remove_backends.clone()),
        bg_refine_openai: options.postprocess.bg_refine_openai,
        bg_refine_openai_required: options.postprocess.bg_refine_openai_required,
    }
}

fn apply_settings_overlay_to_request(
    request: &PipelineRunRequest,
    overlay: &PipelineSettingsOverlay,
) -> PipelineRunRequest {
    let mut out = request.clone();
    if out.options.manifest_path.is_none() {
        out.options.manifest_path = overlay.manifest_path.clone();
    }
    if out.options.postprocess.config_path.is_none() {
        out.options.postprocess.config_path = overlay.postprocess_config_path.clone();
    }
    if !out.options.postprocess.upscale {
        out.options.postprocess.upscale = overlay.post_upscale.unwrap_or(false);
    }
    if out.options.postprocess.upscale_backend.is_none() {
        out.options.postprocess.upscale_backend = overlay
            .upscale_backend
            .as_deref()
            .and_then(parse_pipeline_upscale_backend);
    }
    if !out.options.postprocess.color {
        out.options.postprocess.color = overlay.post_color.unwrap_or(false);
    }
    if out.options.postprocess.color_profile.is_none() {
        out.options.postprocess.color_profile = overlay.color_profile.clone();
    }
    if !out.options.postprocess.bg_remove {
        out.options.postprocess.bg_remove = overlay.post_bg_remove.unwrap_or(false);
    }
    if out.options.postprocess.bg_remove_backends.is_empty() {
        out.options.postprocess.bg_remove_backends =
            overlay.bg_remove_backends.clone().unwrap_or_default();
    }
    if out.options.postprocess.bg_refine_openai.is_none() {
        out.options.postprocess.bg_refine_openai = overlay.bg_refine_openai;
    }
    if out.options.postprocess.bg_refine_openai_required.is_none() {
        out.options.postprocess.bg_refine_openai_required = overlay.bg_refine_openai_required;
    }
    out
}

fn parse_pipeline_upscale_backend(value: &str) -> Option<PipelineUpscaleBackend> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ncnn" => Some(PipelineUpscaleBackend::Ncnn),
        "python" => Some(PipelineUpscaleBackend::Python),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::runtime::{PipelinePostprocessOptions, PipelineRunMode};

    #[test]
    fn apply_settings_overlay_to_request_fills_missing_postprocess_fields() {
        let request = PipelineRunRequest {
            project_slug: String::from("demo"),
            mode: PipelineRunMode::Dry,
            confirm_spend: false,
            options: PipelineRunOptions::default(),
        };
        let overlay = PipelineSettingsOverlay {
            manifest_path: Some(String::from("pipeline/manifest.json")),
            postprocess_config_path: Some(String::from("pipeline/postprocess.config.json")),
            post_upscale: Some(true),
            upscale_backend: Some(String::from("ncnn")),
            post_color: Some(true),
            color_profile: Some(String::from("cinematic")),
            post_bg_remove: Some(true),
            bg_remove_backends: Some(vec![String::from("u2net")]),
            bg_refine_openai: Some(true),
            bg_refine_openai_required: Some(true),
        };

        let out = apply_settings_overlay_to_request(&request, &overlay);
        assert_eq!(
            out.options.manifest_path.as_deref(),
            Some("pipeline/manifest.json")
        );
        assert_eq!(
            out.options.postprocess.config_path.as_deref(),
            Some("pipeline/postprocess.config.json")
        );
        assert_eq!(out.options.postprocess.upscale, true);
        assert_eq!(
            out.options.postprocess.upscale_backend,
            Some(PipelineUpscaleBackend::Ncnn)
        );
        assert_eq!(out.options.postprocess.color, true);
        assert_eq!(
            out.options.postprocess.color_profile.as_deref(),
            Some("cinematic")
        );
        assert_eq!(out.options.postprocess.bg_remove, true);
        assert_eq!(
            out.options.postprocess.bg_remove_backends,
            vec![String::from("u2net")]
        );
        assert_eq!(out.options.postprocess.bg_refine_openai, Some(true));
        assert_eq!(
            out.options.postprocess.bg_refine_openai_required,
            Some(true)
        );
    }

    #[test]
    fn apply_settings_overlay_to_request_keeps_explicit_request_values() {
        let request = PipelineRunRequest {
            project_slug: String::from("demo"),
            mode: PipelineRunMode::Dry,
            confirm_spend: false,
            options: PipelineRunOptions {
                manifest_path: Some(String::from("explicit.json")),
                postprocess: PipelinePostprocessOptions {
                    upscale: true,
                    upscale_backend: Some(PipelineUpscaleBackend::Python),
                    color: true,
                    color_profile: Some(String::from("explicit-profile")),
                    bg_remove: true,
                    bg_remove_backends: vec![String::from("bria")],
                    bg_refine_openai: Some(false),
                    bg_refine_openai_required: Some(false),
                    ..PipelinePostprocessOptions::default()
                },
                ..PipelineRunOptions::default()
            },
        };
        let overlay = PipelineSettingsOverlay {
            manifest_path: Some(String::from("overlay.json")),
            postprocess_config_path: Some(String::from("overlay-config.json")),
            post_upscale: Some(false),
            upscale_backend: Some(String::from("ncnn")),
            post_color: Some(false),
            color_profile: Some(String::from("overlay-profile")),
            post_bg_remove: Some(false),
            bg_remove_backends: Some(vec![String::from("u2net")]),
            bg_refine_openai: Some(true),
            bg_refine_openai_required: Some(true),
        };

        let out = apply_settings_overlay_to_request(&request, &overlay);
        assert_eq!(out.options.manifest_path.as_deref(), Some("explicit.json"));
        assert_eq!(out.options.postprocess.upscale, true);
        assert_eq!(
            out.options.postprocess.upscale_backend,
            Some(PipelineUpscaleBackend::Python)
        );
        assert_eq!(out.options.postprocess.color, true);
        assert_eq!(
            out.options.postprocess.color_profile.as_deref(),
            Some("explicit-profile")
        );
        assert_eq!(out.options.postprocess.bg_remove, true);
        assert_eq!(
            out.options.postprocess.bg_remove_backends,
            vec![String::from("bria")]
        );
        assert_eq!(out.options.postprocess.bg_refine_openai, Some(false));
        assert_eq!(
            out.options.postprocess.bg_refine_openai_required,
            Some(false)
        );
    }
}
