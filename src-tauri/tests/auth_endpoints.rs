use std::path::PathBuf;
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store_auth_mode;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn bootstrap_first_token_allows_single_unauthenticated_creation() {
    let app = build_router_with_projects_store_auth_mode(test_store(), false, true);

    let first = send_json(app.clone(), Method::POST, "/auth/token", Body::from("{}")).await;
    assert_eq!(first.0, StatusCode::OK);
    let token = first.1["auth_token"]["token"]
        .as_str()
        .expect("first token response should include token")
        .to_string();

    let second_unauth = send_json(app.clone(), Method::POST, "/auth/token", Body::from("{}")).await;
    assert_eq!(second_unauth.0, StatusCode::UNAUTHORIZED);

    let third_with_auth = send_json_with_bearer(
        app,
        Method::POST,
        "/auth/token",
        Body::from("{}"),
        token.as_str(),
    )
    .await;
    assert_eq!(third_with_auth.0, StatusCode::OK);
}

#[tokio::test]
async fn bootstrap_first_token_can_be_disabled() {
    let app = build_router_with_projects_store_auth_mode(test_store(), false, false);
    let response = send_json(app, Method::POST, "/auth/token", Body::from("{}")).await;
    assert_eq!(response.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_projects_endpoint_requires_bearer_when_dev_bypass_is_off() {
    let app = build_router_with_projects_store_auth_mode(test_store(), false, true);

    let bootstrap = send_json(app.clone(), Method::POST, "/auth/token", Body::from("{}")).await;
    assert_eq!(bootstrap.0, StatusCode::OK);
    let token = bootstrap.1["auth_token"]["token"]
        .as_str()
        .expect("bootstrap response should include token")
        .to_string();

    let project_created = send_json_with_bearer(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Auth Protected Project","slug":"auth_protected_project"}"#),
        token.as_str(),
    )
    .await;
    assert_eq!(project_created.0, StatusCode::OK);

    let unauthenticated = send_json(
        app.clone(),
        Method::GET,
        "/api/projects/auth_protected_project",
        Body::empty(),
    )
    .await;
    assert_eq!(unauthenticated.0, StatusCode::UNAUTHORIZED);

    let authenticated = send_json_with_bearer(
        app,
        Method::GET,
        "/api/projects/auth_protected_project",
        Body::empty(),
        token.as_str(),
    )
    .await;
    assert_eq!(authenticated.0, StatusCode::OK);
}

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    body: Body,
) -> (StatusCode, Value) {
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
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let parsed = serde_json::from_slice(body.as_ref()).expect("response should be valid JSON");
    (status, parsed)
}

async fn send_json_with_bearer(
    app: axum::Router,
    method: Method,
    uri: &str,
    body: Body,
    bearer_token: &str,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {bearer_token}"))
        .body(body)
        .expect("request should build");

    let response = app
        .oneshot(request)
        .await
        .expect("router should return response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let parsed = serde_json::from_slice(body.as_ref()).expect("response should be valid JSON");
    (status, parsed)
}

fn test_store() -> Arc<ProjectsStore> {
    let suffix = Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("kroma_auth_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
