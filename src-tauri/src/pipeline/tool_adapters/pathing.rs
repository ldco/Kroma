use std::path::{Path, PathBuf};

use super::ToolAdapterError;
use crate::pipeline::pathing as shared;

pub(super) fn resolve_under_root(root: &Path, value: &str) -> PathBuf {
    shared::resolve_under_root(root, value)
}

pub(super) fn resolve_request_path_under_root(
    root: &Path,
    value: &str,
    field: &str,
) -> Result<PathBuf, ToolAdapterError> {
    shared::resolve_request_path_under_root(root, value, field).map_err(ToolAdapterError::Native)
}

pub(super) fn path_for_output(app_root: &Path, path: &Path) -> String {
    shared::path_for_output(app_root, path)
}

pub(super) fn is_image_path(path: &Path) -> bool {
    shared::is_image_path(path)
}

pub(super) fn list_image_files_recursive(input_abs: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    shared::list_image_files_recursive(input_abs)
}
