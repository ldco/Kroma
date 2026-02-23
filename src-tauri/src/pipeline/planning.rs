use std::collections::HashMap;
use std::path::Path;

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
        let mut prompts = HashMap::new();
        prompts.insert(String::from("style_base"), String::from("Base style."));
        prompts.insert(String::from("time_day"), String::from("Day lighting."));
        prompts.insert(String::from("time_night"), String::from("Night lighting."));
        prompts.insert(
            String::from("weather_clear"),
            String::from("Clear weather."),
        );
        prompts.insert(String::from("weather_rain"), String::from("Rain weather."));

        PipelinePlanningManifest {
            prompts,
            scene_refs: vec![
                String::from("var/projects/demo/scenes/Scene 01.png"),
                String::from("var/projects/demo/scenes/Scene 02.png"),
            ],
            style_refs: vec![String::from("var/projects/demo/styles/s1.png")],
            policy: PipelinePlanningPolicy {
                default_no_invention: true,
            },
        }
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
}
