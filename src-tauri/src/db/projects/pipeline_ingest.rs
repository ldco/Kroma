use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    ensure_project, ensure_project_storage_defaults, ensure_user, fetch_project_by_slug,
    normalize_slug, now_iso, ProjectRow, ProjectsRepoError, ProjectsStore,
};

#[derive(Debug, Clone)]
pub struct IngestRunLogInput {
    pub run_log_path: PathBuf,
    pub project_slug: String,
    pub project_name: String,
    pub create_project_if_missing: bool,
    pub compute_hashes: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IngestRunLogResult {
    pub project_slug: String,
    pub run_id: String,
    pub run_log_path: String,
    pub jobs: u64,
    pub candidates: u64,
    pub assets_upserted: u64,
    pub quality_reports_written: u64,
    pub cost_events_written: u64,
    pub status: String,
}

impl ProjectsStore {
    pub fn ingest_run_log(
        &self,
        input: IngestRunLogInput,
    ) -> Result<IngestRunLogResult, ProjectsRepoError> {
        let repo_root = self.repo_root.clone();
        self.with_connection_mut(move |conn| {
            let safe_slug = normalize_slug(input.project_slug.as_str()).ok_or_else(|| {
                ProjectsRepoError::Validation(String::from("Invalid project slug"))
            })?;
            let resolved_run_log_path = if input.run_log_path.is_absolute() {
                input.run_log_path.clone()
            } else {
                repo_root.join(input.run_log_path.as_path())
            };
            if !resolved_run_log_path.is_file() {
                return Err(ProjectsRepoError::Validation(format!(
                    "Run log not found: {}",
                    resolved_run_log_path.display()
                )));
            }

            let run_data = read_run_log_json(resolved_run_log_path.as_path())?;

            let tx = conn.transaction()?;

            let mut project = fetch_project_by_slug(&tx, safe_slug.as_str())?;
            if project.is_none() && input.create_project_if_missing {
                let user_id = ensure_user(&tx, "local", "Local User")?;
                let name = if input.project_name.trim().is_empty() {
                    safe_slug.clone()
                } else {
                    input.project_name.trim().to_string()
                };
                let created =
                    ensure_project(&tx, user_id.as_str(), safe_slug.as_str(), name.as_str(), "")?;
                ensure_project_storage_defaults(&tx, &created)?;
                project = Some(created);
            }
            let project = project.ok_or(ProjectsRepoError::NotFound)?;

            let result = ingest_run_tx(
                &tx,
                repo_root.as_path(),
                &project,
                resolved_run_log_path.as_path(),
                &run_data,
                input.compute_hashes,
            )?;

            tx.commit()?;
            Ok(result)
        })
    }
}

fn read_run_log_json(path: &Path) -> Result<Value, ProjectsRepoError> {
    let raw = fs::read_to_string(path).map_err(|error| {
        ProjectsRepoError::Validation(format!(
            "Failed to read run log '{}': {error}",
            path.display()
        ))
    })?;
    serde_json::from_str(raw.as_str()).map_err(|error| {
        ProjectsRepoError::Validation(format!(
            "Run log JSON parse failed for '{}': {error}",
            path.display()
        ))
    })
}

fn ingest_run_tx(
    conn: &Connection,
    repo_root: &Path,
    project: &ProjectRow,
    run_log_path: &Path,
    run_data: &Value,
    compute_hashes: bool,
) -> Result<IngestRunLogResult, ProjectsRepoError> {
    let rel_run_log_path = path_for_storage(run_log_path, repo_root);
    let run_status = derive_run_status(run_data);
    let ts = now_iso();
    let run_mode = get_str(run_data, "mode");
    let model_name = get_str(run_data, "model");

    if let Some(existing_run_id) = conn
        .query_row(
            "SELECT id FROM runs WHERE project_id = ?1 AND run_log_path = ?2 LIMIT 1",
            params![project.id, rel_run_log_path],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        conn.execute(
            "DELETE FROM quality_reports WHERE run_id = ?1",
            [&existing_run_id],
        )?;
        conn.execute(
            "DELETE FROM cost_events WHERE run_id = ?1",
            [&existing_run_id],
        )?;
        conn.execute(
            "DELETE FROM run_candidates WHERE job_id IN (SELECT id FROM run_jobs WHERE run_id = ?1)",
            [&existing_run_id],
        )?;
        conn.execute(
            "DELETE FROM run_job_candidates WHERE job_id IN (SELECT id FROM run_jobs WHERE run_id = ?1)",
            [&existing_run_id],
        )?;
        conn.execute("DELETE FROM run_jobs WHERE run_id = ?1", [&existing_run_id])?;
        conn.execute("DELETE FROM runs WHERE id = ?1", [&existing_run_id])?;
    }

    let run_id = Uuid::new_v4().to_string();
    let run_meta = json!({
        "timestamp": run_data.get("timestamp").cloned().unwrap_or(Value::Null),
        "generation": run_data.get("generation").cloned().unwrap_or(Value::Null),
        "postprocess": run_data.get("postprocess").cloned().unwrap_or(Value::Null),
        "output_guard": run_data.get("output_guard").cloned().unwrap_or(Value::Null),
    });

    conn.execute(
        "
        INSERT INTO runs
          (id, project_id, run_log_path, mode, run_mode, stage, time_of_day, weather, model, model_name,
           image_size, image_quality, status, meta_json, settings_snapshot_json, created_at)
        VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6, ?7, ?8, ?8, ?9, ?10, ?11, ?12, ?12, ?13)
        ",
        params![
            run_id,
            project.id,
            rel_run_log_path,
            run_mode,
            get_str(run_data, "stage"),
            get_str(run_data, "time"),
            get_str(run_data, "weather"),
            model_name,
            get_str(run_data, "size"),
            get_str(run_data, "quality"),
            run_status,
            serde_json::to_string(&run_meta).unwrap_or_else(|_| String::from("{}")),
            ts,
        ],
    )?;

    let mut inserted_jobs: u64 = 0;
    let mut inserted_candidates: u64 = 0;
    let mut inserted_assets: u64 = 0;
    let mut quality_reports_written: u64 = 0;
    let mut cost_events_written: u64 = 0;

    let jobs = run_data
        .get("jobs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for (idx, job) in jobs.iter().enumerate() {
        let Some(job_obj) = job.as_object() else {
            continue;
        };
        let job_key =
            non_empty_string(job_obj.get("id")).unwrap_or_else(|| format!("job_{}", idx + 1));
        let job_id = Uuid::new_v4().to_string();
        let selected_candidate = job_obj.get("selected_candidate").and_then(as_i64_checked);
        let final_output_rel = job_obj
            .get("final_output")
            .and_then(Value::as_str)
            .and_then(normalize_rel_path_opt);
        let prompt_text = first_non_empty_str(&[job_obj.get("prompt"), job_obj.get("prompt_text")]);

        conn.execute(
            "
            INSERT INTO run_jobs
              (id, run_id, job_key, status, selected_candidate, selected_candidate_index, final_output, final_asset_id,
               prompt_text, meta_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, NULL, ?7, ?8, ?9)
            ",
            params![
                job_id,
                run_id,
                job_key,
                string_or_empty(job_obj.get("status")),
                selected_candidate,
                final_output_rel,
                prompt_text,
                serde_json::to_string(job).unwrap_or_else(|_| String::from("{}")),
                ts,
            ],
        )?;
        inserted_jobs += 1;

        let mut candidates = job_obj
            .get("candidates")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if candidates.is_empty() {
            candidates.push(json!({
                "candidate_index": 1,
                "status": string_or_empty(job_obj.get("status")),
                "output": job_obj.get("output").cloned().unwrap_or(Value::Null),
                "final_output": job_obj.get("final_output").cloned().unwrap_or(Value::Null),
                "rank": {
                    "hard_failures": 0,
                    "soft_warnings": 0,
                    "avg_chroma_exceed": 0.0
                }
            }));
        }

        for (candidate_pos, candidate) in candidates.into_iter().enumerate() {
            let Some(candidate_obj) = candidate.as_object() else {
                continue;
            };

            let next_job_candidate_index = (candidate_pos + 1) as i64;
            let candidate_id = Uuid::new_v4().to_string();
            let rank_obj = candidate_obj
                .get("rank")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            let candidate_index = candidate_obj
                .get("candidate_index")
                .and_then(as_i64_checked)
                .unwrap_or(next_job_candidate_index);
            let output_path = candidate_obj
                .get("output")
                .and_then(Value::as_str)
                .and_then(normalize_rel_path_opt);
            let final_output_path = candidate_obj
                .get("final_output")
                .and_then(Value::as_str)
                .and_then(normalize_rel_path_opt);

            conn.execute(
                "
                INSERT INTO run_job_candidates
                  (id, job_id, candidate_index, status, output_path, final_output_path,
                   rank_hard_failures, rank_soft_warnings, rank_avg_chroma_exceed, meta_json, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ",
                params![
                    candidate_id,
                    job_id,
                    candidate_index,
                    string_or_empty(candidate_obj.get("status")),
                    output_path,
                    final_output_path,
                    number_to_i64(rank_obj.get("hard_failures")),
                    number_to_i64(rank_obj.get("soft_warnings")),
                    number_to_f64(rank_obj.get("avg_chroma_exceed")),
                    serde_json::to_string(&candidate).unwrap_or_else(|_| String::from("{}")),
                    ts,
                ],
            )?;
            inserted_candidates += 1;

            let mut output_asset_id = None::<String>;
            let mut final_asset_id = None::<String>;

            if let Some(output_path) = output_path.as_deref() {
                output_asset_id = upsert_asset_tx(
                    conn,
                    project.id.as_str(),
                    Some(run_id.as_str()),
                    Some(job_id.as_str()),
                    Some(candidate_id.as_str()),
                    "candidate_output",
                    output_path,
                    repo_root,
                    compute_hashes,
                    None,
                )?;
                if output_asset_id.is_some() {
                    inserted_assets += 1;
                }
            }

            if let Some(final_output_path) = final_output_path.as_deref() {
                if Some(final_output_path) == output_path.as_deref() {
                    final_asset_id = output_asset_id.clone();
                } else {
                    final_asset_id = upsert_asset_tx(
                        conn,
                        project.id.as_str(),
                        Some(run_id.as_str()),
                        Some(job_id.as_str()),
                        Some(candidate_id.as_str()),
                        "candidate_final_output",
                        final_output_path,
                        repo_root,
                        compute_hashes,
                        None,
                    )?;
                    if final_asset_id.is_some() {
                        inserted_assets += 1;
                    }
                }
            }

            conn.execute(
                "
                INSERT INTO run_candidates
                  (id, job_id, candidate_index, status, output_asset_id, final_asset_id,
                   output_path, final_output_path, rank_hard_failures, rank_soft_warnings, rank_avg_chroma_exceed, meta_json, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ",
                params![
                    candidate_id,
                    job_id,
                    candidate_index,
                    string_or_empty(candidate_obj.get("status")),
                    output_asset_id,
                    final_asset_id,
                    output_path,
                    final_output_path,
                    number_to_i64(rank_obj.get("hard_failures")),
                    number_to_i64(rank_obj.get("soft_warnings")),
                    number_to_f64(rank_obj.get("avg_chroma_exceed")),
                    serde_json::to_string(&candidate).unwrap_or_else(|_| String::from("{}")),
                    ts,
                ],
            )?;

            let mut report_summary = json!({
                "status": string_or_empty(candidate_obj.get("status")),
                "rank": {
                    "hard_failures": number_to_i64(rank_obj.get("hard_failures")),
                    "soft_warnings": number_to_i64(rank_obj.get("soft_warnings")),
                    "avg_chroma_exceed": number_to_f64(rank_obj.get("avg_chroma_exceed")),
                },
                "output_path": output_path,
                "final_output_path": final_output_path,
            });
            if let Some(obj) = candidate_obj.get("output_guard").and_then(Value::as_object) {
                report_summary["output_guard"] = Value::Object(obj.clone());
            }
            if let Some(obj) = candidate_obj.get("qa").and_then(Value::as_object) {
                report_summary["qa"] = Value::Object(obj.clone());
            }

            insert_quality_report_tx(
                conn,
                project.id.as_str(),
                Some(run_id.as_str()),
                final_asset_id.as_deref().or(output_asset_id.as_deref()),
                "output_guard",
                &report_summary,
                Some(ts.as_str()),
            )?;
            quality_reports_written += 1;
        }

        if let Some(final_output) = final_output_rel.as_deref() {
            let final_asset_id = upsert_asset_tx(
                conn,
                project.id.as_str(),
                Some(run_id.as_str()),
                Some(job_id.as_str()),
                None,
                "job_final_output",
                final_output,
                repo_root,
                compute_hashes,
                Some(
                    json!({"selected_candidate": job_obj.get("selected_candidate").cloned().unwrap_or(Value::Null)}),
                ),
            )?;
            if final_asset_id.is_some() {
                inserted_assets += 1;
            }
            conn.execute(
                "
                UPDATE run_jobs
                SET final_asset_id = ?1, final_output = COALESCE(final_output, ?2)
                WHERE id = ?3
                ",
                params![final_asset_id, final_output, job_id],
            )?;
        }
    }

    if let Some(output_guard) = run_data.get("output_guard").and_then(Value::as_object) {
        let summary = json!({"scope": "run", "output_guard": output_guard});
        insert_quality_report_tx(
            conn,
            project.id.as_str(),
            Some(run_id.as_str()),
            None,
            "output_guard",
            &summary,
            Some(ts.as_str()),
        )?;
        quality_reports_written += 1;
    }

    for row in extract_cost_event_rows(run_data) {
        insert_cost_event_tx(
            conn,
            project.id.as_str(),
            Some(run_id.as_str()),
            row.job_id.as_deref(),
            row.provider_code.as_str(),
            row.model_name.as_deref().unwrap_or(model_name.as_str()),
            row.operation_code.as_str(),
            row.units,
            row.cost_usd,
            row.currency.as_str(),
            row.meta.as_ref(),
            Some(ts.as_str()),
        )?;
        cost_events_written += 1;
    }

    Ok(IngestRunLogResult {
        project_slug: project.slug.clone(),
        run_id,
        run_log_path: rel_run_log_path,
        jobs: inserted_jobs,
        candidates: inserted_candidates,
        assets_upserted: inserted_assets,
        quality_reports_written,
        cost_events_written,
        status: run_status,
    })
}

fn derive_run_status(run_data: &Value) -> String {
    let Some(jobs) = run_data.get("jobs").and_then(Value::as_array) else {
        return String::from("unknown");
    };
    let statuses: Vec<String> = jobs
        .iter()
        .filter_map(Value::as_object)
        .map(|job| {
            string_or_empty(job.get("status"))
                .trim()
                .to_ascii_lowercase()
        })
        .collect();
    if statuses.iter().any(|s| s.starts_with("failed")) {
        return String::from("failed");
    }
    if !statuses.is_empty() && statuses.iter().all(|s| s == "done" || s == "planned") {
        return String::from("ok");
    }
    String::from("partial")
}

fn insert_quality_report_tx(
    conn: &Connection,
    project_id: &str,
    run_id: Option<&str>,
    asset_id: Option<&str>,
    report_type: &str,
    summary: &Value,
    created_at: Option<&str>,
) -> Result<(), ProjectsRepoError> {
    let ts = created_at.unwrap_or_else(|| "");
    let hard_failures = summary
        .get("rank")
        .and_then(|v| v.get("hard_failures"))
        .map(|v| number_to_i64(Some(v)))
        .unwrap_or(0);
    let soft_warnings = summary
        .get("rank")
        .and_then(|v| v.get("soft_warnings"))
        .map(|v| number_to_i64(Some(v)))
        .unwrap_or(0);
    let avg_chroma_exceed = summary
        .get("rank")
        .and_then(|v| v.get("avg_chroma_exceed"))
        .map(|v| number_to_f64(Some(v)))
        .unwrap_or(0.0);
    let summary_json = serde_json::to_string(summary).unwrap_or_else(|_| String::from("{}"));
    conn.execute(
        "
        INSERT INTO quality_reports
          (id, project_id, run_id, asset_id, report_type, grade, hard_failures, soft_warnings, avg_chroma_exceed, summary_json, meta_json, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, ?7, ?8, ?9, ?9, ?10)
        ",
        params![
            Uuid::new_v4().to_string(),
            project_id,
            run_id,
            asset_id,
            report_type,
            hard_failures,
            soft_warnings,
            avg_chroma_exceed,
            summary_json,
            if ts.is_empty() { now_iso() } else { ts.to_string() },
        ],
    )?;
    Ok(())
}

fn insert_cost_event_tx(
    conn: &Connection,
    project_id: &str,
    run_id: Option<&str>,
    job_id: Option<&str>,
    provider_code: &str,
    model_name: &str,
    event_type: &str,
    units: f64,
    total_cost_usd: f64,
    currency: &str,
    meta: Option<&Value>,
    created_at: Option<&str>,
) -> Result<(), ProjectsRepoError> {
    let safe_units = if units.is_finite() { units } else { 0.0 };
    let safe_total = if total_cost_usd.is_finite() {
        total_cost_usd
    } else {
        0.0
    };
    let unit_cost = if safe_units.abs() > f64::EPSILON {
        safe_total / safe_units
    } else {
        0.0
    };
    let ts = created_at.map(ToOwned::to_owned).unwrap_or_else(now_iso);
    let meta_json = serde_json::to_string(meta.unwrap_or(&Value::Object(Default::default())))
        .unwrap_or_else(|_| String::from("{}"));
    conn.execute(
        "
        INSERT INTO cost_events
          (id, project_id, run_id, job_id, provider_code, model_name, event_type, units, unit_cost_usd, total_cost_usd, currency, meta_json, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ",
        params![
            Uuid::new_v4().to_string(),
            project_id,
            run_id,
            job_id,
            non_empty_fallback(provider_code, "unknown"),
            model_name.trim(),
            non_empty_fallback(event_type, "legacy_event"),
            safe_units,
            unit_cost,
            safe_total,
            non_empty_fallback(currency, "USD"),
            meta_json,
            ts,
        ],
    )?;
    Ok(())
}

fn upsert_asset_tx(
    conn: &Connection,
    project_id: &str,
    run_id: Option<&str>,
    job_id: Option<&str>,
    candidate_id: Option<&str>,
    asset_kind: &str,
    rel_path: &str,
    repo_root: &Path,
    compute_hashes: bool,
    extra_meta: Option<Value>,
) -> Result<Option<String>, ProjectsRepoError> {
    let Some(clean_rel) = normalize_rel_path_opt(rel_path) else {
        return Ok(None);
    };

    let abs_path = repo_root.join(clean_rel.as_str());
    let file_hash = if compute_hashes && abs_path.is_file() {
        Some(sha256_of_file(abs_path.as_path()).map_err(|error| {
            ProjectsRepoError::Validation(format!(
                "Failed to hash asset '{}': {error}",
                abs_path.display()
            ))
        })?)
    } else {
        None
    };
    let ts = now_iso();

    let mut payload = serde_json::Map::new();
    payload.insert(String::from("path_exists"), Value::Bool(abs_path.exists()));
    if let Some(Value::Object(extra)) = extra_meta {
        for (k, v) in extra {
            payload.insert(k, v);
        }
    }
    let payload_json = serde_json::to_string(&Value::Object(payload.clone()))
        .unwrap_or_else(|_| String::from("{}"));

    if let Some(existing_id) = conn
        .query_row(
            "
            SELECT id
            FROM assets
            WHERE project_id = ?1 AND (rel_path = ?2 OR storage_uri = ?2)
            ORDER BY COALESCE(created_at, '') DESC
            LIMIT 1
            ",
            params![project_id, clean_rel],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        conn.execute(
            "
            UPDATE assets
            SET run_id = ?1,
                job_id = ?2,
                candidate_id = ?3,
                asset_kind = ?4,
                kind = ?4,
                rel_path = ?5,
                storage_uri = ?5,
                sha256 = ?6,
                meta_json = ?7,
                metadata_json = ?7,
                created_at = ?8
            WHERE id = ?9
            ",
            params![
                run_id,
                job_id,
                candidate_id,
                asset_kind,
                clean_rel,
                file_hash,
                payload_json,
                ts,
                existing_id,
            ],
        )?;
        return Ok(Some(existing_id));
    }

    let asset_id = Uuid::new_v4().to_string();
    conn.execute(
        "
        INSERT INTO assets
          (id, project_id, run_id, job_id, candidate_id, asset_kind, kind, rel_path, storage_uri, sha256, meta_json, metadata_json, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?7, ?8, ?9, ?9, ?10)
        ",
        params![
            asset_id,
            project_id,
            run_id,
            job_id,
            candidate_id,
            asset_kind,
            clean_rel,
            file_hash,
            payload_json,
            ts,
        ],
    )?;
    Ok(Some(asset_id))
}

#[derive(Debug, Clone)]
struct CostEventRow {
    provider_code: String,
    operation_code: String,
    units: f64,
    cost_usd: f64,
    currency: String,
    model_name: Option<String>,
    job_id: Option<String>,
    meta: Option<Value>,
}

fn extract_cost_event_rows(run_data: &Value) -> Vec<CostEventRow> {
    let mut events = Vec::new();

    if let Some(raw_events) = run_data.get("cost_events").and_then(Value::as_array) {
        for item in raw_events {
            let Some(obj) = item.as_object() else {
                continue;
            };
            let provider_code =
                first_non_empty_str(&[obj.get("provider_code"), obj.get("provider")])
                    .unwrap_or_else(|| String::from("unknown"));
            let operation_code = first_non_empty_str(&[
                obj.get("operation_code"),
                obj.get("operation"),
                obj.get("event_type"),
            ])
            .unwrap_or_else(|| String::from("legacy_event"));
            let units = number_to_f64_opt(obj.get("units"))
                .or_else(|| number_to_f64_opt(obj.get("quantity")))
                .unwrap_or(0.0);
            let cost_usd = number_to_f64_opt(obj.get("cost_usd"))
                .or_else(|| number_to_f64_opt(obj.get("amount_cents")).map(|v| v / 100.0))
                .unwrap_or(0.0);
            events.push(CostEventRow {
                provider_code,
                operation_code,
                units,
                cost_usd,
                currency: first_non_empty_str(&[obj.get("currency")])
                    .unwrap_or_else(|| String::from("USD")),
                model_name: first_non_empty_str(&[obj.get("model_name"), obj.get("model")]),
                job_id: first_non_empty_str(&[obj.get("job_id")]),
                meta: Some(item.clone()),
            });
        }
    }

    if let Some(generation) = run_data.get("generation").and_then(Value::as_object) {
        let provider_code =
            first_non_empty_str(&[generation.get("provider_code"), generation.get("provider")])
                .unwrap_or_else(|| String::from("openai"));
        let operation_code = first_non_empty_str(&[generation.get("operation_code")])
            .unwrap_or_else(|| String::from("image_generation"));
        let units = number_to_f64_opt(generation.get("units"))
            .or_else(|| number_to_f64_opt(generation.get("images")))
            .or_else(|| number_to_f64_opt(generation.get("count")))
            .unwrap_or(0.0);
        let cost_usd = number_to_f64_opt(generation.get("cost_usd"))
            .or_else(|| number_to_f64_opt(generation.get("amount_cents")).map(|v| v / 100.0));
        if let Some(cost_usd) = cost_usd {
            events.push(CostEventRow {
                provider_code,
                operation_code,
                units,
                cost_usd,
                currency: first_non_empty_str(&[generation.get("currency")])
                    .unwrap_or_else(|| String::from("USD")),
                model_name: first_non_empty_str(&[
                    generation.get("model_name"),
                    generation.get("model"),
                ]),
                job_id: None,
                meta: Some(Value::Object(generation.clone())),
            });
        }
    }

    if events.is_empty() {
        let top_level_cost = number_to_f64_opt(run_data.get("cost_usd"))
            .or_else(|| number_to_f64_opt(run_data.get("amount_cents")).map(|v| v / 100.0));
        if let Some(cost_usd) = top_level_cost {
            events.push(CostEventRow {
                provider_code: String::from("unknown"),
                operation_code: String::from("run_total"),
                units: 1.0,
                cost_usd,
                currency: get_str(run_data, "currency").if_empty_then(String::from("USD")),
                model_name: None,
                job_id: None,
                meta: Some(json!({"source": "run_log_top_level"})),
            });
        }
    }

    events
}

fn path_for_storage(path: &Path, repo_root: &Path) -> String {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    match resolved.strip_prefix(repo_root) {
        Ok(rel) => normalize_rel_path_string(rel.to_string_lossy().as_ref()),
        Err(_) => normalize_rel_path_string(resolved.to_string_lossy().as_ref()),
    }
}

fn normalize_rel_path_opt(value: &str) -> Option<String> {
    let normalized = normalize_rel_path_string(value);
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_rel_path_string(value: &str) -> String {
    value.replace('\\', "/").trim().to_string()
}

fn sha256_of_file(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

fn get_str(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or_default()
        .to_string()
}

fn string_or_empty(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
        .to_string()
}

fn non_empty_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}

fn first_non_empty_str(values: &[Option<&Value>]) -> Option<String> {
    values.iter().find_map(|v| non_empty_string(*v))
}

fn as_i64_checked(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_u64().and_then(|v| i64::try_from(v).ok())),
        _ => None,
    }
}

fn number_to_i64(value: Option<&Value>) -> i64 {
    value.and_then(as_i64_checked).unwrap_or(0)
}

fn number_to_f64(value: Option<&Value>) -> f64 {
    number_to_f64_opt(value).unwrap_or(0.0)
}

fn number_to_f64_opt(value: Option<&Value>) -> Option<f64> {
    match value {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn non_empty_fallback<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

trait StringExt {
    fn if_empty_then(self, fallback: String) -> String;
}

impl StringExt for String {
    fn if_empty_then(self, fallback: String) -> String {
        if self.trim().is_empty() {
            fallback
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_ingest_test_{stamp}"));
        fs::create_dir_all(root.as_path()).expect("temp root should be created");
        root
    }

    #[test]
    fn ingests_run_log_into_rust_tables() {
        let repo_root = test_root();
        let db_path = repo_root.join("var/backend/app.db");
        let project_root = repo_root.join("var/projects/demo");
        fs::create_dir_all(project_root.join("outputs")).expect("project outputs dir should exist");
        fs::write(project_root.join("outputs/out.png"), b"png").expect("output file should exist");

        let run_log_path = project_root.join("runs/run_1.json");
        fs::create_dir_all(run_log_path.parent().expect("run log parent"))
            .expect("runs dir should exist");
        fs::write(
            run_log_path.as_path(),
            serde_json::to_vec_pretty(&json!({
                "mode": "run",
                "stage": "style",
                "time": "day",
                "weather": "clear",
                "model": "gpt-image-1",
                "jobs": [{
                    "id": "job_a",
                    "status": "done",
                    "prompt": "test prompt",
                    "selected_candidate": 1,
                    "final_output": "var/projects/demo/outputs/out.png",
                    "candidates": [{
                        "candidate_index": 1,
                        "status": "done",
                        "output": "var/projects/demo/outputs/out.png",
                        "final_output": "var/projects/demo/outputs/out.png",
                        "rank": {
                            "hard_failures": 0,
                            "soft_warnings": 1,
                            "avg_chroma_exceed": 0.2
                        }
                    }]
                }],
                "generation": {
                    "provider_code": "openai",
                    "operation_code": "image_generation",
                    "units": 1,
                    "cost_usd": 0.04,
                    "currency": "USD"
                }
            }))
            .expect("run log json should serialize"),
        )
        .expect("run log should be written");

        let store = ProjectsStore::new(db_path, repo_root.clone());
        store.initialize().expect("schema should initialize");

        let result = store
            .ingest_run_log(IngestRunLogInput {
                run_log_path: PathBuf::from("var/projects/demo/runs/run_1.json"),
                project_slug: String::from("demo"),
                project_name: String::from("Demo"),
                create_project_if_missing: true,
                compute_hashes: false,
            })
            .expect("ingest should succeed");

        assert_eq!(result.project_slug, "demo");
        assert_eq!(result.jobs, 1);
        assert_eq!(result.candidates, 1);
        assert_eq!(result.status, "ok");

        let runs = store.list_runs("demo", 10).expect("runs should list");
        assert_eq!(runs.len(), 1);
        let jobs = store
            .list_run_jobs("demo", runs[0].id.as_str())
            .expect("run jobs should list");
        assert_eq!(jobs.len(), 1);

        let assets = store.list_assets("demo", 20).expect("assets should list");
        assert!(!assets.is_empty());

        let quality = store
            .list_quality_reports("demo", 20)
            .expect("quality reports should list");
        assert!(!quality.is_empty());

        let cost = store
            .list_cost_events("demo", 20)
            .expect("cost events should list");
        assert_eq!(cost.len(), 1);

        let _ = fs::remove_dir_all(repo_root);
    }

    #[test]
    fn fallback_candidate_index_resets_per_job() {
        let repo_root = test_root();
        let db_path = repo_root.join("var/backend/app.db");
        let project_root = repo_root.join("var/projects/demo");
        fs::create_dir_all(project_root.join("outputs")).expect("project outputs dir should exist");
        fs::write(project_root.join("outputs/a.png"), b"a").expect("output a should exist");
        fs::write(project_root.join("outputs/b.png"), b"b").expect("output b should exist");

        let run_log_path = project_root.join("runs/run_2.json");
        fs::create_dir_all(run_log_path.parent().expect("run log parent"))
            .expect("runs dir should exist");
        fs::write(
            run_log_path.as_path(),
            serde_json::to_vec_pretty(&json!({
                "mode": "run",
                "jobs": [
                    {
                        "id": "job_a",
                        "status": "done",
                        "candidates": [{
                            "status": "done",
                            "output": "var/projects/demo/outputs/a.png",
                            "final_output": "var/projects/demo/outputs/a.png"
                        }]
                    },
                    {
                        "id": "job_b",
                        "status": "done",
                        "candidates": [{
                            "status": "done",
                            "output": "var/projects/demo/outputs/b.png",
                            "final_output": "var/projects/demo/outputs/b.png"
                        }]
                    }
                ]
            }))
            .expect("run log json should serialize"),
        )
        .expect("run log should be written");

        let store = ProjectsStore::new(db_path, repo_root.clone());
        store.initialize().expect("schema should initialize");
        let ingest = store
            .ingest_run_log(IngestRunLogInput {
                run_log_path: PathBuf::from("var/projects/demo/runs/run_2.json"),
                project_slug: String::from("demo"),
                project_name: String::from("Demo"),
                create_project_if_missing: true,
                compute_hashes: false,
            })
            .expect("ingest should succeed");

        let jobs = store
            .list_run_jobs("demo", ingest.run_id.as_str())
            .expect("run jobs should list");
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].candidates.len(), 1);
        assert_eq!(jobs[1].candidates.len(), 1);
        assert_eq!(jobs[0].candidates[0].candidate_index, 1);
        assert_eq!(jobs[1].candidates[0].candidate_index, 1);

        let _ = fs::remove_dir_all(repo_root);
    }

    #[test]
    fn reingest_same_run_log_path_replaces_run_state_idempotently() {
        let repo_root = test_root();
        let db_path = repo_root.join("var/backend/app.db");
        let project_root = repo_root.join("var/projects/demo");
        fs::create_dir_all(project_root.join("outputs")).expect("project outputs dir should exist");
        fs::write(project_root.join("outputs/v1.png"), b"v1").expect("output v1 should exist");
        fs::write(project_root.join("outputs/v2.png"), b"v2").expect("output v2 should exist");

        let run_log_path = project_root.join("runs/run_reingest.json");
        fs::create_dir_all(run_log_path.parent().expect("run log parent"))
            .expect("runs dir should exist");

        let write_run_log = |final_output: &str, cost_usd: f64| {
            fs::write(
                run_log_path.as_path(),
                serde_json::to_vec_pretty(&json!({
                    "mode": "run",
                    "stage": "style",
                    "time": "day",
                    "weather": "clear",
                    "model": "gpt-image-1",
                    "jobs": [{
                        "id": "job_a",
                        "status": "done",
                        "selected_candidate": 1,
                        "final_output": final_output,
                        "candidates": [{
                            "status": "done",
                            "output": final_output,
                            "final_output": final_output,
                            "rank": {
                                "hard_failures": 0,
                                "soft_warnings": 0,
                                "avg_chroma_exceed": 0.0
                            }
                        }]
                    }],
                    "generation": {
                        "provider_code": "openai",
                        "operation_code": "image_generation",
                        "units": 1,
                        "cost_usd": cost_usd,
                        "currency": "USD"
                    }
                }))
                .expect("run log json should serialize"),
            )
            .expect("run log should be written");
        };

        let store = ProjectsStore::new(db_path, repo_root.clone());
        store.initialize().expect("schema should initialize");

        write_run_log("var/projects/demo/outputs/v1.png", 0.04);
        let first = store
            .ingest_run_log(IngestRunLogInput {
                run_log_path: PathBuf::from("var/projects/demo/runs/run_reingest.json"),
                project_slug: String::from("demo"),
                project_name: String::from("Demo"),
                create_project_if_missing: true,
                compute_hashes: false,
            })
            .expect("first ingest should succeed");

        write_run_log("var/projects/demo/outputs/v2.png", 0.07);
        let second = store
            .ingest_run_log(IngestRunLogInput {
                run_log_path: PathBuf::from("var/projects/demo/runs/run_reingest.json"),
                project_slug: String::from("demo"),
                project_name: String::from("Demo"),
                create_project_if_missing: true,
                compute_hashes: false,
            })
            .expect("second ingest should succeed");

        assert_ne!(first.run_id, second.run_id);

        let runs = store.list_runs("demo", 10).expect("runs should list");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].id, second.run_id);
        assert_eq!(
            runs[0].run_log_path,
            "var/projects/demo/runs/run_reingest.json"
        );

        let jobs = store
            .list_run_jobs("demo", second.run_id.as_str())
            .expect("run jobs should list");
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].candidates.len(), 1);
        assert_eq!(jobs[0].candidates[0].candidate_index, 1);
        assert_eq!(
            jobs[0].candidates[0].final_output_path,
            "var/projects/demo/outputs/v2.png"
        );

        let quality = store
            .list_quality_reports("demo", 20)
            .expect("quality reports should list");
        assert_eq!(quality.len(), 1);
        assert_eq!(quality[0].run_id, second.run_id);

        let cost = store
            .list_cost_events("demo", 20)
            .expect("cost events should list");
        assert_eq!(cost.len(), 1);
        assert_eq!(cost[0].run_id, second.run_id);
        assert!((cost[0].total_cost_usd - 0.07).abs() < f64::EPSILON);

        let _ = fs::remove_dir_all(repo_root);
    }
}
