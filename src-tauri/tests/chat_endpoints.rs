use std::path::PathBuf;
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn chat_sessions_and_messages_support_create_and_read() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Chat Project"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let sessions_before = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/chat/sessions"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(sessions_before["count"], json!(0));

    let created_session = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/chat/sessions"),
        Body::from(json!({"title":"Creative Draft"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let session_id = created_session["session"]["id"]
        .as_str()
        .expect("session id should exist")
        .to_string();

    let session_detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/chat/sessions/{session_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(session_detail["session"]["title"], json!("Creative Draft"));

    let messages_before = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/chat/sessions/{session_id}/messages"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(messages_before["count"], json!(0));

    let first_message = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/chat/sessions/{session_id}/messages"),
        Body::from(json!({"role":"user","content":"Draft a cover concept"}).to_string()),
        StatusCode::OK,
    )
    .await;
    assert_eq!(first_message["message"]["role"], json!("user"));

    let _second_message = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/chat/sessions/{session_id}/messages"),
        Body::from(
            json!({"role":"assistant","content":"Use dynamic composition and bold lighting."})
                .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    let messages_after = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/chat/sessions/{session_id}/messages"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(messages_after["count"], json!(2));
    assert_eq!(messages_after["messages"][0]["role"], json!("user"));
    assert_eq!(messages_after["messages"][1]["role"], json!("assistant"));
}

#[tokio::test]
async fn chat_validation_and_not_found_paths_are_enforced() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Chat Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let created_session = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/chat/sessions"),
        Body::from("{}"),
        StatusCode::OK,
    )
    .await;
    let session_id = created_session["session"]["id"]
        .as_str()
        .expect("session id should exist")
        .to_string();

    let missing_role = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/chat/sessions/{session_id}/messages"),
        Body::from(json!({"content":"Hello"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(missing_role["error"], json!("Field 'role' is required"));

    let missing_content = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/chat/sessions/{session_id}/messages"),
        Body::from(json!({"role":"user"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_content["error"],
        json!("Field 'content' is required")
    );

    let missing_session = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/chat/sessions/missing/messages"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_session["error"], json!("Chat session not found"));
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
    let root = std::env::temp_dir().join(format!("kroma_chat_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
