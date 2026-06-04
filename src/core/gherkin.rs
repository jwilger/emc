use crate::core::effect::{
    Effect, EffectPlan, ProcessArgument, ProcessInvocation, ProgramName, ProjectPath, ReportLine,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GherkinSuite {
    Meta,
    ReviewGate,
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
        vec![Self::ReviewGate, Self::Meta]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Meta => "meta",
            Self::ReviewGate => "review-gate",
        }
    }

    fn scenario_count(&self) -> usize {
        match self {
            Self::Meta => 3,
            Self::ReviewGate => 9,
        }
    }

    fn test_target(&self) -> &'static str {
        match self {
            Self::Meta => "cucumber_runner_config",
            Self::ReviewGate => "review_gate",
        }
    }

    fn feature_paths(&self) -> Vec<ProjectPath> {
        match self {
            Self::Meta => vec![project_path(
                "tests/features/event_model_cucumber_execution.feature",
            )],
            Self::ReviewGate => vec![project_path(
                "tests/features/event_model_review_gate/workflow_review_gate.feature",
            )],
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
