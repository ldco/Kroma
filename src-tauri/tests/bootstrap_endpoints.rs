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
async fn bootstrap_prompt_export_and_import_round_trip() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Bootstrap Demo"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let _seed_provider = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::from(json!({"provider_code":"openai","display_name":"Old Provider"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let _seed_style = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/style-guides"),
        Body::from(
            json!({"name":"Old Style","instructions":"Old instructions","notes":"legacy"})
                .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let _seed_template = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::from(json!({"name":"Old Template","template_text":"Old template text"}).to_string()),
        StatusCode::OK,
    )
    .await;
    let _seed_character = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/characters"),
        Body::from(
            json!({
                "name":"Protagonist",
                "description":"Legacy character description",
                "prompt_text":"Legacy character prompt"
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    let exported = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/bootstrap-prompt"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(exported["ok"], json!(true));
    assert_eq!(
        exported["bootstrap"]["settings"]["provider_accounts"]
            .as_array()
            .expect("provider_accounts should be an array")
            .len(),
        1
    );
    assert_eq!(
        exported["bootstrap"]["settings"]["characters"]
            .as_array()
            .expect("characters should be an array")
            .len(),
        1
    );
    assert!(
        exported["bootstrap"]["prompt"]
            .as_str()
            .expect("prompt should be a string")
            .contains("Return ONLY valid JSON"),
        "prompt should contain strict output instructions"
    );

    let imported = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/bootstrap-import"),
        Body::from(
            json!({
                "mode": "replace",
                "settings": {
                    "project": {
                        "name": "Bootstrap Demo v2",
                        "description": "Imported via bootstrap payload"
                    },
                    "provider_accounts": [
                        {
                            "provider_code": "openai",
                            "display_name": "OpenAI Primary",
                            "base_url": "https://api.openai.com/v1",
                            "enabled": true,
                            "config_json": {"model": "gpt-image-1"}
                        }
                    ],
                    "style_guides": [
                        {
                            "name": "Studio Look",
                            "instructions": "Use cinematic lighting and subtle film grain.",
                            "notes": "Primary style"
                        }
                    ],
                    "characters": [
                        {
                            "name": "Protagonist",
                            "description": "Refined protagonist profile",
                            "prompt_text": "Consistent wardrobe, cinematic framing, same face."
                        }
                    ],
                    "prompt_templates": [
                        {
                            "name": "Hero Prompt",
                            "template_text": "Create a hero image with a consistent art direction."
                        }
                    ]
                }
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(imported["ok"], json!(true));
    assert_eq!(imported["bootstrap_import"]["mode"], json!("replace"));
    assert_eq!(imported["bootstrap_import"]["dry_run"], json!(false));
    assert_eq!(
        imported["bootstrap_import"]["applied"]["provider_accounts"],
        json!(1)
    );
    assert_eq!(
        imported["bootstrap_import"]["changes"]["provider_accounts"]["updated"],
        json!(1)
    );
    assert_eq!(
        imported["bootstrap_import"]["changes"]["provider_accounts"]["deleted"],
        json!(0)
    );
    assert_eq!(
        imported["bootstrap_import"]["changes"]["style_guides"]["replaced"],
        json!(true)
    );
    assert_eq!(
        imported["bootstrap_import"]["applied"]["characters"],
        json!(1)
    );
    assert_eq!(
        imported["bootstrap_import"]["changes"]["characters"]["updated"],
        json!(1)
    );
    assert_eq!(
        imported["bootstrap_import"]["project"]["name"],
        json!("Bootstrap Demo v2")
    );

    let project_detail = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        project_detail["project"]["name"],
        json!("Bootstrap Demo v2")
    );

    let providers = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(providers["count"], json!(1));
    assert_eq!(
        providers["provider_accounts"][0]["display_name"],
        json!("OpenAI Primary")
    );

    let style_guides = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/style-guides"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(style_guides["count"], json!(1));
    assert_eq!(
        style_guides["style_guides"][0]["name"],
        json!("Studio Look")
    );

    let characters = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/characters"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(characters["count"], json!(1));
    assert_eq!(characters["characters"][0]["name"], json!("Protagonist"));
    assert_eq!(
        characters["characters"][0]["description"],
        json!("Refined protagonist profile")
    );

    let templates = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(templates["count"], json!(1));
    assert_eq!(
        templates["prompt_templates"][0]["name"],
        json!("Hero Prompt")
    );
}

#[tokio::test]
async fn bootstrap_import_accepts_ai_response_text_payload() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Bootstrap AI"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let ai_payload = r#"```json
{
  "mode": "merge",
  "settings": {
    "style_guides": [
      {
        "name": "Painterly",
        "instructions": "Painterly brushwork with controlled color palette."
      }
    ],
    "prompt_templates": [
      {
        "name": "Scene Prompt",
        "template_text": "Generate a scene in the configured style."
      }
    ]
  }
}
```"#;

    let imported = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/bootstrap-import"),
        Body::from(json!({"ai_response_text": ai_payload}).to_string()),
        StatusCode::OK,
    )
    .await;
    assert_eq!(imported["bootstrap_import"]["mode"], json!("merge"));
    assert_eq!(
        imported["bootstrap_import"]["applied"]["style_guides"],
        json!(1)
    );
    assert_eq!(
        imported["bootstrap_import"]["applied"]["prompt_templates"],
        json!(1)
    );

    let style_guides = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/style-guides"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(style_guides["count"], json!(1));
    assert_eq!(style_guides["style_guides"][0]["name"], json!("Painterly"));

    let templates = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(templates["count"], json!(1));
    assert_eq!(
        templates["prompt_templates"][0]["name"],
        json!("Scene Prompt")
    );
}

#[tokio::test]
async fn bootstrap_import_validation_is_enforced() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Bootstrap Validation"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let missing_payload = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/bootstrap-import"),
        Body::from("{}"),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        missing_payload["error"],
        json!("Provide either 'settings' or 'ai_response_text'")
    );

    let invalid_mode = send_json(
        app,
        Method::POST,
        &format!("/api/projects/{slug}/bootstrap-import"),
        Body::from(
            json!({
                "mode": "overwrite",
                "settings": {
                    "style_guides": [
                        {"name":"One","instructions":"Two"}
                    ]
                }
            })
            .to_string(),
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        invalid_mode["error"],
        json!("Field 'mode' must be one of: merge, replace")
    );
}

#[tokio::test]
async fn bootstrap_replace_mode_only_replaces_provided_sections() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Bootstrap Replace Scope"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let _seed_provider = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::from(
            json!({"provider_code":"openai","display_name":"Original Provider"}).to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let _seed_style = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/style-guides"),
        Body::from(
            json!({"name":"Old Style","instructions":"Original style instructions"}).to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let _seed_template = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::from(
            json!({"name":"Original Template","template_text":"Original template"}).to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let _seed_character = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/characters"),
        Body::from(json!({"name":"Original Character","description":"Baseline"}).to_string()),
        StatusCode::OK,
    )
    .await;

    let _replace_style_only = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/bootstrap-import"),
        Body::from(
            json!({
                "mode": "replace",
                "settings": {
                    "style_guides": [
                        {
                            "name": "New Style",
                            "instructions": "Replacement style instructions."
                        }
                    ]
                }
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    let providers = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/provider-accounts"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(providers["count"], json!(1));
    assert_eq!(
        providers["provider_accounts"][0]["display_name"],
        json!("Original Provider")
    );

    let styles = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/style-guides"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(styles["count"], json!(1));
    assert_eq!(styles["style_guides"][0]["name"], json!("New Style"));

    let templates = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/prompt-templates"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(templates["count"], json!(1));
    assert_eq!(
        templates["prompt_templates"][0]["name"],
        json!("Original Template")
    );

    let characters = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/characters"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(characters["count"], json!(1));
    assert_eq!(
        characters["characters"][0]["name"],
        json!("Original Character")
    );

    assert_eq!(
        _replace_style_only["bootstrap_import"]["changes"]["provider_accounts"]["provided"],
        json!(false)
    );
    assert_eq!(
        _replace_style_only["bootstrap_import"]["changes"]["provider_accounts"]["deleted"],
        json!(0)
    );
    assert_eq!(
        _replace_style_only["bootstrap_import"]["changes"]["style_guides"]["replaced"],
        json!(true)
    );
    assert_eq!(
        _replace_style_only["bootstrap_import"]["changes"]["characters"]["provided"],
        json!(false)
    );
    assert_eq!(
        _replace_style_only["bootstrap_import"]["changes"]["characters"]["deleted"],
        json!(0)
    );
}

#[tokio::test]
async fn bootstrap_import_dry_run_previews_without_writing() {
    let app = build_router_with_projects_store(test_store());

    let create_project = send_json(
        app.clone(),
        Method::POST,
        "/api/projects",
        Body::from(r#"{"name":"Bootstrap Dry Run"}"#),
        StatusCode::OK,
    )
    .await;
    let slug = create_project["project"]["slug"]
        .as_str()
        .expect("project slug should exist")
        .to_string();

    let _seed_style = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/style-guides"),
        Body::from(
            json!({"name":"Existing Style","instructions":"Baseline instructions"}).to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    let _seed_character = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/characters"),
        Body::from(
            json!({"name":"Existing Character","prompt_text":"Baseline prompt"}).to_string(),
        ),
        StatusCode::OK,
    )
    .await;

    let preview = send_json(
        app.clone(),
        Method::POST,
        &format!("/api/projects/{slug}/bootstrap-import"),
        Body::from(
            json!({
                "mode": "replace",
                "dry_run": true,
                "settings": {
                    "style_guides": [
                        {
                            "name": "Preview Style",
                            "instructions": "Preview-only style instructions."
                        }
                    ],
                    "characters": [
                        {
                            "name": "Preview Character",
                            "description": "Preview-only character",
                            "prompt_text": "Preview-only character prompt"
                        }
                    ]
                }
            })
            .to_string(),
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(preview["bootstrap_import"]["dry_run"], json!(true));
    assert_eq!(preview["bootstrap_import"]["mode"], json!("replace"));
    assert_eq!(
        preview["bootstrap_import"]["changes"]["style_guides"]["created"],
        json!(1)
    );
    assert_eq!(
        preview["bootstrap_import"]["changes"]["style_guides"]["deleted"],
        json!(1)
    );
    assert_eq!(
        preview["bootstrap_import"]["changes"]["style_guides"]["replaced"],
        json!(true)
    );
    assert_eq!(
        preview["bootstrap_import"]["changes"]["characters"]["created"],
        json!(1)
    );
    assert_eq!(
        preview["bootstrap_import"]["changes"]["characters"]["deleted"],
        json!(1)
    );
    assert_eq!(
        preview["bootstrap_import"]["changes"]["characters"]["replaced"],
        json!(true)
    );
    assert_eq!(
        preview["bootstrap_import"]["settings"]["style_guides"][0]["name"],
        json!("Preview Style")
    );
    assert_eq!(
        preview["bootstrap_import"]["settings"]["characters"][0]["name"],
        json!("Preview Character")
    );

    let persisted = send_json(
        app.clone(),
        Method::GET,
        &format!("/api/projects/{slug}/style-guides"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(persisted["count"], json!(1));
    assert_eq!(
        persisted["style_guides"][0]["name"],
        json!("Existing Style")
    );

    let persisted_characters = send_json(
        app,
        Method::GET,
        &format!("/api/projects/{slug}/characters"),
        Body::empty(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(persisted_characters["count"], json!(1));
    assert_eq!(
        persisted_characters["characters"][0]["name"],
        json!("Existing Character")
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
    let root = std::env::temp_dir().join(format!("kroma_bootstrap_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = Arc::new(ProjectsStore::new(db, PathBuf::from(root)));
    store.initialize().expect("store should initialize");
    store
}
