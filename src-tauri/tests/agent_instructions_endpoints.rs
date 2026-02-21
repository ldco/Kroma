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
async fn agent_instruction_lifecycle_is_supported() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Agent Instructions"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let list_before = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/agent/instructions"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_before["count"], json!(0));

    let created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/agent/instructions"),
        Body::from(
            json!({"instruction_text":"Lock style guide v2 and rerender covers"}).to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let instruction_id = created["instruction"]["id"]
        .as_str()
        .expect("instruction id should exist")
        .to_string();
    assert_eq!(created["instruction"]["status"], json!("pending"));

    let events_before = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/agent/instructions/{instruction_id}/events"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(events_before["count"], json!(1));
    assert_eq!(events_before["events"][0]["event_type"], json!("created"));

    let confirmed = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/agent/instructions/{instruction_id}/confirm"),
        Body::from(json!({"message":"Approved by operator"}).to_string()),
        StatusCode::OK,
    )
    .await;
    assert_eq!(confirmed["instruction"]["status"], json!("confirmed"));

    let detail_after = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/agent/instructions/{instruction_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail_after["instruction"]["status"], json!("confirmed"));

    let events_after = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/agent/instructions/{instruction_id}/events"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(events_after["count"], json!(2));
    assert_eq!(events_after["events"][1]["event_type"], json!("confirmed"));
}

#[tokio::test]
async fn agent_instruction_validation_and_not_found_paths_are_enforced() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Agent Instruction Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let missing_instruction_text = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/agent/instructions"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_instruction_text["error"],
        json!("Field 'instruction_text' is required")
    );

    let created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/agent/instructions"),
        Body::from(json!({"instruction_text":"Temporary instruction"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let instruction_id = created["instruction"]["id"]
        .as_str()
        .expect("instruction id should exist")
        .to_string();

    let canceled = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/agent/instructions/{instruction_id}/cancel"),
        Body::from(json!({"message":"User canceled"}).to_string()),
        StatusCode::OK,
    )
    .await;
    assert_eq!(canceled["instruction"]["status"], json!("canceled"));

    let confirm_after_cancel = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/agent/instructions/{instruction_id}/confirm"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        confirm_after_cancel["error"],
        json!("Instruction is already canceled")
    );

    let missing_instruction = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/agent/instructions/missing"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(
        missing_instruction["error"],
        json!("Agent instruction not found")
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
    let root = std::env::temp_dir().join(format!("kroma_agent_instructions_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
