use std::fs;
use std::path::{Path, PathBuf};

use super::pathing::is_image_path;

pub(super) fn resolve_bgremove_output_path(
    input_file_abs: &Path,
    input_root_abs: &Path,
    output_root_abs: &Path,
    input_is_dir: bool,
    format: &str,
) -> Result<PathBuf, std::io::Error> {
    if !input_is_dir {
        let out_meta = fs::metadata(output_root_abs).ok();
        let out_str = output_root_abs.to_string_lossy();
        let as_dir = out_str.ends_with('/')
            || out_str.ends_with('\\')
            || out_meta.as_ref().map(|m| m.is_dir()).unwrap_or(false)
            || !is_image_path(output_root_abs);
        if as_dir {
            fs::create_dir_all(output_root_abs)?;
            let base_raw = input_file_abs
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or("image");
            let base = {
                let s = sanitize_id(base_raw);
                if s.is_empty() {
                    String::from("image")
                } else {
                    s
                }
            };
            return Ok(output_root_abs.join(format!("{base}.{format}")));
        }
        if let Some(parent) = output_root_abs.parent() {
            fs::create_dir_all(parent)?;
        }
        return Ok(output_root_abs.to_path_buf());
    }

    let rel = input_file_abs
        .strip_prefix(input_root_abs)
        .unwrap_or(input_file_abs);
    let mut dst = output_root_abs.join(rel);
    dst.set_extension(format);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(dst)
}

pub(super) fn sanitize_id(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_was_sep = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        let keep = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_';
        if keep {
            out.push(ch);
            last_was_sep = false;
            continue;
        }
        if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

pub(super) fn make_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{:03}", now.as_secs(), now.subsec_millis())
}

pub(super) fn archive_existing_target(
    target_abs: &Path,
    archive_dir_abs: &Path,
    tag: &str,
) -> Result<Option<PathBuf>, std::io::Error> {
    if !target_abs.exists() {
        return Ok(None);
    }
    if !fs::metadata(target_abs)?.is_file() {
        return Ok(None);
    }
    fs::create_dir_all(archive_dir_abs)?;
    let ext = target_abs
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let base_raw = target_abs
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or("file");
    let base = {
        let s = sanitize_id(base_raw);
        if s.is_empty() {
            String::from("file")
        } else {
            s
        }
    };
    let archived = archive_dir_abs.join(format!("{base}_{tag}_{}{}", make_stamp(), ext));
    fs::rename(target_abs, archived.as_path())?;
    Ok(Some(archived))
}

pub(super) fn mime_for_path(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|v| v.to_str())
        .map(|v| v.trim().to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" => String::from("image/png"),
        "jpg" | "jpeg" => String::from("image/jpeg"),
        "webp" => String::from("image/webp"),
        "bmp" => String::from("image/bmp"),
        "tif" | "tiff" => String::from("image/tiff"),
        "gif" => String::from("image/gif"),
        _ => String::from("application/octet-stream"),
    }
}
