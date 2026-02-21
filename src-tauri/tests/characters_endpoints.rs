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
async fn characters_support_crud() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Characters"}"#),
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
        &format!("/api/projects/{slug}/characters"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_before["count"], json!(0));

    let created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/characters"),
        Body::from(
            json!({
                "name": "Ava",
                "description": "Main protagonist",
                "prompt_text": "Confident young pilot, anime style"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(created["character"]["name"], json!("Ava"));
    let character_id = created["character"]["id"]
        .as_str()
        .expect("character id should exist")
        .to_string();

    let detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/characters/{character_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        detail["character"]["description"],
        json!("Main protagonist")
    );

    let updated = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/characters/{character_id}"),
        Body::from(
            json!({
                "name": "Ava Prime",
                "description": "",
                "prompt_text": "Confident space pilot, cinematic lighting"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["character"]["name"], json!("Ava Prime"));
    assert_eq!(updated["character"]["description"], json!(""));

    let deleted = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/characters/{character_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(deleted["ok"], json!(true));

    let missing = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/characters/{character_id}"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing["error"], json!("Character not found"));
}

#[tokio::test]
async fn character_validation_is_enforced() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Character Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let missing_name = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/characters"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(missing_name["error"], json!("Field 'name' is required"));

    let _created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/characters"),
        Body::from(json!({"name":"Duplicate"}).to_string()),
        StatusCode::OK,
    )
    .await;

    let duplicate_name = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/characters"),
        Body::from(json!({"name":"Duplicate"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        duplicate_name["error"],
        json!("Character name already exists")
    );

    let empty_update = send_json(
        app,
        Method::PUT,
        &format!("/api/projects/{slug}/characters/missing"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        empty_update["error"],
        json!("Provide at least one of: name, description, prompt_text")
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
    let root = std::env::temp_dir().join(format!("kroma_characters_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
