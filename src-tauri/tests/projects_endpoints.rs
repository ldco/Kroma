use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn project_endpoints_support_create_list_and_detail() {
    let app = build_router_with_projects_store(test_repo());

    let list_before = send_json(
        app.clone(),
        Method::GET,
        "/api/projects",
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_before["ok"], json!(true));
    assert_eq!(list_before["count"], json!(0));

    let create_payload = json!({
        "name": "Alpha Project",
        "description": "Primary project for parity tests",
    });
    let created = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(create_payload.to_string()),
        StatusCode::OK,
    )
    .await;
    assert_eq!(created["ok"], json!(true));
    assert_eq!(created["project"]["slug"], json!("alpha_project"));
    assert_eq!(created["project"]["name"], json!("Alpha Project"));

    let list_after = send_json(
        app.clone(),
        Method::GET,
        "/api/projects",
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_after["ok"], json!(true));
    assert_eq!(list_after["count"], json!(1));
    assert_eq!(list_after["projects"][0]["slug"], json!("alpha_project"));

    let detail = send_json(
        app,
        Method::GET,
        "/api/projects/alpha_project",
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail["ok"], json!(true));
    assert_eq!(detail["project"]["slug"], json!("alpha_project"));
    assert_eq!(
        detail["project"]["description"],
        json!("Primary project for parity tests")
    );
    assert_eq!(detail["counts"]["runs"], json!(0));
    assert_eq!(detail["counts"]["jobs"], json!(0));
    assert_eq!(detail["counts"]["assets"], json!(0));
}

#[tokio::test]
async fn project_create_requires_name() {
    let app = build_router_with_projects_store(test_repo());
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

fn test_repo() -> Arc<ProjectsStore> {
    let suffix = Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("kroma_project_api_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, root));
    store.initialize().expect("store should initialize");
    store
}
