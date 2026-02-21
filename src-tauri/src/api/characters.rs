use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;

use super::handler_utils::{internal_error, into_json, map_repo_error, ApiObject};
use crate::db::projects::{CharacterSummary, CreateCharacterInput, UpdateCharacterInput};

#[derive(Debug, Clone, Deserialize)]
pub struct SlugCharacterPath {
    pub slug: String,
    #[serde(rename = "characterId")]
    pub character_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListCharactersResponse {
    ok: bool,
    count: usize,
    characters: Vec<CharacterSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct CharacterResponse {
    ok: bool,
    character: CharacterSummary,
}

#[derive(Debug, Clone, Serialize)]
struct DeleteCharacterResponse {
    ok: bool,
    deleted: bool,
    id: String,
}

pub async fn list_characters_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || store.list_characters(slug.as_str())).await;

    match result {
        Ok(Ok(characters)) => (
            StatusCode::OK,
            into_json(ListCharactersResponse {
                ok: true,
                count: characters.len(),
                characters,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("character listing task failed: {join_error}")),
    }
}

pub async fn create_character_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateCharacterInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.create_character(slug.as_str(), payload)).await;

    match result {
        Ok(Ok(character)) => (
            StatusCode::OK,
            into_json(CharacterResponse {
                ok: true,
                character,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!("character create task failed: {join_error}")),
    }
}

pub async fn get_character_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugCharacterPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_character_detail(path.slug.as_str(), path.character_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(character)) => (
            StatusCode::OK,
            into_json(CharacterResponse {
                ok: true,
                character,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Character not found"),
        Err(join_error) => internal_error(format!("character detail task failed: {join_error}")),
    }
}

pub async fn update_character_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugCharacterPath>,
    Json(payload): Json<UpdateCharacterInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_character(path.slug.as_str(), path.character_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(character)) => (
            StatusCode::OK,
            into_json(CharacterResponse {
                ok: true,
                character,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Character not found"),
        Err(join_error) => internal_error(format!("character update task failed: {join_error}")),
    }
}

pub async fn delete_character_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugCharacterPath>,
) -> ApiObject<Value> {
    let character_id = path.character_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_character(path.slug.as_str(), path.character_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeleteCharacterResponse {
                ok: true,
                deleted: true,
                id: character_id,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Character not found"),
        Err(join_error) => internal_error(format!("character delete task failed: {join_error}")),
    }
}
