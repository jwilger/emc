use std::collections::BTreeSet;

use crate::core::effect::{
    Effect, EffectPlan, ProcessArgument, ProcessInvocation, ProgramName, ReportLine,
};
use crate::core::layout::ModeledWorkflowLayout;
use crate::core::types::WorkflowSliceDetail;

pub fn verify_project(
    modeled_workflows: Vec<ModeledWorkflowLayout>,
    workflow_slice_details: Vec<WorkflowSliceDetail>,
) -> EffectPlan {
    EffectPlan::new(
        modeled_workflows
            .into_iter()
            .flat_map(verify_modeled_workflow)
            .chain(verify_modeled_slices(workflow_slice_details))
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

fn verify_modeled_slices(workflow_slice_details: Vec<WorkflowSliceDetail>) -> Vec<Effect> {
    workflow_slice_details
        .into_iter()
        .map(|slice| module_name_from_raw(slice.name().as_ref()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .flat_map(verify_modeled_slice)
        .collect()
}

fn verify_modeled_slice(module_name: String) -> Vec<Effect> {
    vec![
        Effect::RunProcess(ProcessInvocation::new(
            program_name("lake"),
            vec![
                process_argument("env"),
                process_argument("lean"),
                process_argument(format!("model/lean/slices/{module_name}.lean")),
            ],
            report_line("Lean4 artifacts verified"),
        )),
        Effect::RunProcess(ProcessInvocation::new(
            program_name("quint"),
            vec![
                process_argument("verify"),
                process_argument("--invariant"),
                process_argument("sliceIdentityStable"),
                process_argument(format!("model/quint/slices/{module_name}.qnt")),
            ],
            report_line("Quint artifacts verified"),
        )),
    ]
}

fn module_name_from_raw(raw: &str) -> String {
    raw.split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters
                .next()
                .map(|first| {
                    first.to_ascii_uppercase().to_string()
                        + characters.as_str().to_ascii_lowercase().as_str()
                })
                .unwrap_or_default()
        })
        .collect()
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
