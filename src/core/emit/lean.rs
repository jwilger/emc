use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, WorkflowSliceDetail, WorkflowSlug,
    WorkflowTransitionLabel,
};

pub fn emit_workflow_module(
    module_name: LeanModuleName,
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_slice_details: Vec<WorkflowSliceDetail>,
    workflow_transitions: Vec<WorkflowTransitionLabel>,
    digest: ArtifactDigest,
) -> FileContents {
    let slice_list = slice_list(&workflow_slice_details);
    let slice_detail_list = slice_detail_list(&workflow_slice_details);
    let transition_list = transition_list(workflow_transitions);
    file_contents(format!(
        "namespace {module_name}\n\n-- EMC-DIGEST: {digest}\n-- EMC generated Lean4 business workflow model.\ndef workflowName := {workflow_name_json}\n\ndef workflowSlug := {workflow_slug_json}\n\ndef workflowDescription := {workflow_description_json}\n\ndef workflowSlices : List String := {slice_list}\n\ndef workflowSliceDetails : List (String × String × String × String) := {slice_detail_list}\n\ndef workflowTransitions : List (String × String × String × String) := {transition_list}\n\ntheorem workflowIdentityIsStable : workflowName = {workflow_name_json} := rfl\n\ntheorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl\n\ntheorem workflowTransitionsAreStructured : workflowTransitions.all (fun transition => transition.1.isEmpty == false && transition.2.1.isEmpty == false && transition.2.2.1.isEmpty == false && transition.2.2.2.isEmpty == false) = true := rfl\n\nend {module_name}\n",
        module_name = module_name.as_ref(),
        digest = digest.as_ref(),
        workflow_name_json = quoted(workflow_name.as_ref()),
        workflow_slug_json = quoted(workflow_slug.as_ref()),
        workflow_description_json = quoted(workflow_description.as_ref()),
        slice_list = slice_list,
        slice_detail_list = slice_detail_list,
        transition_list = transition_list,
    ))
}

fn file_contents(value: impl Into<String>) -> FileContents {
    FileContents::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Lean4 file contents must be valid: {error}");
    })
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated Lean4 string literal must be valid: {error}");
    })
}

fn slice_list(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    format!(
        "[{}]",
        workflow_slice_details
            .iter()
            .map(|slice| quoted(slice.slug().as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn slice_detail_list(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    format!(
        "[{}]",
        workflow_slice_details
            .iter()
            .map(|slice| {
                format!(
                    "({}, {}, {}, {})",
                    quoted(slice.slug().as_ref()),
                    quoted(slice.name().as_ref()),
                    quoted(slice.kind().as_ref()),
                    quoted(slice.description().as_ref())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn transition_list(workflow_transitions: Vec<WorkflowTransitionLabel>) -> String {
    format!(
        "[{}]",
        workflow_transitions
            .iter()
            .map(|transition| transition_record(transition.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn transition_record(transition: &str) -> String {
    let (source, tail) = transition.split_once("->").unwrap_or_else(|| {
        unreachable!("EMC generated workflow transition must contain a source and target");
    });
    let (target, kind, trigger) = transition_parts(tail);
    format!(
        "({}, {}, {}, {})",
        quoted(source),
        quoted(target),
        quoted(kind.as_ref()),
        quoted(trigger.as_ref())
    )
}

fn transition_parts(tail: &str) -> (&str, String, String) {
    let parts = tail.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        [target, kind, trigger] => (target, (*kind).to_owned(), (*trigger).to_owned()),
        [target, "workflow_exit", exit_kind, trigger] => (
            target,
            format!("workflow_exit:{exit_kind}"),
            (*trigger).to_owned(),
        ),
        _ => unreachable!("EMC generated workflow transition must have canonical parts"),
    }
}
