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

#[derive(Debug, Clone)]
struct SeedIds {
    run_id: String,
    job_id: String,
    candidate_id: String,
    asset_id: String,
}

#[tokio::test]
async fn runs_and_assets_endpoints_return_seeded_records() {
    let (store, db_path) = test_store_with_db_path();
    let app = build_router_with_projects_store_dev_bypass(store);

    let create = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Runs Assets"}"#),
        StatusCode::OK,
    )
    .await;
    let project_id = create["project"]["id"]
        .as_str()
        .expect("project id should exist")
        .to_string();
    let slug = create["project"]["slug"]
        .as_str()
        .expect("project slug should exist");

    let seeded = seed_run_asset_data(db_path.as_path(), project_id.as_str());

    let runs = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/runs?limit=10"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(runs["ok"], json!(true));
    assert_eq!(runs["count"], json!(1));
    assert_eq!(runs["runs"][0]["id"], json!(seeded.run_id));
    assert_eq!(runs["runs"][0]["model_name"], json!("gpt-image-1"));

    let run_detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/runs/{run_id}", run_id = seeded.run_id),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(run_detail["ok"], json!(true));
    assert_eq!(run_detail["run"]["id"], json!(seeded.run_id));
    assert_eq!(run_detail["jobs"][0]["id"], json!(seeded.job_id));
    assert_eq!(
        run_detail["jobs"][0]["candidates"][0]["id"],
        json!(seeded.candidate_id)
    );

    let run_jobs = send_json(
        app.clone(),
        Method::GET,
        &format!(
            "/api/projects/{slug}/runs/{run_id}/jobs",
            run_id = seeded.run_id
        ),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(run_jobs["ok"], json!(true));
    assert_eq!(run_jobs["run_id"], json!(seeded.run_id));
    assert_eq!(run_jobs["count"], json!(1));

    let assets = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/assets?limit=20"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(assets["ok"], json!(true));
    assert_eq!(assets["count"], json!(1));
    assert_eq!(assets["assets"][0]["id"], json!(seeded.asset_id));

    let asset_detail = send_json(
        app,
        Method::GET,
        &format!(
            "/api/projects/{slug}/assets/{asset_id}",
            asset_id = seeded.asset_id
        ),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(asset_detail["ok"], json!(true));
    assert_eq!(asset_detail["asset"]["id"], json!(seeded.asset_id));
    assert_eq!(
        asset_detail["asset"]["metadata_json"]["source"],
        json!("seed")
    );
}

#[tokio::test]
async fn run_and_asset_detail_return_not_found() {
    let (store, _) = test_store_with_db_path();
    let app = build_router_with_projects_store_dev_bypass(store);

    let create = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Missing Records"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let run_not_found = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/runs/missing-run"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(run_not_found["ok"], json!(false));
    assert_eq!(run_not_found["error"], json!("Run not found"));

    let asset_not_found = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/assets/missing-asset"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(asset_not_found["ok"], json!(false));
    assert_eq!(asset_not_found["error"], json!("Asset not found"));
}

fn seed_run_asset_data(db_path: &Path, project_id: &str) -> SeedIds {
    let run_id = format!("run_{}", Uuid::new_v4().simple());
    let job_id = format!("job_{}", Uuid::new_v4().simple());
    let candidate_id = format!("cand_{}", Uuid::new_v4().simple());
    let asset_id = format!("asset_{}", Uuid::new_v4().simple());

    let conn = Connection::open(db_path).expect("sqlite connection should open");

    conn.execute(
        "
        INSERT INTO runs (
            id, project_id, run_mode, status, stage, time_of_day, weather,
            model_name, provider_code, settings_snapshot_json, started_at,
            finished_at, created_at, run_log_path, image_size, image_quality
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
        ",
        params![
            run_id,
            project_id,
            "generate",
            "completed",
            "done",
            "day",
            "clear",
            "gpt-image-1",
            "openai",
            r#"{"seed":true,"steps":24}"#,
            "2026-02-21T00:00:00Z",
            "2026-02-21T00:01:00Z",
            "2026-02-21T00:00:00Z",
            "runs/demo/log.txt",
            "1024x1024",
            "high"
        ],
    )
    .expect("run should insert");

    conn.execute(
        "
        INSERT INTO run_jobs (
            id, run_id, job_key, status, prompt_text, selected_candidate_index,
            final_asset_id, final_output, meta_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ",
        params![
            job_id,
            run_id,
            "cover",
            "completed",
            "cinematic hero portrait",
            0_i64,
            asset_id,
            "assets/final/cover.png",
            r#"{"job_type":"cover"}"#,
            "2026-02-21T00:00:10Z"
        ],
    )
    .expect("job should insert");

    conn.execute(
        "
        INSERT INTO run_candidates (
            id, job_id, candidate_index, status, output_asset_id, final_asset_id,
            output_path, final_output_path, rank_hard_failures,
            rank_soft_warnings, rank_avg_chroma_exceed, meta_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ",
        params![
            candidate_id,
            job_id,
            0_i64,
            "accepted",
            asset_id,
            asset_id,
            "assets/candidates/cover_0.png",
            "assets/final/cover.png",
            0_i64,
            0_i64,
            0.0_f64,
            r#"{"score":0.98}"#,
            "2026-02-21T00:00:20Z"
        ],
    )
    .expect("candidate should insert");

    conn.execute(
        "
        INSERT INTO assets (
            id, project_id, kind, asset_kind, storage_uri, rel_path,
            storage_backend, mime_type, width, height, sha256,
            run_id, job_id, candidate_id, metadata_json, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
        ",
        params![
            asset_id,
            project_id,
            "image",
            "render",
            "file:///tmp/kroma/final/cover.png",
            "assets/final/cover.png",
            "local",
            "image/png",
            1024_i64,
            1024_i64,
            "abcdef0123456789",
            run_id,
            job_id,
            candidate_id,
            r#"{"source":"seed"}"#,
            "2026-02-21T00:01:00Z"
        ],
    )
    .expect("asset should insert");

    SeedIds {
        run_id,
        job_id,
        candidate_id,
        asset_id,
    }
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
    let root = std::env::temp_dir().join(format!("kroma_runs_assets_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db.clone(), root));
    store.initialize().expect("store should initialize");
    (store, db)
}
