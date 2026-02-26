use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::pipeline::execution::{
    finalize_job_from_candidates, ExecutionCandidateJobOutputs, ExecutionCandidateJobResult,
    ExecutionCandidateRank, ExecutionCandidateStatus, ExecutionPlannedRunLogRecord,
};
use crate::pipeline::runlog::write_pretty_json_with_newline;

pub fn normalize_script_run_log_job_finalizations_file(
    app_root: &Path,
    run_log_path: &Path,
) -> Result<(), String> {
    let abs = resolve_run_log_abs(app_root, run_log_path);
    if !abs.is_file() {
        return Ok(());
    }

    let raw = fs::read_to_string(abs.as_path())
        .map_err(|e| format!("read run log '{}': {e}", abs.display()))?;
    let mut value: Value = serde_json::from_str(raw.as_str())
        .map_err(|e| format!("parse run log '{}': {e}", abs.display()))?;
    let changed = normalize_script_run_log_job_finalizations_value(&mut value)?;
    if changed {
        write_pretty_json_with_newline(abs.as_path(), &value)
            .map_err(|e| format!("write normalized run log '{}': {e}", abs.display()))?;
    }
    Ok(())
}

pub fn patch_script_run_log_planned_metadata_file(
    app_root: &Path,
    run_log_path: &Path,
    planned: &ExecutionPlannedRunLogRecord,
) -> Result<(), String> {
    let abs = resolve_run_log_abs(app_root, run_log_path);
    if !abs.is_file() {
        return Ok(());
    }
    let raw = fs::read_to_string(abs.as_path())
        .map_err(|e| format!("read run log '{}': {e}", abs.display()))?;
    let mut value: Value = serde_json::from_str(raw.as_str())
        .map_err(|e| format!("parse run log '{}': {e}", abs.display()))?;
    let changed = patch_run_log_planned_metadata_value(&mut value, planned)?;
    if changed {
        write_pretty_json_with_newline(abs.as_path(), &value)
            .map_err(|e| format!("write patched run log '{}': {e}", abs.display()))?;
    }
    Ok(())
}

pub fn normalize_script_run_log_job_finalization(job: &mut Value) -> Result<bool, String> {
    let Some(job_obj) = job.as_object_mut() else {
        return Ok(false);
    };
    let Some(candidates) = job_obj.get("candidates").and_then(Value::as_array).cloned() else {
        return Ok(false);
    };
    if candidates.is_empty() {
        return Ok(false);
    }

    let parsed_candidates = candidates
        .iter()
        .map(parse_execution_candidate_from_run_log)
        .collect::<Result<Vec<_>, _>>()?;
    let finalized = finalize_job_from_candidates(parsed_candidates.as_slice())
        .map_err(|e| format!("finalize job from candidates: {e}"))?;

    let winner_candidate_json = finalized.selected_candidate.and_then(|selected| {
        candidates.iter().find(|candidate| {
            candidate
                .get("candidate_index")
                .and_then(|v| v.as_u64())
                .and_then(|v| u8::try_from(v).ok())
                == Some(selected)
        })
    });

    let mut changed = false;
    changed |= upsert_json_field(job_obj, "status", json!(finalized.status.as_str()));

    if let Some(selected) = finalized.selected_candidate {
        changed |= upsert_json_field(job_obj, "selected_candidate", json!(selected));
    } else {
        changed |= upsert_json_field(job_obj, "selected_candidate", Value::Null);
    }

    if let Some(final_output) = finalized
        .final_output
        .as_deref()
        .map(path_to_run_log_string)
    {
        changed |= upsert_json_field(job_obj, "final_output", json!(final_output));
        changed |= upsert_json_field(job_obj, "output", json!(final_output));
    } else {
        changed |= upsert_json_field(job_obj, "final_output", Value::Null);
        changed |= remove_json_field(job_obj, "output");
    }

    if let Some(reason) = finalized.failure_reason {
        changed |= upsert_json_field(job_obj, "failure_reason", json!(reason));
    } else {
        changed |= remove_json_field(job_obj, "failure_reason");
    }

    changed |= sync_winner_passthrough_fields(job_obj, winner_candidate_json);
    Ok(changed)
}

fn resolve_run_log_abs(app_root: &Path, run_log_path: &Path) -> PathBuf {
    if run_log_path.is_absolute() {
        run_log_path.to_path_buf()
    } else {
        app_root.join(run_log_path)
    }
}

fn normalize_script_run_log_job_finalizations_value(run_log: &mut Value) -> Result<bool, String> {
    let Some(jobs) = run_log.get_mut("jobs").and_then(Value::as_array_mut) else {
        return Ok(false);
    };
    let mut changed = false;
    for job in jobs {
        changed |= normalize_script_run_log_job_finalization(job)?;
    }
    Ok(changed)
}

fn patch_run_log_planned_metadata_value(
    run_log: &mut Value,
    planned: &ExecutionPlannedRunLogRecord,
) -> Result<bool, String> {
    let Some(run_obj) = run_log.as_object_mut() else {
        return Err(String::from("run log root must be an object"));
    };
    let planned_json =
        serde_json::to_value(planned).map_err(|e| format!("serialize planned metadata: {e}"))?;
    let planned_obj = planned_json
        .as_object()
        .ok_or_else(|| String::from("planned metadata root must be object"))?;

    let mut changed = false;
    for key in ["generation", "postprocess", "output_guard", "storage"] {
        if let Some(value) = planned_obj.get(key).cloned() {
            changed |= upsert_json_field(run_obj, key, value);
        }
    }

    let mut planned_jobs_by_id = Vec::<(String, Value)>::new();
    if let Some(planned_jobs) = planned_obj.get("jobs").and_then(Value::as_array) {
        for job in planned_jobs {
            let Some(id) = job.get("id").and_then(Value::as_str) else {
                continue;
            };
            planned_jobs_by_id.push((id.to_string(), job.clone()));
        }
    }

    let Some(run_jobs) = run_obj.get_mut("jobs").and_then(Value::as_array_mut) else {
        return Ok(changed);
    };
    for job in run_jobs {
        let Some(job_obj) = job.as_object_mut() else {
            continue;
        };
        let Some(id) = job_obj.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some((_, planned_job)) = planned_jobs_by_id
            .iter()
            .find(|(planned_id, _)| planned_id == id)
        else {
            continue;
        };
        for key in [
            "planned_generation",
            "planned_postprocess",
            "planned_output_guard",
        ] {
            if let Some(value) = planned_job.get(key).cloned() {
                changed |= upsert_json_field(job_obj, key, value);
            }
        }
    }

    Ok(changed)
}

fn parse_execution_candidate_from_run_log(
    candidate: &Value,
) -> Result<ExecutionCandidateJobResult, String> {
    let obj = candidate
        .as_object()
        .ok_or_else(|| String::from("candidate must be an object"))?;
    let candidate_index = obj
        .get("candidate_index")
        .and_then(Value::as_u64)
        .and_then(|v| u8::try_from(v).ok())
        .ok_or_else(|| String::from("candidate_index missing or invalid"))?;
    let status = match obj
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("generated")
        .trim()
    {
        "done" => ExecutionCandidateStatus::Done,
        "failed_output_guard" => ExecutionCandidateStatus::FailedOutputGuard,
        _ => ExecutionCandidateStatus::Generated,
    };

    let rank_obj = obj.get("rank").and_then(Value::as_object);
    let rank = ExecutionCandidateRank {
        hard_failures: rank_obj
            .and_then(|m| m.get("hard_failures"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        soft_warnings: rank_obj
            .and_then(|m| m.get("soft_warnings"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        avg_chroma_exceed: rank_obj
            .and_then(|m| m.get("avg_chroma_exceed"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
    };

    Ok(ExecutionCandidateJobResult {
        candidate: crate::pipeline::execution::ExecutionCandidateResult {
            candidate_index,
            status,
            rank,
        },
        outputs: ExecutionCandidateJobOutputs {
            output: parse_candidate_output_path(obj.get("output")),
            final_output: parse_candidate_output_path(obj.get("final_output")),
            bg_remove: parse_candidate_output_path(obj.get("bg_remove")),
            upscale: parse_candidate_output_path(obj.get("upscale")),
            color: parse_candidate_output_path(obj.get("color")),
        },
    })
}

fn parse_candidate_output_path(value: Option<&Value>) -> Option<PathBuf> {
    match value {
        Some(Value::String(path)) => Some(PathBuf::from(path)),
        Some(Value::Object(obj)) => obj.get("output").and_then(Value::as_str).map(PathBuf::from),
        _ => None,
    }
}

fn path_to_run_log_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn upsert_json_field(
    obj: &mut serde_json::Map<String, Value>,
    key: &str,
    new_value: Value,
) -> bool {
    if obj.get(key) == Some(&new_value) {
        return false;
    }
    obj.insert(String::from(key), new_value);
    true
}

fn remove_json_field(obj: &mut serde_json::Map<String, Value>, key: &str) -> bool {
    obj.remove(key).is_some()
}

fn sync_winner_passthrough_fields(
    job_obj: &mut serde_json::Map<String, Value>,
    winner_candidate_json: Option<&Value>,
) -> bool {
    const KEYS: [&str; 4] = ["bg_remove", "upscale", "color", "output_guard"];
    let mut changed = false;
    let winner_obj = winner_candidate_json.and_then(Value::as_object);
    for key in KEYS {
        if let Some(value) = winner_obj.and_then(|obj| obj.get(key)).cloned() {
            changed |= upsert_json_field(job_obj, key, value);
        } else {
            changed |= remove_json_field(job_obj, key);
        }
    }
    changed
}
