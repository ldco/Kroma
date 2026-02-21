use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{
    AssetLinkSummary, CreateAssetLinkInput, ProjectsRepoError, UpdateAssetLinkInput,
};

type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListAssetLinksQuery {
    pub asset_id: Option<String>,
    pub link_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlugLinkPath {
    pub slug: String,
    #[serde(rename = "linkId")]
    pub link_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListAssetLinksResponse {
    ok: bool,
    count: usize,
    asset_links: Vec<AssetLinkSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct AssetLinkResponse {
    ok: bool,
    asset_link: AssetLinkSummary,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteAssetLinkResponse {
    ok: bool,
    deleted: bool,
    id: String,
}

pub async fn list_asset_links_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListAssetLinksQuery>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let asset_id = query.asset_id.clone();
    let link_type = query.link_type.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.list_asset_links(slug.as_str(), asset_id.as_deref(), link_type.as_deref())
    })
    .await;

    match result {
        Ok(Ok(asset_links)) => (
            StatusCode::OK,
            into_json(ListAssetLinksResponse {
                ok: true,
                count: asset_links.len(),
                asset_links,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("asset-link listing task failed: {join_error}")),
    }
}

pub async fn create_asset_link_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateAssetLinkInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.create_asset_link(slug.as_str(), payload)).await;

    match result {
        Ok(Ok(asset_link)) => (
            StatusCode::OK,
            into_json(AssetLinkResponse {
                ok: true,
                asset_link,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("asset-link create task failed: {join_error}")),
    }
}

pub async fn get_asset_link_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugLinkPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_asset_link_detail(path.slug.as_str(), path.link_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(asset_link)) => (
            StatusCode::OK,
            into_json(AssetLinkResponse {
                ok: true,
                asset_link,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Asset link not found"),
        Err(join_error) => internal_error(format!("asset-link detail task failed: {join_error}")),
    }
}

pub async fn update_asset_link_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugLinkPath>,
    Json(payload): Json<UpdateAssetLinkInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_asset_link(path.slug.as_str(), path.link_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(asset_link)) => (
            StatusCode::OK,
            into_json(AssetLinkResponse {
                ok: true,
                asset_link,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Asset link not found"),
        Err(join_error) => internal_error(format!("asset-link update task failed: {join_error}")),
    }
}

pub async fn delete_asset_link_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugLinkPath>,
) -> ApiObject<Value> {
    let link_id = path.link_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_asset_link(path.slug.as_str(), path.link_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeleteAssetLinkResponse {
                ok: true,
                deleted: true,
                id: link_id,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Asset link not found"),
        Err(join_error) => internal_error(format!("asset-link delete task failed: {join_error}")),
    }
}

fn map_repo_error(error: ProjectsRepoError, not_found_message: &str) -> ApiObject<Value> {
    match error {
        ProjectsRepoError::NotFound => (
            StatusCode::NOT_FOUND,
            into_json(ErrorResponse {
                ok: false,
                error: String::from(not_found_message),
            }),
        ),
        ProjectsRepoError::Validation(message) => (
            StatusCode::BAD_REQUEST,
            into_json(ErrorResponse {
                ok: false,
                error: message,
            }),
        ),
        ProjectsRepoError::Sqlite(source) => internal_error(format!("database error: {source}")),
    }
}

fn internal_error(message: String) -> ApiObject<Value> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        into_json(ErrorResponse {
            ok: false,
            error: message,
        }),
    )
}

fn into_json(payload: impl Serialize) -> Json<Value> {
    Json(serde_json::to_value(payload).expect("api payload should serialize"))
}
