use crate::core::effect::{Effect, EffectPlan, ProjectPath, ReportLine};

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
    EffectPlan::new(vec![Effect::Fail(report_line(format!(
        "undefined, pending, or skipped Gherkin steps are failures for {}; attempted {} configured {} scenarios",
        suite.label(),
        suite.scenario_count(),
        suite.label()
    )))])
}

impl GherkinSuite {
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

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC static feature report line must be valid: {error}");
    })
}
