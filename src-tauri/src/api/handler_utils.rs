use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::Value;
use tracing::error;

use crate::db::projects::ProjectsRepoError;

pub type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

pub fn map_repo_error(error: ProjectsRepoError, not_found_message: &str) -> ApiObject<Value> {
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

pub fn internal_error(message: impl Into<String>) -> ApiObject<Value> {
    let detail = message.into();
    error!(detail = %detail, "internal api error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        into_json(ErrorResponse {
            ok: false,
            error: String::from("Internal server error"),
        }),
    )
}

pub fn into_json(payload: impl Serialize) -> Json<Value> {
    Json(serde_json::to_value(payload).expect("api payload should serialize"))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;

    use super::{internal_error, map_repo_error};
    use crate::db::projects::ProjectsRepoError;

    #[test]
    fn map_repo_error_maps_not_found_with_custom_message() {
        let (status, payload) = map_repo_error(ProjectsRepoError::NotFound, "Thing not found");
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(payload.0["ok"], json!(false));
        assert_eq!(payload.0["error"], json!("Thing not found"));
    }

    #[test]
    fn map_repo_error_maps_validation_message() {
        let (status, payload) = map_repo_error(
            ProjectsRepoError::Validation(String::from("bad payload")),
            "ignored",
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload.0["ok"], json!(false));
        assert_eq!(payload.0["error"], json!("bad payload"));
    }

    #[test]
    fn internal_errors_are_sanitized() {
        let (status, payload) = internal_error("sensitive detail");
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(payload.0["ok"], json!(false));
        assert_eq!(payload.0["error"], json!("Internal server error"));
    }
}
