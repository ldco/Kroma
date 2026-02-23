use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::audit::write_project_audit_event;
use crate::api::auth::AuthPrincipal;
use crate::api::server::AppState;

use super::handler_utils::{internal_error, into_json, map_repo_error, ApiObject};
use crate::db::projects::{
    AgentInstructionActionInput, AgentInstructionEventSummary, AgentInstructionSummary,
    CreateAgentInstructionInput,
};

#[derive(Debug, Clone, Deserialize)]
pub struct SlugInstructionPath {
    pub slug: String,
    #[serde(rename = "instructionId")]
    pub instruction_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListInstructionsResponse {
    ok: bool,
    count: usize,
    instructions: Vec<AgentInstructionSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct InstructionResponse {
    ok: bool,
    instruction: AgentInstructionSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct InstructionEventsResponse {
    ok: bool,
    instruction_id: String,
    count: usize,
    events: Vec<AgentInstructionEventSummary>,
}

pub async fn list_agent_instructions_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result =
        tokio::task::spawn_blocking(move || store.list_agent_instructions(slug.as_str())).await;

    match result {
        Ok(Ok(instructions)) => (
            StatusCode::OK,
            into_json(ListInstructionsResponse {
                ok: true,
                count: instructions.len(),
                instructions,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!(
            "agent instruction listing task failed: {join_error}"
        )),
    }
}

pub async fn create_agent_instruction_handler(
    State(state): State<AppState>,
    Extension(actor): Extension<AuthPrincipal>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateAgentInstructionInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let slug_for_store = slug.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.create_agent_instruction(slug_for_store.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(instruction)) => {
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                slug.as_str(),
                "agent_instruction.create",
                json!({"instruction_id": instruction.id, "status": instruction.status}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => {
                    return internal_error(format!("instruction audit failed: {message}"))
                }
            };
            (
                StatusCode::OK,
                into_json(InstructionResponse {
                    ok: true,
                    instruction,
                    audit_id,
                }),
            )
        }
        Ok(Err(error)) => map_repo_error(error, "Project not found"),
        Err(join_error) => internal_error(format!(
            "agent instruction create task failed: {join_error}"
        )),
    }
}

pub async fn get_agent_instruction_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugInstructionPath>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.get_agent_instruction_detail(path.slug.as_str(), path.instruction_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(instruction)) => (
            StatusCode::OK,
            into_json(InstructionResponse {
                ok: true,
                instruction,
                audit_id: None,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Agent instruction not found"),
        Err(join_error) => internal_error(format!(
            "agent instruction detail task failed: {join_error}"
        )),
    }
}

pub async fn list_agent_instruction_events_handler(
    State(state): State<AppState>,
    Path(path): Path<SlugInstructionPath>,
) -> ApiObject<Value> {
    let instruction_id = path.instruction_id.clone();
    let store = state.projects_store.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.list_agent_instruction_events(path.slug.as_str(), path.instruction_id.as_str())
    })
    .await;

    match result {
        Ok(Ok(events)) => (
            StatusCode::OK,
            into_json(InstructionEventsResponse {
                ok: true,
                instruction_id,
                count: events.len(),
                events,
            }),
        ),
        Ok(Err(error)) => map_repo_error(error, "Agent instruction not found"),
        Err(join_error) => internal_error(format!(
            "agent instruction events task failed: {join_error}"
        )),
    }
}

pub async fn confirm_agent_instruction_handler(
    State(state): State<AppState>,
    Extension(actor): Extension<AuthPrincipal>,
    Path(path): Path<SlugInstructionPath>,
    Json(payload): Json<AgentInstructionActionInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let audit_slug = path.slug.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.confirm_agent_instruction(path.slug.as_str(), path.instruction_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(instruction)) => {
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                audit_slug.as_str(),
                "agent_instruction.confirm",
                json!({"instruction_id": instruction.id, "status": instruction.status}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => {
                    return internal_error(format!("instruction audit failed: {message}"))
                }
            };
            (
                StatusCode::OK,
                into_json(InstructionResponse {
                    ok: true,
                    instruction,
                    audit_id,
                }),
            )
        }
        Ok(Err(error)) => map_repo_error(error, "Agent instruction not found"),
        Err(join_error) => internal_error(format!(
            "agent instruction confirm task failed: {join_error}"
        )),
    }
}

pub async fn cancel_agent_instruction_handler(
    State(state): State<AppState>,
    Extension(actor): Extension<AuthPrincipal>,
    Path(path): Path<SlugInstructionPath>,
    Json(payload): Json<AgentInstructionActionInput>,
) -> ApiObject<Value> {
    let store = state.projects_store.clone();
    let audit_slug = path.slug.clone();
    let result = tokio::task::spawn_blocking(move || {
        store.cancel_agent_instruction(path.slug.as_str(), path.instruction_id.as_str(), payload)
    })
    .await;

    match result {
        Ok(Ok(instruction)) => {
            let audit_id = match write_project_audit_event(
                &state,
                Some(&actor),
                audit_slug.as_str(),
                "agent_instruction.cancel",
                json!({"instruction_id": instruction.id, "status": instruction.status}),
            )
            .await
            {
                Ok(id) => Some(id),
                Err(message) => {
                    return internal_error(format!("instruction audit failed: {message}"))
                }
            };
            (
                StatusCode::OK,
                into_json(InstructionResponse {
                    ok: true,
                    instruction,
                    audit_id,
                }),
            )
        }
        Ok(Err(error)) => map_repo_error(error, "Agent instruction not found"),
        Err(join_error) => internal_error(format!(
            "agent instruction cancel task failed: {join_error}"
        )),
    }
}
