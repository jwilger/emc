use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, SliceSlug, WorkflowSlug, WorkflowTransitionLabel,
};

pub fn emit_workflow_module(
    module_name: LeanModuleName,
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_slices: Vec<SliceSlug>,
    workflow_transitions: Vec<WorkflowTransitionLabel>,
    digest: ArtifactDigest,
) -> FileContents {
    let slice_list = slice_list(workflow_slices);
    let transition_list = transition_list(workflow_transitions);
    file_contents(format!(
        "namespace {module_name}\n\n-- EMC-DIGEST: {digest}\n-- EMC generated Lean4 business workflow model.\ndef workflowName := {workflow_name_json}\n\ndef workflowSlug := {workflow_slug_json}\n\ndef workflowDescription := {workflow_description_json}\n\ndef workflowSlices := {slice_list}\n\ndef workflowTransitions := {transition_list}\n\ntheorem workflowIdentityIsStable : workflowName = {workflow_name_json} := rfl\n\nend {module_name}\n",
        module_name = module_name.as_ref(),
        digest = digest.as_ref(),
        workflow_name_json = quoted(workflow_name.as_ref()),
        workflow_slug_json = quoted(workflow_slug.as_ref()),
        workflow_description_json = quoted(workflow_description.as_ref()),
        slice_list = slice_list,
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

fn slice_list(workflow_slices: Vec<SliceSlug>) -> String {
    format!(
        "[{}]",
        workflow_slices
            .iter()
            .map(|slice| quoted(slice.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn transition_list(workflow_transitions: Vec<WorkflowTransitionLabel>) -> String {
    format!(
        "[{}]",
        workflow_transitions
            .iter()
            .map(|transition| quoted(transition.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}
