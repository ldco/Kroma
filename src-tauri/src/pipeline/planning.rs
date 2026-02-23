use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;
use thiserror::Error;

use crate::pipeline::runtime::{PipelineStageFilter, PipelineTimeFilter, PipelineWeatherFilter};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PipelinePlanningPolicy {
    pub default_no_invention: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PipelinePlanningManifest {
    pub prompts: HashMap<String, String>,
    pub scene_refs: Vec<String>,
    pub style_refs: Vec<String>,
    pub policy: PipelinePlanningPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedGenerationJob {
    pub id: String,
    pub mode: String,
    pub time: String,
    pub weather: String,
    pub input_images: Vec<String>,
    pub prompt: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlanningError {
    #[error("missing prompts.style_base")]
    MissingStyleBasePrompt,
    #[error("missing prompts.{0}")]
    MissingPrompt(String),
    #[error("no scene references configured")]
    NoSceneReferences,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlanningManifestError {
    #[error("planning manifest must be a JSON object")]
    RootMustBeObject,
    #[error("planning manifest field '{field}' must be an array of strings")]
    ArrayOfStrings { field: String },
    #[error("planning manifest field '{field}' must not contain empty strings")]
    EmptyStringInArray { field: String },
    #[error("planning manifest field '{field}' must be a string")]
    StringField { field: String },
    #[error("planning manifest field '{field}' must be an object")]
    ObjectField { field: String },
    #[error("planning manifest field '{field}' must be a boolean")]
    BoolField { field: String },
    #[error("failed to read planning manifest '{path}': {message}")]
    ReadFile { path: String, message: String },
    #[error("failed to parse planning manifest JSON '{path}': {message}")]
    ParseJson { path: String, message: String },
}

pub fn default_planning_manifest() -> PipelinePlanningManifest {
    let mut prompts = HashMap::new();
    prompts.insert(
        String::from("style_base"),
        String::from(
            "Preserve geometry and perspective. Apply one coherent noir drawing hand. No text, no logos, no watermark.",
        ),
    );
    prompts.insert(
        String::from("time_day"),
        String::from("Daylight scene with clear readability and stable contrast family."),
    );
    prompts.insert(
        String::from("time_night"),
        String::from("Night scene with controlled deep shadows, no topology changes."),
    );
    prompts.insert(
        String::from("weather_clear"),
        String::from(
            "Dry clear atmosphere. No rain streaks, no snow particles, no diagonal sky hatching.",
        ),
    );
    prompts.insert(
        String::from("weather_rain"),
        String::from("Visible wet surfaces, puddles, and reflection cues; no style drift."),
    );

    PipelinePlanningManifest {
        prompts,
        scene_refs: Vec::new(),
        style_refs: Vec::new(),
        policy: PipelinePlanningPolicy {
            default_no_invention: true,
        },
    }
}

pub fn parse_planning_manifest_json(
    value: &Value,
) -> Result<PipelinePlanningManifest, PlanningManifestError> {
    let root = value
        .as_object()
        .ok_or(PlanningManifestError::RootMustBeObject)?;
    let mut manifest = default_planning_manifest();

    if let Some(scene_refs) = root.get("scene_refs") {
        manifest.scene_refs = parse_string_array_field(scene_refs, "scene_refs")?;
    }
    if let Some(style_refs) = root.get("style_refs") {
        manifest.style_refs = parse_string_array_field(style_refs, "style_refs")?;
    }
    if let Some(policy) = root.get("policy") {
        let policy_obj = policy
            .as_object()
            .ok_or_else(|| PlanningManifestError::ObjectField {
                field: String::from("policy"),
            })?;
        if let Some(default_no_invention) = policy_obj.get("default_no_invention") {
            manifest.policy.default_no_invention =
                default_no_invention
                    .as_bool()
                    .ok_or_else(|| PlanningManifestError::BoolField {
                        field: String::from("policy.default_no_invention"),
                    })?;
        }
    }
    if let Some(prompts) = root.get("prompts") {
        let prompts_obj =
            prompts
                .as_object()
                .ok_or_else(|| PlanningManifestError::ObjectField {
                    field: String::from("prompts"),
                })?;
        for (key, raw_value) in prompts_obj {
            let prompt_value = raw_value.as_str().map(str::trim).ok_or_else(|| {
                PlanningManifestError::StringField {
                    field: format!("prompts.{key}"),
                }
            })?;
            manifest
                .prompts
                .insert(key.clone(), prompt_value.to_string());
        }
    }

    Ok(manifest)
}

pub fn load_planning_manifest_file(
    path: &Path,
) -> Result<PipelinePlanningManifest, PlanningManifestError> {
    let raw = fs::read_to_string(path).map_err(|error| PlanningManifestError::ReadFile {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    let parsed = serde_json::from_str::<Value>(raw.as_str()).map_err(|error| {
        PlanningManifestError::ParseJson {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    parse_planning_manifest_json(&parsed)
}

pub fn compose_prompt(
    prompts: &HashMap<String, String>,
    stage: PipelineStageFilter,
    time: PipelineTimeFilter,
    weather: PipelineWeatherFilter,
    no_invention: bool,
) -> Result<String, PlanningError> {
    let mut chunks = Vec::new();

    let style_base = prompts
        .get("style_base")
        .map(String::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(PlanningError::MissingStyleBasePrompt)?;
    chunks.push(style_base.to_string());

    if matches!(
        stage,
        PipelineStageFilter::Time | PipelineStageFilter::Weather
    ) {
        let key = format!("time_{}", time.as_str());
        let value = prompts
            .get(key.as_str())
            .map(String::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| PlanningError::MissingPrompt(key.clone()))?;
        chunks.push(value.to_string());
    }

    if matches!(stage, PipelineStageFilter::Weather) {
        let key = format!("weather_{}", weather.as_str());
        let value = prompts
            .get(key.as_str())
            .map(String::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| PlanningError::MissingPrompt(key.clone()))?;
        chunks.push(value.to_string());
    }

    if no_invention {
        chunks.push(String::from("Do not invent new object categories."));
    }

    Ok(chunks.join(" "))
}

pub fn build_generation_jobs(
    manifest: &PipelinePlanningManifest,
    stage: PipelineStageFilter,
    time: PipelineTimeFilter,
    weather: PipelineWeatherFilter,
) -> Result<Vec<PlannedGenerationJob>, PlanningError> {
    if manifest.scene_refs.is_empty() {
        return Err(PlanningError::NoSceneReferences);
    }

    let prompt = compose_prompt(
        &manifest.prompts,
        stage,
        time,
        weather,
        manifest.policy.default_no_invention,
    )?;

    Ok(manifest
        .scene_refs
        .iter()
        .enumerate()
        .map(|(idx, scene)| {
            let id_suffix = sanitize_scene_id(scene).unwrap_or_else(|| (idx + 1).to_string());
            let mut input_images = Vec::with_capacity(1 + manifest.style_refs.len());
            input_images.push(scene.clone());
            input_images.extend(manifest.style_refs.iter().cloned());
            PlannedGenerationJob {
                id: format!("{}_{}_{}", stage.as_str(), idx + 1, id_suffix),
                mode: stage.as_str().to_string(),
                time: time.as_str().to_string(),
                weather: weather.as_str().to_string(),
                input_images,
                prompt: prompt.clone(),
            }
        })
        .collect())
}

fn sanitize_scene_id(scene_path: &str) -> Option<String> {
    let file_name = Path::new(scene_path).file_name()?.to_str()?;
    let stem = Path::new(file_name).file_stem()?.to_str()?;
    sanitize_id(stem)
}

fn parse_string_array_field(
    value: &Value,
    field: &str,
) -> Result<Vec<String>, PlanningManifestError> {
    let arr = value
        .as_array()
        .ok_or_else(|| PlanningManifestError::ArrayOfStrings {
            field: field.to_string(),
        })?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let item_str =
            item.as_str()
                .map(str::trim)
                .ok_or_else(|| PlanningManifestError::ArrayOfStrings {
                    field: field.to_string(),
                })?;
        if item_str.is_empty() {
            return Err(PlanningManifestError::EmptyStringInArray {
                field: field.to_string(),
            });
        }
        out.push(item_str.to_string());
    }
    Ok(out)
}

fn sanitize_id(value: &str) -> Option<String> {
    let mut out = String::new();
    let mut last_was_underscore = false;

    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            ch.to_ascii_lowercase()
        } else {
            '_'
        };

        if normalized == '_' {
            if out.is_empty() || last_was_underscore {
                last_was_underscore = true;
                continue;
            }
            out.push('_');
            last_was_underscore = true;
            continue;
        }

        out.push(normalized);
        last_was_underscore = false;
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest() -> PipelinePlanningManifest {
        let mut manifest = default_planning_manifest();
        manifest
            .prompts
            .insert(String::from("style_base"), String::from("Base style."));
        manifest
            .prompts
            .insert(String::from("time_day"), String::from("Day lighting."));
        manifest
            .prompts
            .insert(String::from("time_night"), String::from("Night lighting."));
        manifest.prompts.insert(
            String::from("weather_clear"),
            String::from("Clear weather."),
        );
        manifest
            .prompts
            .insert(String::from("weather_rain"), String::from("Rain weather."));
        manifest.scene_refs = vec![
            String::from("var/projects/demo/scenes/Scene 01.png"),
            String::from("var/projects/demo/scenes/Scene 02.png"),
        ];
        manifest.style_refs = vec![String::from("var/projects/demo/styles/s1.png")];
        manifest.policy.default_no_invention = true;
        manifest
    }

    #[test]
    fn compose_prompt_follows_stage_rules() {
        let manifest = test_manifest();

        let prompt = compose_prompt(
            &manifest.prompts,
            PipelineStageFilter::Weather,
            PipelineTimeFilter::Night,
            PipelineWeatherFilter::Rain,
            true,
        )
        .expect("prompt should compose");

        assert!(prompt.contains("Base style."));
        assert!(prompt.contains("Night lighting."));
        assert!(prompt.contains("Rain weather."));
        assert!(prompt.contains("Do not invent new object categories."));
    }

    #[test]
    fn build_generation_jobs_uses_per_scene_ids_and_includes_style_refs() {
        let manifest = test_manifest();

        let jobs = build_generation_jobs(
            &manifest,
            PipelineStageFilter::Style,
            PipelineTimeFilter::Day,
            PipelineWeatherFilter::Clear,
        )
        .expect("jobs should build");

        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].id, "style_1_scene_01");
        assert_eq!(jobs[1].id, "style_2_scene_02");
        assert_eq!(jobs[0].input_images.len(), 2);
        assert_eq!(
            jobs[0].input_images[0],
            "var/projects/demo/scenes/Scene 01.png"
        );
        assert_eq!(jobs[0].input_images[1], "var/projects/demo/styles/s1.png");
        assert_eq!(jobs[0].mode, "style");
        assert_eq!(jobs[0].time, "day");
        assert_eq!(jobs[0].weather, "clear");
    }

    #[test]
    fn build_generation_jobs_errors_without_scenes() {
        let mut manifest = test_manifest();
        manifest.scene_refs.clear();

        let err = build_generation_jobs(
            &manifest,
            PipelineStageFilter::Style,
            PipelineTimeFilter::Day,
            PipelineWeatherFilter::Clear,
        )
        .expect_err("missing scenes should fail");

        assert_eq!(err, PlanningError::NoSceneReferences);
    }

    #[test]
    fn parse_planning_manifest_json_merges_defaults_and_overrides() {
        let parsed = parse_planning_manifest_json(&serde_json::json!({
            "scene_refs": ["a.png", " b.png "],
            "style_refs": ["style.png"],
            "policy": { "default_no_invention": false },
            "prompts": {
                "style_base": "Custom base",
                "time_day": "Custom day"
            }
        }))
        .expect("manifest should parse");

        assert_eq!(parsed.scene_refs, vec!["a.png", "b.png"]);
        assert_eq!(parsed.style_refs, vec!["style.png"]);
        assert!(!parsed.policy.default_no_invention);
        assert_eq!(
            parsed.prompts.get("style_base").map(String::as_str),
            Some("Custom base")
        );
        assert_eq!(
            parsed.prompts.get("time_day").map(String::as_str),
            Some("Custom day")
        );
        assert!(parsed.prompts.contains_key("weather_rain"));
    }

    #[test]
    fn parse_planning_manifest_json_rejects_invalid_types() {
        let err = parse_planning_manifest_json(&serde_json::json!({
            "scene_refs": ["a.png", 1]
        }))
        .expect_err("non-string scene ref should fail");
        assert_eq!(
            err,
            PlanningManifestError::ArrayOfStrings {
                field: String::from("scene_refs")
            }
        );

        let err = parse_planning_manifest_json(&serde_json::json!({
            "policy": { "default_no_invention": "yes" }
        }))
        .expect_err("non-bool policy should fail");
        assert_eq!(
            err,
            PlanningManifestError::BoolField {
                field: String::from("policy.default_no_invention")
            }
        );
    }

    #[test]
    fn parse_planning_manifest_json_rejects_empty_array_entries() {
        let err = parse_planning_manifest_json(&serde_json::json!({
            "scene_refs": ["a.png", "   "]
        }))
        .expect_err("empty scene ref entry should fail");

        assert_eq!(
            err,
            PlanningManifestError::EmptyStringInArray {
                field: String::from("scene_refs")
            }
        );
    }
}
