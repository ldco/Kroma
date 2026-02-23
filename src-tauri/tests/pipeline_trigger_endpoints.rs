use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::{
    build_router_with_projects_store, build_router_with_projects_store_and_pipeline_trigger,
};
use kroma_backend_core::db::projects::{ProjectsStore, UpdateStorageLocalInput};
use kroma_backend_core::pipeline::runtime::{
    PipelineInputSource, PipelineOrchestrator, PipelineRunRequest, PipelineRunResult,
    PipelineRuntimeError, PipelineStageFilter, PipelineTimeFilter, PipelineWeatherFilter,
};
use kroma_backend_core::pipeline::trigger::PipelineTriggerService;

#[tokio::test]
async fn pipeline_trigger_validates_mode_before_execution() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(json!({"mode":"fast"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Field 'mode' must be one of: dry, run")
    );
}

#[tokio::test]
async fn pipeline_trigger_returns_not_found_for_missing_project() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(json!({"mode":"dry","scene_refs":["a.png"]}).to_string()),
        StatusCode::NOT_FOUND,
    )
    .await;

    assert_eq!(response["error"], json!("Project not found"));
}

#[tokio::test]
async fn pipeline_validate_config_returns_not_found_for_missing_project() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/validate-config",
        Body::from(json!({"project_root":"var/projects/any"}).to_string()),
        StatusCode::NOT_FOUND,
    )
    .await;

    assert_eq!(response["error"], json!("Project not found"));
}

#[tokio::test]
async fn pipeline_trigger_run_mode_requires_spend_confirmation() {
    let app = build_router_with_projects_store(test_store());

    let created = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Pipeline Trigger"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = created["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let response = send_json(
        app,
        Method::POST,
        &format!("/api/projects/{slug}/runs/trigger"),
        Body::from(json!({"mode":"run","scene_refs":["a.png"]}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Run mode requires explicit spend confirmation")
    );
}

#[tokio::test]
async fn pipeline_validate_config_uses_project_storage_root_when_missing() {
    let store = test_store();
    let app = build_router_with_projects_store(store.clone());

    let created = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Validate Config Root"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = created["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let temp_root = temp_test_dir("kroma_validate_config_endpoint");
    let project_root = temp_root.join("project-root");
    fs::create_dir_all(project_root.join(".kroma")).expect("project settings dir");
    let manifest_path = temp_root.join("manifest.json");
    let post_path = temp_root.join("post.json");
    fs::write(
        &manifest_path,
        r#"{"scene_refs":["a.png"],"prompts":{"style_base":"ok"}}"#,
    )
    .expect("manifest write");
    fs::write(&post_path, r#"{}"#).expect("postprocess write");
    fs::write(
        project_root.join(".kroma/pipeline.settings.json"),
        json!({
            "pipeline": {
                "manifest_path": manifest_path.to_string_lossy(),
                "postprocess_config_path": post_path.to_string_lossy()
            }
        })
        .to_string(),
    )
    .expect("project settings write");

    store
        .update_project_storage_local(
            slug.as_str(),
            UpdateStorageLocalInput {
                project_root: Some(project_root.to_string_lossy().to_string()),
                ..UpdateStorageLocalInput::default()
            },
        )
        .expect("project storage local root should update");

    let response = send_json(
        app,
        Method::POST,
        &format!("/api/projects/{slug}/runs/validate-config"),
        Body::from(
            json!({
                "app_settings_path": temp_root.join("missing-app-settings.toml").to_string_lossy()
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["summary"]["project_settings_loaded"], json!(true));
    assert_eq!(
        response["summary"]["resolved_manifest_path"],
        json!(manifest_path.to_string_lossy().to_string())
    );
    assert_eq!(
        response["summary"]["resolved_postprocess_config_path"],
        json!(post_path.to_string_lossy().to_string())
    );

    let _ = fs::remove_dir_all(temp_root);
}

#[tokio::test]
async fn pipeline_validate_config_returns_bad_request_for_invalid_manifest_override() {
    let app = build_router_with_projects_store(test_store());
    let created = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Validate Config Invalid"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = created["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let temp_root = temp_test_dir("kroma_validate_config_invalid");
    let missing_manifest = temp_root.join("missing-manifest.json");
    let post_path = temp_root.join("post.json");
    fs::write(&post_path, r#"{}"#).expect("postprocess write");

    let response = send_json(
        app,
        Method::POST,
        &format!("/api/projects/{slug}/runs/validate-config"),
        Body::from(
            json!({
                "project_root": temp_root.join("project-root").to_string_lossy(),
                "app_settings_path": temp_root.join("missing-app-settings.toml").to_string_lossy(),
                "manifest_path": missing_manifest.to_string_lossy(),
                "postprocess_config_path": post_path.to_string_lossy()
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(response["ok"], json!(false));
    let error = response["error"]
        .as_str()
        .expect("error should be string")
        .to_string();
    assert!(error.contains("planning manifest validation failed"));

    let _ = fs::remove_dir_all(temp_root);
}

#[tokio::test]
async fn pipeline_trigger_success_path_can_use_injected_fake_orchestrator() {
    let store = test_store();
    let fake = Arc::new(FakeOrchestrator::with_next(Ok(PipelineRunResult {
        status_code: 0,
        stdout: String::from("{\"preview\":true}"),
        stderr: String::new(),
    })));
    let pipeline_trigger = PipelineTriggerService::new(fake.clone());
    let app = build_router_with_projects_store_and_pipeline_trigger(store, pipeline_trigger);

    let created = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Pipeline Trigger Success"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = created["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let response = send_json(
        app,
        Method::POST,
        &format!("/api/projects/{slug}/runs/trigger"),
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":["a.png"]
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["pipeline_trigger"]["mode"], json!("dry"));
    assert_eq!(response["pipeline_trigger"]["status_code"], json!(0));
    assert_eq!(
        response["pipeline_trigger"]["adapter"],
        json!("script_fallback")
    );
    assert_eq!(
        response["pipeline_trigger"]["stdout"],
        json!("{\"preview\":true}")
    );

    let seen = fake.take_seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].project_slug, slug);
    assert_eq!(seen[0].mode.as_str(), "dry");
    assert!(!seen[0].confirm_spend);
    assert_eq!(
        seen[0].options.input_source,
        Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")]))
    );
}

#[tokio::test]
async fn pipeline_trigger_typed_fields_translate_to_cli_args() {
    let store = test_store();
    let fake = Arc::new(FakeOrchestrator::with_next(Ok(PipelineRunResult {
        status_code: 0,
        stdout: String::new(),
        stderr: String::new(),
    })));
    let pipeline_trigger = PipelineTriggerService::new(fake.clone());
    let app = build_router_with_projects_store_and_pipeline_trigger(store, pipeline_trigger);

    let created = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Typed Trigger"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = created["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let response = send_json(
        app,
        Method::POST,
        &format!("/api/projects/{slug}/runs/trigger"),
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":["scene_a.png","scene_b.png"],
                "style_refs":["style_1.png"],
                "stage":"weather",
                "time":"day",
                "weather":"clear",
                "candidates":2
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    assert_eq!(response["ok"], json!(true));
    let seen = fake.take_seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].options.input_source,
        Some(PipelineInputSource::SceneRefs(vec![
            String::from("scene_a.png"),
            String::from("scene_b.png"),
        ]))
    );
    assert_eq!(
        seen[0].options.style_refs,
        vec![String::from("style_1.png")]
    );
    assert_eq!(seen[0].options.stage, Some(PipelineStageFilter::Weather));
    assert_eq!(seen[0].options.time, Some(PipelineTimeFilter::Day));
    assert_eq!(seen[0].options.weather, Some(PipelineWeatherFilter::Clear));
    assert_eq!(seen[0].options.candidates, Some(2));
}

#[tokio::test]
async fn pipeline_trigger_injects_project_root_from_rust_storage_when_missing() {
    let store = test_store();
    let fake = Arc::new(FakeOrchestrator::with_next(Ok(PipelineRunResult {
        status_code: 0,
        stdout: String::new(),
        stderr: String::new(),
    })));
    let pipeline_trigger = PipelineTriggerService::new(fake.clone());
    let app =
        build_router_with_projects_store_and_pipeline_trigger(store.clone(), pipeline_trigger);

    let created = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Trigger Root Injection"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = created["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    store
        .update_project_storage_local(
            slug.as_str(),
            UpdateStorageLocalInput {
                project_root: Some(String::from("var/projects/custom-demo")),
                ..UpdateStorageLocalInput::default()
            },
        )
        .expect("project storage local root should update");

    let response = send_json(
        app,
        Method::POST,
        &format!("/api/projects/{slug}/runs/trigger"),
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":["scene_a.png"]
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    assert_eq!(response["ok"], json!(true));
    let seen = fake.take_seen();
    assert_eq!(seen.len(), 1);
    let injected_root = seen[0]
        .options
        .project_root
        .as_deref()
        .expect("project root should be injected");
    assert!(injected_root.ends_with("/var/projects/custom-demo"));
}

#[tokio::test]
async fn pipeline_trigger_typed_fields_validate_before_execution() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":["a.png", ""]
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Field 'scene_refs' must not contain empty values")
    );
}

#[tokio::test]
async fn pipeline_trigger_rejects_empty_scene_refs_array() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":[]
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Field 'scene_refs' must not be empty")
    );
}

#[tokio::test]
async fn pipeline_trigger_rejects_empty_style_refs_array() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":["a.png"],
                "style_refs":[]
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Field 'style_refs' must not be empty")
    );
}

#[tokio::test]
async fn pipeline_trigger_requires_one_input_source_before_execution() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(json!({"mode":"dry"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Provide one of: input, scene_refs")
    );
}

#[tokio::test]
async fn pipeline_trigger_rejects_conflicting_typed_input_fields() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(
            json!({
                "mode":"dry",
                "input":"scene_dir",
                "scene_refs":["a.png"]
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Provide only one of: input, scene_refs")
    );
}

#[tokio::test]
async fn pipeline_trigger_rejects_out_of_range_candidates() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":["a.png"],
                "candidates": 9
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Field 'candidates' must be between 1 and 6")
    );
}

#[tokio::test]
async fn pipeline_trigger_rejects_time_without_time_or_weather_stage() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":["a.png"],
                "time":"night"
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Field 'time' requires stage 'time' or 'weather'")
    );
}

#[tokio::test]
async fn pipeline_trigger_rejects_weather_without_weather_stage() {
    let app = build_router_with_projects_store(test_store());

    let response = send_json(
        app,
        Method::POST,
        "/api/projects/missing-project/runs/trigger",
        Body::from(
            json!({
                "mode":"dry",
                "scene_refs":["a.png"],
                "stage":"time",
                "weather":"rain"
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(
        response["error"],
        json!("Field 'weather' requires stage 'weather'")
    );
}

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    body: Body,
    expected_status: StatusCode,
) -> Value {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(body)
        .expect("request should build");

    let response = app
        .oneshot(request)
        .await
        .expect("router should return response");
    assert_eq!(response.status(), expected_status);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    serde_json::from_slice(body.as_ref()).expect("response should be valid JSON")
}

fn test_store() -> Arc<ProjectsStore> {
    let suffix = Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("kroma_pipeline_trigger_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}_{stamp}_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("temp dir should be creatable");
    dir
}

#[derive(Default)]
struct FakeOrchestrator {
    seen: Mutex<Vec<PipelineRunRequest>>,
    next: Mutex<Option<Result<PipelineRunResult, PipelineRuntimeError>>>,
}

impl FakeOrchestrator {
    fn with_next(result: Result<PipelineRunResult, PipelineRuntimeError>) -> Self {
        Self {
            seen: Mutex::new(Vec::new()),
            next: Mutex::new(Some(result)),
        }
    }

    fn take_seen(&self) -> Vec<PipelineRunRequest> {
        std::mem::take(&mut *self.seen.lock().expect("fake orchestrator mutex poisoned"))
    }
}

impl PipelineOrchestrator for FakeOrchestrator {
    fn execute(
        &self,
        request: &PipelineRunRequest,
    ) -> Result<PipelineRunResult, PipelineRuntimeError> {
        self.seen
            .lock()
            .expect("fake orchestrator mutex poisoned")
            .push(request.clone());
        self.next
            .lock()
            .expect("fake orchestrator mutex poisoned")
            .take()
            .unwrap_or_else(|| {
                Ok(PipelineRunResult {
                    status_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                })
            })
    }
}
