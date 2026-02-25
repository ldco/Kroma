use std::path::PathBuf;
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use rusqlite::params;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store_dev_bypass;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn secrets_support_upsert_list_and_delete() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Secrets Project"}"#),
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
        &format!("/api/projects/{slug}/secrets"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_before["count"], json!(0));

    let created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/secrets"),
        Body::from(
            json!({
                "provider_code": "OpenAI",
                "secret_name": "api_key",
                "secret_value": "sk-test-123"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(created["secret"]["provider_code"], json!("openai"));
    assert_eq!(created["secret"]["secret_name"], json!("api_key"));
    assert_eq!(created["secret"]["has_value"], json!(true));

    let listed = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/secrets"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(listed["count"], json!(1));
    assert_eq!(listed["secrets"][0]["provider_code"], json!("openai"));
    assert_eq!(listed["secrets"][0]["secret_name"], json!("api_key"));
    assert_eq!(listed["secrets"][0]["has_value"], json!(true));

    let updated = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/secrets"),
        Body::from(
            json!({
                "provider_code": "openai",
                "secret_name": "api_key",
                "secret_value": "sk-test-456"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["secret"]["provider_code"], json!("openai"));
    assert_eq!(updated["secret"]["secret_name"], json!("api_key"));
    assert_eq!(updated["secret"]["has_value"], json!(true));

    let deleted = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/secrets/openai/api_key"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(deleted["ok"], json!(true));
    assert_eq!(deleted["deleted"], json!(true));

    let list_after_delete = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/secrets"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_after_delete["count"], json!(0));
}

#[tokio::test]
async fn secrets_support_rotation_endpoint() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Secrets Rotate"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let _created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/secrets"),
        Body::from(
            json!({
                "provider_code": "openai",
                "secret_name": "api_key",
                "secret_value": "sk-test-rotate"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    let rotated = send_json(
        app,
        Method::POST,
        &format!("/api/projects/{slug}/secrets/rotate"),
        Body::from(
            json!({
                "force": true
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(rotated["ok"], json!(true));
    assert_eq!(rotated["rotation"]["scanned"], json!(1));
    assert_eq!(rotated["rotation"]["rotated"], json!(1));
    assert_eq!(rotated["rotation"]["skipped_empty"], json!(0));
}

#[tokio::test]
async fn secrets_rotation_status_reports_plaintext_and_key_refs() {
    let (store, db) = test_store_with_db_path();
    let app = build_router_with_projects_store_dev_bypass(store.clone());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Secrets Rotation Status"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let _created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/secrets"),
        Body::from(
            json!({
                "provider_code": "openai",
                "secret_name": "api_key",
                "secret_value": "sk-test-status"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    let conn = rusqlite::Connection::open(db.as_path()).expect("db should open");
    conn.execute(
        "
        UPDATE project_secrets
        SET secret_value = 'legacy-plaintext', key_ref = 'legacy-key'
        WHERE provider_code = 'openai' AND secret_name = 'api_key'
    ",
        params![],
    )
    .expect("legacy row update should succeed");

    let status = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/secrets/rotation-status"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(status["ok"], json!(true));
    assert_eq!(status["status"]["total"], json!(1));
    assert_eq!(status["status"]["encrypted"], json!(0));
    assert_eq!(status["status"]["plaintext"], json!(1));
    assert_eq!(status["status"]["empty"], json!(0));
    assert_eq!(
        status["status"]["key_refs"][0]["key_ref"],
        json!("legacy-key")
    );
    assert_eq!(status["status"]["key_refs"][0]["count"], json!(1));
}

#[tokio::test]
async fn secrets_validation_and_not_found_paths_are_enforced() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Secrets Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let missing_provider = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/secrets"),
        Body::from(
            json!({
                "secret_name": "api_key",
                "secret_value": "sk-test-123"
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_provider["error"],
        json!("Field 'provider_code' is required")
    );

    let missing_name = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/secrets"),
        Body::from(
            json!({
                "provider_code": "openai",
                "secret_value": "sk-test-123"
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_name["error"],
        json!("Field 'secret_name' is required")
    );

    let missing_value = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/secrets"),
        Body::from(
            json!({
                "provider_code": "openai",
                "secret_name": "api_key"
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_value["error"],
        json!("Field 'secret_value' is required")
    );

    let missing_secret = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/secrets/openai/api_key"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_secret["error"], json!("Secret not found"));

    let missing_project = send_json(
        app,
        Method::GET,
        "/api/projects/missing-project/secrets",
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_project["error"], json!("Project not found"));
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
    let (store, _) = test_store_with_db_path();
    store
}

fn test_store_with_db_path() -> (Arc<ProjectsStore>, PathBuf) {
    let suffix = Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("kroma_secrets_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db.clone(), PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    (store, db)
}
