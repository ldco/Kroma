use std::path::PathBuf;

use thiserror::Error;

use crate::pipeline::backend_ops::{
    BackendCommandResult, BackendIngestRunRequest, BackendIngestRunResponse, BackendOpsError,
    BackendSyncProjectS3Request, BackendSyncProjectS3Response, SharedPipelineBackendOps,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRunIngestParams {
    pub run_log_path: PathBuf,
    pub project_slug: String,
    pub project_name: String,
    pub create_project_if_missing: bool,
    pub compute_hashes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRunSyncS3Params {
    pub project_slug: String,
    pub dry_run: bool,
    pub delete: bool,
    pub allow_missing_local: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRunFinalizeParams {
    pub ingest: PostRunIngestParams,
    pub sync_s3: Option<PostRunSyncS3Params>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRunFinalizeResult {
    pub ingest: BackendIngestRunResponse,
    pub sync_s3: Option<BackendSyncProjectS3Response>,
}

#[derive(Clone)]
pub struct PipelinePostRunService {
    backend_ops: SharedPipelineBackendOps,
}

impl PipelinePostRunService {
    pub fn new(backend_ops: SharedPipelineBackendOps) -> Self {
        Self { backend_ops }
    }

    pub fn ingest_run(
        &self,
        params: PostRunIngestParams,
    ) -> Result<BackendIngestRunResponse, PipelinePostRunError> {
        let result = self
            .backend_ops
            .ingest_run(&BackendIngestRunRequest {
                run_log_path: params.run_log_path,
                project_slug: params.project_slug,
                project_name: params.project_name,
                create_project_if_missing: params.create_project_if_missing,
                compute_hashes: params.compute_hashes,
            })
            .map_err(PipelinePostRunError::BackendOps)?;
        parse_typed(result)
    }

    pub fn sync_project_s3(
        &self,
        params: PostRunSyncS3Params,
    ) -> Result<BackendSyncProjectS3Response, PipelinePostRunError> {
        let result = self
            .backend_ops
            .sync_project_s3(&BackendSyncProjectS3Request {
                project_slug: params.project_slug,
                dry_run: params.dry_run,
                delete: params.delete,
                allow_missing_local: params.allow_missing_local,
            })
            .map_err(PipelinePostRunError::BackendOps)?;
        parse_typed(result)
    }

    pub fn finalize_run(
        &self,
        params: PostRunFinalizeParams,
    ) -> Result<PostRunFinalizeResult, PipelinePostRunError> {
        let ingest = self.ingest_run(params.ingest)?;
        let sync_s3 = match params.sync_s3 {
            Some(sync) => Some(self.sync_project_s3(sync)?),
            None => None,
        };
        Ok(PostRunFinalizeResult { ingest, sync_s3 })
    }
}

fn parse_typed<T>(result: BackendCommandResult) -> Result<T, PipelinePostRunError>
where
    T: serde::de::DeserializeOwned,
{
    result
        .parse_json_as()
        .map_err(PipelinePostRunError::BackendOps)
}

#[derive(Debug, Error)]
pub enum PipelinePostRunError {
    #[error(transparent)]
    BackendOps(#[from] BackendOpsError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::backend_ops::PipelineBackendOps;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct FakeBackendOps {
        seen_ingest: Mutex<Vec<BackendIngestRunRequest>>,
        seen_sync: Mutex<Vec<BackendSyncProjectS3Request>>,
        next_ingest: Mutex<Option<Result<BackendCommandResult, BackendOpsError>>>,
        next_sync: Mutex<Option<Result<BackendCommandResult, BackendOpsError>>>,
    }

    impl FakeBackendOps {
        fn with_ingest_json(payload: serde_json::Value) -> Self {
            Self {
                next_ingest: Mutex::new(Some(Ok(BackendCommandResult {
                    stdout: payload.to_string(),
                    stderr: String::new(),
                    json: Some(payload),
                }))),
                ..Self::default()
            }
        }
    }

    impl PipelineBackendOps for FakeBackendOps {
        fn ingest_run(
            &self,
            request: &BackendIngestRunRequest,
        ) -> Result<BackendCommandResult, BackendOpsError> {
            self.seen_ingest
                .lock()
                .expect("fake backend ops ingest mutex poisoned")
                .push(request.clone());
            self.next_ingest
                .lock()
                .expect("fake backend ops ingest result mutex poisoned")
                .take()
                .unwrap_or_else(|| {
                    Ok(BackendCommandResult {
                        stdout: String::from("{\"ok\":true}"),
                        stderr: String::new(),
                        json: Some(json!({"ok": true})),
                    })
                })
        }

        fn sync_project_s3(
            &self,
            request: &BackendSyncProjectS3Request,
        ) -> Result<BackendCommandResult, BackendOpsError> {
            self.seen_sync
                .lock()
                .expect("fake backend ops sync mutex poisoned")
                .push(request.clone());
            self.next_sync
                .lock()
                .expect("fake backend ops sync result mutex poisoned")
                .take()
                .unwrap_or_else(|| {
                    Ok(BackendCommandResult {
                        stdout: String::from(
                            "{\"ok\":true,\"project_slug\":\"demo\",\"project_root\":\"/tmp/demo\",\"destination\":\"s3://bucket/demo/\",\"dry_run\":true,\"delete\":false}",
                        ),
                        stderr: String::new(),
                        json: Some(json!({
                            "ok": true,
                            "project_slug": "demo",
                            "project_root": "/tmp/demo",
                            "destination": "s3://bucket/demo/",
                            "dry_run": true,
                            "delete": false
                        })),
                    })
                })
        }
    }

    #[test]
    fn ingest_run_parses_typed_response() {
        let backend_ops = Arc::new(FakeBackendOps::with_ingest_json(json!({
            "ok": true,
            "project_slug": "demo",
            "run_id": "run_1",
            "run_log_path": "var/projects/demo/runs/run_1.json",
            "jobs": 1,
            "candidates": 1,
            "assets_upserted": 1,
            "quality_reports_written": 1,
            "cost_events_written": 0,
            "status": "ok"
        })));
        let service = PipelinePostRunService::new(backend_ops.clone());

        let result = service
            .ingest_run(PostRunIngestParams {
                run_log_path: PathBuf::from("var/projects/demo/runs/run_1.json"),
                project_slug: String::from("demo"),
                project_name: String::from("Demo"),
                create_project_if_missing: true,
                compute_hashes: false,
            })
            .expect("ingest should parse");

        assert_eq!(result.project_slug, "demo");
        assert_eq!(result.jobs, 1);
        assert_eq!(
            backend_ops
                .seen_ingest
                .lock()
                .expect("fake backend ops ingest mutex poisoned")
                .len(),
            1
        );
    }

    #[test]
    fn finalize_run_executes_ingest_then_optional_sync() {
        let backend_ops = Arc::new(FakeBackendOps::with_ingest_json(json!({
            "ok": true,
            "project_slug": "demo",
            "run_id": "run_1",
            "run_log_path": "var/projects/demo/runs/run_1.json",
            "jobs": 2,
            "candidates": 2,
            "assets_upserted": 2,
            "quality_reports_written": 2,
            "cost_events_written": 1,
            "status": "ok"
        })));
        let service = PipelinePostRunService::new(backend_ops.clone());

        let result = service
            .finalize_run(PostRunFinalizeParams {
                ingest: PostRunIngestParams {
                    run_log_path: PathBuf::from("var/projects/demo/runs/run_1.json"),
                    project_slug: String::from("demo"),
                    project_name: String::from("Demo"),
                    create_project_if_missing: true,
                    compute_hashes: false,
                },
                sync_s3: Some(PostRunSyncS3Params {
                    project_slug: String::from("demo"),
                    dry_run: true,
                    delete: false,
                    allow_missing_local: true,
                }),
            })
            .expect("finalize should succeed");

        assert_eq!(result.ingest.run_id, "run_1");
        assert_eq!(
            result
                .sync_s3
                .expect("sync result should exist")
                .project_slug,
            Some(String::from("demo"))
        );
    }
}
