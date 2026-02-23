use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionProjectDirs {
    pub root: PathBuf,
    pub outputs: PathBuf,
    pub runs: PathBuf,
    pub upscaled: PathBuf,
    pub color: PathBuf,
    pub bg_remove: PathBuf,
    pub archive_bad: PathBuf,
    pub archive_replaced: PathBuf,
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

pub fn execution_project_dirs(project_root: &Path) -> ExecutionProjectDirs {
    ExecutionProjectDirs {
        root: project_root.to_path_buf(),
        outputs: project_root.join("outputs"),
        runs: project_root.join("runs"),
        upscaled: project_root.join("upscaled"),
        color: project_root.join("color_corrected"),
        bg_remove: project_root.join("background_removed"),
        archive_bad: project_root.join("archive").join("bad"),
        archive_replaced: project_root.join("archive").join("replaced"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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

    #[test]
    fn execution_project_dirs_match_script_layout() {
        let dirs = execution_project_dirs(Path::new("/tmp/demo"));

        assert_eq!(dirs.outputs, PathBuf::from("/tmp/demo/outputs"));
        assert_eq!(dirs.runs, PathBuf::from("/tmp/demo/runs"));
        assert_eq!(dirs.upscaled, PathBuf::from("/tmp/demo/upscaled"));
        assert_eq!(dirs.color, PathBuf::from("/tmp/demo/color_corrected"));
        assert_eq!(
            dirs.bg_remove,
            PathBuf::from("/tmp/demo/background_removed")
        );
        assert_eq!(dirs.archive_bad, PathBuf::from("/tmp/demo/archive/bad"));
        assert_eq!(
            dirs.archive_replaced,
            PathBuf::from("/tmp/demo/archive/replaced")
        );
    }
}
