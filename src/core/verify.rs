use crate::core::effect::{
    Effect, EffectPlan, ProcessArgument, ProcessInvocation, ProgramName, ReportLine,
};
use crate::core::layout::ModeledWorkflowLayout;

pub fn verify_project(modeled_workflows: Vec<ModeledWorkflowLayout>) -> EffectPlan {
    EffectPlan::new(
        modeled_workflows
            .into_iter()
            .flat_map(verify_modeled_workflow)
            .collect(),
    )
}

fn verify_modeled_workflow(workflow: ModeledWorkflowLayout) -> Vec<Effect> {
    vec![
        Effect::RunProcess(ProcessInvocation::new(
            program_name("lake"),
            vec![
                process_argument("env"),
                process_argument("lean"),
                process_argument(workflow.lean_artifact_path().as_ref().to_owned()),
            ],
            report_line("Lean4 artifacts verified"),
        )),
        Effect::RunProcess(ProcessInvocation::new(
            program_name("quint"),
            vec![
                process_argument("verify"),
                process_argument("--invariant"),
                process_argument(
                    "workflowIdentityStable,workflowSliceDetailsComplete,workflowTransitionsStructured",
                ),
                process_argument(workflow.quint_artifact_path().as_ref().to_owned()),
            ],
            report_line("Quint artifacts verified"),
        )),
    ]
}

fn program_name(value: impl Into<String>) -> ProgramName {
    ProgramName::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated program name must be valid: {error}");
    })
}

fn process_argument(value: impl Into<String>) -> ProcessArgument {
    ProcessArgument::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated process argument must be valid: {error}");
    })
}

fn report_line(value: impl Into<String>) -> ReportLine {
    ReportLine::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated report line must be valid: {error}");
    })
}
