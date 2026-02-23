use serde_json::Value;

use crate::api::auth::AuthPrincipal;
use crate::api::server::AppState;
use crate::db::projects::AppendAuditEventInput;

pub async fn write_project_audit_event(
    state: &AppState,
    actor: Option<&AuthPrincipal>,
    project_slug: &str,
    event_code: &str,
    payload_json: Value,
) -> Result<String, String> {
    let store = state.projects_store.clone();
    let actor_user_id = actor
        .and_then(AuthPrincipal::actor_user_id)
        .map(ToOwned::to_owned);
    let project_slug = project_slug.to_string();
    let event_code = event_code.to_string();
    tokio::task::spawn_blocking(move || {
        store.append_audit_event(AppendAuditEventInput {
            project_slug: Some(project_slug),
            actor_user_id,
            event_code,
            payload_json,
        })
    })
    .await
    .map_err(|e| format!("audit task failed: {e}"))?
    .map_err(|e| e.to_string())
}
