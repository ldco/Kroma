use std::path::PathBuf;
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store_dev_bypass;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn contract_smoke_test_full_flow() {
    let app = build_router_with_projects_store_dev_bypass(test_store());
    let slug = "contract_demo";

    let _project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(json!({"name": "Contract Demo", "slug": slug}).to_string()),
        StatusCode::OK,
    )
    .await;

    let created_template = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::from(json!({"name": "default-shot", "template_text": "A cinematic shot of {subject}"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let template_id = created_template["prompt_template"]["id"]
        .as_str()
        .expect("template id should exist")
        .to_string();

    let _templates_list = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;

    let _template_detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/prompt-templates/{template_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;

    let _updated = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/prompt-templates/{template_id}"),
        Body::from(json!({"template_text": "A cinematic close-up of {subject}"}).to_string()),
        StatusCode::OK,
    )
    .await;

    let _deleted = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/prompt-templates/{template_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;

    let sess = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/chat/sessions"),
        Body::from(json!({"title": "Contract session"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let session_id = sess["session"]["id"]
        .as_str()
        .expect("session id should exist")
        .to_string();

    let _message = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/chat/sessions/{session_id}/messages"),
        Body::from(json!({"role": "user", "content": "Create instruction"}).to_string()),
        StatusCode::OK,
    )
    .await;

    let ins = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/agent/instructions"),
        Body::from(json!({"instruction_text": "Test instruction"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let instr_id = ins["instruction"]["id"]
        .as_str()
        .expect("instruction id should exist")
        .to_string();

    let _events = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/agent/instructions/{instr_id}/events"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;

    let _secret_created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/secrets"),
        Body::from(json!({"provider_code": "openai", "secret_name": "api_key", "secret_value": "sk-test-contract-123456"}).to_string()),
        StatusCode::OK,
    )
    .await;

    let secrets_list = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/secrets"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(secrets_list["count"].as_i64().expect("count should be integer"), 1);

    let _secret_deleted = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/secrets/openai/api_key"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;

    let exports = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/exports"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(exports["ok"], json!(true));

    println!("[contract-smoke] ok project={slug} template={template_id} session={session_id} instruction={instr_id}");
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
    assert_eq!(response.status(), expected_status, "Expected status {} but got {}", expected_status, response.status());
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    serde_json::from_slice(body.as_ref()).expect("response should be valid JSON")
}

fn test_store() -> Arc<ProjectsStore> {
    let suffix = Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("kroma_contract_smoke_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
