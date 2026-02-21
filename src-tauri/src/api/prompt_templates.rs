use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::server::AppState;

use super::handler_utils::{internal_error, into_json, map_repo_error, ApiObject};
use crate::db::projects::{
    CreatePromptTemplateInput, PromptTemplateSummary, UpdatePromptTemplateInput,
};

#[derive(Debug, Clone, Deserialize)]
pub struct SlugTemplatePath {
    pub slug: String,
    #[serde(rename = "templateId")]
    pub template_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListPromptTemplatesResponse {
    ok: bool,
    count: usize,
    prompt_templates: Vec<PromptTemplateSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct PromptTemplateResponse {
    ok: bool,
    prompt_template: PromptTemplateSummary,
}

#[derive(Debug, Clone, Serialize)]
struct DeletePromptTemplateResponse {
    ok: bool,
    deleted: bool,
    id: String,
}

pub async fn list_prompt_templates_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.list_prompt_templates(slug.as_str())).await;

    match result {
        Ok(Ok(prompt_templates)) => (
            StatusCode::OK,
            into_json(ListPromptTemplatesResponse {
                ok: true,
                count: prompt_templates.len(),
                prompt_templates,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => {
            internal_error(format!("prompt template listing task failed: {join_error}"))
        }
    }
}

pub async fn create_prompt_template_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreatePromptTemplateInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.create_prompt_template(slug.as_str(), payload))
            .await;

    match result {
        Ok(Ok(prompt_template)) => (
            StatusCode::OK,
            into_json(PromptTemplateResponse {
                ok: true,
                prompt_template,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => {
            internal_error(format!("prompt template create task failed: {join_error}"))
        }
    }
}

pub async fn get_prompt_template_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugTemplatePath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_prompt_template_detail(path.slug.as_str(), path.template_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(prompt_template)) => (
            StatusCode::OK,
            into_json(PromptTemplateResponse {
                ok: true,
                prompt_template,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Prompt template not found"),
        Err(join_error) => {
            internal_error(format!("prompt template detail task failed: {join_error}"))
        }
    }
}

pub async fn update_prompt_template_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugTemplatePath>,
    Json(payload): Json<UpdatePromptTemplateInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.update_prompt_template(path.slug.as_str(), path.template_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(prompt_template)) => (
            StatusCode::OK,
            into_json(PromptTemplateResponse {
                ok: true,
                prompt_template,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Prompt template not found"),
        Err(join_error) => {
            internal_error(format!("prompt template update task failed: {join_error}"))
        }
    }
}

pub async fn delete_prompt_template_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugTemplatePath>,
) -> ApiObject<Value> {
    let template_id = path.template_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.delete_prompt_template(path.slug.as_str(), path.template_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            into_json(DeletePromptTemplateResponse {
                ok: true,
                deleted: true,
                id: template_id,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Prompt template not found"),
        Err(join_error) => {
            internal_error(format!("prompt template delete task failed: {join_error}"))
        }
    }
}
