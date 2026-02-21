use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;
use uuid::Uuid;

use kroma_backend_core::api::routes::route_catalog;
use kroma_backend_core::api::server::build_router_with_projects_store;
use kroma_backend_core::contract::HttpMethod;
use kroma_backend_core::db::projects::ProjectsStore;

#[tokio::test]
async fn every_contract_route_is_http_mounted() {
    let app = build_router_with_projects_store(test_repo());

    for route in route_catalog() {
        let request_path = materialize_path(route.spec.path.as_str());
        let request = Request::builder()
            .method(to_http_method(route.spec.method))
            .uri(request_path)
            .header("content-type", "application/json")
            .body(request_body(route.spec.method))
            .expect("request should build");

        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("router should return response");

        let status = response.status();
        let expected = expected_status(route.spec.method, route.spec.path.as_str());
        assert_eq!(
            status, expected,
            "unexpected status for {} {}",
            route.spec.method, route.spec.path
        );
    }
}

fn to_http_method(method: HttpMethod) -> Method {
    match method {
        HttpMethod::Get => Method::GET,
        HttpMethod::Post => Method::POST,
        HttpMethod::Put => Method::PUT,
        HttpMethod::Delete => Method::DELETE,
        HttpMethod::Patch => Method::PATCH,
        HttpMethod::Options => Method::OPTIONS,
        HttpMethod::Head => Method::HEAD,
    }
}

fn request_body(method: HttpMethod) -> Body {
    match method {
        HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => Body::from("{}"),
        _ => Body::empty(),
    }
}

fn materialize_path(path_template: &str) -> String {
    let mut output = String::with_capacity(path_template.len());
    let mut in_param = false;

    for ch in path_template.chars() {
        match ch {
            '{' => {
                in_param = true;
                output.push('x');
            }
            '}' => in_param = false,
            _ if in_param => {}
            _ => output.push(ch),
        }
    }

    output
}

fn expected_status(method: HttpMethod, path: &str) -> StatusCode {
    match (method, path) {
        (HttpMethod::Get, "/health") => StatusCode::OK,
        (HttpMethod::Get, "/api/projects") => StatusCode::OK,
        (HttpMethod::Post, "/api/projects") => StatusCode::BAD_REQUEST,
        (HttpMethod::Get, "/api/projects/{slug}") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/storage") => StatusCode::NOT_FOUND,
        (HttpMethod::Put, "/api/projects/{slug}/storage/local") => StatusCode::BAD_REQUEST,
        (HttpMethod::Put, "/api/projects/{slug}/storage/s3") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/runs") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/runs/{runId}") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/runs/{runId}/jobs") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/assets") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/assets/{assetId}") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/asset-links") => StatusCode::NOT_FOUND,
        (HttpMethod::Post, "/api/projects/{slug}/asset-links") => StatusCode::BAD_REQUEST,
        (HttpMethod::Get, "/api/projects/{slug}/asset-links/{linkId}") => StatusCode::NOT_FOUND,
        (HttpMethod::Put, "/api/projects/{slug}/asset-links/{linkId}") => StatusCode::BAD_REQUEST,
        (HttpMethod::Delete, "/api/projects/{slug}/asset-links/{linkId}") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/quality-reports") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/cost-events") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/exports") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/exports/{exportId}") => StatusCode::NOT_FOUND,
        (HttpMethod::Get, "/api/projects/{slug}/prompt-templates") => StatusCode::NOT_FOUND,
        (HttpMethod::Post, "/api/projects/{slug}/prompt-templates") => StatusCode::BAD_REQUEST,
        (HttpMethod::Get, "/api/projects/{slug}/prompt-templates/{templateId}") => {
            StatusCode::NOT_FOUND
        }
        (HttpMethod::Put, "/api/projects/{slug}/prompt-templates/{templateId}") => {
            StatusCode::BAD_REQUEST
        }
        (HttpMethod::Delete, "/api/projects/{slug}/prompt-templates/{templateId}") => {
            StatusCode::NOT_FOUND
        }
        (HttpMethod::Get, "/api/projects/{slug}/provider-accounts") => StatusCode::NOT_FOUND,
        (HttpMethod::Post, "/api/projects/{slug}/provider-accounts") => StatusCode::BAD_REQUEST,
        (HttpMethod::Get, "/api/projects/{slug}/provider-accounts/{providerCode}") => {
            StatusCode::NOT_FOUND
        }
        (HttpMethod::Put, "/api/projects/{slug}/provider-accounts/{providerCode}") => {
            StatusCode::BAD_REQUEST
        }
        (HttpMethod::Delete, "/api/projects/{slug}/provider-accounts/{providerCode}") => {
            StatusCode::NOT_FOUND
        }
        _ => StatusCode::NOT_IMPLEMENTED,
    }
}

fn test_repo() -> std::sync::Arc<ProjectsStore> {
    let suffix = Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("kroma_router_test_{suffix}"));
    let db = root.join("var/backend/app.db");
    std::fs::create_dir_all(root.as_path()).expect("temp test root must be creatable");
    let store = std::sync::Arc::new(ProjectsStore::new(db, root));
    store.initialize().expect("store should initialize");
    store
}
