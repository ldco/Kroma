use std::path::Path;

use crate::pipeline::execution::{
    build_planned_run_log_record, ExecutionPlannedJob, ExecutionPlannedOutputGuardRecord,
    ExecutionPlannedPostprocessRecord, ExecutionPlannedRunLogContext, ExecutionPlannedRunLogRecord,
};
use crate::pipeline::pathing::{path_for_output, resolve_under_root};
use crate::pipeline::planning::{PipelinePlanningOutputGuard, PlannedGenerationJob};

pub struct RunLogPlannedTemplateInput {
    pub project_slug: String,
    pub stage: String,
    pub time: String,
    pub weather: String,
    pub project_root: String,
    pub resolved_from_backend: bool,
    pub candidate_count: u64,
    pub max_candidate_count: u64,
    pub planned_postprocess: ExecutionPlannedPostprocessRecord,
    pub planned_output_guard: ExecutionPlannedOutputGuardRecord,
    pub jobs: Vec<PlannedGenerationJob>,
}

pub struct RunLogPlannedTemplateRequestInput {
    pub project_slug: String,
    pub project_root_override: Option<String>,
    pub stage: String,
    pub time: String,
    pub weather: String,
    pub requested_candidate_count: Option<u64>,
    pub manifest_candidate_count: u64,
    pub manifest_max_candidates: u64,
    pub planned_postprocess: ExecutionPlannedPostprocessRecord,
    pub manifest_output_guard: PipelinePlanningOutputGuard,
    pub jobs: Vec<PlannedGenerationJob>,
}

pub fn build_planned_template(input: RunLogPlannedTemplateInput) -> ExecutionPlannedRunLogRecord {
    let execution_jobs = input
        .jobs
        .into_iter()
        .map(ExecutionPlannedJob::from)
        .collect::<Vec<_>>();
    build_planned_run_log_record(
        ExecutionPlannedRunLogContext {
            timestamp: String::new(),
            project_slug: input.project_slug,
            stage: input.stage,
            time: input.time,
            weather: input.weather,
            project_root: input.project_root,
            resolved_from_backend: input.resolved_from_backend,
            candidate_count: input.candidate_count,
            max_candidate_count: input.max_candidate_count,
            planned_postprocess: input.planned_postprocess,
            planned_output_guard: input.planned_output_guard,
        },
        execution_jobs.as_slice(),
    )
}

pub fn build_planned_template_from_request(
    app_root: &Path,
    input: RunLogPlannedTemplateRequestInput,
) -> ExecutionPlannedRunLogRecord {
    let project_root_abs = input
        .project_root_override
        .as_deref()
        .map(|value| resolve_under_root(app_root, value))
        .unwrap_or_else(|| {
            app_root
                .join("var/projects")
                .join(input.project_slug.as_str())
        });
    let candidate_count = input
        .requested_candidate_count
        .unwrap_or(input.manifest_candidate_count);
    build_planned_template(RunLogPlannedTemplateInput {
        project_slug: input.project_slug,
        stage: input.stage,
        time: input.time,
        weather: input.weather,
        project_root: path_for_output(app_root, project_root_abs.as_path()),
        resolved_from_backend: input.project_root_override.is_some(),
        candidate_count,
        max_candidate_count: input.manifest_max_candidates,
        planned_postprocess: input.planned_postprocess,
        planned_output_guard: planned_output_guard_from_manifest(&input.manifest_output_guard),
        jobs: input.jobs,
    })
}

pub(crate) fn planned_output_guard_from_manifest(
    cfg: &PipelinePlanningOutputGuard,
) -> ExecutionPlannedOutputGuardRecord {
    ExecutionPlannedOutputGuardRecord {
        enabled: true,
        enforce_grayscale: cfg.enforce_grayscale,
        max_chroma_delta: cfg.max_chroma_delta,
        fail_on_chroma_exceed: cfg.fail_on_chroma_exceed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::planning::{PipelinePlanningOutputGuard, PlannedGenerationJob};

    fn sample_planned_postprocess() -> ExecutionPlannedPostprocessRecord {
        ExecutionPlannedPostprocessRecord {
            pipeline_order: vec![String::from("generate")],
            upscale: false,
            upscale_backend: None,
            color: false,
            color_profile: None,
            bg_remove: false,
            bg_remove_backends: Vec::new(),
            bg_refine_openai: false,
            bg_refine_openai_required: false,
        }
    }

    fn sample_jobs() -> Vec<PlannedGenerationJob> {
        vec![PlannedGenerationJob {
            id: String::from("style_1_demo"),
            mode: String::from("style"),
            time: String::from("day"),
            weather: String::from("clear"),
            input_images: vec![String::from("var/projects/demo/scenes/a.png")],
            prompt: String::from("Prompt"),
        }]
    }

    #[test]
    fn build_planned_template_shapes_jobs_and_context() {
        let template = build_planned_template(RunLogPlannedTemplateInput {
            project_slug: String::from("demo"),
            stage: String::from("style"),
            time: String::from("day"),
            weather: String::from("clear"),
            project_root: String::from("var/projects/demo"),
            resolved_from_backend: true,
            candidate_count: 2,
            max_candidate_count: 6,
            planned_postprocess: sample_planned_postprocess(),
            planned_output_guard: ExecutionPlannedOutputGuardRecord {
                enabled: true,
                enforce_grayscale: false,
                max_chroma_delta: 2.0,
                fail_on_chroma_exceed: false,
            },
            jobs: sample_jobs(),
        });

        assert_eq!(template.project, "demo");
        assert_eq!(template.stage, "style");
        assert_eq!(template.storage.project_root, "var/projects/demo");
        assert_eq!(template.generation.candidates, 2);
        assert_eq!(template.generation.max_candidates, 6);
        assert_eq!(template.jobs.len(), 1);
        assert_eq!(template.jobs[0].id, "style_1_demo");
        assert_eq!(template.jobs[0].planned_generation.candidates, 2);
    }

    #[test]
    fn build_planned_template_from_request_resolves_defaults_and_guard() {
        let template = build_planned_template_from_request(
            Path::new("/app-root"),
            RunLogPlannedTemplateRequestInput {
                project_slug: String::from("demo"),
                project_root_override: Some(String::from("custom/root")),
                stage: String::from("style"),
                time: String::from("day"),
                weather: String::from("clear"),
                requested_candidate_count: None,
                manifest_candidate_count: 3,
                manifest_max_candidates: 6,
                planned_postprocess: sample_planned_postprocess(),
                manifest_output_guard: PipelinePlanningOutputGuard {
                    enforce_grayscale: true,
                    max_chroma_delta: 1.5,
                    fail_on_chroma_exceed: true,
                },
                jobs: sample_jobs(),
            },
        );

        assert_eq!(template.project, "demo");
        assert_eq!(template.storage.project_root, "custom/root");
        assert_eq!(template.storage.resolved_from_backend, true);
        assert_eq!(template.generation.candidates, 3);
        assert_eq!(template.output_guard.enabled, true);
        assert_eq!(template.output_guard.enforce_grayscale, true);
        assert_eq!(template.output_guard.max_chroma_delta, 1.5);
        assert_eq!(template.output_guard.fail_on_chroma_exceed, true);
    }
}
