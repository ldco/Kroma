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
async fn voice_stt_and_tts_requests_support_create_and_detail() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Voice Project"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let created_stt = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/voice/stt"),
        Body::from(
            json!({
                "audio_uri": "file:///tmp/input.wav",
                "language": "en",
                "mock_transcript": "hello from speech"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    let stt_request_id = created_stt["request"]["id"]
        .as_str()
        .expect("stt request id should exist")
        .to_string();
    assert_eq!(created_stt["request"]["request_type"], json!("stt"));
    assert_eq!(created_stt["request"]["status"], json!("completed"));
    assert_eq!(created_stt["request"]["input_text"], json!("en"));
    assert_eq!(
        created_stt["request"]["output_text"],
        json!("hello from speech")
    );

    let stt_detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/voice/requests/{stt_request_id}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(stt_detail["request"]["id"], json!(stt_request_id));
    assert_eq!(
        stt_detail["request"]["audio_uri"],
        json!("file:///tmp/input.wav")
    );

    let created_tts = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/voice/tts"),
        Body::from(
            json!({
                "text": "Generate a spoken summary",
                "voice": "alloy",
                "format": "mp3"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let tts_request_id = created_tts["request"]["id"]
        .as_str()
        .expect("tts request id should exist")
        .to_string();
    assert_eq!(created_tts["request"]["request_type"], json!("tts"));
    assert_eq!(
        created_tts["request"]["input_text"],
        json!("Generate a spoken summary")
    );

    let expected_audio_uri_prefix = format!("voice://alloy/{tts_request_id}");
    let tts_audio_uri = created_tts["request"]["audio_uri"]
        .as_str()
        .expect("tts audio uri should exist");
    assert!(tts_audio_uri.starts_with(expected_audio_uri_prefix.as_str()));
    assert!(tts_audio_uri.ends_with(".mp3"));
}

#[tokio::test]
async fn voice_validation_and_not_found_paths_are_enforced() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Voice Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let missing_tts_text = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/voice/tts"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(missing_tts_text["error"], json!("Field 'text' is required"));

    let missing_voice_request = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/voice/requests/missing"),
        Body::empty(),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(
        missing_voice_request["error"],
        json!("Voice request not found")
    );

    let unknown_project = send_json(
        app,
        Method::POST,
        "/api/projects/unknown-slug/voice/stt",
        Body::from(json!({"audio_uri":"file:///tmp/input.wav"}).to_string()),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(unknown_project["error"], json!("Project not found"));
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
    let root = std::env::temp_dir().join(format!("kroma_voice_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
