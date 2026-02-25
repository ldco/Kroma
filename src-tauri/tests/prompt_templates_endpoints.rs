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
async fn prompt_template_endpoints_support_crud() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Prompt Demo"}"#),
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
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_before["count"], json!(0));

    let created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::from(
            json!({
                "name": "Cover Prompt",
                "template_text": "Create a cinematic cover image for {title}."
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(created["ok"], json!(true));
    assert_eq!(created["prompt_template"]["name"], json!("Cover Prompt"));
    let template_id = created["prompt_template"]["id"]
        .as_str()
        .expect("template id should exist")
        .to_string();

    let list_after = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_after["count"], json!(1));

    let detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/prompt-templates/{template_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail["prompt_template"]["id"], json!(template_id));

    let updated = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/prompt-templates/{template_id}"),
        Body::from(
            json!({
                "name": "Cover Prompt V2",
                "template_text": "Create a vivid cinematic cover image for {title}."
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["prompt_template"]["name"], json!("Cover Prompt V2"));

    let deleted = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/prompt-templates/{template_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(deleted["ok"], json!(true));
    assert_eq!(deleted["deleted"], json!(true));

    let missing_after_delete = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/prompt-templates/{template_id}"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(
        missing_after_delete["error"],
        json!("Prompt template not found")
    );
}

#[tokio::test]
async fn prompt_template_validation_is_enforced() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Prompt Validation"}"#),
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
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(missing_name["error"], json!("Field 'name' is required"));

    let missing_template_text = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::from(json!({"name":"Only Name","template_text":"   "}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_template_text["error"],
        json!("Field 'template_text' is required")
    );

    let _created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::from(json!({"name":"Duplicate","template_text":"One"}).to_string()),
        StatusCode::OK,
    )
    .await;

    let duplicate_name = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::from(json!({"name":"Duplicate","template_text":"Two"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        duplicate_name["error"],
        json!("Prompt template name already exists")
    );

    let empty_update = send_json(
        app,
        Method::PUT,
        &format!("/api/projects/{slug}/prompt-templates/missing"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        empty_update["error"],
        json!("Provide at least one of: name, template_text")
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
    let root = std::env::temp_dir().join(format!("kroma_prompt_templates_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
