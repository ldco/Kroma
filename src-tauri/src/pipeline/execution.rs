use std::cmp::Ordering;
use std::fs;
use std::io;
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
pub struct FileOutputPathTarget {
    pub path: PathBuf,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionUpscalePathConfig {
    pub scale: u8,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionPostprocessPathConfig {
    pub bg_remove_format: Option<String>,
    pub upscale: Option<ExecutionUpscalePathConfig>,
    pub color_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionCandidatePathPlan {
    pub generated: PathBuf,
    pub bg_remove: Option<PathBuf>,
    pub upscale: Option<PathBuf>,
    pub color: Option<PathBuf>,
    pub final_output: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionCandidateStatus {
    Generated,
    Done,
    FailedOutputGuard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionCandidateRank {
    pub hard_failures: u64,
    pub soft_warnings: u64,
    pub avg_chroma_exceed: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionCandidateResult {
    pub candidate_index: u8,
    pub status: ExecutionCandidateStatus,
    pub rank: ExecutionCandidateRank,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExecutionPlanningError {
    #[error("candidate index must be >= 1")]
    InvalidCandidateIndex,
    #[error("total candidates must be >= 1")]
    InvalidTotalCandidates,
    #[error("upscale scale must be >= 1")]
    InvalidUpscaleScale,
    #[error("upscale format must not be empty")]
    EmptyUpscaleFormat,
    #[error("background remove format must not be empty")]
    EmptyBgRemoveFormat,
    #[error("color profile must not be empty")]
    EmptyColorProfile,
}

pub fn candidate_output_file_name(
    job_id: &str,
    candidate_index: u8,
    total_candidates: u8,
) -> Result<CandidateOutputTarget, ExecutionPlanningError> {
    if candidate_index == 0 {
        return Err(ExecutionPlanningError::InvalidCandidateIndex);
    }
    if total_candidates == 0 {
        return Err(ExecutionPlanningError::InvalidTotalCandidates);
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

pub fn build_file_output_path(
    output_dir: &Path,
    file_name: &str,
    suffix: Option<&str>,
    extension: &str,
) -> FileOutputPathTarget {
    let base_name = Path::new(file_name)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or(file_name);
    let base = sanitize_id(base_name);
    let base = if base.is_empty() {
        String::from("image")
    } else {
        base
    };

    // Keep script parity: any non-empty suffix string reserves the "_" even if sanitization empties it.
    let safe_suffix = suffix
        .filter(|value| !value.is_empty())
        .map(|value| format!("_{}", sanitize_id(value)))
        .unwrap_or_default();

    FileOutputPathTarget {
        path: output_dir.join(format!("{base}{safe_suffix}{extension}")),
    }
}

pub fn plan_candidate_output_paths(
    dirs: &ExecutionProjectDirs,
    job_id: &str,
    candidate_index: u8,
    total_candidates: u8,
    post: &ExecutionPostprocessPathConfig,
) -> Result<ExecutionCandidatePathPlan, ExecutionPlanningError> {
    let generated_name = candidate_output_file_name(job_id, candidate_index, total_candidates)?;
    let generated = dirs.outputs.join(generated_name.file_name);

    let mut current = generated.clone();
    let mut bg_remove = None;
    let mut upscale = None;
    let mut color = None;

    if let Some(format) = post.bg_remove_format.as_deref() {
        let format = format.trim();
        if format.is_empty() {
            return Err(ExecutionPlanningError::EmptyBgRemoveFormat);
        }
        let next = build_file_output_path(
            dirs.bg_remove.as_path(),
            current
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("image.png"),
            Some("nobg"),
            &format!(".{format}"),
        )
        .path;
        current = next.clone();
        bg_remove = Some(next);
    }

    if let Some(upscale_cfg) = post.upscale.as_ref() {
        if upscale_cfg.scale == 0 {
            return Err(ExecutionPlanningError::InvalidUpscaleScale);
        }
        let format = upscale_cfg.format.trim();
        if format.is_empty() {
            return Err(ExecutionPlanningError::EmptyUpscaleFormat);
        }
        let next = build_file_output_path(
            dirs.upscaled.as_path(),
            current
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("image.png"),
            Some(format!("x{}", upscale_cfg.scale).as_str()),
            &format!(".{format}"),
        )
        .path;
        current = next.clone();
        upscale = Some(next);
    }

    if let Some(profile) = post.color_profile.as_deref() {
        let profile = profile.trim();
        if profile.is_empty() {
            return Err(ExecutionPlanningError::EmptyColorProfile);
        }
        let next = build_file_output_path(
            dirs.color.as_path(),
            current
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("image.png"),
            Some(profile),
            ".png",
        )
        .path;
        current = next.clone();
        color = Some(next);
    }

    Ok(ExecutionCandidatePathPlan {
        generated,
        bg_remove,
        upscale,
        color,
        final_output: current,
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

pub fn ensure_generation_mode_dirs(dirs: &ExecutionProjectDirs) -> io::Result<()> {
    fs::create_dir_all(dirs.outputs.as_path())?;
    fs::create_dir_all(dirs.runs.as_path())?;
    fs::create_dir_all(dirs.archive_bad.as_path())?;
    fs::create_dir_all(dirs.archive_replaced.as_path())?;
    Ok(())
}

pub fn pick_best_candidate(
    candidates: &[ExecutionCandidateResult],
) -> Option<&ExecutionCandidateResult> {
    candidates
        .iter()
        .filter(|candidate| matches!(candidate.status, ExecutionCandidateStatus::Done))
        .min_by(|a, b| compare_candidate_rank(a, b))
}

fn compare_candidate_rank(a: &ExecutionCandidateResult, b: &ExecutionCandidateResult) -> Ordering {
    a.rank
        .hard_failures
        .cmp(&b.rank.hard_failures)
        .then_with(|| a.rank.soft_warnings.cmp(&b.rank.soft_warnings))
        .then_with(|| {
            a.rank
                .avg_chroma_exceed
                .total_cmp(&b.rank.avg_chroma_exceed)
        })
        .then_with(|| a.candidate_index.cmp(&b.candidate_index))
}

fn sanitize_id(value: &str) -> String {
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
    fn candidate_file_name_rejects_zero_total_candidates() {
        let err =
            candidate_output_file_name("job", 1, 0).expect_err("zero total candidates should fail");
        assert_eq!(err, ExecutionPlanningError::InvalidTotalCandidates);
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

    #[test]
    fn ensure_generation_mode_dirs_creates_script_parity_directories() {
        let root = std::env::temp_dir().join(format!(
            "kroma_execution_dirs_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be monotonic")
                .as_nanos()
        ));
        let dirs = execution_project_dirs(root.as_path());

        ensure_generation_mode_dirs(&dirs).expect("generation dirs should be created");

        assert!(dirs.outputs.is_dir());
        assert!(dirs.runs.is_dir());
        assert!(dirs.archive_bad.is_dir());
        assert!(dirs.archive_replaced.is_dir());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn build_file_output_path_matches_script_sanitization() {
        let target = build_file_output_path(
            Path::new("/tmp/out"),
            "Foo Bar (Final).PNG",
            Some("x4"),
            ".png",
        );
        assert_eq!(target.path, PathBuf::from("/tmp/out/foo_bar_final_x4.png"));
    }

    #[test]
    fn build_file_output_path_keeps_underscore_for_non_empty_suffix_that_sanitizes_empty() {
        let target = build_file_output_path(Path::new("/tmp/out"), "A.png", Some("!!!"), ".png");
        assert_eq!(target.path, PathBuf::from("/tmp/out/a_.png"));
    }

    #[test]
    fn pick_best_candidate_matches_script_ranking_rules() {
        let candidates = vec![
            ExecutionCandidateResult {
                candidate_index: 3,
                status: ExecutionCandidateStatus::Done,
                rank: ExecutionCandidateRank {
                    hard_failures: 0,
                    soft_warnings: 2,
                    avg_chroma_exceed: 0.5,
                },
            },
            ExecutionCandidateResult {
                candidate_index: 2,
                status: ExecutionCandidateStatus::FailedOutputGuard,
                rank: ExecutionCandidateRank {
                    hard_failures: 9,
                    soft_warnings: 9,
                    avg_chroma_exceed: 9.0,
                },
            },
            ExecutionCandidateResult {
                candidate_index: 1,
                status: ExecutionCandidateStatus::Done,
                rank: ExecutionCandidateRank {
                    hard_failures: 0,
                    soft_warnings: 1,
                    avg_chroma_exceed: 0.8,
                },
            },
        ];

        let winner = pick_best_candidate(&candidates).expect("winner should exist");
        assert_eq!(winner.candidate_index, 1);
    }

    #[test]
    fn pick_best_candidate_breaks_ties_by_candidate_index() {
        let candidates = vec![
            ExecutionCandidateResult {
                candidate_index: 2,
                status: ExecutionCandidateStatus::Done,
                rank: ExecutionCandidateRank {
                    hard_failures: 0,
                    soft_warnings: 0,
                    avg_chroma_exceed: 0.0,
                },
            },
            ExecutionCandidateResult {
                candidate_index: 1,
                status: ExecutionCandidateStatus::Done,
                rank: ExecutionCandidateRank {
                    hard_failures: 0,
                    soft_warnings: 0,
                    avg_chroma_exceed: 0.0,
                },
            },
        ];

        let winner = pick_best_candidate(&candidates).expect("winner should exist");
        assert_eq!(winner.candidate_index, 1);
    }

    #[test]
    fn plan_candidate_output_paths_matches_generate_only_script_behavior() {
        let dirs = execution_project_dirs(Path::new("/tmp/demo"));
        let plan = plan_candidate_output_paths(
            &dirs,
            "style_1_scene_01",
            1,
            1,
            &ExecutionPostprocessPathConfig::default(),
        )
        .expect("plan should build");

        assert_eq!(
            plan.generated,
            PathBuf::from("/tmp/demo/outputs/style_1_scene_01.png")
        );
        assert_eq!(plan.bg_remove, None);
        assert_eq!(plan.upscale, None);
        assert_eq!(plan.color, None);
        assert_eq!(plan.final_output, plan.generated);
    }

    #[test]
    fn plan_candidate_output_paths_matches_script_pass_order_and_naming() {
        let dirs = execution_project_dirs(Path::new("/tmp/demo"));
        let plan = plan_candidate_output_paths(
            &dirs,
            "style_1_scene_01",
            2,
            3,
            &ExecutionPostprocessPathConfig {
                bg_remove_format: Some(String::from("webp")),
                upscale: Some(ExecutionUpscalePathConfig {
                    scale: 4,
                    format: String::from("png"),
                }),
                color_profile: Some(String::from("cinematic-v2")),
            },
        )
        .expect("plan should build");

        assert_eq!(
            plan.generated,
            PathBuf::from("/tmp/demo/outputs/style_1_scene_01__c2.png")
        );
        assert_eq!(
            plan.bg_remove,
            Some(PathBuf::from(
                "/tmp/demo/background_removed/style_1_scene_01__c2_nobg.webp"
            ))
        );
        assert_eq!(
            plan.upscale,
            Some(PathBuf::from(
                "/tmp/demo/upscaled/style_1_scene_01__c2_nobg_x4.png"
            ))
        );
        assert_eq!(
            plan.color,
            Some(PathBuf::from(
                "/tmp/demo/color_corrected/style_1_scene_01__c2_nobg_x4_cinematic-v2.png"
            ))
        );
        assert_eq!(
            plan.final_output,
            PathBuf::from(
                "/tmp/demo/color_corrected/style_1_scene_01__c2_nobg_x4_cinematic-v2.png"
            )
        );
    }

    #[test]
    fn plan_candidate_output_paths_rejects_empty_postprocess_values() {
        let dirs = execution_project_dirs(Path::new("/tmp/demo"));
        let err = plan_candidate_output_paths(
            &dirs,
            "job",
            1,
            1,
            &ExecutionPostprocessPathConfig {
                bg_remove_format: Some(String::from(" ")),
                ..ExecutionPostprocessPathConfig::default()
            },
        )
        .expect_err("empty bg-remove format should fail");
        assert_eq!(err, ExecutionPlanningError::EmptyBgRemoveFormat);

        let err = plan_candidate_output_paths(
            &dirs,
            "job",
            1,
            1,
            &ExecutionPostprocessPathConfig {
                upscale: Some(ExecutionUpscalePathConfig {
                    scale: 0,
                    format: String::from("png"),
                }),
                ..ExecutionPostprocessPathConfig::default()
            },
        )
        .expect_err("zero upscale scale should fail");
        assert_eq!(err, ExecutionPlanningError::InvalidUpscaleScale);

        let err = plan_candidate_output_paths(
            &dirs,
            "job",
            1,
            1,
            &ExecutionPostprocessPathConfig {
                color_profile: Some(String::from("")),
                ..ExecutionPostprocessPathConfig::default()
            },
        )
        .expect_err("empty color profile should fail");
        assert_eq!(err, ExecutionPlanningError::EmptyColorProfile);
    }
}
