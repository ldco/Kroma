use std::path::Path;

use crate::pipeline::execution::{
    build_planned_run_log_record, ensure_generation_mode_dirs, execution_project_dirs,
    ExecutionPlannedJob, ExecutionPlannedRunLogContext,
};
use crate::pipeline::pathing::path_for_output;
use crate::pipeline::planning_preflight::RustPlanningPreflightSummary;
use crate::pipeline::request_settings::default_project_root_for_request;
use crate::pipeline::runlog::{
    format_summary_marker, iso_like_timestamp_now, run_log_stamp_now,
    write_pretty_json_with_newline, PipelineRunSummaryMarkerPayload,
};
use crate::pipeline::runlog_enrich::planned_output_guard_from_manifest;
use crate::pipeline::runtime::{
    PipelineRunRequest, PipelineRunResult, PipelineRuntimeError, PipelineStageFilter,
    PipelineTimeFilter, PipelineWeatherFilter,
};

pub(crate) fn execute_rust_dry_run_with_preflight(
    app_root: &Path,
    request: &PipelineRunRequest,
    planned: &RustPlanningPreflightSummary,
) -> Result<PipelineRunResult, PipelineRuntimeError> {
    let project_root_abs = default_project_root_for_request(app_root, request);
    let project_dirs = execution_project_dirs(project_root_abs.as_path());
    ensure_generation_mode_dirs(&project_dirs).map_err(PipelineRuntimeError::Io)?;
    let run_log_path_abs = project_dirs
        .runs
        .join(format!("run_{}.json", run_log_stamp_now()));

    let stage = request.options.stage.unwrap_or(PipelineStageFilter::Style);
    let time = request.options.time.unwrap_or(PipelineTimeFilter::Day);
    let weather = request
        .options
        .weather
        .unwrap_or(PipelineWeatherFilter::Clear);
    let candidate_count = request
        .options
        .candidates
        .map(u64::from)
        .unwrap_or(planned.manifest_candidate_count);
    let project_root_display = path_for_output(app_root, project_dirs.root.as_path());
    let run_log_display = path_for_output(app_root, run_log_path_abs.as_path());
    let timestamp = iso_like_timestamp_now();

    let execution_jobs = planned
        .jobs
        .iter()
        .cloned()
        .map(ExecutionPlannedJob::from)
        .collect::<Vec<_>>();
    let run_meta = build_planned_run_log_record(
        ExecutionPlannedRunLogContext {
            timestamp,
            project_slug: request.project_slug.clone(),
            stage: stage.as_str().to_string(),
            time: time.as_str().to_string(),
            weather: weather.as_str().to_string(),
            project_root: project_root_display.clone(),
            resolved_from_backend: request.options.project_root.is_some(),
            candidate_count,
            max_candidate_count: planned.manifest_max_candidates,
            planned_postprocess: planned.planned_postprocess.clone(),
            planned_output_guard: planned_output_guard_from_manifest(
                &planned.manifest_output_guard,
            ),
        },
        execution_jobs.as_slice(),
    );

    write_pretty_json_with_newline(run_log_path_abs.as_path(), &run_meta)
        .map_err(|e| PipelineRuntimeError::Io(std::io::Error::other(e.to_string())))?;
    let marker = format_summary_marker(&PipelineRunSummaryMarkerPayload {
        run_log_path: run_log_display.clone(),
        project_slug: request.project_slug.clone(),
        project_root: project_root_display.clone(),
        jobs: planned.job_count(),
        mode: String::from("dry"),
    })
    .map_err(|e| PipelineRuntimeError::Io(std::io::Error::other(e.to_string())))?;

    let stdout = [
        format!("Run log: {run_log_display}"),
        format!("Project: {}", request.project_slug),
        format!("Project root: {project_root_display}"),
        format!("Jobs: {} (dry/planned)", planned.job_count()),
        marker,
    ]
    .join("\n");

    Ok(PipelineRunResult {
        status_code: 0,
        stdout,
        stderr: String::new(),
    })
}
