use std::path::PathBuf;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::db::projects::{IngestRunLogInput, ProjectsRepoError, ProjectsStore};
use crate::pipeline::runtime::{
    default_app_root_from_manifest_dir, CommandOutput, CommandSpec, PipelineCommandRunner,
    PipelineRuntimeError, StdPipelineCommandRunner,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendIngestRunRequest {
    pub run_log_path: PathBuf,
    pub project_slug: String,
    pub project_name: String,
    pub create_project_if_missing: bool,
    pub compute_hashes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendSyncProjectS3Request {
    pub project_slug: String,
    pub dry_run: bool,
    pub delete: bool,
    pub allow_missing_local: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackendCommandResult {
    pub stdout: String,
    pub stderr: String,
    pub json: Option<Value>,
}

impl BackendCommandResult {
    pub fn parse_json_as<T>(&self) -> Result<T, BackendOpsError>
    where
        T: DeserializeOwned,
    {
        let payload = self
            .json
            .as_ref()
            .ok_or(BackendOpsError::MissingJsonOutput)?;
        serde_json::from_value(payload.clone()).map_err(BackendOpsError::JsonDecode)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendIngestRunResponse {
    pub ok: bool,
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct BackendSyncProjectS3Response {
    pub ok: bool,
    #[serde(default)]
    pub project_slug: Option<String>,
    pub project_root: String,
    pub destination: String,
    #[serde(default)]
    pub dry_run: Option<bool>,
    #[serde(default)]
    pub delete: Option<bool>,
    #[serde(default)]
    pub skipped: Option<bool>,
    #[serde(default)]
    pub reason: Option<String>,
}

pub trait PipelineBackendOps: Send + Sync + 'static {
    fn ingest_run(
        &self,
        request: &BackendIngestRunRequest,
    ) -> Result<BackendCommandResult, BackendOpsError>;
    fn sync_project_s3(
        &self,
        request: &BackendSyncProjectS3Request,
    ) -> Result<BackendCommandResult, BackendOpsError>;
}

pub type SharedPipelineBackendOps = Arc<dyn PipelineBackendOps>;

#[derive(Clone)]
pub struct NativeIngestScriptSyncBackendOps<R> {
    projects_store: Arc<ProjectsStore>,
    script_sync_ops: ScriptPipelineBackendOps<R>,
}

impl<R> NativeIngestScriptSyncBackendOps<R>
where
    R: PipelineCommandRunner,
{
    pub fn new(
        projects_store: Arc<ProjectsStore>,
        script_sync_ops: ScriptPipelineBackendOps<R>,
    ) -> Self {
        Self {
            projects_store,
            script_sync_ops,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptPipelineBackendOps<R> {
    runner: R,
    app_root: PathBuf,
    python_binary: String,
    script_rel_path: PathBuf,
    db_rel_path: PathBuf,
}

impl<R> ScriptPipelineBackendOps<R>
where
    R: PipelineCommandRunner,
{
    pub fn new(app_root: PathBuf, runner: R) -> Self {
        Self {
            runner,
            app_root,
            python_binary: String::from("python3"),
            script_rel_path: PathBuf::from("scripts/backend.py"),
            db_rel_path: PathBuf::from("var/backend/app.db"),
        }
    }

    pub fn with_python_binary(mut self, python_binary: impl Into<String>) -> Self {
        self.python_binary = python_binary.into();
        self
    }

    pub fn with_script_rel_path(mut self, script_rel_path: impl Into<PathBuf>) -> Self {
        self.script_rel_path = script_rel_path.into();
        self
    }

    pub fn with_db_rel_path(mut self, db_rel_path: impl Into<PathBuf>) -> Self {
        self.db_rel_path = db_rel_path.into();
        self
    }

    pub fn build_ingest_run_command(
        &self,
        request: &BackendIngestRunRequest,
    ) -> Result<CommandSpec, BackendOpsError> {
        validate_project_slug(request.project_slug.as_str())?;
        let script_path = self.script_abs_path()?;

        let mut args = vec![
            script_path.to_string_lossy().to_string(),
            String::from("--db"),
            self.db_rel_path.to_string_lossy().to_string(),
            String::from("ingest-run"),
            String::from("--run-log"),
            request.run_log_path.to_string_lossy().to_string(),
            String::from("--project-slug"),
            request.project_slug.clone(),
            String::from("--project-name"),
            if request.project_name.trim().is_empty() {
                request.project_slug.clone()
            } else {
                request.project_name.clone()
            },
        ];

        if !request.create_project_if_missing {
            args.push(String::from("--no-create-project-if-missing"));
        }
        if request.compute_hashes {
            args.push(String::from("--compute-hashes"));
        }

        Ok(CommandSpec {
            program: self.python_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    pub fn build_sync_project_s3_command(
        &self,
        request: &BackendSyncProjectS3Request,
    ) -> Result<CommandSpec, BackendOpsError> {
        validate_project_slug(request.project_slug.as_str())?;
        let script_path = self.script_abs_path()?;

        let mut args = vec![
            script_path.to_string_lossy().to_string(),
            String::from("--db"),
            self.db_rel_path.to_string_lossy().to_string(),
            String::from("sync-project-s3"),
            String::from("--project-slug"),
            request.project_slug.clone(),
        ];

        if request.dry_run {
            args.push(String::from("--dry-run"));
        }
        if request.delete {
            args.push(String::from("--delete"));
        }
        if request.allow_missing_local {
            args.push(String::from("--allow-missing-local"));
        }

        Ok(CommandSpec {
            program: self.python_binary.clone(),
            args,
            cwd: self.app_root.clone(),
        })
    }

    fn script_abs_path(&self) -> Result<PathBuf, BackendOpsError> {
        let script_path = self.app_root.join(self.script_rel_path.as_path());
        if !script_path.is_file() {
            return Err(BackendOpsError::ScriptNotFound(script_path));
        }
        Ok(script_path)
    }

    fn run_command(&self, spec: CommandSpec) -> Result<BackendCommandResult, BackendOpsError> {
        let output = self
            .runner
            .run(&spec)
            .map_err(BackendOpsError::CommandRunner)?;

        if output.status_code != 0 {
            return Err(BackendOpsError::CommandFailed {
                program: spec.program,
                status_code: output.status_code,
                stderr: output.stderr,
            });
        }

        Ok(parse_backend_command_result(output))
    }

    pub fn ingest_run_typed(
        &self,
        request: &BackendIngestRunRequest,
    ) -> Result<BackendIngestRunResponse, BackendOpsError> {
        let result = self.ingest_run(request)?;
        result.parse_json_as()
    }

    pub fn sync_project_s3_typed(
        &self,
        request: &BackendSyncProjectS3Request,
    ) -> Result<BackendSyncProjectS3Response, BackendOpsError> {
        let result = self.sync_project_s3(request)?;
        result.parse_json_as()
    }
}

impl<R> PipelineBackendOps for ScriptPipelineBackendOps<R>
where
    R: PipelineCommandRunner,
{
    fn ingest_run(
        &self,
        request: &BackendIngestRunRequest,
    ) -> Result<BackendCommandResult, BackendOpsError> {
        let spec = self.build_ingest_run_command(request)?;
        self.run_command(spec)
    }

    fn sync_project_s3(
        &self,
        request: &BackendSyncProjectS3Request,
    ) -> Result<BackendCommandResult, BackendOpsError> {
        let spec = self.build_sync_project_s3_command(request)?;
        self.run_command(spec)
    }
}

impl<R> PipelineBackendOps for NativeIngestScriptSyncBackendOps<R>
where
    R: PipelineCommandRunner,
{
    fn ingest_run(
        &self,
        request: &BackendIngestRunRequest,
    ) -> Result<BackendCommandResult, BackendOpsError> {
        validate_project_slug(request.project_slug.as_str())?;
        let result = self
            .projects_store
            .ingest_run_log(IngestRunLogInput {
                run_log_path: request.run_log_path.clone(),
                project_slug: request.project_slug.clone(),
                project_name: request.project_name.clone(),
                create_project_if_missing: request.create_project_if_missing,
                compute_hashes: request.compute_hashes,
            })
            .map_err(BackendOpsError::ProjectsRepo)?;

        let payload = serde_json::to_value(BackendIngestRunResponse {
            ok: true,
            project_slug: result.project_slug,
            run_id: result.run_id,
            run_log_path: result.run_log_path,
            jobs: result.jobs,
            candidates: result.candidates,
            assets_upserted: result.assets_upserted,
            quality_reports_written: result.quality_reports_written,
            cost_events_written: result.cost_events_written,
            status: result.status,
        })
        .map_err(BackendOpsError::JsonEncode)?;

        Ok(BackendCommandResult {
            stdout: payload.to_string(),
            stderr: String::new(),
            json: Some(payload),
        })
    }

    fn sync_project_s3(
        &self,
        request: &BackendSyncProjectS3Request,
    ) -> Result<BackendCommandResult, BackendOpsError> {
        validate_project_slug(request.project_slug.as_str())?;
        match self.sync_project_s3_precheck(request)? {
            SyncPrecheckOutcome::Skipped(result) => Ok(result),
            SyncPrecheckOutcome::Ready(ready) => self.execute_aws_sync(request, ready),
        }
    }
}

impl<R> NativeIngestScriptSyncBackendOps<R>
where
    R: PipelineCommandRunner,
{
    fn sync_project_s3_precheck(
        &self,
        request: &BackendSyncProjectS3Request,
    ) -> Result<SyncPrecheckOutcome, BackendOpsError> {
        let payload = self
            .projects_store
            .get_project_storage(request.project_slug.as_str())
            .map_err(BackendOpsError::ProjectsRepo)?;

        let s3 = &payload.storage.s3;
        if !s3.enabled {
            return Err(BackendOpsError::SyncPrecheck(String::from(
                "S3 storage is disabled for this project. Enable via set-project-storage-s3.",
            )));
        }
        let bucket = s3.bucket.trim();
        if bucket.is_empty() {
            return Err(BackendOpsError::SyncPrecheck(String::from(
                "S3 bucket is not configured for this project.",
            )));
        }

        let prefix = s3.prefix.trim().trim_matches('/');
        let dst = if prefix.is_empty() {
            format!("s3://{bucket}/{}/", payload.project.id)
        } else {
            format!("s3://{bucket}/{prefix}/{}/", payload.project.id)
        };

        let local_root = PathBuf::from(payload.storage.local.project_root.clone());
        if !local_root.exists() {
            if request.allow_missing_local {
                let payload = json!({
                    "ok": true,
                    "skipped": true,
                    "reason": "missing_local_project_root",
                    "project_root": payload.storage.local.project_root,
                    "destination": dst,
                });
                return Ok(SyncPrecheckOutcome::Skipped(BackendCommandResult {
                    stdout: payload.to_string(),
                    stderr: String::new(),
                    json: Some(payload),
                }));
            }
            return Err(BackendOpsError::SyncPrecheck(format!(
                "Local project root not found: {}",
                local_root.display()
            )));
        }
        if !local_root.is_dir() {
            return Err(BackendOpsError::SyncPrecheck(format!(
                "Local project root is not a directory: {}",
                local_root.display()
            )));
        }

        Ok(SyncPrecheckOutcome::Ready(SyncReadyContext {
            project_slug: payload.project.slug,
            project_root: payload.storage.local.project_root,
            destination: dst,
            region: s3.region.clone(),
            profile: s3.profile.clone(),
            endpoint_url: s3.endpoint_url.clone(),
        }))
    }

    fn execute_aws_sync(
        &self,
        request: &BackendSyncProjectS3Request,
        ready: SyncReadyContext,
    ) -> Result<BackendCommandResult, BackendOpsError> {
        let mut args = vec![
            String::from("s3"),
            String::from("sync"),
            ready.project_root.clone(),
            ready.destination.clone(),
            String::from("--only-show-errors"),
        ];
        if request.delete {
            args.push(String::from("--delete"));
        }
        if request.dry_run {
            args.push(String::from("--dryrun"));
        }
        if !ready.region.trim().is_empty() {
            args.push(String::from("--region"));
            args.push(ready.region.clone());
        }
        if !ready.profile.trim().is_empty() {
            args.push(String::from("--profile"));
            args.push(ready.profile.clone());
        }
        if !ready.endpoint_url.trim().is_empty() {
            args.push(String::from("--endpoint-url"));
            args.push(ready.endpoint_url.clone());
        }

        let spec = CommandSpec {
            program: String::from("aws"),
            args,
            cwd: self.script_sync_ops.app_root.clone(),
        };
        let output = self
            .script_sync_ops
            .runner
            .run(&spec)
            .map_err(|err| match err {
                PipelineRuntimeError::Io(source)
                    if source.kind() == std::io::ErrorKind::NotFound =>
                {
                    BackendOpsError::SyncPrecheck(String::from(
                        "AWS CLI not found. Install aws cli v2 to use sync-project-s3.",
                    ))
                }
                other => BackendOpsError::CommandRunner(other),
            })?;

        if output.status_code != 0 {
            let stderr = if output.stderr.trim().is_empty() {
                output.stdout
            } else {
                output.stderr
            };
            return Err(BackendOpsError::CommandFailed {
                program: spec.program,
                status_code: output.status_code,
                stderr,
            });
        }

        let payload = json!({
            "ok": true,
            "project_slug": ready.project_slug,
            "project_root": ready.project_root,
            "destination": ready.destination,
            "dry_run": request.dry_run,
            "delete": request.delete,
        });
        Ok(BackendCommandResult {
            stdout: payload.to_string(),
            stderr: String::new(),
            json: Some(payload),
        })
    }
}

enum SyncPrecheckOutcome {
    Skipped(BackendCommandResult),
    Ready(SyncReadyContext),
}

struct SyncReadyContext {
    project_slug: String,
    project_root: String,
    destination: String,
    region: String,
    profile: String,
    endpoint_url: String,
}

fn parse_backend_command_result(output: CommandOutput) -> BackendCommandResult {
    let json = serde_json::from_str::<Value>(output.stdout.as_str()).ok();
    BackendCommandResult {
        stdout: output.stdout,
        stderr: output.stderr,
        json,
    }
}

#[derive(Debug, Error)]
pub enum BackendOpsError {
    #[error("invalid project slug for backend operation")]
    InvalidProjectSlug,
    #[error("backend script not found: {0}")]
    ScriptNotFound(PathBuf),
    #[error("backend command runner error: {0}")]
    CommandRunner(#[source] PipelineRuntimeError),
    #[error("backend command failed ({program}) with exit code {status_code}: {stderr}")]
    CommandFailed {
        program: String,
        status_code: i32,
        stderr: String,
    },
    #[error("backend command produced no JSON stdout payload")]
    MissingJsonOutput,
    #[error("backend command JSON decode failed: {0}")]
    JsonDecode(#[source] serde_json::Error),
    #[error("backend command JSON encode failed: {0}")]
    JsonEncode(#[source] serde_json::Error),
    #[error("backend projects repo error: {0}")]
    ProjectsRepo(#[source] ProjectsRepoError),
    #[error("{0}")]
    SyncPrecheck(String),
}

fn validate_project_slug(value: &str) -> Result<(), BackendOpsError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(BackendOpsError::InvalidProjectSlug);
    }
    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        Ok(())
    } else {
        Err(BackendOpsError::InvalidProjectSlug)
    }
}

pub fn default_script_backend_ops() -> ScriptPipelineBackendOps<StdPipelineCommandRunner> {
    ScriptPipelineBackendOps::new(
        default_app_root_from_manifest_dir(),
        StdPipelineCommandRunner,
    )
}

pub fn default_backend_ops_with_native_ingest(
    projects_store: Arc<ProjectsStore>,
) -> NativeIngestScriptSyncBackendOps<StdPipelineCommandRunner> {
    NativeIngestScriptSyncBackendOps::new(projects_store, default_script_backend_ops())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::{UpdateStorageS3Input, UpsertProjectInput};
    use crate::pipeline::runtime::CommandOutput;
    use std::fs;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone, Default)]
    struct FakeRunner {
        seen: Arc<Mutex<Vec<CommandSpec>>>,
        next: Arc<Mutex<Option<Result<CommandOutput, PipelineRuntimeError>>>>,
    }

    impl FakeRunner {
        fn with_next(result: Result<CommandOutput, PipelineRuntimeError>) -> Self {
            Self {
                seen: Arc::new(Mutex::new(Vec::new())),
                next: Arc::new(Mutex::new(Some(result))),
            }
        }

        fn take_seen(&self) -> Vec<CommandSpec> {
            std::mem::take(&mut *self.seen.lock().expect("fake runner mutex poisoned"))
        }
    }

    impl PipelineCommandRunner for FakeRunner {
        fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, PipelineRuntimeError> {
            self.seen
                .lock()
                .expect("fake runner mutex poisoned")
                .push(spec.clone());
            self.next
                .lock()
                .expect("fake runner mutex poisoned")
                .take()
                .unwrap_or_else(|| {
                    Ok(CommandOutput {
                        status_code: 0,
                        stdout: String::new(),
                        stderr: String::new(),
                    })
                })
        }
    }

    fn test_ops(runner: FakeRunner) -> ScriptPipelineBackendOps<FakeRunner> {
        ScriptPipelineBackendOps::new(default_app_root_from_manifest_dir(), runner)
    }

    fn temp_repo_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("kroma_backend_ops_test_{stamp}"));
        fs::create_dir_all(root.as_path()).expect("temp repo root should be created");
        root
    }

    #[test]
    fn builds_ingest_run_command() {
        let ops = test_ops(FakeRunner::default());
        let cmd = ops
            .build_ingest_run_command(&BackendIngestRunRequest {
                run_log_path: PathBuf::from("/tmp/run.json"),
                project_slug: String::from("demo"),
                project_name: String::from("Demo"),
                create_project_if_missing: true,
                compute_hashes: false,
            })
            .expect("ingest command should build");

        assert_eq!(cmd.program, "python3");
        assert!(cmd.args.iter().any(|arg| arg == "ingest-run"));
        assert!(cmd.args.iter().any(|arg| arg == "--run-log"));
        assert!(!cmd
            .args
            .iter()
            .any(|arg| arg == "--no-create-project-if-missing"));
    }

    #[test]
    fn builds_sync_project_s3_command_with_flags() {
        let ops = test_ops(FakeRunner::default());
        let cmd = ops
            .build_sync_project_s3_command(&BackendSyncProjectS3Request {
                project_slug: String::from("demo"),
                dry_run: true,
                delete: true,
                allow_missing_local: true,
            })
            .expect("sync command should build");

        assert!(cmd.args.iter().any(|arg| arg == "sync-project-s3"));
        assert!(cmd.args.iter().any(|arg| arg == "--dry-run"));
        assert!(cmd.args.iter().any(|arg| arg == "--delete"));
        assert!(cmd.args.iter().any(|arg| arg == "--allow-missing-local"));
    }

    #[test]
    fn parses_json_stdout_when_present() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from("{\"ok\":true}"),
            stderr: String::new(),
        }));
        let ops = test_ops(runner.clone());

        let result = ops
            .sync_project_s3(&BackendSyncProjectS3Request {
                project_slug: String::from("demo"),
                dry_run: false,
                delete: false,
                allow_missing_local: false,
            })
            .expect("sync should succeed");

        assert_eq!(result.json, Some(serde_json::json!({"ok": true})));
        assert_eq!(runner.take_seen().len(), 1);
    }

    #[test]
    fn parses_typed_ingest_run_response() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from(
                "{\"ok\":true,\"project_slug\":\"demo\",\"run_id\":\"r1\",\"run_log_path\":\"var/projects/demo/runs/run.json\",\"jobs\":1,\"candidates\":2,\"assets_upserted\":3,\"quality_reports_written\":4,\"cost_events_written\":5,\"status\":\"ok\"}",
            ),
            stderr: String::new(),
        }));
        let ops = test_ops(runner);

        let parsed = ops
            .ingest_run_typed(&BackendIngestRunRequest {
                run_log_path: PathBuf::from("var/projects/demo/runs/run.json"),
                project_slug: String::from("demo"),
                project_name: String::from("Demo"),
                create_project_if_missing: true,
                compute_hashes: false,
            })
            .expect("typed ingest response should parse");

        assert_eq!(parsed.project_slug, "demo");
        assert_eq!(parsed.jobs, 1);
        assert_eq!(parsed.status, "ok");
    }

    #[test]
    fn parses_typed_sync_project_s3_skipped_response() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from(
                "{\"ok\":true,\"skipped\":true,\"reason\":\"missing_local_project_root\",\"project_root\":\"/tmp/demo\",\"destination\":\"s3://bucket/demo/\"}",
            ),
            stderr: String::new(),
        }));
        let ops = test_ops(runner);

        let parsed = ops
            .sync_project_s3_typed(&BackendSyncProjectS3Request {
                project_slug: String::from("demo"),
                dry_run: false,
                delete: false,
                allow_missing_local: true,
            })
            .expect("typed sync response should parse");

        assert_eq!(parsed.ok, true);
        assert_eq!(parsed.skipped, Some(true));
        assert_eq!(parsed.reason.as_deref(), Some("missing_local_project_root"));
        assert!(parsed.project_slug.is_none());
    }

    #[test]
    fn typed_parse_errors_when_stdout_is_not_json() {
        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::from("not-json"),
            stderr: String::new(),
        }));
        let ops = test_ops(runner);

        let err = ops
            .sync_project_s3_typed(&BackendSyncProjectS3Request {
                project_slug: String::from("demo"),
                dry_run: false,
                delete: false,
                allow_missing_local: false,
            })
            .expect_err("typed parse should fail");

        assert!(matches!(err, BackendOpsError::MissingJsonOutput));
    }

    #[test]
    fn hybrid_sync_precheck_skips_missing_local_without_calling_script() {
        let repo_root = temp_repo_root();
        let db_path = repo_root.join("var/backend/app.db");
        let store = Arc::new(ProjectsStore::new(db_path, repo_root.clone()));
        store.initialize().expect("schema should initialize");
        store
            .upsert_project(UpsertProjectInput {
                name: String::from("Demo"),
                slug: Some(String::from("demo")),
                ..UpsertProjectInput::default()
            })
            .expect("project should be created");
        store
            .update_project_storage_s3(
                "demo",
                UpdateStorageS3Input {
                    enabled: Some(true),
                    bucket: Some(String::from("bucket")),
                    prefix: Some(String::from("iat-projects")),
                    ..UpdateStorageS3Input::default()
                },
            )
            .expect("s3 storage should be configured");

        let runner = FakeRunner::default();
        let hybrid = NativeIngestScriptSyncBackendOps::new(store, test_ops(runner.clone()));

        let result = hybrid
            .sync_project_s3(&BackendSyncProjectS3Request {
                project_slug: String::from("demo"),
                dry_run: false,
                delete: false,
                allow_missing_local: true,
            })
            .expect("missing local root should return skipped json");

        let parsed: BackendSyncProjectS3Response = result
            .parse_json_as()
            .expect("skipped payload should parse");
        assert_eq!(parsed.skipped, Some(true));
        assert_eq!(parsed.reason.as_deref(), Some("missing_local_project_root"));
        assert!(runner.take_seen().is_empty());

        let _ = fs::remove_dir_all(repo_root);
    }

    #[test]
    fn hybrid_sync_precheck_rejects_disabled_s3_without_calling_script() {
        let repo_root = temp_repo_root();
        let db_path = repo_root.join("var/backend/app.db");
        let store = Arc::new(ProjectsStore::new(db_path, repo_root.clone()));
        store.initialize().expect("schema should initialize");
        store
            .upsert_project(UpsertProjectInput {
                name: String::from("Demo"),
                slug: Some(String::from("demo")),
                ..UpsertProjectInput::default()
            })
            .expect("project should be created");

        let runner = FakeRunner::default();
        let hybrid = NativeIngestScriptSyncBackendOps::new(store, test_ops(runner.clone()));

        let err = hybrid
            .sync_project_s3(&BackendSyncProjectS3Request {
                project_slug: String::from("demo"),
                dry_run: false,
                delete: false,
                allow_missing_local: true,
            })
            .expect_err("disabled s3 should fail in rust precheck");

        match err {
            BackendOpsError::SyncPrecheck(message) => {
                assert!(message.contains("S3 storage is disabled"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(runner.take_seen().is_empty());

        let _ = fs::remove_dir_all(repo_root);
    }

    #[test]
    fn hybrid_sync_executes_aws_cli_when_ready() {
        let repo_root = temp_repo_root();
        let db_path = repo_root.join("var/backend/app.db");
        let store = Arc::new(ProjectsStore::new(db_path, repo_root.clone()));
        store.initialize().expect("schema should initialize");
        store
            .upsert_project(UpsertProjectInput {
                name: String::from("Demo"),
                slug: Some(String::from("demo")),
                ..UpsertProjectInput::default()
            })
            .expect("project should be created");
        store
            .update_project_storage_s3(
                "demo",
                UpdateStorageS3Input {
                    enabled: Some(true),
                    bucket: Some(String::from("bucket")),
                    prefix: Some(String::from("iat-projects")),
                    region: Some(String::from("us-east-1")),
                    profile: Some(String::from("test-profile")),
                    endpoint_url: Some(String::from("http://localhost:9000")),
                },
            )
            .expect("s3 storage should be configured");

        fs::create_dir_all(repo_root.join("var/projects/demo"))
            .expect("local project root should exist for sync");

        let runner = FakeRunner::with_next(Ok(CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }));
        let hybrid = NativeIngestScriptSyncBackendOps::new(store, test_ops(runner.clone()));

        let result = hybrid
            .sync_project_s3(&BackendSyncProjectS3Request {
                project_slug: String::from("demo"),
                dry_run: true,
                delete: true,
                allow_missing_local: false,
            })
            .expect("ready sync should run aws command");

        let parsed: BackendSyncProjectS3Response =
            result.parse_json_as().expect("sync response should parse");
        assert_eq!(parsed.project_slug.as_deref(), Some("demo"));
        assert_eq!(parsed.dry_run, Some(true));
        assert_eq!(parsed.delete, Some(true));

        let seen = runner.take_seen();
        assert_eq!(seen.len(), 1);
        let cmd = &seen[0];
        assert_eq!(cmd.program, "aws");
        assert!(cmd.args.iter().any(|arg| arg == "s3"));
        assert!(cmd.args.iter().any(|arg| arg == "sync"));
        assert!(cmd.args.iter().any(|arg| arg == "--dryrun"));
        assert!(cmd.args.iter().any(|arg| arg == "--delete"));
        assert!(cmd.args.iter().any(|arg| arg == "--region"));
        assert!(cmd.args.iter().any(|arg| arg == "us-east-1"));
        assert!(cmd.args.iter().any(|arg| arg == "--profile"));
        assert!(cmd.args.iter().any(|arg| arg == "test-profile"));
        assert!(cmd.args.iter().any(|arg| arg == "--endpoint-url"));
        assert!(cmd.args.iter().any(|arg| arg == "http://localhost:9000"));

        let _ = fs::remove_dir_all(repo_root);
    }

    #[test]
    fn hybrid_sync_precheck_rejects_file_project_root_without_calling_aws() {
        let repo_root = temp_repo_root();
        let db_path = repo_root.join("var/backend/app.db");
        let store = Arc::new(ProjectsStore::new(db_path, repo_root.clone()));
        store.initialize().expect("schema should initialize");
        store
            .upsert_project(UpsertProjectInput {
                name: String::from("Demo"),
                slug: Some(String::from("demo")),
                ..UpsertProjectInput::default()
            })
            .expect("project should be created");
        store
            .update_project_storage_s3(
                "demo",
                UpdateStorageS3Input {
                    enabled: Some(true),
                    bucket: Some(String::from("bucket")),
                    ..UpdateStorageS3Input::default()
                },
            )
            .expect("s3 storage should be configured");

        let project_root_file = repo_root.join("var/projects/demo");
        fs::create_dir_all(project_root_file.parent().expect("project root parent"))
            .expect("parent dir should exist");
        fs::write(project_root_file.as_path(), b"not-a-dir")
            .expect("file project root should exist");

        let runner = FakeRunner::default();
        let hybrid = NativeIngestScriptSyncBackendOps::new(store, test_ops(runner.clone()));

        let err = hybrid
            .sync_project_s3(&BackendSyncProjectS3Request {
                project_slug: String::from("demo"),
                dry_run: false,
                delete: false,
                allow_missing_local: false,
            })
            .expect_err("file project root should fail precheck");

        match err {
            BackendOpsError::SyncPrecheck(message) => {
                assert!(message.contains("not a directory"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(runner.take_seen().is_empty());

        let _ = fs::remove_dir_all(repo_root);
    }
}
