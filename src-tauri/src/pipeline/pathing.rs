use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn resolve_under_root(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

/// Resolve a request path under root with canonical path containment check.
/// Prevents symlink traversal attacks by resolving the final target and verifying
/// it remains within the app root boundary.
/// For non-existent paths, validates the nearest existing ancestor against canonical root.
pub fn resolve_request_path_under_root(
    root: &Path,
    value: &str,
    field: &str,
) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(format!("{field} must be a relative path under app root"));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!("{field} must stay within app root"));
    }
    
    let candidate = root.join(path);
    
    // Canonical path containment check to prevent symlink traversal
    // For existing paths: check the path itself
    // For non-existent paths: check the nearest existing ancestor
    let canonical_root = root.canonicalize()
        .map_err(|e| format!("Failed to resolve app root: {e}"))?;
    
    let check_path = if candidate.exists() {
        candidate.canonicalize()
            .map_err(|e| format!("Failed to resolve path: {e}"))?
    } else {
        // Find nearest existing ancestor
        let mut ancestor = candidate.as_path();
        while !ancestor.exists() {
            ancestor = ancestor.parent()
                .ok_or_else(|| format!("{field} has no valid ancestor within root"))?;
        }
        ancestor.canonicalize()
            .map_err(|e| format!("Failed to resolve ancestor path: {e}"))?
    };
    
    if !check_path.starts_with(&canonical_root) {
        return Err(format!("{field} resolved outside app root (symlink traversal detected)"));
    }
    
    Ok(candidate)
}

pub fn path_for_output(app_root: &Path, path: &Path) -> String {
    let value = match path.strip_prefix(app_root) {
        Ok(rel) => rel.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    };
    value.replace('\\', "/")
}

pub fn is_image_path(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tif" | "tiff"
    )
}

pub fn list_image_files_recursive(input_abs: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
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
