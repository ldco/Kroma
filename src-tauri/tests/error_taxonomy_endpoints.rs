use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store_dev_bypass;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn project_create_validation_error_has_taxonomy_fields() {
    let app = build_router_with_projects_store_dev_bypass(test_store());
    let response = send_json(
        app,
        Method::POST,
        "/api/projects",
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(response["ok"], json!(false));
    assert_eq!(response["error"], json!("Field 'name' is required"));
    assert_eq!(response["error_kind"], json!("validation"));
    assert_eq!(response["error_code"], json!("validation_error"));
}

#[tokio::test]
async fn not_found_errors_have_not_found_taxonomy_fields() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let runs_response = send_json(
        app.clone(),
        Method::GET,
        "/api/projects/missing/runs",
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(runs_response["ok"], json!(false));
    assert_eq!(runs_response["error"], json!("Project not found"));
    assert_eq!(runs_response["error_kind"], json!("validation"));
    assert_eq!(runs_response["error_code"], json!("not_found"));

    let exports_response = send_json(
        app,
        Method::GET,
        "/api/projects/missing/exports",
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(exports_response["ok"], json!(false));
    assert_eq!(exports_response["error"], json!("Project not found"));
    assert_eq!(exports_response["error_kind"], json!("validation"));
    assert_eq!(exports_response["error_code"], json!("not_found"));
}

#[tokio::test]
async fn trigger_spend_confirmation_error_has_policy_taxonomy_fields() {
    let app = build_router_with_projects_store_dev_bypass(test_store());
    let created = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Policy Taxonomy"}"#),
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
        format!("/api/projects/{slug}/runs/trigger").as_str(),
        Body::from(json!({"mode":"run","scene_refs":["a.png"]}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(response["ok"], json!(false));
    assert_eq!(
        response["error"],
        json!("Run mode requires explicit spend confirmation")
    );
    assert_eq!(response["error_kind"], json!("policy"));
    assert_eq!(response["error_code"], json!("spend_confirmation_required"));
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
    let root = std::env::temp_dir().join(format!("kroma_error_taxonomy_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, root));
    store.initialize().expect("store should initialize");
    store
}
