use thiserror::Error;

use crate::pipeline::runtime::{
    PipelineInputSource, PipelineRunMode, PipelineRunOptions, PipelineRunRequest,
    PipelineRunResult, PipelineRuntimeError, PipelineStageFilter, PipelineTimeFilter,
    PipelineWeatherFilter, SharedPipelineOrchestrator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerMode {
    Dry,
    Run,
}

impl TriggerMode {
    fn to_pipeline_mode(self) -> PipelineRunMode {
        match self {
            Self::Dry => PipelineRunMode::Dry,
            Self::Run => PipelineRunMode::Run,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerStage {
    Style,
    Time,
    Weather,
}

impl TriggerStage {
    fn to_pipeline_stage(self) -> PipelineStageFilter {
        match self {
            Self::Style => PipelineStageFilter::Style,
            Self::Time => PipelineStageFilter::Time,
            Self::Weather => PipelineStageFilter::Weather,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerTime {
    Day,
    Night,
}

impl TriggerTime {
    fn to_pipeline_time(self) -> PipelineTimeFilter {
        match self {
            Self::Day => PipelineTimeFilter::Day,
            Self::Night => PipelineTimeFilter::Night,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerWeather {
    Clear,
    Rain,
}

impl TriggerWeather {
    fn to_pipeline_weather(self) -> PipelineWeatherFilter {
        match self {
            Self::Clear => PipelineWeatherFilter::Clear,
            Self::Rain => PipelineWeatherFilter::Rain,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TriggerRunParams {
    pub project_root: Option<String>,
    pub input: Option<String>,
    pub scene_refs: Option<Vec<String>>,
    pub style_refs: Vec<String>,
    pub stage: Option<TriggerStage>,
    pub time: Option<TriggerTime>,
    pub weather: Option<TriggerWeather>,
    pub candidates: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerPipelineInput {
    pub project_slug: String,
    pub mode: TriggerMode,
    pub confirm_spend: bool,
    pub params: TriggerRunParams,
}

#[derive(Clone)]
pub struct PipelineTriggerService {
    orchestrator: SharedPipelineOrchestrator,
}

impl PipelineTriggerService {
    pub fn new(orchestrator: SharedPipelineOrchestrator) -> Self {
        Self { orchestrator }
    }

    pub fn trigger(
        &self,
        input: TriggerPipelineInput,
    ) -> Result<PipelineRunResult, PipelineTriggerError> {
        let TriggerPipelineInput {
            project_slug,
            mode,
            confirm_spend,
            params,
        } = input;
        validate_trigger_input(mode, confirm_spend, &params)?;

        let TriggerRunParams {
            project_root,
            input,
            scene_refs,
            style_refs,
            stage,
            time,
            weather,
            candidates,
        } = params;

        let confirm_spend = matches!(mode, TriggerMode::Run);

        let input_source = if let Some(input_path) = input {
            Some(PipelineInputSource::InputPath(input_path))
        } else {
            scene_refs.map(PipelineInputSource::SceneRefs)
        };

        self.orchestrator
            .execute(&PipelineRunRequest {
                project_slug,
                mode: mode.to_pipeline_mode(),
                confirm_spend,
                options: PipelineRunOptions {
                    project_root,
                    input_source,
                    style_refs,
                    stage: stage.map(TriggerStage::to_pipeline_stage),
                    time: time.map(TriggerTime::to_pipeline_time),
                    weather: weather.map(TriggerWeather::to_pipeline_weather),
                    candidates,
                    backend_db_ingest: None,
                    storage_sync_s3: None,
                },
            })
            .map_err(PipelineTriggerError::Runtime)
    }
}

pub fn validate_trigger_input(
    mode: TriggerMode,
    confirm_spend: bool,
    params: &TriggerRunParams,
) -> Result<(), PipelineTriggerError> {
    if params.input.is_some() && params.scene_refs.is_some() {
        return Err(PipelineTriggerError::InvalidRequest(String::from(
            "Provide only one of: input, scene_refs",
        )));
    }
    if params.input.is_none() && params.scene_refs.is_none() {
        return Err(PipelineTriggerError::InvalidRequest(String::from(
            "Provide one of: input, scene_refs",
        )));
    }

    validate_stage_parameter_usage(params.stage, params.time, params.weather)?;

    if matches!(mode, TriggerMode::Run) && !confirm_spend {
        return Err(PipelineTriggerError::MissingSpendConfirmation);
    }

    Ok(())
}

fn validate_stage_parameter_usage(
    stage: Option<TriggerStage>,
    time: Option<TriggerTime>,
    weather: Option<TriggerWeather>,
) -> Result<(), PipelineTriggerError> {
    if time.is_some()
        && !matches!(
            stage,
            Some(TriggerStage::Time) | Some(TriggerStage::Weather)
        )
    {
        return Err(PipelineTriggerError::InvalidRequest(String::from(
            "Field 'time' requires stage 'time' or 'weather'",
        )));
    }

    if weather.is_some() && !matches!(stage, Some(TriggerStage::Weather)) {
        return Err(PipelineTriggerError::InvalidRequest(String::from(
            "Field 'weather' requires stage 'weather'",
        )));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum PipelineTriggerError {
    #[error("run mode requires explicit spend confirmation")]
    MissingSpendConfirmation,
    #[error("{0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Runtime(#[from] PipelineRuntimeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::runtime::{PipelineOrchestrator, PipelineRunRequest};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct FakeOrchestrator {
        seen: Mutex<Vec<PipelineRunRequest>>,
        next: Mutex<Option<Result<PipelineRunResult, PipelineRuntimeError>>>,
    }

    impl FakeOrchestrator {
        fn with_next(result: Result<PipelineRunResult, PipelineRuntimeError>) -> Self {
            Self {
                seen: Mutex::new(Vec::new()),
                next: Mutex::new(Some(result)),
            }
        }

        fn take_seen(&self) -> Vec<PipelineRunRequest> {
            std::mem::take(&mut *self.seen.lock().expect("fake orchestrator mutex poisoned"))
        }
    }

    impl PipelineOrchestrator for FakeOrchestrator {
        fn execute(
            &self,
            request: &PipelineRunRequest,
        ) -> Result<PipelineRunResult, PipelineRuntimeError> {
            self.seen
                .lock()
                .expect("fake orchestrator mutex poisoned")
                .push(request.clone());
            self.next
                .lock()
                .expect("fake orchestrator mutex poisoned")
                .take()
                .unwrap_or_else(|| {
                    Ok(PipelineRunResult {
                        status_code: 0,
                        stdout: String::new(),
                        stderr: String::new(),
                    })
                })
        }
    }

    #[test]
    fn dry_mode_does_not_require_confirmation() {
        let orchestrator = Arc::new(FakeOrchestrator::with_next(Ok(PipelineRunResult {
            status_code: 0,
            stdout: String::from("ok"),
            stderr: String::new(),
        })));
        let service = PipelineTriggerService::new(orchestrator.clone());

        let result = service
            .trigger(TriggerPipelineInput {
                project_slug: String::from("demo"),
                mode: TriggerMode::Dry,
                confirm_spend: false,
                params: TriggerRunParams {
                    scene_refs: Some(vec![String::from("a.png")]),
                    ..TriggerRunParams::default()
                },
            })
            .expect("dry trigger should succeed");
        assert_eq!(result.stdout, "ok");

        let seen = orchestrator.take_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].mode, PipelineRunMode::Dry);
        assert!(!seen[0].confirm_spend);
        assert_eq!(
            seen[0].options.input_source,
            Some(PipelineInputSource::SceneRefs(vec![String::from("a.png")]))
        );
    }

    #[test]
    fn run_mode_requires_confirmation() {
        let orchestrator = Arc::new(FakeOrchestrator::default());
        let service = PipelineTriggerService::new(orchestrator.clone());

        let err = service
            .trigger(TriggerPipelineInput {
                project_slug: String::from("demo"),
                mode: TriggerMode::Run,
                confirm_spend: false,
                params: TriggerRunParams {
                    scene_refs: Some(vec![String::from("a.png")]),
                    ..TriggerRunParams::default()
                },
            })
            .expect_err("run mode without confirmation should fail");

        assert!(matches!(
            err,
            PipelineTriggerError::MissingSpendConfirmation
        ));
        assert!(orchestrator.take_seen().is_empty());
    }

    #[test]
    fn run_mode_injects_confirm_flag_once() {
        let orchestrator = Arc::new(FakeOrchestrator::with_next(Ok(PipelineRunResult {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        })));
        let service = PipelineTriggerService::new(orchestrator.clone());

        service
            .trigger(TriggerPipelineInput {
                project_slug: String::from("demo"),
                mode: TriggerMode::Run,
                confirm_spend: true,
                params: TriggerRunParams {
                    scene_refs: Some(vec![String::from("a.png")]),
                    ..TriggerRunParams::default()
                },
            })
            .expect("run trigger should succeed");

        let seen = orchestrator.take_seen();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].mode, PipelineRunMode::Run);
        assert!(seen[0].confirm_spend);
    }

    #[test]
    fn rejects_conflicting_input_and_scene_refs() {
        let orchestrator = Arc::new(FakeOrchestrator::default());
        let service = PipelineTriggerService::new(orchestrator.clone());

        let err = service
            .trigger(TriggerPipelineInput {
                project_slug: String::from("demo"),
                mode: TriggerMode::Dry,
                confirm_spend: false,
                params: TriggerRunParams {
                    input: Some(String::from("folder")),
                    scene_refs: Some(vec![String::from("a.png")]),
                    ..TriggerRunParams::default()
                },
            })
            .expect_err("conflicting inputs should fail");

        assert!(matches!(err, PipelineTriggerError::InvalidRequest(_)));
        assert!(orchestrator.take_seen().is_empty());
    }

    #[test]
    fn rejects_missing_input_and_scene_refs() {
        let orchestrator = Arc::new(FakeOrchestrator::default());
        let service = PipelineTriggerService::new(orchestrator.clone());

        let err = service
            .trigger(TriggerPipelineInput {
                project_slug: String::from("demo"),
                mode: TriggerMode::Dry,
                confirm_spend: false,
                params: TriggerRunParams::default(),
            })
            .expect_err("missing input source should fail");

        assert!(matches!(err, PipelineTriggerError::InvalidRequest(_)));
        assert!(orchestrator.take_seen().is_empty());
    }

    #[test]
    fn rejects_time_without_time_or_weather_stage() {
        let orchestrator = Arc::new(FakeOrchestrator::default());
        let service = PipelineTriggerService::new(orchestrator.clone());

        let err = service
            .trigger(TriggerPipelineInput {
                project_slug: String::from("demo"),
                mode: TriggerMode::Dry,
                confirm_spend: false,
                params: TriggerRunParams {
                    scene_refs: Some(vec![String::from("a.png")]),
                    time: Some(TriggerTime::Night),
                    ..TriggerRunParams::default()
                },
            })
            .expect_err("time without matching stage should fail");

        assert!(matches!(err, PipelineTriggerError::InvalidRequest(_)));
        assert!(orchestrator.take_seen().is_empty());
    }

    #[test]
    fn rejects_weather_without_weather_stage() {
        let orchestrator = Arc::new(FakeOrchestrator::default());
        let service = PipelineTriggerService::new(orchestrator.clone());

        let err = service
            .trigger(TriggerPipelineInput {
                project_slug: String::from("demo"),
                mode: TriggerMode::Dry,
                confirm_spend: false,
                params: TriggerRunParams {
                    scene_refs: Some(vec![String::from("a.png")]),
                    stage: Some(TriggerStage::Time),
                    weather: Some(TriggerWeather::Rain),
                    ..TriggerRunParams::default()
                },
            })
            .expect_err("weather without weather stage should fail");

        assert!(matches!(err, PipelineTriggerError::InvalidRequest(_)));
        assert!(orchestrator.take_seen().is_empty());
    }
}
