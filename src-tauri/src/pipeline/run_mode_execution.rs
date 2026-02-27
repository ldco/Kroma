use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use uuid::Uuid;

use crate::pipeline::execution::{
    build_run_log_output_guard_record, ensure_generation_mode_dirs, execution_project_dirs,
    finalize_job_from_candidates, plan_job_candidate_output_paths, summarize_output_guard_report,
    ExecutionCandidateJobOutputs, ExecutionCandidateJobResult, ExecutionCandidateRank,
    ExecutionCandidateResult, ExecutionCandidateStatus, ExecutionOutputGuardReport,
    ExecutionOutputGuardReportFile, ExecutionOutputGuardReportSummary,
};
use crate::pipeline::pathing::path_for_output;
use crate::pipeline::planning_preflight::RustPlanningPreflightSummary;
use crate::pipeline::request_settings::default_project_root_for_request;
use crate::pipeline::runlog::{
    format_summary_marker, write_pretty_json_with_newline, PipelineRunSummaryMarkerPayload,
};
use crate::pipeline::runlog_enrich::planned_output_guard_from_manifest;
use crate::pipeline::runlog_patch::normalize_script_run_log_job_finalization;
use crate::pipeline::runtime::{
    PipelineRunRequest, PipelineRunResult, PipelineRuntimeError, PipelineStageFilter,
    PipelineTimeFilter, PipelineWeatherFilter,
};
use crate::pipeline::tool_adapters::{
    ArchiveBadRequest, BackgroundRemovePassRequest, ColorPassRequest, GenerateOneImageRequest,
    PipelineToolAdapterOps, QaCheckRequest, ToolAdapterError, UpscalePassRequest,
};

pub(crate) fn execute_rust_run_mode_with_tool_adapters(
    app_root: &Path,
    tools: &dyn PipelineToolAdapterOps,
    request: &PipelineRunRequest,
    planned: &RustPlanningPreflightSummary,
) -> Result<PipelineRunResult, PipelineRuntimeError> {
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
    if candidate_count < 1 {
        return Err(PipelineRuntimeError::PlanningPreflight(String::from(
            "Invalid candidate count: expected >= 1",
        )));
    }
    if candidate_count > planned.manifest_max_candidates {
        return Err(PipelineRuntimeError::PlanningPreflight(format!(
            "Candidate count {} exceeds limit {}.",
            candidate_count, planned.manifest_max_candidates
        )));
    }
    let candidate_count_u8 = u8::try_from(candidate_count).map_err(|_| {
        PipelineRuntimeError::PlanningPreflight(format!(
            "Candidate count {} exceeds Rust execution limit 255.",
            candidate_count
        ))
    })?;

    let project_root_abs = default_project_root_for_request(app_root, request);
    let project_dirs = execution_project_dirs(project_root_abs.as_path());
    ensure_generation_mode_dirs(&project_dirs).map_err(PipelineRuntimeError::Io)?;
    let run_log_path_abs = project_dirs
        .runs
        .join(format!("run_{}.json", make_run_log_stamp()));

    let project_root_display = path_for_output(app_root, project_dirs.root.as_path());
    let run_log_display = path_for_output(app_root, run_log_path_abs.as_path());
    let timestamp = iso_like_timestamp();
    let output_guard_cfg = &planned.manifest_output_guard;
    let planned_output_guard = planned_output_guard_from_manifest(output_guard_cfg);
    let mut failed_output_guard_jobs = 0_u64;
    let mut jobs_json = Vec::<Value>::with_capacity(planned.jobs.len());

    let mut model_name = String::new();
    let mut image_size = String::new();
    let mut image_quality = String::new();

    let project_root_arg = path_for_output(app_root, project_dirs.root.as_path());
    let manifest_path_for_qa = request.options.manifest_path.clone();

    for job in planned.jobs.iter() {
        let candidate_plans = plan_job_candidate_output_paths(
            &project_dirs,
            job.id.as_str(),
            candidate_count_u8,
            &planned.postprocess_path_config,
        )
        .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;

        let input_images_file_path = write_json_temp_file(
            app_root,
            "pipeline_input_images",
            &serde_json::to_value(&job.input_images)
                .map_err(|e| PipelineRuntimeError::PlannedJobsTempFile(e.to_string()))?,
        )?;
        let input_images_file_arg = path_for_output(app_root, input_images_file_path.as_path());

        let mut candidate_records = Vec::<Value>::with_capacity(candidate_plans.len());
        let mut execution_candidates =
            Vec::<ExecutionCandidateJobResult>::with_capacity(candidate_plans.len());

        for plan in candidate_plans {
            let generated_rel = path_for_output(app_root, plan.generated.as_path());
            let generate_resp = tools
                .generate_one(&GenerateOneImageRequest {
                    project_slug: request.project_slug.clone(),
                    project_root: Some(project_root_arg.clone()),
                    prompt: job.prompt.clone(),
                    input_images_file: input_images_file_arg.clone(),
                    output_path: generated_rel.clone(),
                    model: None,
                    size: None,
                    quality: None,
                })
                .map_err(tool_adapter_error_to_runtime)?;
            if model_name.is_empty() {
                model_name = generate_resp.model.clone();
                image_size = generate_resp.size.clone();
                image_quality = generate_resp.quality.clone();
            }

            let mut current_path = PathBuf::from(generate_resp.output.clone());
            let mut candidate_json = json!({
                "candidate_index": plan.candidate_index,
                "output": generate_resp.output,
                "status": "generated",
                "rank": {
                    "hard_failures": 0,
                    "soft_warnings": 0,
                    "avg_chroma_exceed": 0.0
                }
            });
            let mut bg_remove_output = None::<PathBuf>;
            let mut upscale_output = None::<PathBuf>;
            let mut color_output = None::<PathBuf>;

            if let Some(bg_remove_path) = plan.bg_remove.as_ref() {
                let bg_resp = tools
                    .bgremove(&BackgroundRemovePassRequest {
                        project_slug: request.project_slug.clone(),
                        project_root: Some(project_root_arg.clone()),
                        input_path: path_for_output(app_root, current_path.as_path()),
                        output_path: path_for_output(app_root, bg_remove_path.as_path()),
                        postprocess_config_path: request.options.postprocess.config_path.clone(),
                        backends: planned.planned_postprocess.bg_remove_backends.clone(),
                        bg_refine_openai: Some(planned.planned_postprocess.bg_refine_openai),
                        bg_refine_openai_required: Some(
                            planned.planned_postprocess.bg_refine_openai_required,
                        ),
                    })
                    .map_err(tool_adapter_error_to_runtime)?;
                let single = bg_resp.results.first().ok_or_else(|| {
                    PipelineRuntimeError::PlanningPreflight(String::from(
                        "bgremove adapter returned no per-file result",
                    ))
                })?;
                let bg_meta = json!({
                    "input": single.input,
                    "output": single.output,
                    "backend": single.backend,
                    "backends_tried": bg_resp.backends,
                    "refine_openai": single.refine_openai,
                    "refine_error": single.refine_error
                });
                candidate_json["bg_remove"] = bg_meta.clone();
                let next = bg_meta
                    .get("output")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        PipelineRuntimeError::PlanningPreflight(String::from(
                            "bgremove adapter JSON missing output",
                        ))
                    })?;
                current_path = PathBuf::from(next);
                bg_remove_output = Some(current_path.clone());
            }

            if let Some(upscale_path) = plan.upscale.as_ref() {
                let upscale_resp = tools
                    .upscale(&UpscalePassRequest {
                        project_slug: request.project_slug.clone(),
                        project_root: Some(project_root_arg.clone()),
                        input_path: path_for_output(app_root, current_path.as_path()),
                        output_path: path_for_output(app_root, upscale_path.as_path()),
                        postprocess_config_path: request.options.postprocess.config_path.clone(),
                        upscale_backend: planned.planned_postprocess.upscale_backend.clone(),
                        upscale_scale: planned
                            .postprocess_path_config
                            .upscale
                            .as_ref()
                            .map(|cfg| cfg.scale),
                        upscale_format: planned
                            .postprocess_path_config
                            .upscale
                            .as_ref()
                            .map(|cfg| cfg.format.clone()),
                    })
                    .map_err(tool_adapter_error_to_runtime)?;
                candidate_json["upscale"] = serde_json::to_value(&upscale_resp)
                    .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;
                current_path = PathBuf::from(upscale_resp.output);
                upscale_output = Some(current_path.clone());
            }

            if let Some(color_path) = plan.color.as_ref() {
                let color_resp = tools
                    .color(&ColorPassRequest {
                        project_slug: request.project_slug.clone(),
                        project_root: Some(project_root_arg.clone()),
                        input_path: path_for_output(app_root, current_path.as_path()),
                        output_path: path_for_output(app_root, color_path.as_path()),
                        postprocess_config_path: request.options.postprocess.config_path.clone(),
                        profile: planned.planned_postprocess.color_profile.clone(),
                        color_settings_path: None,
                    })
                    .map_err(tool_adapter_error_to_runtime)?;
                candidate_json["color"] = serde_json::to_value(&color_resp)
                    .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;
                current_path = PathBuf::from(color_resp.output);
                color_output = Some(current_path.clone());
            }

            let mut status = ExecutionCandidateStatus::Done;
            let mut final_output = Some(current_path.clone());
            let mut rank = ExecutionCandidateRank {
                hard_failures: 0,
                soft_warnings: 0,
                avg_chroma_exceed: 0.0,
            };

            if planned_output_guard.enabled {
                let qa_resp = tools
                    .qa(&QaCheckRequest {
                        project_slug: request.project_slug.clone(),
                        project_root: Some(project_root_arg.clone()),
                        input_path: path_for_output(app_root, current_path.as_path()),
                        manifest_path: manifest_path_for_qa.clone(),
                        output_guard_enabled: Some(true),
                        enforce_grayscale: Some(output_guard_cfg.enforce_grayscale),
                        max_chroma_delta: Some(output_guard_cfg.max_chroma_delta),
                        fail_on_chroma_exceed: Some(output_guard_cfg.fail_on_chroma_exceed),
                    })
                    .map_err(tool_adapter_error_to_runtime)?;
                let guard_report_value = qa_resp.report.as_ref().ok_or_else(|| {
                    PipelineRuntimeError::PlanningPreflight(String::from(
                        "qa adapter response missing report payload",
                    ))
                })?;
                let guard_report = parse_execution_output_guard_report(guard_report_value);
                rank =
                    summarize_output_guard_report(&guard_report, output_guard_cfg.max_chroma_delta);
                candidate_json["rank"] = serde_json::to_value(&rank)
                    .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;

                let mut bad_archive = None::<PathBuf>;
                if rank.hard_failures > 0 {
                    let archive_resp = tools
                        .archive_bad(&ArchiveBadRequest {
                            project_slug: request.project_slug.clone(),
                            project_root: Some(project_root_arg.clone()),
                            input_path: path_for_output(app_root, current_path.as_path()),
                        })
                        .map_err(tool_adapter_error_to_runtime)?;
                    bad_archive = archive_resp
                        .moved
                        .first()
                        .map(|m| PathBuf::from(m.to.clone()));
                    status = ExecutionCandidateStatus::FailedOutputGuard;
                    final_output = None;
                }

                let guard_record = build_run_log_output_guard_record(
                    &guard_report,
                    current_path.as_path(),
                    bad_archive.as_deref(),
                    |p| path_for_output(app_root, p),
                );
                candidate_json["output_guard"] = serde_json::to_value(&guard_record)
                    .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;
            }

            candidate_json["status"] = json!(status.as_str());
            if let Some(final_output_path) = final_output.as_ref() {
                candidate_json["final_output"] =
                    json!(path_for_output(app_root, final_output_path.as_path()));
            }

            execution_candidates.push(ExecutionCandidateJobResult {
                candidate: ExecutionCandidateResult {
                    candidate_index: plan.candidate_index,
                    status,
                    rank: rank.clone(),
                },
                outputs: ExecutionCandidateJobOutputs {
                    output: Some(PathBuf::from(
                        candidate_json
                            .get("output")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                    )),
                    final_output,
                    bg_remove: bg_remove_output,
                    upscale: upscale_output,
                    color: color_output,
                },
            });
            candidate_records.push(candidate_json);
        }

        let _ = fs::remove_file(input_images_file_path.as_path());

        let finalized = finalize_job_from_candidates(execution_candidates.as_slice())
            .map_err(|e| PipelineRuntimeError::PlanningPreflight(e.to_string()))?;
        failed_output_guard_jobs += finalized.failed_output_guard_jobs_increment;

        let mut job_json = json!({
            "id": job.id,
            "mode": job.mode,
            "time": job.time,
            "weather": job.weather,
            "input_images": job.input_images,
            "prompt": job.prompt,
            "status": "running",
            "selected_candidate": Value::Null,
            "final_output": Value::Null,
            "candidates": candidate_records,
            "planned_generation": { "candidates": candidate_count },
            "planned_postprocess": planned.planned_postprocess,
            "planned_output_guard": planned_output_guard
        });
        normalize_script_run_log_job_finalization(&mut job_json)
            .map_err(PipelineRuntimeError::PlanningPreflight)?;
        jobs_json.push(job_json);
    }

    let mut run_meta = json!({
        "timestamp": timestamp,
        "project": request.project_slug,
        "mode": "run",
        "stage": stage.as_str(),
        "time": time.as_str(),
        "weather": weather.as_str(),
        "model": model_name,
        "size": image_size,
        "quality": image_quality,
        "generation": {
            "candidates": candidate_count,
            "max_candidates": planned.manifest_max_candidates
        },
        "postprocess": planned.planned_postprocess,
        "output_guard": planned_output_guard,
        "storage": {
            "project_root": project_root_display,
            "resolved_from_backend": request.options.project_root.is_some()
        },
        "jobs": jobs_json
    });

    write_pretty_json_with_newline(run_log_path_abs.as_path(), &run_meta)
        .map_err(|e| PipelineRuntimeError::Io(std::io::Error::other(e.to_string())))?;
    let marker = format_summary_marker(&PipelineRunSummaryMarkerPayload {
        run_log_path: run_log_display.clone(),
        project_slug: request.project_slug.clone(),
        project_root: project_root_display.clone(),
        jobs: planned.job_count(),
        mode: String::from("run"),
    })
    .map_err(|e| PipelineRuntimeError::Io(std::io::Error::other(e.to_string())))?;
    let stdout = [
        format!("Run log: {run_log_display}"),
        format!("Project: {}", request.project_slug),
        format!("Project root: {project_root_display}"),
        format!("Jobs: {} (run/completed)", planned.job_count()),
        marker,
    ]
    .join("\n");

    if failed_output_guard_jobs > 0 {
        return Err(PipelineRuntimeError::CommandFailed {
            program: String::from("rust-pipeline"),
            status_code: 1,
            stdout,
            stderr: format!(
                "Output guard failed for {} job(s). Bad outputs moved to {}",
                failed_output_guard_jobs,
                path_for_output(app_root, project_dirs.archive_bad.as_path())
            ),
        });
    }

    // Keep post-run wrapper path compatible by returning script-like summary lines.
    // Also ensure planned metadata normalization wrapper stays idempotent on Rust-owned logs.
    let _ = run_meta.as_object_mut();
    Ok(PipelineRunResult {
        status_code: 0,
        stdout,
        stderr: String::new(),
    })
}

fn tool_adapter_error_to_runtime(error: ToolAdapterError) -> PipelineRuntimeError {
    match error {
        ToolAdapterError::CommandRunner(source) => source,
        ToolAdapterError::CommandFailed {
            program,
            status_code,
            stdout,
            stderr,
        } => PipelineRuntimeError::CommandFailed {
            program,
            status_code,
            stdout,
            stderr,
        },
        other => PipelineRuntimeError::PlanningPreflight(other.to_string()),
    }
}

fn write_json_temp_file(
    app_root: &Path,
    prefix: &str,
    value: &Value,
) -> Result<PathBuf, PipelineRuntimeError> {
    let dir = app_root.join("var/tmp");
    fs::create_dir_all(dir.as_path()).map_err(|error| {
        PipelineRuntimeError::PlannedJobsTempFile(format!(
            "create dir '{}': {error}",
            dir.display()
        ))
    })?;
    let path = dir.join(format!("{prefix}_{}.json", Uuid::new_v4()));
    write_pretty_json_with_newline(path.as_path(), value)
        .map_err(|e| PipelineRuntimeError::PlannedJobsTempFile(e.to_string()))?;
    Ok(path)
}

fn parse_execution_output_guard_report(value: &Value) -> ExecutionOutputGuardReport {
    let summary_obj = value.get("summary").and_then(Value::as_object);
    let summary = summary_obj.map(|obj| ExecutionOutputGuardReportSummary {
        total_files: obj.get("total_files").and_then(Value::as_u64).unwrap_or(0),
        hard_failures: obj
            .get("hard_failures")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        soft_warnings: obj
            .get("soft_warnings")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    });
    let files = value
        .get("files")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|file| {
            let obj = file.as_object().cloned().unwrap_or_default();
            ExecutionOutputGuardReportFile {
                file: obj.get("file").and_then(Value::as_str).map(PathBuf::from),
                chroma_delta: obj.get("chroma_delta").and_then(Value::as_f64),
            }
        })
        .collect::<Vec<_>>();
    ExecutionOutputGuardReport { summary, files }
}

fn iso_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", now.as_secs(), now.subsec_millis())
}

fn make_run_log_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{:03}", now.as_secs(), now.subsec_millis())
}
