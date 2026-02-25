use std::fs;
use std::path::Path;

use serde_json::Value;

use super::pathing::{path_for_output, resolve_request_path_under_root};
use super::ToolAdapterError;

pub(super) fn load_color_adapter_config(
    app_root: &Path,
    config_path: Option<&str>,
) -> Result<(Option<String>, Option<String>), ToolAdapterError> {
    let Some(config_path) = config_path.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok((None, None));
    };
    let path = resolve_request_path_under_root(app_root, config_path, "postprocess_config_path")?;
    if !path.is_file() {
        return Err(ToolAdapterError::Native(format!(
            "postprocess config not found: {}",
            path_for_output(app_root, path.as_path())
        )));
    }
    let raw = fs::read_to_string(path.as_path()).map_err(ToolAdapterError::Io)?;
    let parsed: Value =
        serde_json::from_str(raw.as_str()).map_err(|source| ToolAdapterError::JsonDecode {
            source,
            stdout: raw,
        })?;
    let color = parsed.get("color").and_then(Value::as_object);
    let default_profile = color
        .and_then(|obj| obj.get("default_profile"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let settings_file = color
        .and_then(|obj| obj.get("settings_file"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    Ok((default_profile, settings_file))
}
