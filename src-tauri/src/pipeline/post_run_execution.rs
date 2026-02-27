use std::path::Path;

use crate::pipeline::planning_preflight::build_rust_planning_preflight_summary;
use crate::pipeline::post_run::{
    PipelinePostRunService, PostRunFinalizeParams, PostRunIngestParams, PostRunSyncS3Params,
};
use crate::pipeline::request_settings::effective_pipeline_request_with_layered_settings;
use crate::pipeline::runlog_enrich::{
    build_planned_template_from_request, RunLogPlannedTemplateRequestInput,
};
use crate::pipeline::runlog_parse::{append_stderr_line, parse_script_run_summary_from_stdout};
use crate::pipeline::runlog_patch::{
    normalize_script_run_log_job_finalizations_file, patch_script_run_log_planned_metadata_file,
};
use crate::pipeline::runtime::{
    PipelineRunMode, PipelineRunRequest, PipelineStageFilter, PipelineTimeFilter,
    PipelineWeatherFilter,
};

pub(crate) fn run_post_run_finalize_best_effort(
    app_root: &Path,
    post_run: &PipelinePostRunService,
    request: &PipelineRunRequest,
    stdout: &str,
    stderr: &mut String,
) {
    let Some(summary) = parse_script_run_summary_from_stdout(stdout) else {
        append_stderr_line(
            stderr,
            "Rust post-run finalize skipped: missing summary marker or 'Run log:' line in pipeline stdout",
        );
        return;
    };
    if let Some(project_slug) = summary.project_slug.as_deref() {
        if project_slug != request.project_slug {
            append_stderr_line(
                stderr,
                format!(
                    "Rust post-run ingest warning: script stdout project '{}' does not match request '{}'",
                    project_slug, request.project_slug
                ),
            );
        }
    }
    normalize_script_run_log_best_effort(app_root, summary.run_log_path.as_path(), stderr);
    enrich_script_run_log_planned_metadata_best_effort(
        app_root,
        request,
        summary.run_log_path.as_path(),
        stderr,
    );

    let finalize = post_run.finalize_run(PostRunFinalizeParams {
        ingest: PostRunIngestParams {
            run_log_path: summary.run_log_path,
            project_slug: request.project_slug.clone(),
            project_name: request.project_slug.clone(),
            create_project_if_missing: true,
            compute_hashes: false,
        },
        sync_s3: build_post_run_sync_s3_params(request),
    });

    if let Err(error) = finalize {
        append_stderr_line(stderr, format!("Rust post-run finalize skipped: {error}"));
    }
}

fn build_post_run_sync_s3_params(request: &PipelineRunRequest) -> Option<PostRunSyncS3Params> {
    if !matches!(request.mode, PipelineRunMode::Run) {
        return None;
    }
    if !request.options.storage_sync_s3.unwrap_or(false) {
        return None;
    }
    Some(PostRunSyncS3Params {
        project_slug: request.project_slug.clone(),
        dry_run: false,
        delete: false,
        allow_missing_local: false,
    })
}

fn normalize_script_run_log_best_effort(app_root: &Path, run_log_path: &Path, stderr: &mut String) {
    if let Err(error) = normalize_script_run_log_job_finalizations_file(app_root, run_log_path) {
        append_stderr_line(
            stderr,
            format!("Rust run-log normalization skipped: {error}"),
        );
    }
}

fn enrich_script_run_log_planned_metadata_best_effort(
    app_root: &Path,
    request: &PipelineRunRequest,
    run_log_path: &Path,
    stderr: &mut String,
) {
    if let Err(error) = enrich_script_run_log_planned_metadata_file(app_root, request, run_log_path)
    {
        append_stderr_line(
            stderr,
            format!("Rust planned-metadata run-log patch skipped: {error}"),
        );
    }
}

fn enrich_script_run_log_planned_metadata_file(
    app_root: &Path,
    request: &PipelineRunRequest,
    run_log_path: &Path,
) -> Result<(), String> {
    let effective = effective_pipeline_request_with_layered_settings(app_root, request)
        .map_err(|e| format!("resolve layered settings: {e}"))?;
    let Some(planned) = build_rust_planning_preflight_summary(app_root, &effective)
        .map_err(|e| format!("build planning preflight summary: {e}"))?
    else {
        return Ok(());
    };
    if planned.jobs.is_empty() {
        return Ok(());
    }

    let stage = effective
        .options
        .stage
        .unwrap_or(PipelineStageFilter::Style);
    let time = effective.options.time.unwrap_or(PipelineTimeFilter::Day);
    let weather = effective
        .options
        .weather
        .unwrap_or(PipelineWeatherFilter::Clear);
    let planned_template = build_planned_template_from_request(
        app_root,
        RunLogPlannedTemplateRequestInput {
            project_slug: effective.project_slug.clone(),
            project_root_override: effective.options.project_root.clone(),
            stage: stage.as_str().to_string(),
            time: time.as_str().to_string(),
            weather: weather.as_str().to_string(),
            requested_candidate_count: effective.options.candidates.map(u64::from),
            manifest_candidate_count: planned.manifest_candidate_count,
            manifest_max_candidates: planned.manifest_max_candidates,
            planned_postprocess: planned.planned_postprocess.clone(),
            manifest_output_guard: planned.manifest_output_guard.clone(),
            jobs: planned.jobs.clone(),
        },
    );

    patch_script_run_log_planned_metadata_file(app_root, run_log_path, &planned_template)
}
