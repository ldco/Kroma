use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::server::build_router_with_projects_store_dev_bypass;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn asset_links_support_crud_and_filters() {
    let (store, db_path) = test_store_with_db_path();
    let app = build_router_with_projects_store_dev_bypass(store);

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Asset Links Demo"}"#),
        StatusCode::OK,
    )
    .await;
    let project_id = create_project["project"]["id"]
        .as_str()
        .expect("project id should exist")
        .to_string();
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let (asset_a, asset_b, asset_c) = seed_assets(db_path.as_path(), project_id.as_str());

    let created = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/asset-links"),
        Body::from(
            json!({
                "parent_asset_id": asset_a,
                "child_asset_id": asset_b,
                "link_type": "derived_from"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(created["ok"], json!(true));
    assert_eq!(created["asset_link"]["parent_asset_id"], json!(asset_a));
    assert_eq!(created["asset_link"]["child_asset_id"], json!(asset_b));
    assert_eq!(created["asset_link"]["link_type"], json!("derived_from"));
    let link_id = created["asset_link"]["id"]
        .as_str()
        .expect("link id should exist")
        .to_string();

    let list_all = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/asset-links"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_all["ok"], json!(true));
    assert_eq!(list_all["count"], json!(1));

    let list_filtered_asset = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/asset-links?asset_id={asset_b}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_filtered_asset["count"], json!(1));

    let list_filtered_type = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/asset-links?link_type=derived_from"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list_filtered_type["count"], json!(1));

    let detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/asset-links/{link_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail["asset_link"]["id"], json!(link_id));

    let updated = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/asset-links/{link_id}"),
        Body::from(
            json!({
                "child_asset_id": asset_c,
                "link_type": "variant_of"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["asset_link"]["child_asset_id"], json!(asset_c));
    assert_eq!(updated["asset_link"]["link_type"], json!("variant_of"));

    let deleted = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/asset-links/{link_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(deleted["ok"], json!(true));
    assert_eq!(deleted["deleted"], json!(true));

    let missing_after_delete = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/asset-links/{link_id}"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_after_delete["ok"], json!(false));
    assert_eq!(missing_after_delete["error"], json!("Asset link not found"));
    assert_eq!(missing_after_delete["error_kind"], json!("validation"));
    assert_eq!(missing_after_delete["error_code"], json!("not_found"));
}

#[tokio::test]
async fn asset_link_validation_returns_bad_request() {
    let (store, db_path) = test_store_with_db_path();
    let app = build_router_with_projects_store_dev_bypass(store);

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Asset Link Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let project_id = create_project["project"]["id"]
        .as_str()
        .expect("project id should exist")
        .to_string();
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let (asset_a, _, _) = seed_assets(db_path.as_path(), project_id.as_str());

    let same_parent_child = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/asset-links"),
        Body::from(
            json!({
                "parent_asset_id": asset_a,
                "child_asset_id": asset_a
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(same_parent_child["ok"], json!(false));
    assert_eq!(
        same_parent_child["error"],
        json!("parent_asset_id and child_asset_id must differ")
    );
    assert_eq!(same_parent_child["error_kind"], json!("validation"));
    assert_eq!(same_parent_child["error_code"], json!("validation_error"));

    let invalid_link_type = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/asset-links"),
        Body::from(
            json!({
                "parent_asset_id": asset_a,
                "child_asset_id": "asset_missing",
                "link_type": "invalid_type"
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(invalid_link_type["ok"], json!(false));
    assert_eq!(
        invalid_link_type["error"],
        json!("Field 'link_type' must be one of: derived_from, variant_of, mask_for, reference_of")
    );
    assert_eq!(invalid_link_type["error_kind"], json!("validation"));
    assert_eq!(invalid_link_type["error_code"], json!("validation_error"));

    let empty_update = send_json(
        app,
        Method::PUT,
        &format!("/api/projects/{slug}/asset-links/missing"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(empty_update["ok"], json!(false));
    assert_eq!(
        empty_update["error"],
        json!("Provide at least one of: parent_asset_id, child_asset_id, link_type")
    );
    assert_eq!(empty_update["error_kind"], json!("validation"));
    assert_eq!(empty_update["error_code"], json!("validation_error"));
}

fn seed_assets(db_path: &Path, project_id: &str) -> (String, String, String) {
    let asset_a = format!("asset_{}", Uuid::new_v4().simple());
    let asset_b = format!("asset_{}", Uuid::new_v4().simple());
    let asset_c = format!("asset_{}", Uuid::new_v4().simple());

    let conn = Connection::open(db_path).expect("sqlite connection should open");

    for (asset_id, rel_path) in [
        (asset_a.as_str(), "assets/a.png"),
        (asset_b.as_str(), "assets/b.png"),
        (asset_c.as_str(), "assets/c.png"),
    ] {
        conn.execute(
            "
            INSERT INTO assets (
                id, project_id, kind, asset_kind, storage_uri, rel_path,
                storage_backend, mime_type, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            params![
                asset_id,
                project_id,
                "image",
                "render",
                format!("file:///tmp/kroma/{rel_path}"),
                rel_path,
                "local",
                "image/png",
                "2026-02-21T00:00:00Z"
            ],
        )
        .expect("asset should insert");
    }

    (asset_a, asset_b, asset_c)
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
    let root = std::env::temp_dir().join(format!("kroma_asset_links_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db.clone(), root));
    store.initialize().expect("store should initialize");
    (store, db)
}
