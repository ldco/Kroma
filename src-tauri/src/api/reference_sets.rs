use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;
use crate::db::projects::{
    CreateReferenceSetInput, CreateReferenceSetItemInput, ProjectsRepoError,
    ReferenceSetItemSummary, ReferenceSetSummary, UpdateReferenceSetInput,
    UpdateReferenceSetItemInput,
};

type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Deserialize)]
pub struct SlugReferenceSetPath {
    pub slug: String,
    #[serde(rename = "referenceSetId")]
    pub reference_set_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlugReferenceSetItemPath {
    pub slug: String,
    #[serde(rename = "referenceSetId")]
    pub reference_set_id: String,
    #[serde(rename = "itemId")]
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListReferenceSetsResponse {
    ok: bool,
    count: usize,
    reference_sets: Vec<ReferenceSetSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ReferenceSetResponse {
    ok: bool,
    reference_set: ReferenceSetSummary,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteReferenceSetResponse {
    ok: bool,
    deleted: bool,
    id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListReferenceSetItemsResponse {
    ok: bool,
    reference_set_id: String,
    count: usize,
    items: Vec<ReferenceSetItemSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ReferenceSetItemResponse {
    ok: bool,
    item: ReferenceSetItemSummary,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteReferenceSetItemResponse {
    ok: bool,
    deleted: bool,
    id: String,
}

pub async fn list_reference_sets_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.list_reference_sets(slug.as_str())).await;

    match result {
        Ok(Ok(reference_sets)) => (
            StatusCode::OK,
            into_json(ListReferenceSetsResponse {
                ok: true,
                count: reference_sets.len(),
                reference_sets,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => {
            internal_error(format!("reference set listing task failed: {join_error}"))
        }
    }
}

pub async fn create_reference_set_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateReferenceSetInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.create_reference_set(slug.as_str(), payload))
            .await;

    match result {
        Ok(Ok(reference_set)) => (
            StatusCode::OK,
            into_json(ReferenceSetResponse {
                ok: true,
                reference_set,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => {
            internal_error(format!("reference set create task failed: {join_error}"))
        }
    }
}

pub async fn get_reference_set_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugReferenceSetPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_reference_set_detail(path.slug.as_str(), path.reference_set_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(reference_set)) => (
            StatusCode::OK,
            into_json(ReferenceSetResponse {
                ok: true,
                reference_set,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Reference set not found"),
        Err(join_error) => {
            internal_error(format!("reference set detail task failed: {join_error}"))
        }
    }
}

pub async fn update_reference_set_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugReferenceSetPath>,
    Json(payload): Json<UpdateReferenceSetInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_reference_set(path.slug.as_str(), path.reference_set_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(reference_set)) => (
            StatusCode::OK,
            into_json(ReferenceSetResponse {
                ok: true,
                reference_set,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Reference set not found"),
        Err(join_error) => {
            internal_error(format!("reference set update task failed: {join_error}"))
        }
    }
}

pub async fn delete_reference_set_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugReferenceSetPath>,
) -> ApiObject<Value> {
    let reference_set_id = path.reference_set_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_reference_set(path.slug.as_str(), path.reference_set_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeleteReferenceSetResponse {
                ok: true,
                deleted: true,
                id: reference_set_id,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Reference set not found"),
        Err(join_error) => {
            internal_error(format!("reference set delete task failed: {join_error}"))
        }
    }
}

pub async fn list_reference_set_items_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugReferenceSetPath>,
) -> ApiObject<Value> {
    let reference_set_id = path.reference_set_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.list_reference_set_items(path.slug.as_str(), path.reference_set_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(items)) => (
            StatusCode::OK,
            into_json(ListReferenceSetItemsResponse {
                ok: true,
                reference_set_id,
                count: items.len(),
                items,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Reference set not found"),
        Err(join_error) => internal_error(format!(
            "reference set item listing task failed: {join_error}"
        )),
    }
}

pub async fn create_reference_set_item_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugReferenceSetPath>,
    Json(payload): Json<CreateReferenceSetItemInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.create_reference_set_item(path.slug.as_str(), path.reference_set_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(item)) => (
            StatusCode::OK,
            into_json(ReferenceSetItemResponse { ok: true, item }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Reference set not found"),
        Err(join_error) => internal_error(format!(
            "reference set item create task failed: {join_error}"
        )),
    }
}

pub async fn get_reference_set_item_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugReferenceSetItemPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_reference_set_item_detail(
            path.slug.as_str(),
            path.reference_set_id.as_str(),
            path.item_id.as_str(),
        )
    })
    .await;

    match result {
        Ok(Ok(item)) => (
            StatusCode::OK,
            into_json(ReferenceSetItemResponse { ok: true, item }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Reference set item not found"),
        Err(join_error) => internal_error(format!(
            "reference set item detail task failed: {join_error}"
        )),
    }
}

pub async fn update_reference_set_item_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugReferenceSetItemPath>,
    Json(payload): Json<UpdateReferenceSetItemInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_reference_set_item(
            path.slug.as_str(),
            path.reference_set_id.as_str(),
            path.item_id.as_str(),
            payload,
        )
    })
    .await;

    match result {
        Ok(Ok(item)) => (
            StatusCode::OK,
            into_json(ReferenceSetItemResponse { ok: true, item }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Reference set item not found"),
        Err(join_error) => internal_error(format!(
            "reference set item update task failed: {join_error}"
        )),
    }
}

pub async fn delete_reference_set_item_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugReferenceSetItemPath>,
) -> ApiObject<Value> {
    let item_id = path.item_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_reference_set_item(
            path.slug.as_str(),
            path.reference_set_id.as_str(),
            path.item_id.as_str(),
        )
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeleteReferenceSetItemResponse {
                ok: true,
                deleted: true,
                id: item_id,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Reference set item not found"),
        Err(join_error) => internal_error(format!(
            "reference set item delete task failed: {join_error}"
        )),
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
