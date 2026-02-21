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
async fn analytics_endpoints_return_seeded_records() {
    let (store, db_path) = test_store_with_db_path();
    let app = build_router_with_projects_store(store);

    let create = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Analytics Demo"}"#),
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

    seed_analytics_data(db_path.as_path(), project_id.as_str());

    let quality_reports = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/quality-reports?limit=10"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(quality_reports["ok"], json!(true));
    assert_eq!(quality_reports["count"], json!(2));
    assert_eq!(quality_reports["quality_reports"][0]["grade"], json!("A"));

    let cost_events = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/cost-events?limit=10"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(cost_events["ok"], json!(true));
    assert_eq!(cost_events["count"], json!(2));
    assert_eq!(
        cost_events["cost_events"][0]["provider_code"],
        json!("openai")
    );
    assert_eq!(cost_events["cost_events"][0]["currency"], json!("USD"));
}

#[tokio::test]
async fn analytics_endpoints_return_not_found_for_unknown_project() {
    let (store, _) = test_store_with_db_path();
    let app = build_router_with_projects_store(store);

    let quality_not_found = send_json(
        app.clone(),
        Method::GET,
        "/api/projects/missing/quality-reports",
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(quality_not_found["ok"], json!(false));
    assert_eq!(quality_not_found["error"], json!("Project not found"));

    let cost_not_found = send_json(
        app,
        Method::GET,
        "/api/projects/missing/cost-events",
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(cost_not_found["ok"], json!(false));
    assert_eq!(cost_not_found["error"], json!("Project not found"));
}

fn seed_analytics_data(db_path: &Path, project_id: &str) {
    let conn = Connection::open(db_path).expect("sqlite connection should open");

    let report_id_a = format!("qr_{}", Uuid::new_v4().simple());
    let report_id_b = format!("qr_{}", Uuid::new_v4().simple());

    conn.execute(
        "
        INSERT INTO quality_reports (
            id, project_id, run_id, asset_id, report_type, grade,
            hard_failures, soft_warnings, avg_chroma_exceed,
            summary_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ",
        params![
            report_id_b,
            project_id,
            "run_old",
            "asset_old",
            "compliance",
            "B",
            1_i64,
            2_i64,
            0.12_f64,
            r#"{"threshold":"warn"}"#,
            "2026-02-21T00:00:00Z"
        ],
    )
    .expect("quality report should insert");

    conn.execute(
        "
        INSERT INTO quality_reports (
            id, project_id, run_id, asset_id, report_type, grade,
            hard_failures, soft_warnings, avg_chroma_exceed,
            summary_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ",
        params![
            report_id_a,
            project_id,
            "run_new",
            "asset_new",
            "compliance",
            "A",
            0_i64,
            1_i64,
            0.02_f64,
            r#"{"threshold":"pass"}"#,
            "2026-02-21T00:01:00Z"
        ],
    )
    .expect("quality report should insert");

    let cost_id_a = format!("ce_{}", Uuid::new_v4().simple());
    let cost_id_b = format!("ce_{}", Uuid::new_v4().simple());

    conn.execute(
        "
        INSERT INTO cost_events (
            id, project_id, run_id, job_id, provider_code, model_name,
            event_type, units, unit_cost_usd, total_cost_usd, currency,
            meta_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ",
        params![
            cost_id_b,
            project_id,
            "run_old",
            "job_old",
            "openai",
            "gpt-image-1",
            "generation",
            800.0_f64,
            0.00004_f64,
            0.032_f64,
            "USD",
            r#"{"batch":1}"#,
            "2026-02-21T00:00:30Z"
        ],
    )
    .expect("cost event should insert");

    conn.execute(
        "
        INSERT INTO cost_events (
            id, project_id, run_id, job_id, provider_code, model_name,
            event_type, units, unit_cost_usd, total_cost_usd, currency,
            meta_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ",
        params![
            cost_id_a,
            project_id,
            "run_new",
            "job_new",
            "openai",
            "gpt-image-1",
            "generation",
            1000.0_f64,
            0.00004_f64,
            0.040_f64,
            "USD",
            r#"{"batch":2}"#,
            "2026-02-21T00:01:30Z"
        ],
    )
    .expect("cost event should insert");
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
    let root = std::env::temp_dir().join(format!("kroma_analytics_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db.clone(), root));
    store.initialize().expect("store should initialize");
    (store, db)
}
