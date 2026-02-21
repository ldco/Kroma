use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;

use super::{
    fetch_project_by_slug, normalize_slug, parse_json_value, row_string_from_columns,
    ProjectsRepoError, ProjectsStore,
};

#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub id: String,
    pub project_id: String,
    pub run_mode: String,
    pub status: String,
    pub stage: String,
    pub time_of_day: String,
    pub weather: String,
    pub model_name: String,
    pub provider_code: String,
    pub settings_snapshot_json: Value,
    pub started_at: String,
    pub finished_at: String,
    pub created_at: String,
    pub run_log_path: String,
    pub image_size: String,
    pub image_quality: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunCandidateSummary {
    pub id: String,
    pub job_id: String,
    pub candidate_index: i64,
    pub status: String,
    pub output_asset_id: String,
    pub final_asset_id: String,
    pub output_path: String,
    pub final_output_path: String,
    pub rank_hard_failures: i64,
    pub rank_soft_warnings: i64,
    pub rank_avg_chroma_exceed: f64,
    pub meta_json: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunJobSummary {
    pub id: String,
    pub run_id: String,
    pub job_key: String,
    pub status: String,
    pub prompt_text: String,
    pub selected_candidate_index: Option<i64>,
    pub final_asset_id: String,
    pub final_output: String,
    pub meta_json: Value,
    pub created_at: String,
    pub candidates: Vec<RunCandidateSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetSummary {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub asset_kind: String,
    pub storage_uri: String,
    pub rel_path: String,
    pub storage_backend: String,
    pub mime_type: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub sha256: String,
    pub run_id: String,
    pub job_id: String,
    pub candidate_id: String,
    pub metadata_json: Value,
    pub created_at: String,
}

impl ProjectsStore {
    pub fn list_runs(&self, slug: &str, limit: i64) -> Result<Vec<RunSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 1000);
            let mut stmt = conn.prepare(
                "
                SELECT *
                FROM runs
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows = stmt.query_map(params![project.id, capped], row_to_run_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn get_run_detail(
        &self,
        slug: &str,
        run_id: &str,
    ) -> Result<(RunSummary, Vec<RunJobSummary>), ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let run = conn
                .query_row(
                    "
                    SELECT *
                    FROM runs
                    WHERE id = ?1 AND project_id = ?2
                    LIMIT 1
                ",
                    params![run_id, project.id],
                    row_to_run_summary,
                )
                .optional()?
                .ok_or(ProjectsRepoError::NotFound)?;

            let jobs = fetch_jobs_with_candidates(conn, run_id)?;
            Ok((run, jobs))
        })
    }

    pub fn list_run_jobs(
        &self,
        slug: &str,
        run_id: &str,
    ) -> Result<Vec<RunJobSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let run_exists = conn
                .query_row(
                    "SELECT id FROM runs WHERE id = ?1 AND project_id = ?2 LIMIT 1",
                    params![run_id, project.id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .is_some();
            if !run_exists {
                return Err(ProjectsRepoError::NotFound);
            }

            fetch_jobs_with_candidates(conn, run_id)
        })
    }

    pub fn list_assets(
        &self,
        slug: &str,
        limit: i64,
    ) -> Result<Vec<AssetSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 2000);

            let mut stmt = conn.prepare(
                "
                SELECT *
                FROM assets
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows = stmt.query_map(params![project.id, capped], row_to_asset_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn get_asset_detail(
        &self,
        slug: &str,
        asset_id: &str,
    ) -> Result<AssetSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            conn.query_row(
                "
                SELECT *
                FROM assets
                WHERE id = ?1 AND project_id = ?2
                LIMIT 1
            ",
                params![asset_id, project.id],
                row_to_asset_summary,
            )
            .optional()?
            .ok_or(ProjectsRepoError::NotFound)
        })
    }
}

fn row_to_run_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunSummary> {
    let settings_snapshot_json = parse_json_value(
        row.get::<_, Option<String>>("settings_snapshot_json")?
            .or(row.get::<_, Option<String>>("meta_json")?),
    );

    Ok(RunSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        run_mode: row_string_from_columns(row, &["run_mode", "mode"])?,
        status: row_string_from_columns(row, &["status"])?,
        stage: row_string_from_columns(row, &["stage"])?,
        time_of_day: row_string_from_columns(row, &["time_of_day"])?,
        weather: row_string_from_columns(row, &["weather"])?,
        model_name: row_string_from_columns(row, &["model_name", "model"])?,
        provider_code: row_string_from_columns(row, &["provider_code"])?,
        settings_snapshot_json,
        started_at: row_string_from_columns(row, &["started_at"])?,
        finished_at: row_string_from_columns(row, &["finished_at"])?,
        created_at: row_string_from_columns(row, &["created_at"])?,
        run_log_path: row_string_from_columns(row, &["run_log_path"])?,
        image_size: row_string_from_columns(row, &["image_size"])?,
        image_quality: row_string_from_columns(row, &["image_quality"])?,
    })
}

fn row_to_run_candidate_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunCandidateSummary> {
    Ok(RunCandidateSummary {
        id: row.get("id")?,
        job_id: row.get("job_id")?,
        candidate_index: row
            .get::<_, Option<i64>>("candidate_index")?
            .unwrap_or_default(),
        status: row.get::<_, Option<String>>("status")?.unwrap_or_default(),
        output_asset_id: row
            .get::<_, Option<String>>("output_asset_id")?
            .unwrap_or_default(),
        final_asset_id: row
            .get::<_, Option<String>>("final_asset_id")?
            .unwrap_or_default(),
        output_path: row
            .get::<_, Option<String>>("output_path")?
            .unwrap_or_default(),
        final_output_path: row
            .get::<_, Option<String>>("final_output_path")?
            .unwrap_or_default(),
        rank_hard_failures: row
            .get::<_, Option<i64>>("rank_hard_failures")?
            .unwrap_or_default(),
        rank_soft_warnings: row
            .get::<_, Option<i64>>("rank_soft_warnings")?
            .unwrap_or_default(),
        rank_avg_chroma_exceed: row
            .get::<_, Option<f64>>("rank_avg_chroma_exceed")?
            .unwrap_or(0.0),
        meta_json: parse_json_value(row.get::<_, Option<String>>("meta_json")?),
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
    })
}

fn row_to_asset_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssetSummary> {
    let kind = row_string_from_columns(row, &["kind", "asset_kind"])?;
    let asset_kind = row_string_from_columns(row, &["asset_kind", "kind"])?;
    let metadata_json = parse_json_value(
        row.get::<_, Option<String>>("metadata_json")?
            .or(row.get::<_, Option<String>>("meta_json")?),
    );

    Ok(AssetSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        kind,
        asset_kind,
        storage_uri: row
            .get::<_, Option<String>>("storage_uri")?
            .unwrap_or_default(),
        rel_path: row
            .get::<_, Option<String>>("rel_path")?
            .unwrap_or_default(),
        storage_backend: row
            .get::<_, Option<String>>("storage_backend")?
            .unwrap_or_default(),
        mime_type: row
            .get::<_, Option<String>>("mime_type")?
            .unwrap_or_default(),
        width: row.get::<_, Option<i64>>("width")?,
        height: row.get::<_, Option<i64>>("height")?,
        sha256: row.get::<_, Option<String>>("sha256")?.unwrap_or_default(),
        run_id: row.get::<_, Option<String>>("run_id")?.unwrap_or_default(),
        job_id: row.get::<_, Option<String>>("job_id")?.unwrap_or_default(),
        candidate_id: row
            .get::<_, Option<String>>("candidate_id")?
            .unwrap_or_default(),
        metadata_json,
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
    })
}

fn fetch_jobs_with_candidates(
    conn: &Connection,
    run_id: &str,
) -> Result<Vec<RunJobSummary>, ProjectsRepoError> {
    let mut candidates_by_job: HashMap<String, Vec<RunCandidateSummary>> = HashMap::new();

    let mut candidate_stmt = conn.prepare(
        "
        SELECT
          id,
          job_id,
          candidate_index,
          status,
          output_asset_id,
          final_asset_id,
          output_path,
          final_output_path,
          rank_hard_failures,
          rank_soft_warnings,
          rank_avg_chroma_exceed,
          meta_json,
          created_at
        FROM run_candidates
        WHERE job_id IN (
          SELECT id FROM run_jobs WHERE run_id = ?1
        )
        ORDER BY COALESCE(candidate_index, 0) ASC, id ASC
    ",
    )?;
    let mut candidate_rows = candidate_stmt.query([run_id])?;
    while let Some(row) = candidate_rows.next()? {
        let candidate = row_to_run_candidate_summary(row)?;
        candidates_by_job
            .entry(candidate.job_id.clone())
            .or_default()
            .push(candidate);
    }

    let mut stmt = conn.prepare(
        "
        SELECT
          id,
          run_id,
          job_key,
          status,
          prompt_text,
          selected_candidate_index,
          selected_candidate,
          final_asset_id,
          final_output,
          meta_json,
          created_at
        FROM run_jobs
        WHERE run_id = ?1
        ORDER BY COALESCE(created_at, '') ASC, id ASC
    ",
    )?;

    let mut rows = stmt.query([run_id])?;
    let mut out = Vec::new();

    while let Some(row) = rows.next()? {
        let job_id: String = row.get("id")?;
        let selected_candidate_index = row
            .get::<_, Option<i64>>("selected_candidate_index")?
            .or(row.get::<_, Option<i64>>("selected_candidate")?);

        out.push(RunJobSummary {
            id: job_id.clone(),
            run_id: row.get("run_id")?,
            job_key: row.get::<_, Option<String>>("job_key")?.unwrap_or_default(),
            status: row.get::<_, Option<String>>("status")?.unwrap_or_default(),
            prompt_text: row
                .get::<_, Option<String>>("prompt_text")?
                .unwrap_or_default(),
            selected_candidate_index,
            final_asset_id: row
                .get::<_, Option<String>>("final_asset_id")?
                .unwrap_or_default(),
            final_output: row
                .get::<_, Option<String>>("final_output")?
                .unwrap_or_default(),
            meta_json: parse_json_value(row.get::<_, Option<String>>("meta_json")?),
            created_at: row
                .get::<_, Option<String>>("created_at")?
                .unwrap_or_default(),
            candidates: candidates_by_job
                .remove(job_id.as_str())
                .unwrap_or_default(),
        });
    }

    Ok(out)
}
