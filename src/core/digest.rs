use crate::core::effect::ArtifactDigest;
use crate::core::types::{
    ModelDescription, ModelName, SliceKindName, SliceSlug, WorkflowSliceDetail,
    WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionRecord, WorkflowTransitionRecords,
};

pub fn artifact_digest(
    workflow_name: ModelName,
    workflow_slug: WorkflowSlug,
    workflow_description: ModelDescription,
    workflow_slice_details: WorkflowSliceDetails,
    workflow_transitions: WorkflowTransitionRecords,
) -> ArtifactDigest {
    ArtifactDigest::try_new(format!(
        "workflow:name={workflow_name};slug={workflow_slug};description={workflow_description};slices={};transitions={}",
        slice_details_digest(workflow_slice_details.as_slice()),
        transitions_digest(workflow_transitions.as_slice())
    ))
    .unwrap_or_else(|error| {
        unreachable!("EMC generated artifact digest must be valid: {error}");
    })
}

pub fn slice_artifact_digest(
    slice_name: ModelName,
    slice_slug: SliceSlug,
    slice_kind: SliceKindName,
    slice_description: ModelDescription,
) -> ArtifactDigest {
    ArtifactDigest::try_new(format!(
        "slice:name={slice_name};slug={slice_slug};kind={slice_kind};description={slice_description}"
    ))
    .unwrap_or_else(|error| {
        unreachable!("EMC generated slice artifact digest must be valid: {error}");
    })
}

fn slice_details_digest(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    workflow_slice_details
        .iter()
        .map(|slice| {
            [
                slice.slug().as_ref(),
                slice.name().as_ref(),
                slice.kind().as_ref(),
                slice.description().as_ref(),
            ]
            .join("|")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn transitions_digest(workflow_transitions: &[WorkflowTransitionRecord]) -> String {
    workflow_transitions
        .iter()
        .map(|transition| {
            format!(
                "{}->{}:{}:{}:{}",
                transition.source().as_ref(),
                transition.target().as_ref(),
                transition.kind().as_ref(),
                transition.trigger().as_ref(),
                transition
                    .rationale()
                    .map_or("", |rationale| rationale.as_ref())
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}
