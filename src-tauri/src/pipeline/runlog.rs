use std::fs;
use std::path::Path;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PipelineRunSummaryMarkerPayload {
    pub run_log_path: String,
    pub project_slug: String,
    pub project_root: String,
    pub jobs: u64,
    pub mode: String,
}

#[derive(Debug, Error)]
pub enum RunLogError {
    #[error("failed to create run-log parent directory '{path}': {message}")]
    CreateParent { path: String, message: String },
    #[error("failed to serialize run log JSON: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to write run log '{path}': {message}")]
    WriteFile { path: String, message: String },
}

pub fn write_pretty_json_with_newline<T>(path: &Path, value: &T) -> Result<(), RunLogError>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| RunLogError::CreateParent {
            path: parent.display().to_string(),
            message: error.to_string(),
        })?;
    }

    let mut bytes = serde_json::to_vec_pretty(value).map_err(RunLogError::Serialize)?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| RunLogError::WriteFile {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    Ok(())
}

pub fn format_summary_marker(
    payload: &PipelineRunSummaryMarkerPayload,
) -> Result<String, RunLogError> {
    let json = serde_json::to_string(payload).map_err(RunLogError::Serialize)?;
    Ok(format!("KROMA_PIPELINE_SUMMARY_JSON: {json}"))
}

pub(crate) fn iso_like_timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", now.as_secs(), now.subsec_millis())
}

pub(crate) fn run_log_stamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{:03}", now.as_secs(), now.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("kroma_runlog_test_{stamp}"))
            .join("runs/run.json")
    }

    #[test]
    fn writes_pretty_json_with_trailing_newline() {
        let path = temp_file_path();
        write_pretty_json_with_newline(path.as_path(), &json!({"ok": true}))
            .expect("run log should write");

        let raw = fs::read_to_string(path.as_path()).expect("run log should be readable");
        assert!(raw.ends_with('\n'));
        assert!(raw.contains("\"ok\": true"));

        let _ = fs::remove_dir_all(
            path.parent()
                .and_then(Path::parent)
                .unwrap_or_else(|| Path::new("/tmp")),
        );
    }

    #[test]
    fn formats_summary_marker_line() {
        let line = format_summary_marker(&PipelineRunSummaryMarkerPayload {
            run_log_path: String::from("var/projects/demo/runs/run_1.json"),
            project_slug: String::from("demo"),
            project_root: String::from("var/projects/demo"),
            jobs: 2,
            mode: String::from("dry"),
        })
        .expect("marker should format");

        assert!(line.starts_with("KROMA_PIPELINE_SUMMARY_JSON: "));
        assert!(line.contains("\"project_slug\":\"demo\""));
    }

    #[test]
    fn iso_like_timestamp_now_matches_expected_shape() {
        let value = iso_like_timestamp_now();
        let (secs, millis_and_z) = value
            .split_once('.')
            .expect("timestamp should include fractional separator");
        assert!(secs.parse::<u64>().is_ok());
        assert!(millis_and_z.ends_with('Z'));
        let millis = &millis_and_z[..millis_and_z.len() - 1];
        assert_eq!(millis.len(), 3);
        assert!(millis.chars().all(|ch| ch.is_ascii_digit()));
    }

    #[test]
    fn run_log_stamp_now_matches_expected_shape() {
        let value = run_log_stamp_now();
        let (secs, millis) = value
            .split_once('-')
            .expect("stamp should include millis separator");
        assert!(secs.parse::<u64>().is_ok());
        assert_eq!(millis.len(), 3);
        assert!(millis.chars().all(|ch| ch.is_ascii_digit()));
    }
}
