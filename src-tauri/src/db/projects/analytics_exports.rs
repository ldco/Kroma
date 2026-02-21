use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;

use super::{
    ensure_column, fetch_project_by_slug, normalize_slug, parse_json_value, ProjectsRepoError,
    ProjectsStore,
};

#[derive(Debug, Clone, Serialize)]
pub struct QualityReportSummary {
    pub id: String,
    pub project_id: String,
    pub run_id: String,
    pub asset_id: String,
    pub report_type: String,
    pub grade: String,
    pub hard_failures: i64,
    pub soft_warnings: i64,
    pub avg_chroma_exceed: f64,
    pub summary_json: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostEventSummary {
    pub id: String,
    pub project_id: String,
    pub run_id: String,
    pub job_id: String,
    pub provider_code: String,
    pub model_name: String,
    pub event_type: String,
    pub units: f64,
    pub unit_cost_usd: f64,
    pub total_cost_usd: f64,
    pub currency: String,
    pub meta_json: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectExportSummary {
    pub id: String,
    pub project_id: String,
    pub run_id: String,
    pub status: String,
    pub export_format: String,
    pub storage_uri: String,
    pub rel_path: String,
    pub file_size_bytes: i64,
    pub checksum_sha256: String,
    pub manifest_json: Value,
    pub created_at: String,
    pub completed_at: String,
}

impl ProjectsStore {
    pub fn list_quality_reports(
        &self,
        slug: &str,
        limit: i64,
    ) -> Result<Vec<QualityReportSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 2000);

            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  project_id,
                  run_id,
                  asset_id,
                  report_type,
                  grade,
                  hard_failures,
                  soft_warnings,
                  avg_chroma_exceed,
                  summary_json,
                  created_at
                FROM quality_reports
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows =
                stmt.query_map(params![project.id, capped], row_to_quality_report_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn list_cost_events(
        &self,
        slug: &str,
        limit: i64,
    ) -> Result<Vec<CostEventSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 2000);

            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  project_id,
                  run_id,
                  job_id,
                  provider_code,
                  model_name,
                  event_type,
                  units,
                  unit_cost_usd,
                  total_cost_usd,
                  currency,
                  meta_json,
                  created_at
                FROM cost_events
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows = stmt.query_map(params![project.id, capped], row_to_cost_event_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn list_project_exports(
        &self,
        slug: &str,
        limit: i64,
    ) -> Result<Vec<ProjectExportSummary>, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let capped = limit.clamp(1, 2000);

            let mut stmt = conn.prepare(
                "
                SELECT
                  id,
                  project_id,
                  run_id,
                  status,
                  export_format,
                  storage_uri,
                  rel_path,
                  file_size_bytes,
                  checksum_sha256,
                  manifest_json,
                  created_at,
                  completed_at
                FROM project_exports
                WHERE project_id = ?1
                ORDER BY COALESCE(created_at, '') DESC, id DESC
                LIMIT ?2
            ",
            )?;
            let rows =
                stmt.query_map(params![project.id, capped], row_to_project_export_summary)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
    }

    pub fn get_project_export_detail(
        &self,
        slug: &str,
        export_id: &str,
    ) -> Result<ProjectExportSummary, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            conn.query_row(
                "
                SELECT
                  id,
                  project_id,
                  run_id,
                  status,
                  export_format,
                  storage_uri,
                  rel_path,
                  file_size_bytes,
                  checksum_sha256,
                  manifest_json,
                  created_at,
                  completed_at
                FROM project_exports
                WHERE id = ?1 AND project_id = ?2
                LIMIT 1
            ",
                params![export_id, project.id],
                row_to_project_export_summary,
            )
            .optional()?
            .ok_or(ProjectsRepoError::NotFound)
        })
    }
}

pub(super) fn ensure_analytics_export_tables(conn: &Connection) -> Result<(), ProjectsRepoError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS quality_reports (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          asset_id TEXT,
          report_type TEXT,
          grade TEXT,
          hard_failures INTEGER,
          soft_warnings INTEGER,
          avg_chroma_exceed REAL,
          summary_json TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS cost_events (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          job_id TEXT,
          provider_code TEXT,
          model_name TEXT,
          event_type TEXT,
          units REAL,
          unit_cost_usd REAL,
          total_cost_usd REAL,
          currency TEXT,
          meta_json TEXT,
          created_at TEXT
        );

        CREATE TABLE IF NOT EXISTS project_exports (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          status TEXT,
          export_format TEXT,
          storage_uri TEXT,
          rel_path TEXT,
          file_size_bytes INTEGER,
          checksum_sha256 TEXT,
          manifest_json TEXT,
          created_at TEXT,
          completed_at TEXT
        );
    ",
    )?;

    Ok(())
}

pub(super) fn ensure_analytics_export_columns(conn: &Connection) -> Result<(), ProjectsRepoError> {
    ensure_column(conn, "quality_reports", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "quality_reports", "run_id", "TEXT")?;
    ensure_column(conn, "quality_reports", "asset_id", "TEXT")?;
    ensure_column(conn, "quality_reports", "report_type", "TEXT")?;
    ensure_column(conn, "quality_reports", "grade", "TEXT")?;
    ensure_column(conn, "quality_reports", "hard_failures", "INTEGER")?;
    ensure_column(conn, "quality_reports", "soft_warnings", "INTEGER")?;
    ensure_column(conn, "quality_reports", "avg_chroma_exceed", "REAL")?;
    ensure_column(conn, "quality_reports", "summary_json", "TEXT")?;
    ensure_column(conn, "quality_reports", "meta_json", "TEXT")?;
    ensure_column(conn, "quality_reports", "created_at", "TEXT")?;

    ensure_column(conn, "cost_events", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "cost_events", "run_id", "TEXT")?;
    ensure_column(conn, "cost_events", "job_id", "TEXT")?;
    ensure_column(conn, "cost_events", "provider_code", "TEXT")?;
    ensure_column(conn, "cost_events", "model_name", "TEXT")?;
    ensure_column(conn, "cost_events", "event_type", "TEXT")?;
    ensure_column(conn, "cost_events", "units", "REAL")?;
    ensure_column(conn, "cost_events", "unit_cost_usd", "REAL")?;
    ensure_column(conn, "cost_events", "total_cost_usd", "REAL")?;
    ensure_column(conn, "cost_events", "currency", "TEXT")?;
    ensure_column(conn, "cost_events", "meta_json", "TEXT")?;
    ensure_column(conn, "cost_events", "created_at", "TEXT")?;

    ensure_column(conn, "project_exports", "project_id", "TEXT NOT NULL")?;
    ensure_column(conn, "project_exports", "run_id", "TEXT")?;
    ensure_column(conn, "project_exports", "status", "TEXT")?;
    ensure_column(conn, "project_exports", "export_format", "TEXT")?;
    ensure_column(conn, "project_exports", "storage_uri", "TEXT")?;
    ensure_column(conn, "project_exports", "rel_path", "TEXT")?;
    ensure_column(conn, "project_exports", "file_size_bytes", "INTEGER")?;
    ensure_column(conn, "project_exports", "checksum_sha256", "TEXT")?;
    ensure_column(conn, "project_exports", "manifest_json", "TEXT")?;
    ensure_column(conn, "project_exports", "meta_json", "TEXT")?;
    ensure_column(conn, "project_exports", "created_at", "TEXT")?;
    ensure_column(conn, "project_exports", "completed_at", "TEXT")?;

    Ok(())
}

fn row_to_quality_report_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<QualityReportSummary> {
    Ok(QualityReportSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        run_id: row.get::<_, Option<String>>("run_id")?.unwrap_or_default(),
        asset_id: row
            .get::<_, Option<String>>("asset_id")?
            .unwrap_or_default(),
        report_type: row
            .get::<_, Option<String>>("report_type")?
            .unwrap_or_default(),
        grade: row.get::<_, Option<String>>("grade")?.unwrap_or_default(),
        hard_failures: row
            .get::<_, Option<i64>>("hard_failures")?
            .unwrap_or_default(),
        soft_warnings: row
            .get::<_, Option<i64>>("soft_warnings")?
            .unwrap_or_default(),
        avg_chroma_exceed: row
            .get::<_, Option<f64>>("avg_chroma_exceed")?
            .unwrap_or(0.0),
        summary_json: parse_json_value(row.get::<_, Option<String>>("summary_json")?),
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
    })
}

fn row_to_cost_event_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<CostEventSummary> {
    Ok(CostEventSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        run_id: row.get::<_, Option<String>>("run_id")?.unwrap_or_default(),
        job_id: row.get::<_, Option<String>>("job_id")?.unwrap_or_default(),
        provider_code: row
            .get::<_, Option<String>>("provider_code")?
            .unwrap_or_default(),
        model_name: row
            .get::<_, Option<String>>("model_name")?
            .unwrap_or_default(),
        event_type: row
            .get::<_, Option<String>>("event_type")?
            .unwrap_or_default(),
        units: row.get::<_, Option<f64>>("units")?.unwrap_or(0.0),
        unit_cost_usd: row.get::<_, Option<f64>>("unit_cost_usd")?.unwrap_or(0.0),
        total_cost_usd: row.get::<_, Option<f64>>("total_cost_usd")?.unwrap_or(0.0),
        currency: row
            .get::<_, Option<String>>("currency")?
            .unwrap_or_default(),
        meta_json: parse_json_value(row.get::<_, Option<String>>("meta_json")?),
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
    })
}

fn row_to_project_export_summary(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<ProjectExportSummary> {
    Ok(ProjectExportSummary {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        run_id: row.get::<_, Option<String>>("run_id")?.unwrap_or_default(),
        status: row.get::<_, Option<String>>("status")?.unwrap_or_default(),
        export_format: row
            .get::<_, Option<String>>("export_format")?
            .unwrap_or_default(),
        storage_uri: row
            .get::<_, Option<String>>("storage_uri")?
            .unwrap_or_default(),
        rel_path: row
            .get::<_, Option<String>>("rel_path")?
            .unwrap_or_default(),
        file_size_bytes: row
            .get::<_, Option<i64>>("file_size_bytes")?
            .unwrap_or_default(),
        checksum_sha256: row
            .get::<_, Option<String>>("checksum_sha256")?
            .unwrap_or_default(),
        manifest_json: parse_json_value(row.get::<_, Option<String>>("manifest_json")?),
        created_at: row
            .get::<_, Option<String>>("created_at")?
            .unwrap_or_default(),
        completed_at: row
            .get::<_, Option<String>>("completed_at")?
            .unwrap_or_default(),
    })
}
