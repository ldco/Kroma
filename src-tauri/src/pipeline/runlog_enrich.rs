use crate::pipeline::execution::{
    build_planned_run_log_record, ExecutionPlannedJob, ExecutionPlannedOutputGuardRecord,
    ExecutionPlannedPostprocessRecord, ExecutionPlannedRunLogContext, ExecutionPlannedRunLogRecord,
};
use crate::pipeline::planning::PlannedGenerationJob;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::planning::PlannedGenerationJob;

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
            planned_postprocess: ExecutionPlannedPostprocessRecord {
                pipeline_order: vec![String::from("generate")],
                upscale: false,
                upscale_backend: None,
                color: false,
                color_profile: None,
                bg_remove: false,
                bg_remove_backends: Vec::new(),
                bg_refine_openai: false,
                bg_refine_openai_required: false,
            },
            planned_output_guard: ExecutionPlannedOutputGuardRecord {
                enabled: true,
                enforce_grayscale: false,
                max_chroma_delta: 2.0,
                fail_on_chroma_exceed: false,
            },
            jobs: vec![PlannedGenerationJob {
                id: String::from("style_1_demo"),
                mode: String::from("style"),
                time: String::from("day"),
                weather: String::from("clear"),
                input_images: vec![String::from("var/projects/demo/scenes/a.png")],
                prompt: String::from("Prompt"),
            }],
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
}
