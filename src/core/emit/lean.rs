use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{
    LeanModuleName, ModelDescription, ModelName, SliceKindName, SliceSlug, WorkflowSliceDetail,
    WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionRecord, WorkflowTransitionRecords,
};

pub fn emit_workflow_module(
    module_name: LeanModuleName,
    workflow_name: ModelName,
    workflow_description: ModelDescription,
    workflow_slug: WorkflowSlug,
    workflow_slice_details: WorkflowSliceDetails,
    workflow_transitions: WorkflowTransitionRecords,
    digest: ArtifactDigest,
) -> FileContents {
    let slice_list = slice_list(workflow_slice_details.as_slice());
    let slice_detail_list = slice_detail_list(workflow_slice_details.as_slice());
    let transition_list = transition_list(workflow_transitions);
    file_contents(format!(
        "namespace {module_name}\n\n-- EMC-DIGEST: {digest}\n-- EMC generated Lean4 business workflow model.\ndef workflowName := {workflow_name_json}\n\ndef workflowSlug := {workflow_slug_json}\n\ndef workflowDescription := {workflow_description_json}\n\ndef workflowSlices : List String := {slice_list}\n\ndef workflowSliceDetails : List (String × String × String × String) := {slice_detail_list}\n\nstructure WorkflowTransition where\n  source : String\n  target : String\n  kind : String\n  trigger : String\n  rationale : String\n\ndef workflowTransitions : List WorkflowTransition := {transition_list}\n\ntheorem workflowIdentityIsStable : workflowName = {workflow_name_json} := rfl\n\ntheorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl\n\ntheorem workflowTransitionsAreStructured : workflowTransitions.all (fun transition => transition.source.isEmpty == false && transition.target.isEmpty == false && transition.kind.isEmpty == false && transition.trigger.isEmpty == false) = true := rfl\n\nend {module_name}\n",
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

pub fn emit_slice_module(
    module_name: LeanModuleName,
    slice_name: ModelName,
    slice_description: ModelDescription,
    slice_slug: SliceSlug,
    slice_kind: SliceKindName,
    digest: ArtifactDigest,
) -> FileContents {
    file_contents(format!(
        "namespace {module_name}\n\n-- EMC-DIGEST: {digest}\n-- EMC generated Lean4 business slice model.\ndef sliceName := {slice_name_json}\n\ndef sliceSlug := {slice_slug_json}\n\ndef sliceKind := {slice_kind_json}\n\ndef sliceDescription := {slice_description_json}\n\ntheorem sliceIdentityIsStable : sliceName = {slice_name_json} := rfl\n\nend {module_name}\n",
        module_name = module_name.as_ref(),
        digest = digest.as_ref(),
        slice_name_json = quoted(slice_name.as_ref()),
        slice_slug_json = quoted(slice_slug.as_ref()),
        slice_kind_json = quoted(slice_kind.as_ref()),
        slice_description_json = quoted(slice_description.as_ref()),
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

fn transition_list(workflow_transitions: WorkflowTransitionRecords) -> String {
    format!(
        "[{}]",
        workflow_transitions
            .as_slice()
            .iter()
            .map(transition_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn transition_record(transition: &WorkflowTransitionRecord) -> String {
    format!(
        "{{ source := {}, target := {}, kind := {}, trigger := {}, rationale := {} }}",
        quoted(transition.source().as_ref()),
        quoted(transition.target().as_ref()),
        quoted(transition.kind().as_ref()),
        quoted(transition.trigger().as_ref()),
        quoted(
            transition
                .rationale()
                .map_or("", |rationale| rationale.as_ref())
        )
    )
}
