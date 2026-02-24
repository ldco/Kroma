use std::fs;
use std::path::{Component, Path, PathBuf};

use super::ToolAdapterError;

pub(super) fn resolve_under_root(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

pub(super) fn resolve_request_path_under_root(
    root: &Path,
    value: &str,
    field: &str,
) -> Result<PathBuf, ToolAdapterError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ToolAdapterError::Native(format!(
            "{field} must not be empty"
        )));
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(ToolAdapterError::Native(format!(
            "{field} must be a relative path under app root"
        )));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(ToolAdapterError::Native(format!(
            "{field} must stay within app root"
        )));
    }
    Ok(root.join(path))
}

pub(super) fn path_for_output(app_root: &Path, path: &Path) -> String {
    let value = match path.strip_prefix(app_root) {
        Ok(rel) => rel.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    };
    value.replace('\\', "/")
}

pub(super) fn is_image_path(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tif" | "tiff"
    )
}

pub(super) fn list_image_files_recursive(input_abs: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let meta = fs::metadata(input_abs)?;
    if meta.is_file() {
        return Ok(if is_image_path(input_abs) {
            vec![input_abs.to_path_buf()]
        } else {
            Vec::new()
        });
    }

    let mut out = Vec::new();
    let mut entries = fs::read_dir(input_abs)?.collect::<Result<Vec<_>, std::io::Error>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            out.extend(list_image_files_recursive(path.as_path())?);
            continue;
        }
        if file_type.is_file() && is_image_path(path.as_path()) {
            out.push(path);
        }
    }
    Ok(out)
}
