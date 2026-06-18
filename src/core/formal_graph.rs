// Copyright 2026 John Wilger

use crate::core::types::{
    ModelDescription, ModelName, WorkflowCommandErrorRecords, WorkflowEntryLifecycleStateRecords,
    WorkflowOutcomeRecords, WorkflowOwnedDefinitionRecords, WorkflowSliceDetails, WorkflowSlug,
    WorkflowTransitionEvidenceRecords, WorkflowTransitionRecords,
};

/// The components of a `FormalWorkflowGraph`, supplied when projecting one from
/// the event-log `ProjectedModel` (rather than parsing artifact text).
pub(crate) struct FormalWorkflowGraphComponents {
    pub(crate) name: ModelName,
    pub(crate) slug: WorkflowSlug,
    pub(crate) description: ModelDescription,
    pub(crate) slice_details: WorkflowSliceDetails,
    pub(crate) transitions: WorkflowTransitionRecords,
    pub(crate) outcomes: WorkflowOutcomeRecords,
    pub(crate) command_errors: WorkflowCommandErrorRecords,
    pub(crate) owned_definitions: WorkflowOwnedDefinitionRecords,
    pub(crate) transition_evidences: WorkflowTransitionEvidenceRecords,
    pub(crate) entry_lifecycle_required: bool,
    pub(crate) entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalWorkflowGraph {
    name: ModelName,
    slug: WorkflowSlug,
    description: ModelDescription,
    slice_details: WorkflowSliceDetails,
    transitions: WorkflowTransitionRecords,
    outcomes: WorkflowOutcomeRecords,
    command_errors: WorkflowCommandErrorRecords,
    owned_definitions: WorkflowOwnedDefinitionRecords,
    transition_evidences: WorkflowTransitionEvidenceRecords,
    entry_lifecycle_required: bool,
    entry_lifecycle_states: WorkflowEntryLifecycleStateRecords,
}

impl FormalWorkflowGraph {
    /// Build a workflow graph directly from projected components (the event-log
    /// `ProjectedModel`) rather than by parsing Lean/Quint artifact text. This is
    /// how command decisions source workflow state from the authoritative event
    /// log; the Lean/Quint artifacts are write-only projections and are never
    /// parsed back to drive a decision.
    pub(crate) fn from_components(components: FormalWorkflowGraphComponents) -> Self {
        Self {
            name: components.name,
            slug: components.slug,
            description: components.description,
            slice_details: components.slice_details,
            transitions: components.transitions,
            outcomes: components.outcomes,
            command_errors: components.command_errors,
            owned_definitions: components.owned_definitions,
            transition_evidences: components.transition_evidences,
            entry_lifecycle_required: components.entry_lifecycle_required,
            entry_lifecycle_states: components.entry_lifecycle_states,
        }
    }

    pub(crate) fn name(&self) -> &ModelName {
        &self.name
    }

    pub(crate) fn slug(&self) -> &WorkflowSlug {
        &self.slug
    }

    pub(crate) fn description(&self) -> &ModelDescription {
        &self.description
    }

    pub(crate) fn slice_details(&self) -> &WorkflowSliceDetails {
        &self.slice_details
    }

    pub(crate) fn transitions(&self) -> &WorkflowTransitionRecords {
        &self.transitions
    }

    pub(crate) fn outcomes(&self) -> &WorkflowOutcomeRecords {
        &self.outcomes
    }

    pub(crate) fn command_errors(&self) -> &WorkflowCommandErrorRecords {
        &self.command_errors
    }

    pub(crate) fn owned_definitions(&self) -> &WorkflowOwnedDefinitionRecords {
        &self.owned_definitions
    }

    pub(crate) fn transition_evidences(&self) -> &WorkflowTransitionEvidenceRecords {
        &self.transition_evidences
    }

    pub(crate) fn entry_lifecycle_required(&self) -> bool {
        self.entry_lifecycle_required
    }

    pub(crate) fn entry_lifecycle_states(&self) -> &WorkflowEntryLifecycleStateRecords {
        &self.entry_lifecycle_states
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormalWorkflowGraphs {
    graphs: Vec<FormalWorkflowGraph>,
}

impl FormalWorkflowGraphs {
    pub(crate) fn from_graphs(graphs: impl IntoIterator<Item = FormalWorkflowGraph>) -> Self {
        Self {
            graphs: graphs.into_iter().collect(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[FormalWorkflowGraph] {
        &self.graphs
    }

    pub(crate) fn into_inner(self) -> Vec<FormalWorkflowGraph> {
        self.graphs
    }
}
