use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store_dev_bypass;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn storage_endpoints_support_read_and_update() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Storage Demo"}"#),
        StatusCode::OK,
    )
    .await;
    assert_eq!(create["project"]["slug"], json!("storage_demo"));

    let read_default = send_json(
        app.clone(),
        Method::GET,
        "/api/projects/storage_demo/storage",
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(read_default["ok"], json!(true));
    assert_eq!(
        read_default["storage"]["local"]["base_dir"],
        json!("var/projects")
    );

    let update_local = send_json(
        app.clone(),
        Method::PUT,
        "/api/projects/storage_demo/storage/local",
        Body::from(r#"{"project_root":"/tmp/kroma-storage-demo"}"#),
        StatusCode::OK,
    )
    .await;
    assert_eq!(update_local["updated"], json!("local"));
    assert_eq!(
        update_local["storage"]["local"]["project_root"],
        json!("/tmp/kroma-storage-demo")
    );

    let update_s3 = send_json(
        app.clone(),
        Method::PUT,
        "/api/projects/storage_demo/storage/s3",
        Body::from(
            r#"{"enabled":true,"bucket":"kroma-assets","prefix":"prod","region":"us-east-1"}"#,
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(update_s3["updated"], json!("s3"));
    assert_eq!(update_s3["storage"]["s3"]["enabled"], json!(true));
    assert_eq!(update_s3["storage"]["s3"]["bucket"], json!("kroma-assets"));
    assert_eq!(update_s3["storage"]["s3"]["region"], json!("us-east-1"));

    let read_updated = send_json(
        app,
        Method::GET,
        "/api/projects/storage_demo/storage",
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        read_updated["storage"]["local"]["project_root"],
        json!("/tmp/kroma-storage-demo")
    );
    assert_eq!(read_updated["storage"]["s3"]["enabled"], json!(true));
}

#[tokio::test]
async fn storage_local_update_requires_payload_fields() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let _ = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Validation Demo"}"#),
        StatusCode::OK,
    )
    .await;

    let response = send_json(
        app,
        Method::PUT,
        "/api/projects/validation_demo/storage/local",
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;

    assert_eq!(response["ok"], json!(false));
    assert_eq!(
        response["error"],
        json!("Provide at least one of: base_dir, project_root")
    );
    assert_eq!(response["error_kind"], json!("validation"));
    assert_eq!(response["error_code"], json!("validation_error"));
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
    let root = std::env::temp_dir().join(format!("kroma_storage_api_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, root));
    store.initialize().expect("store should initialize");
    store
}
