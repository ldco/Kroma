use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::Value;
use tracing::error;

use crate::api::error::ErrorKind;
use crate::db::projects::ProjectsRepoError;

pub type ApiObject<T> = (StatusCode, Json<T>);

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_kind: Option<ErrorKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
}

pub fn error_response(
    status: StatusCode,
    kind: ErrorKind,
    code: impl Into<String>,
    message: impl Into<String>,
) -> ApiObject<Value> {
    (
        status,
        into_json(ErrorResponse {
            ok: false,
            error: message.into(),
            error_kind: Some(kind),
            error_code: Some(code.into()),
        }),
    )
}

pub fn map_repo_error(error: ProjectsRepoError, not_found_message: &str) -> ApiObject<Value> {
    match error {
        ProjectsRepoError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            ErrorKind::Validation,
            "not_found",
            not_found_message,
        ),
        ProjectsRepoError::Validation(message) => error_response(
            StatusCode::BAD_REQUEST,
            ErrorKind::Validation,
            "validation_error",
            message,
        ),
        ProjectsRepoError::Internal(message) => internal_error(message),
        ProjectsRepoError::Sqlite(source) => internal_error(format!("database error: {source}")),
    }
}

pub fn internal_error(message: impl Into<String>) -> ApiObject<Value> {
    let detail = message.into();
    error!(detail = %detail, "internal api error");
    error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::Infra,
        "internal_error",
        "Internal server error",
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
        assert_eq!(payload.0["error_kind"], json!("validation"));
        assert_eq!(payload.0["error_code"], json!("not_found"));
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
        assert_eq!(payload.0["error_kind"], json!("validation"));
        assert_eq!(payload.0["error_code"], json!("validation_error"));
    }

    #[test]
    fn internal_errors_are_sanitized() {
        let (status, payload) = internal_error("sensitive detail");
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(payload.0["ok"], json!(false));
        assert_eq!(payload.0["error"], json!("Internal server error"));
        assert_eq!(payload.0["error_kind"], json!("infra"));
        assert_eq!(payload.0["error_code"], json!("internal_error"));
    }
}
