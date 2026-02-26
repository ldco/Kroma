pub mod backend_ops;
pub mod config_validation;
pub mod execution;
pub mod pathing;
pub mod planning;
pub mod post_run;
pub mod postprocess_planning;
pub mod runlog;
pub mod runlog_patch;
pub mod runtime;
pub mod settings_layer;
pub mod tool_adapters;
pub mod trigger;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PipelineStage {
    Style,
    Time,
    Weather,
    OptionalPasses,
}

impl PipelineStage {
    pub const PRODUCTION_ORDER: [Self; 4] =
        [Self::Style, Self::Time, Self::Weather, Self::OptionalPasses];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Style => "style",
            Self::Time => "time",
            Self::Weather => "weather",
            Self::OptionalPasses => "optional_passes",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagePlan {
    pub stages: Vec<PipelineStage>,
}

impl StagePlan {
    pub fn production_default() -> Self {
        Self {
            stages: PipelineStage::PRODUCTION_ORDER.to_vec(),
        }
    }
}
