use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{LeanModuleName, ModelDescription, ModelName, SliceSlug, WorkflowSlug};

pub fn emit_workflow_module(
    module_name: LeanModuleName,
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_slices: Vec<SliceSlug>,
    digest: ArtifactDigest,
) -> FileContents {
    let slice_list = slice_list(workflow_slices);
    file_contents(format!(
        "namespace {module_name}\n\n-- EMC-DIGEST: {digest}\n-- EMC generated Lean4 business workflow model.\ndef workflowName := {workflow_name_json}\n\ndef workflowSlug := {workflow_slug_json}\n\ndef workflowDescription := {workflow_description_json}\n\ndef workflowSlices := {slice_list}\n\ntheorem workflowIdentityIsStable : workflowName = {workflow_name_json} := rfl\n\nend {module_name}\n",
        module_name = module_name.as_ref(),
        digest = digest.as_ref(),
        workflow_name_json = quoted(workflow_name.as_ref()),
        workflow_slug_json = quoted(workflow_slug.as_ref()),
        workflow_description_json = quoted(workflow_description.as_ref()),
        slice_list = slice_list,
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
