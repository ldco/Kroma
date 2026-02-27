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
async fn provider_accounts_support_lifecycle() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Provider Accounts"}"#),
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
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_before["count"], json!(0));

    let created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::from(
            json!({
                "provider_code": "OpenAI",
                "display_name": "OpenAI Main",
                "account_ref": "acct-main",
                "base_url": "https://api.openai.com/v1",
                "enabled": true,
                "config_json": {"tier": "prod"}
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        created["provider_account"]["provider_code"],
        json!("openai")
    );
    assert_eq!(created["provider_account"]["enabled"], json!(true));

    let list_after = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_after["count"], json!(1));

    let detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/provider-accounts/openai"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        detail["provider_account"]["display_name"],
        json!("OpenAI Main")
    );

    let updated = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/provider-accounts/openai"),
        Body::from(
            json!({
                "display_name": "OpenAI Backup",
                "enabled": false,
                "config_json": {"tier": "backup"}
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        updated["provider_account"]["display_name"],
        json!("OpenAI Backup")
    );
    assert_eq!(updated["provider_account"]["enabled"], json!(false));
    assert_eq!(
        updated["provider_account"]["config_json"]["tier"],
        json!("backup")
    );

    let deleted = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/provider-accounts/openai"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(deleted["ok"], json!(true));
    assert_eq!(deleted["deleted"], json!(true));

    let missing_after_delete = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/provider-accounts/openai"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(
        missing_after_delete["error"],
        json!("Provider account not found")
    );
    assert_eq!(missing_after_delete["error_kind"], json!("validation"));
    assert_eq!(missing_after_delete["error_code"], json!("not_found"));
}

#[tokio::test]
async fn provider_account_validation_is_enforced() {
    let app = build_router_with_projects_store_dev_bypass(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Provider Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let missing_provider_code = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_provider_code["error"],
        json!("Field 'provider_code' is required")
    );
    assert_eq!(missing_provider_code["error_kind"], json!("validation"));
    assert_eq!(
        missing_provider_code["error_code"],
        json!("validation_error")
    );

    let _created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::from(json!({"provider_code":"openai"}).to_string()),
        StatusCode::OK,
    )
    .await;

    let empty_update = send_json(
        app,
        Method::PUT,
        &format!("/api/projects/{slug}/provider-accounts/openai"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        empty_update["error"],
        json!("Provide at least one of: display_name, account_ref, base_url, enabled, config_json")
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
    let root = std::env::temp_dir().join(format!("kroma_provider_accounts_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
