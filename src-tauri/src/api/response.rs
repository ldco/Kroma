use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::Value;

use crate::api::error::{ApiError, ErrorKind};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ApiEnvelope<T> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

pub type ApiJson<T> = (StatusCode, Json<ApiEnvelope<T>>);

pub fn success<T>(payload: T) -> ApiJson<T>
where
    T: Serialize,
{
    (
        StatusCode::OK,
        Json(ApiEnvelope {
            ok: true,
            data: Some(payload),
            error: None,
        }),
    )
}

pub fn failure(
    status: StatusCode,
    kind: ErrorKind,
    code: impl Into<String>,
    message: impl Into<String>,
    details: Option<Value>,
) -> ApiJson<Value> {
    (
        status,
        Json(ApiEnvelope {
            ok: false,
            data: None,
            error: Some(ApiError::new(kind, code, message, details)),
        }),
    )
}
