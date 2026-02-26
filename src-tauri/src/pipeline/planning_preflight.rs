use std::path::{Path, PathBuf};

use crate::pipeline::execution::{
    ExecutionPlannedPostprocessRecord, ExecutionPostprocessPathConfig, ExecutionUpscalePathConfig,
};
use crate::pipeline::pathing::{list_image_files_recursive, resolve_under_root};
use crate::pipeline::planning::{
    build_generation_jobs, default_planning_manifest, load_planned_jobs_file,
    load_planning_manifest_file, PipelinePlanningOutputGuard, PlannedGenerationJob,
};
use crate::pipeline::postprocess_planning::{
    load_postprocess_planning_config, resolve_planned_postprocess_record,
    PostprocessPlanningConfig, PostprocessPlanningOverrides,
};
use crate::pipeline::runtime::{
    PipelineInputSource, PipelineRunRequest, PipelineRuntimeError, PipelineStageFilter,
    PipelineTimeFilter, PipelineWeatherFilter,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RustPlanningPreflightSummary {
    pub(crate) job_ids: Vec<String>,
    pub(crate) manifest_output_guard: PipelinePlanningOutputGuard,
    pub(crate) planned_postprocess: ExecutionPlannedPostprocessRecord,
    pub(crate) postprocess_path_config: ExecutionPostprocessPathConfig,
    pub(crate) manifest_candidate_count: u64,
    pub(crate) manifest_max_candidates: u64,
    pub(crate) jobs: Vec<PlannedGenerationJob>,
}

impl RustPlanningPreflightSummary {
    pub(crate) fn job_count(&self) -> u64 {
        self.job_ids.len() as u64
    }
}

pub(crate) fn build_rust_planning_preflight_summary(
    app_root: &Path,
    request: &PipelineRunRequest,
) -> Result<Option<RustPlanningPreflightSummary>, PipelineRuntimeError> {
    let has_manifest = request.options.manifest_path.is_some();
    let has_jobs_file = request.options.jobs_file.is_some();
    let has_scene_refs = matches!(
        request.options.input_source,
        Some(PipelineInputSource::SceneRefs(_))
    );
    let has_input_path = matches!(
        request.options.input_source,
        Some(PipelineInputSource::InputPath(_))
    );
    if !has_manifest && !has_jobs_file && !has_scene_refs && !has_input_path {
        return Ok(None);
    }

    let mut manifest = if let Some(manifest_path_raw) = request.options.manifest_path.as_deref() {
        let manifest_path = resolve_under_app_root(app_root, manifest_path_raw);
        load_planning_manifest_file(manifest_path.as_path()).map_err(|error| {
            PipelineRuntimeError::PlanningPreflight(format!(
                "manifest parse failed ({}): {error}",
                manifest_path.display()
            ))
        })?
    } else {
        default_planning_manifest()
    };

    match request.options.input_source.as_ref() {
        Some(PipelineInputSource::SceneRefs(values)) => {
            manifest.scene_refs = values.clone();
        }
        Some(PipelineInputSource::InputPath(path)) => {
            let input_abs = resolve_under_app_root(app_root, path);
            if !input_abs.exists() {
                return Err(PipelineRuntimeError::PlanningPreflight(format!(
                    "input not found: {}",
                    path
                )));
            }
            manifest.scene_refs = list_image_files_recursive(input_abs.as_path())
                .map_err(PipelineRuntimeError::Io)?
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
        }
        None => {}
    }

    if !has_jobs_file && !request.options.style_refs.is_empty() {
        manifest.style_refs = request.options.style_refs.clone();
    }

    let stage = request.options.stage.unwrap_or(PipelineStageFilter::Style);
    let time = request.options.time.unwrap_or(PipelineTimeFilter::Day);
    let weather = request
        .options
        .weather
        .unwrap_or(PipelineWeatherFilter::Clear);
    let jobs = if let Some(jobs_file_raw) = request.options.jobs_file.as_deref() {
        let jobs_path = resolve_under_app_root(app_root, jobs_file_raw);
        load_planned_jobs_file(jobs_path.as_path())
            .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?
    } else {
        build_generation_jobs(&manifest, stage, time, weather).map_err(|error| {
            PipelineRuntimeError::PlanningPreflight(format!(
                "manifest planning preflight failed: {error}"
            ))
        })?
    };
    if (jobs.len() as u64) > manifest.safe_batch_limit {
        return Err(PipelineRuntimeError::PlanningPreflight(format!(
            "Batch exceeds safety limit ({}). Use --allow-large-batch to override.",
            manifest.safe_batch_limit
        )));
    }

    let postprocess_cfg = load_postprocess_planning_config(
        app_root,
        request.options.postprocess.config_path.as_deref(),
    )
    .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?;
    let planned_postprocess = resolve_planned_postprocess_record(
        &postprocess_cfg,
        &PostprocessPlanningOverrides {
            post_upscale: request.options.postprocess.upscale,
            post_color: request.options.postprocess.color,
            post_bg_remove: request.options.postprocess.bg_remove,
            upscale_backend: request
                .options
                .postprocess
                .upscale_backend
                .map(|v| v.as_str().to_string()),
            color_profile: request.options.postprocess.color_profile.clone(),
            bg_remove_backends: request.options.postprocess.bg_remove_backends.clone(),
            bg_refine_openai: request.options.postprocess.bg_refine_openai,
            bg_refine_openai_required: request.options.postprocess.bg_refine_openai_required,
        },
    )
    .map_err(|error| PipelineRuntimeError::PlanningPreflight(error.to_string()))?;

    Ok(Some(RustPlanningPreflightSummary {
        job_ids: jobs.iter().map(|job| job.id.clone()).collect(),
        manifest_output_guard: manifest.output_guard.clone(),
        postprocess_path_config: execution_postprocess_path_config_from_planning(
            &postprocess_cfg,
            &planned_postprocess,
        ),
        planned_postprocess,
        manifest_candidate_count: manifest.generation.candidates,
        manifest_max_candidates: manifest.generation.max_candidates,
        jobs,
    }))
}

fn execution_postprocess_path_config_from_planning(
    cfg: &PostprocessPlanningConfig,
    planned: &ExecutionPlannedPostprocessRecord,
) -> ExecutionPostprocessPathConfig {
    ExecutionPostprocessPathConfig {
        bg_remove_format: planned.bg_remove.then(|| cfg.bg_remove_format.clone()),
        upscale: planned.upscale.then(|| ExecutionUpscalePathConfig {
            scale: cfg.upscale_scale,
            format: cfg.upscale_format.clone(),
        }),
        color_profile: planned.color.then(|| {
            planned
                .color_profile
                .clone()
                .unwrap_or_else(|| cfg.color_default_profile.clone())
        }),
    }
}

fn resolve_under_app_root(app_root: &Path, value: &str) -> PathBuf {
    resolve_under_root(app_root, value)
}
