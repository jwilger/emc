use crate::core::effect::{
    Effect, EffectPlan, ProcessArgument, ProcessInvocation, ProgramName, ProjectPath, ReportLine,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GherkinSuite {
    Browser,
    Meta,
    ReviewGate,
    Validator,
}

pub fn list_gherkin_features(suite: GherkinSuite) -> EffectPlan {
    EffectPlan::new(
        suite
            .feature_paths()
            .iter()
            .map(|path| Effect::Report(report_line(path.as_ref())))
            .collect(),
    )
}

pub fn run_gherkin_suite(suite: GherkinSuite) -> EffectPlan {
    EffectPlan::new(vec![run_suite_effect(suite)])
}

pub fn run_all_gherkin_suites() -> EffectPlan {
    EffectPlan::new(
        GherkinSuite::all()
            .into_iter()
            .map(run_suite_effect)
            .collect(),
    )
}

fn run_suite_effect(suite: GherkinSuite) -> Effect {
    let success = report_line(format!(
        "{} Gherkin suite passed; attempted {} configured {} scenarios",
        suite.label(),
        suite.scenario_count(),
        suite.label()
    ));
    Effect::RunProcess(ProcessInvocation::new(
        program_name("cargo"),
        vec![
            process_argument("test"),
            process_argument("--test"),
            process_argument(suite.test_target()),
        ],
        success,
    ))
}

impl GherkinSuite {
    fn all() -> Vec<Self> {
        vec![Self::Browser, Self::ReviewGate, Self::Validator, Self::Meta]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Meta => "meta",
            Self::ReviewGate => "review-gate",
            Self::Validator => "validator",
        }
    }

    fn scenario_count(&self) -> usize {
        match self {
            Self::Browser => 11,
            Self::Meta => 6,
            Self::ReviewGate => 9,
            Self::Validator => 159,
        }
    }

    fn test_target(&self) -> &'static str {
        match self {
            Self::Browser => "browser_composition",
            Self::Meta => "cucumber_runner_config",
            Self::ReviewGate => "review_gate",
            Self::Validator => "validate_event_model",
        }
    }

    fn feature_paths(&self) -> Vec<ProjectPath> {
        match self {
            Self::Browser => vec![project_path(
                "tests/features/event_model_browser/timeline_rendering.feature",
            )],
            Self::Meta => vec![project_path(
                "tests/features/event_model_cucumber_execution.feature",
            )],
            Self::ReviewGate => vec![project_path(
                "tests/features/event_model_review_gate/workflow_review_gate.feature",
            )],
            Self::Validator => vec![
                project_path(
                    "tests/features/event_model_validator/board_timeline_and_workflow.feature",
                ),
                project_path(
                    "tests/features/event_model_validator/outcomes_errors_and_review.feature",
                ),
                project_path("tests/features/event_model_validator/slice_architecture.feature"),
                project_path("tests/features/event_model_validator/structure_and_sources.feature"),
                project_path(
                    "tests/features/event_model_validator/views_controls_and_information.feature",
                ),
            ],
        }
    }
}

fn project_path(value: impl Into<String>) -> ProjectPath {
    ProjectPath::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static feature path must be valid: {error}");
    })
}

fn program_name(value: impl Into<String>) -> ProgramName {
    ProgramName::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Gherkin runner program name must be valid: {error}");
    })
}

fn process_argument(value: impl Into<String>) -> ProcessArgument {
    ProcessArgument::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Gherkin runner argument must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static feature report line must be valid: {error}");
    })
}
