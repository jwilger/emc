use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{
    ModelDescription, ModelName, QuintModuleName, WorkflowSliceDetail, WorkflowSlug,
    WorkflowTransitionLabel,
};

pub fn emit_workflow_module(
    module_name: QuintModuleName,
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
        "module {module_name} {{\n  // EMC-DIGEST: {digest}\n  val workflowName = {workflow_name_json}\n  val workflowSlug = {workflow_slug_json}\n  val workflowDescription = {workflow_description_json}\n  val workflowSlices = {slice_list}\n  val workflowSliceDetails = {slice_detail_list}\n  val workflowTransitions = {transition_list}\n  val workflowIdentityStable = workflowName == {workflow_name_json}\n  val workflowSlicesHaveDetails = length(workflowSlices) == length(workflowSliceDetails)\n  val workflowSliceDetailsComplete = workflowSlicesHaveDetails\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}}\n",
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
        unreachable!("EMC generated Quint file contents must be valid: {error}");
    })
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated Quint string literal must be valid: {error}");
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
                    "{{ slug: {}, name: {}, kind: {}, description: {} }}",
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
            .map(|transition| quoted(transition.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}
