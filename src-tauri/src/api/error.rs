use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Validation,
    Provider,
    Infra,
    Policy,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ApiError {
    pub kind: ErrorKind,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(
        kind: ErrorKind,
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<Value>,
    ) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            details,
        }
    }
}
