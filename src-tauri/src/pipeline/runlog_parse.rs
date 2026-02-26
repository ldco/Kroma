use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PipelineScriptRunSummary {
    pub run_log_path: PathBuf,
    pub project_slug: Option<String>,
    pub project_root: Option<String>,
    pub jobs: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PipelineScriptRunSummaryMarker {
    run_log_path: String,
    #[serde(default)]
    project_slug: Option<String>,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    jobs: Option<u64>,
}

pub(crate) fn parse_script_run_summary_from_stdout(
    stdout: &str,
) -> Option<PipelineScriptRunSummary> {
    const MARKER: &str = "KROMA_PIPELINE_SUMMARY_JSON:";
    if let Some(marker_line) = stdout
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with(MARKER))
    {
        let payload = marker_line.trim_start_matches(MARKER).trim();
        if !payload.is_empty() {
            if let Ok(parsed) = serde_json::from_str::<PipelineScriptRunSummaryMarker>(payload) {
                return Some(PipelineScriptRunSummary {
                    run_log_path: PathBuf::from(parsed.run_log_path),
                    project_slug: parsed.project_slug.filter(|v| !v.trim().is_empty()),
                    project_root: parsed.project_root.filter(|v| !v.trim().is_empty()),
                    jobs: parsed.jobs,
                });
            }
        }
    }

    let mut run_log_path = None;
    let mut project_slug = None;
    let mut project_root = None;
    let mut jobs = None;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("Run log:") {
            let value = value.trim();
            if !value.is_empty() {
                run_log_path = Some(PathBuf::from(value));
            }
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Project:") {
            let value = value.trim();
            if !value.is_empty() {
                project_slug = Some(value.to_string());
            }
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Project root:") {
            let value = value.trim();
            if !value.is_empty() {
                project_root = Some(value.to_string());
            }
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Jobs:") {
            let count_token = value.split_whitespace().next().unwrap_or_default();
            if let Ok(parsed) = count_token.parse::<u64>() {
                jobs = Some(parsed);
            }
        }
    }

    Some(PipelineScriptRunSummary {
        run_log_path: run_log_path?,
        project_slug,
        project_root,
        jobs,
    })
}

pub(crate) fn append_stderr_line(stderr: &mut String, line: impl AsRef<str>) {
    if !stderr.trim().is_empty() {
        stderr.push('\n');
    }
    stderr.push_str(line.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_script_run_summary_from_stdout_prefers_marker_payload() {
        let parsed = parse_script_run_summary_from_stdout(
            "KROMA_PIPELINE_SUMMARY_JSON: {\"run_log_path\":\"var/projects/demo/runs/run_1.json\",\"project_slug\":\"demo\",\"project_root\":\"var/projects/demo\",\"jobs\":3,\"mode\":\"run\"}\n",
        )
        .expect("summary should parse");

        assert_eq!(
            parsed.run_log_path,
            PathBuf::from("var/projects/demo/runs/run_1.json")
        );
        assert_eq!(parsed.project_slug.as_deref(), Some("demo"));
        assert_eq!(parsed.project_root.as_deref(), Some("var/projects/demo"));
        assert_eq!(parsed.jobs, Some(3));
    }

    #[test]
    fn append_stderr_line_appends_with_newline_separator() {
        let mut stderr = String::from("first");
        append_stderr_line(&mut stderr, "second");
        assert_eq!(stderr, "first\nsecond");
    }
}
