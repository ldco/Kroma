use thiserror::Error;

use crate::pipeline::planning::PlannedGenerationJob;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRunContext {
    pub project_slug: String,
    pub candidates: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlannedJob {
    pub id: String,
    pub input_images: Vec<String>,
    pub prompt: String,
}

impl From<PlannedGenerationJob> for ExecutionPlannedJob {
    fn from(value: PlannedGenerationJob) -> Self {
        Self {
            id: value.id,
            input_images: value.input_images,
            prompt: value.prompt,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateOutputTarget {
    pub file_name: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExecutionPlanningError {
    #[error("candidate index must be >= 1")]
    InvalidCandidateIndex,
}

pub fn candidate_output_file_name(
    job_id: &str,
    candidate_index: u8,
    total_candidates: u8,
) -> Result<CandidateOutputTarget, ExecutionPlanningError> {
    if candidate_index == 0 {
        return Err(ExecutionPlanningError::InvalidCandidateIndex);
    }

    let suffix = if total_candidates > 1 {
        format!("__c{candidate_index}")
    } else {
        String::new()
    };

    Ok(CandidateOutputTarget {
        file_name: format!("{job_id}{suffix}.png"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_file_name_matches_single_candidate_script_behavior() {
        let target = candidate_output_file_name("style_1_scene_01", 1, 1)
            .expect("single candidate file name should build");
        assert_eq!(target.file_name, "style_1_scene_01.png");
    }

    #[test]
    fn candidate_file_name_matches_multi_candidate_script_behavior() {
        let c1 =
            candidate_output_file_name("style_1_scene_01", 1, 3).expect("candidate 1 should build");
        let c3 =
            candidate_output_file_name("style_1_scene_01", 3, 3).expect("candidate 3 should build");

        assert_eq!(c1.file_name, "style_1_scene_01__c1.png");
        assert_eq!(c3.file_name, "style_1_scene_01__c3.png");
    }

    #[test]
    fn candidate_file_name_rejects_zero_index() {
        let err = candidate_output_file_name("job", 0, 1).expect_err("zero index should fail");
        assert_eq!(err, ExecutionPlanningError::InvalidCandidateIndex);
    }
}
