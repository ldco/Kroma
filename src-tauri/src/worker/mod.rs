use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::{json, Value};
use thiserror::Error;

use crate::db::projects::{AgentInstructionWorkerLease, ProjectsRepoError, ProjectsStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionState {
    Draft,
    Queued,
    Running,
    Done,
    Failed,
    Canceled,
}

impl InstructionState {
    pub fn can_transition_to(self, next: Self) -> bool {
        use InstructionState::{Canceled, Done, Draft, Failed, Queued, Running};

        matches!(
            (self, next),
            (Draft, Queued)
                | (Draft, Canceled)
                | (Queued, Running)
                | (Queued, Canceled)
                | (Running, Done)
                | (Running, Failed)
                | (Running, Canceled)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentWorkerOptions {
    pub worker_id: String,
    pub once: bool,
    pub poll_interval_seconds: f64,
    pub max_locked_seconds: i64,
    pub default_max_attempts: i64,
    pub retry_backoff_seconds: i64,
    pub dispatch_timeout_seconds: f64,
    pub dispatch_retries: i64,
    pub dispatch_backoff_seconds: f64,
    pub agent_api_url: Option<String>,
    pub agent_api_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentWorkerRunSummary {
    pub ok: bool,
    pub worker_id: String,
    pub processed: usize,
}

#[derive(Debug, Error)]
pub enum AgentWorkerError {
    #[error(transparent)]
    Repo(#[from] ProjectsRepoError),
    #[error("http dispatch build failed: {0}")]
    DispatchBuild(String),
}

pub fn run_agent_worker_loop(
    store: &ProjectsStore,
    options: &AgentWorkerOptions,
) -> Result<AgentWorkerRunSummary, AgentWorkerError> {
    let mut processed = 0usize;
    loop {
        let lease = store.reserve_next_agent_instruction(
            options.worker_id.as_str(),
            options.max_locked_seconds,
            options.default_max_attempts,
        )?;
        let Some(lease) = lease else {
            if options.once {
                break;
            }
            thread::sleep(Duration::from_secs_f64(
                options.poll_interval_seconds.max(0.1),
            ));
            continue;
        };

        process_one_instruction(store, options, &lease)?;
        processed += 1;
        if options.once {
            break;
        }
    }

    Ok(AgentWorkerRunSummary {
        ok: true,
        worker_id: options.worker_id.clone(),
        processed,
    })
}

fn process_one_instruction(
    store: &ProjectsStore,
    options: &AgentWorkerOptions,
    lease: &AgentInstructionWorkerLease,
) -> Result<(), AgentWorkerError> {
    let attempts = lease.attempts + 1;
    let max_attempts = lease.max_attempts.max(options.default_max_attempts).max(1);
    let (target_url, target_token) = resolve_agent_target(store, options, lease)?;
    let dispatch = dispatch_instruction_http(
        options,
        lease,
        target_url.as_deref(),
        target_token.as_deref(),
    )?;

    if dispatch.ok {
        let remote_status = map_remote_status(
            dispatch
                .response
                .as_ref()
                .and_then(|v| v.get("status"))
                .and_then(Value::as_str)
                .unwrap_or("done"),
        );
        let response_payload = dispatch.response.unwrap_or_else(|| json!({}));
        store.complete_agent_instruction_success(
            lease,
            attempts,
            max_attempts,
            remote_status,
            &response_payload,
            dispatch.http_status,
        )?;
        return Ok(());
    }

    store.complete_agent_instruction_retry_or_fail(
        lease,
        attempts,
        max_attempts,
        options.retry_backoff_seconds,
        dispatch
            .error
            .as_deref()
            .unwrap_or("unknown_dispatch_error"),
    )?;
    Ok(())
}

fn resolve_agent_target(
    store: &ProjectsStore,
    options: &AgentWorkerOptions,
    lease: &AgentInstructionWorkerLease,
) -> Result<(Option<String>, Option<String>), AgentWorkerError> {
    let mut target_url = options
        .agent_api_url
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned);
    let mut token = options
        .agent_api_token
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned);

    if target_url.is_none() {
        target_url = store
            .get_project_secret_value(lease.project_slug.as_str(), "agent_api", "url")?
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
    }
    if token.is_none() {
        token = store
            .get_project_secret_value(lease.project_slug.as_str(), "agent_api", "token")?
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
    }

    Ok((target_url, token))
}

fn map_remote_status(remote_status: &str) -> &'static str {
    match remote_status.trim().to_ascii_lowercase().as_str() {
        "done" => "done",
        "failed" => "failed",
        "running" => "running",
        "accepted" | "queued" => "done",
        _ => "done",
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DispatchResult {
    ok: bool,
    http_status: Option<u16>,
    response: Option<Value>,
    error: Option<String>,
}

fn dispatch_instruction_http(
    options: &AgentWorkerOptions,
    lease: &AgentInstructionWorkerLease,
    target_url: Option<&str>,
    token: Option<&str>,
) -> Result<DispatchResult, AgentWorkerError> {
    let target_url = target_url.map(str::trim).unwrap_or_default();
    if target_url.is_empty() {
        return Ok(DispatchResult {
            ok: false,
            http_status: None,
            response: None,
            error: Some(String::from("missing_agent_api_url")),
        });
    }

    let payload = json!({
        "instruction_id": lease.instruction_id,
        "project_slug": lease.project_slug,
        "instruction_type": "agent_instruction",
        "objective": lease.instruction_text,
        "payload": {
            "instruction_text": lease.instruction_text
        }
    });

    let client = Client::builder()
        .timeout(Duration::from_secs_f64(
            options.dispatch_timeout_seconds.max(0.1),
        ))
        .build()
        .map_err(|error| AgentWorkerError::DispatchBuild(error.to_string()))?;

    let mut attempt = 0i64;
    let max_attempts = options.dispatch_retries.max(0) + 1;
    let mut last_error = String::from("dispatch_failed");

    while attempt < max_attempts {
        attempt += 1;
        let mut request = client
            .post(target_url)
            .header("Content-Type", "application/json")
            .json(&payload);
        if let Some(token) = token.map(str::trim).filter(|v| !v.is_empty()) {
            request = request.bearer_auth(token);
        }
        match request.send() {
            Ok(response) => {
                let status = response.status().as_u16();
                let body_text = response.text().unwrap_or_default();
                if (200..300).contains(&status) {
                    let parsed = serde_json::from_str::<Value>(body_text.as_str())
                        .unwrap_or_else(|_| json!({}));
                    return Ok(DispatchResult {
                        ok: true,
                        http_status: Some(status),
                        response: Some(parsed),
                        error: None,
                    });
                }
                last_error = format!("http_{status}:{body_text}");
            }
            Err(error) => {
                last_error = error.to_string();
            }
        }

        if attempt < max_attempts {
            thread::sleep(Duration::from_secs_f64(
                options.dispatch_backoff_seconds.max(0.1) * attempt as f64,
            ));
        }
    }

    Ok(DispatchResult {
        ok: false,
        http_status: None,
        response: None,
        error: Some(last_error),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_remote_status_accepts_known_values() {
        assert_eq!(map_remote_status("done"), "done");
        assert_eq!(map_remote_status("  DONE "), "done");
        assert_eq!(map_remote_status("failed"), "failed");
        assert_eq!(map_remote_status(" failed "), "failed");
        assert_eq!(map_remote_status("running"), "running");
        assert_eq!(map_remote_status(" Running "), "running");
        assert_eq!(map_remote_status("accepted"), "done");
        assert_eq!(map_remote_status("queued"), "done");
    }
}
