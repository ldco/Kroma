use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(super) fn load_dotenv_map(app_root: &Path) -> Result<HashMap<String, String>, std::io::Error> {
    let path = app_root.join(".env");
    if !path.is_file() {
        return Ok(HashMap::new());
    }
    let raw = fs::read_to_string(path)?;
    Ok(parse_dotenv_content(raw.as_str()))
}

pub(super) fn parse_dotenv_content(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in raw.lines() {
        let mut value = line.trim();
        if value.is_empty() || value.starts_with('#') {
            continue;
        }
        if let Some(rest) = value.strip_prefix("export ") {
            value = rest.trim_start();
        }
        let Some((key_raw, value_raw)) = value.split_once('=') else {
            continue;
        };
        let key = key_raw.trim();
        if key.is_empty() {
            continue;
        }
        let mut parsed = value_raw.trim().to_string();
        if (parsed.starts_with('"') && parsed.ends_with('"'))
            || (parsed.starts_with('\'') && parsed.ends_with('\''))
        {
            if parsed.len() >= 2 {
                parsed = parsed[1..parsed.len() - 1].to_string();
            }
        } else if let Some((before_comment, _)) = parsed.split_once(" #") {
            parsed = before_comment.trim_end().to_string();
        }
        out.insert(key.to_string(), parsed);
    }
    out
}
