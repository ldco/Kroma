use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn exports_endpoints_return_seeded_records() {
    let (store, db_path) = test_store_with_db_path();
    let app = build_router_with_projects_store(store);

    let create = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Exports Demo"}"#),
        StatusCode::OK,
    )
    .await;
    let project_id = create["project"]["id"]
        .as_str()
        .expect("project id should exist")
        .to_string();
    let slug = create["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let export_id = seed_exports_data(db_path.as_path(), project_id.as_str());

    let list = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/exports?limit=10"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list["ok"], json!(true));
    assert_eq!(list["count"], json!(2));
    assert_eq!(list["exports"][0]["status"], json!("completed"));

    let detail = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/exports/{export_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail["ok"], json!(true));
    assert_eq!(detail["export"]["id"], json!(export_id));
    assert_eq!(detail["export"]["export_format"], json!("zip"));
}

#[tokio::test]
async fn export_detail_returns_not_found_when_missing() {
    let (store, _) = test_store_with_db_path();
    let app = build_router_with_projects_store(store);

    let create = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Missing Export"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let missing_export = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/exports/missing"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_export["ok"], json!(false));
    assert_eq!(missing_export["error"], json!("Project export not found"));

    let missing_project = send_json(
        app,
        Method::GET,
        "/api/projects/missing/exports",
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_project["ok"], json!(false));
    assert_eq!(missing_project["error"], json!("Project not found"));
}

fn seed_exports_data(db_path: &Path, project_id: &str) -> String {
    let conn = Connection::open(db_path).expect("sqlite connection should open");

    let export_id_old = format!("exp_{}", Uuid::new_v4().simple());
    let export_id_new = format!("exp_{}", Uuid::new_v4().simple());

    conn.execute(
        "
        INSERT INTO project_exports (
            id, project_id, run_id, status, export_format, storage_uri,
            rel_path, file_size_bytes, checksum_sha256, manifest_json,
            created_at, completed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ",
        params![
            export_id_old,
            project_id,
            "run_old",
            "queued",
            "zip",
            "file:///tmp/kroma/old.zip",
            "exports/old.zip",
            100_i64,
            "checksum_old",
            r#"{"items":12}"#,
            "2026-02-21T00:00:00Z",
            ""
        ],
    )
    .expect("old export should insert");

    conn.execute(
        "
        INSERT INTO project_exports (
            id, project_id, run_id, status, export_format, storage_uri,
            rel_path, file_size_bytes, checksum_sha256, manifest_json,
            created_at, completed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ",
        params![
            export_id_new,
            project_id,
            "run_new",
            "completed",
            "zip",
            "file:///tmp/kroma/new.zip",
            "exports/new.zip",
            256_i64,
            "checksum_new",
            r#"{"items":18}"#,
            "2026-02-21T00:01:00Z",
            "2026-02-21T00:02:00Z"
        ],
    )
    .expect("new export should insert");

    export_id_new
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

fn test_store_with_db_path() -> (Arc<ProjectsStore>, PathBuf) {
    let suffix = Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("kroma_exports_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db.clone(), root));
    store.initialize().expect("store should initialize");
    (store, db)
}
