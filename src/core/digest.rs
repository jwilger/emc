// Copyright 2026 John Wilger

use crate::core::effect::ArtifactDigest;
use crate::core::types::{
    ModelDescription, ModelName, SliceKindName, SliceSlug, WorkflowCommandErrorRecord,
    WorkflowCommandErrorRecords, WorkflowEntryLifecycleStateRecord,
    WorkflowEntryLifecycleStateRecords, WorkflowOutcomeRecord, WorkflowOutcomeRecords,
    WorkflowOwnedDefinitionRecord, WorkflowOwnedDefinitionRecords, WorkflowSliceDetail,
    WorkflowSliceDetails, WorkflowSlug, WorkflowTransitionEvidenceRecord,
    WorkflowTransitionEvidenceRecords, WorkflowTransitionRecord, WorkflowTransitionRecords,
};

pub struct WorkflowArtifactDigestInput {
    pub workflow_name: ModelName,
    pub workflow_slug: WorkflowSlug,
    pub workflow_description: ModelDescription,
    pub workflow_slice_details: WorkflowSliceDetails,
    pub workflow_transitions: WorkflowTransitionRecords,
    pub workflow_outcomes: WorkflowOutcomeRecords,
    pub workflow_command_errors: WorkflowCommandErrorRecords,
    pub workflow_owned_definitions: WorkflowOwnedDefinitionRecords,
    pub workflow_transition_evidences: WorkflowTransitionEvidenceRecords,
    pub workflow_requires_entry_lifecycle_coverage: bool,
    pub workflow_entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
}

pub fn artifact_digest(input: WorkflowArtifactDigestInput) -> ArtifactDigest {
    ArtifactDigest::try_new(format!(
        "workflow:name={};slug={};description={};slices={};transitions={};outcomes={};command_errors={};owned_definitions={};transition_evidences={};entry_lifecycle_required={};entry_lifecycle_states={}",
        input.workflow_name,
        input.workflow_slug,
        input.workflow_description,
        slice_details_digest(input.workflow_slice_details.as_slice()),
        transitions_digest(input.workflow_transitions.as_slice()),
        outcomes_digest(input.workflow_outcomes.as_slice()),
        command_errors_digest(input.workflow_command_errors.as_slice()),
        owned_definitions_digest(input.workflow_owned_definitions.as_slice()),
        transition_evidences_digest(input.workflow_transition_evidences.as_slice()),
        input.workflow_requires_entry_lifecycle_coverage,
        entry_lifecycle_states_digest(input.workflow_entry_lifecycle_states.as_slice())
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
                slice.relationship().as_ref(),
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
                "{}->{}:{}:{}:{}:{}",
                transition.source().as_ref(),
                transition.target().as_ref(),
                transition.kind().as_ref(),
                transition.trigger().as_ref(),
                transition
                    .rationale()
                    .map_or("", |rationale| rationale.as_ref()),
                transition
                    .payload_contract()
                    .map_or("", |payload_contract| payload_contract.as_ref())
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn outcomes_digest(workflow_outcomes: &[WorkflowOutcomeRecord]) -> String {
    workflow_outcomes
        .iter()
        .map(|outcome| {
            format!(
                "{}:{}:{}",
                outcome.source_slice().as_ref(),
                outcome.label().as_ref(),
                outcome.externally_relevant()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn command_errors_digest(workflow_command_errors: &[WorkflowCommandErrorRecord]) -> String {
    workflow_command_errors
        .iter()
        .map(|error| {
            format!(
                "{}:{}:{}",
                error.source_slice().as_ref(),
                error.command_name().as_ref(),
                error.error_name().as_ref()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn owned_definitions_digest(
    workflow_owned_definitions: &[WorkflowOwnedDefinitionRecord],
) -> String {
    workflow_owned_definitions
        .iter()
        .map(|definition| {
            format!(
                "{}:{}:{}:{}:{}",
                definition.source_slice().as_ref(),
                definition.definition_kind().as_ref(),
                definition.definition_name().as_ref(),
                definition
                    .definition_stream()
                    .map_or("", |definition_stream| definition_stream.as_ref()),
                definition
                    .source_provenance()
                    .map_or("", |source_provenance| source_provenance.as_ref())
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn transition_evidences_digest(
    workflow_transition_evidences: &[WorkflowTransitionEvidenceRecord],
) -> String {
    workflow_transition_evidences
        .iter()
        .map(|evidence| {
            format!(
                "{}->{}:{}:{}:{}:{}",
                evidence.source().as_ref(),
                evidence.target().as_ref(),
                evidence.kind().as_ref(),
                evidence.trigger().as_ref(),
                evidence.source_evidence().as_ref(),
                evidence.target_evidence().as_ref()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn entry_lifecycle_states_digest(
    workflow_entry_lifecycle_states: &[WorkflowEntryLifecycleStateRecord],
) -> String {
    workflow_entry_lifecycle_states
        .iter()
        .map(|coverage| {
            format!(
                "{}:{}:{}",
                coverage.state().as_ref(),
                coverage.step().as_ref(),
                coverage.evidence().as_ref()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}
