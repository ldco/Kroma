pub mod openapi;

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Options => "OPTIONS",
            Self::Head => "HEAD",
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HttpMethod {
    type Err = ContractError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "OPTIONS" => Ok(Self::Options),
            "HEAD" => Ok(Self::Head),
            other => Err(ContractError::UnsupportedHttpMethod(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RouteSpec {
    pub method: HttpMethod,
    pub path: String,
}

impl RouteSpec {
    pub fn new(method: HttpMethod, path: impl Into<String>) -> Result<Self, ContractError> {
        let path = path.into();
        if !path.starts_with('/') {
            return Err(ContractError::InvalidRoutePath(path));
        }
        Ok(Self { method, path })
    }
}

impl fmt::Display for RouteSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.method, self.path)
    }
}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("unsupported HTTP method: {0}")]
    UnsupportedHttpMethod(String),

    #[error("route path must start with '/' but was '{0}'")]
    InvalidRoutePath(String),
}
