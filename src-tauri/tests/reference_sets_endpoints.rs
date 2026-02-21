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
async fn reference_sets_and_items_support_crud() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Reference Sets"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let created_set = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/reference-sets"),
        Body::from(
            json!({
                "name": "Hero Faces",
                "description": "Face references for main hero"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let reference_set_id = created_set["reference_set"]["id"]
        .as_str()
        .expect("reference set id should exist")
        .to_string();

    let set_list = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/reference-sets"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(set_list["count"], json!(1));

    let set_detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(set_detail["reference_set"]["name"], json!("Hero Faces"));

    let updated_set = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}"),
        Body::from(
            json!({
                "name": "Hero Faces V2",
                "description": "Updated set"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated_set["reference_set"]["name"], json!("Hero Faces V2"));

    let created_item = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items"),
        Body::from(
            json!({
                "label": "Ava Closeup",
                "content_text": "Short hair, determined expression, profile angle",
                "sort_order": 1,
                "metadata_json": {"source": "manual"}
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let item_id = created_item["item"]["id"]
        .as_str()
        .expect("item id should exist")
        .to_string();

    let item_list = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(item_list["count"], json!(1));

    let item_detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items/{item_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(item_detail["item"]["label"], json!("Ava Closeup"));

    let updated_item = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items/{item_id}"),
        Body::from(
            json!({
                "label": "Ava Closeup V2",
                "content_uri": "file:///tmp/refs/ava_closeup_v2.png",
                "sort_order": 2
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated_item["item"]["label"], json!("Ava Closeup V2"));
    assert_eq!(updated_item["item"]["sort_order"], json!(2));

    let delete_item = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items/{item_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(delete_item["deleted"], json!(true));

    let delete_set = send_json(
        app.clone(),
        Method::DELETE,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(delete_set["deleted"], json!(true));

    let missing_set = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_set["error"], json!("Reference set not found"));
}

#[tokio::test]
async fn reference_set_validation_is_enforced() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Reference Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let missing_set_name = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/reference-sets"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(missing_set_name["error"], json!("Field 'name' is required"));

    let created_set = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/reference-sets"),
        Body::from(json!({"name":"Dup Set"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let reference_set_id = created_set["reference_set"]["id"]
        .as_str()
        .expect("reference set id should exist")
        .to_string();

    let duplicate_set_name = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/reference-sets"),
        Body::from(json!({"name":"Dup Set"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        duplicate_set_name["error"],
        json!("Reference set name already exists")
    );

    let empty_set_update = send_json(
        app.clone(),
        Method::PUT,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        empty_set_update["error"],
        json!("Provide at least one of: name, description")
    );

    let missing_item_label = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_item_label["error"],
        json!("Field 'label' is required")
    );

    let missing_item_content = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items"),
        Body::from(json!({"label":"No Content"}).to_string()),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_item_content["error"],
        json!("Provide at least one of: content_uri, content_text")
    );

    let created_item = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items"),
        Body::from(json!({"label":"Valid","content_text":"abc"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let item_id = created_item["item"]["id"]
        .as_str()
        .expect("item id should exist")
        .to_string();

    let empty_item_update = send_json(
        app,
        Method::PUT,
        &format!("/api/projects/{slug}/reference-sets/{reference_set_id}/items/{item_id}"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        empty_item_update["error"],
        json!(
            "Provide at least one of: label, content_uri, content_text, sort_order, metadata_json"
        )
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
    let root = std::env::temp_dir().join(format!("kroma_reference_sets_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
