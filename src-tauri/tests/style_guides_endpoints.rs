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
async fn style_guides_support_crud() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Style Guides"}"#),
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
        &format!("/api/projects/{slug}/style-guides"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_before["count"], json!(0));

    let created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/style-guides"),
        Body::from(
            json!({
                "name": "Painterly",
                "instructions": "Use painterly brush texture, soft gradients, cinematic framing.",
                "notes": "Primary style for covers"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(created["style_guide"]["name"], json!("Painterly"));
    let style_guide_id = created["style_guide"]["id"]
        .as_str()
        .expect("style guide id should exist")
        .to_string();

    let detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/style-guides/{style_guide_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        detail["style_guide"]["notes"],
        json!("Primary style for covers")
    );

    let updated = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/style-guides/{style_guide_id}"),
        Body::from(
            json!({
                "name": "Painterly V2",
                "instructions": "Use painterly texture with stronger contrast.",
                "notes": ""
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["style_guide"]["name"], json!("Painterly V2"));
    assert_eq!(updated["style_guide"]["notes"], json!(""));

    let deleted = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/style-guides/{style_guide_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(deleted["ok"], json!(true));
    assert_eq!(deleted["deleted"], json!(true));

    let missing = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/style-guides/{style_guide_id}"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing["error"], json!("Style guide not found"));
    assert_eq!(missing["error_kind"], json!("validation"));
    assert_eq!(missing["error_code"], json!("not_found"));
}

#[tokio::test]
async fn style_guide_validation_is_enforced() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Style Guide Validation"}"#),
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
        &format!("/api/projects/{slug}/style-guides"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(missing_name["error"], json!("Field 'name' is required"));
    assert_eq!(missing_name["error_kind"], json!("validation"));
    assert_eq!(missing_name["error_code"], json!("validation_error"));

    let missing_instructions = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/style-guides"),
        Body::from(json!({"name":"Only Name","instructions":"  "}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_instructions["error"],
        json!("Field 'instructions' is required")
    );
    assert_eq!(missing_instructions["error_kind"], json!("validation"));
    assert_eq!(
        missing_instructions["error_code"],
        json!("validation_error")
    );

    let _created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/style-guides"),
        Body::from(json!({"name":"Duplicate","instructions":"One"}).to_string()),
        StatusCode::OK,
    )
    .await;

    let duplicate_name = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/style-guides"),
        Body::from(json!({"name":"Duplicate","instructions":"Two"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        duplicate_name["error"],
        json!("Style guide name already exists")
    );
    assert_eq!(duplicate_name["error_kind"], json!("validation"));
    assert_eq!(duplicate_name["error_code"], json!("validation_error"));

    let empty_update = send_json(
        app,
        Method::PUT,
        &format!("/api/projects/{slug}/style-guides/missing"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        empty_update["error"],
        json!("Provide at least one of: name, instructions, notes")
    );
    assert_eq!(empty_update["error_kind"], json!("validation"));
    assert_eq!(empty_update["error_code"], json!("validation_error"));
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
    let root = std::env::temp_dir().join(format!("kroma_style_guides_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
