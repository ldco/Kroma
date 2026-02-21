use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use super::{ContractError, HttpMethod, RouteSpec};

#[derive(Debug, Error)]
pub enum OpenApiContractError {
    #[error("failed to read OpenAPI file '{path}': {source}")]
    Io { path: PathBuf, source: io::Error },

    #[error("failed to parse OpenAPI YAML '{path}': {source}")]
    Parse {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    #[error("invalid OpenAPI route '{path}': {source}")]
    InvalidRoute { path: String, source: ContractError },

    #[error("duplicate OpenAPI route: {method} {path}")]
    DuplicateRoute { method: HttpMethod, path: String },
}

#[derive(Debug, Deserialize)]
struct OpenApiDocument {
    #[serde(default)]
    paths: BTreeMap<String, PathItem>,
}

#[derive(Debug, Default, Deserialize)]
struct PathItem {
    #[serde(default)]
    get: Option<Operation>,
    #[serde(default)]
    post: Option<Operation>,
    #[serde(default)]
    put: Option<Operation>,
    #[serde(default)]
    delete: Option<Operation>,
    #[serde(default)]
    patch: Option<Operation>,
    #[serde(default)]
    options: Option<Operation>,
    #[serde(default)]
    head: Option<Operation>,
}

#[derive(Debug, Default, Deserialize)]
struct Operation {
    #[allow(dead_code)]
    summary: Option<String>,
}

pub fn load_routes(path: &Path) -> Result<Vec<RouteSpec>, OpenApiContractError> {
    let raw = fs::read_to_string(path).map_err(|source| OpenApiContractError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let document: OpenApiDocument =
        serde_yaml::from_str(&raw).map_err(|source| OpenApiContractError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

    let mut routes = Vec::new();
    let mut seen = BTreeSet::new();

    for (route_path, path_item) in document.paths {
        push_route(
            &mut routes,
            &mut seen,
            HttpMethod::Get,
            &route_path,
            path_item.get.is_some(),
        )?;
        push_route(
            &mut routes,
            &mut seen,
            HttpMethod::Post,
            &route_path,
            path_item.post.is_some(),
        )?;
        push_route(
            &mut routes,
            &mut seen,
            HttpMethod::Put,
            &route_path,
            path_item.put.is_some(),
        )?;
        push_route(
            &mut routes,
            &mut seen,
            HttpMethod::Delete,
            &route_path,
            path_item.delete.is_some(),
        )?;
        push_route(
            &mut routes,
            &mut seen,
            HttpMethod::Patch,
            &route_path,
            path_item.patch.is_some(),
        )?;
        push_route(
            &mut routes,
            &mut seen,
            HttpMethod::Options,
            &route_path,
            path_item.options.is_some(),
        )?;
        push_route(
            &mut routes,
            &mut seen,
            HttpMethod::Head,
            &route_path,
            path_item.head.is_some(),
        )?;
    }

    routes.sort();
    Ok(routes)
}

fn push_route(
    out: &mut Vec<RouteSpec>,
    seen: &mut BTreeSet<RouteSpec>,
    method: HttpMethod,
    path: &str,
    present: bool,
) -> Result<(), OpenApiContractError> {
    if !present {
        return Ok(());
    }

    let spec =
        RouteSpec::new(method, path).map_err(|source| OpenApiContractError::InvalidRoute {
            path: path.to_string(),
            source,
        })?;

    if !seen.insert(spec.clone()) {
        return Err(OpenApiContractError::DuplicateRoute {
            method,
            path: path.to_string(),
        });
    }

    out.push(spec);
    Ok(())
}
