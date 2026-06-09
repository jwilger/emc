// Copyright 2026 John Wilger

use crate::core::types::{
    WorkflowEntryLifecycleStateName, WorkflowOwnedDefinitionKind, WorkflowStepRelationshipName,
    WorkflowTransitionKind,
};

pub(crate) mod lean;
pub(crate) mod quint;

pub(crate) fn lean_workflow_transition_kind(kind: WorkflowTransitionKind) -> &'static str {
    match kind {
        WorkflowTransitionKind::Command => "WorkflowTransitionKind.command",
        WorkflowTransitionKind::Event => "WorkflowTransitionKind.event",
        WorkflowTransitionKind::Navigation => "WorkflowTransitionKind.navigation",
        WorkflowTransitionKind::ExternalTrigger => "WorkflowTransitionKind.externalTrigger",
        WorkflowTransitionKind::Outcome => "WorkflowTransitionKind.outcome",
        WorkflowTransitionKind::WorkflowExitCommand => "WorkflowTransitionKind.workflowExitCommand",
        WorkflowTransitionKind::WorkflowExitEvent => "WorkflowTransitionKind.workflowExitEvent",
        WorkflowTransitionKind::WorkflowExitNavigation => {
            "WorkflowTransitionKind.workflowExitNavigation"
        }
        WorkflowTransitionKind::WorkflowExitExternalTrigger => {
            "WorkflowTransitionKind.workflowExitExternalTrigger"
        }
        WorkflowTransitionKind::WorkflowExitOutcome => "WorkflowTransitionKind.workflowExitOutcome",
    }
}

pub(crate) fn quint_workflow_transition_kind(kind: WorkflowTransitionKind) -> &'static str {
    match kind {
        WorkflowTransitionKind::Command => "Command",
        WorkflowTransitionKind::Event => "Event",
        WorkflowTransitionKind::Navigation => "Navigation",
        WorkflowTransitionKind::ExternalTrigger => "ExternalTrigger",
        WorkflowTransitionKind::Outcome => "Outcome",
        WorkflowTransitionKind::WorkflowExitCommand => "WorkflowExitCommand",
        WorkflowTransitionKind::WorkflowExitEvent => "WorkflowExitEvent",
        WorkflowTransitionKind::WorkflowExitNavigation => "WorkflowExitNavigation",
        WorkflowTransitionKind::WorkflowExitExternalTrigger => "WorkflowExitExternalTrigger",
        WorkflowTransitionKind::WorkflowExitOutcome => "WorkflowExitOutcome",
    }
}

pub(crate) fn lean_workflow_owned_definition_kind(
    kind: WorkflowOwnedDefinitionKind,
) -> &'static str {
    match kind {
        WorkflowOwnedDefinitionKind::Command => "WorkflowOwnedDefinitionKind.command",
        WorkflowOwnedDefinitionKind::Event => "WorkflowOwnedDefinitionKind.event",
        WorkflowOwnedDefinitionKind::View => "WorkflowOwnedDefinitionKind.view",
        WorkflowOwnedDefinitionKind::Control => "WorkflowOwnedDefinitionKind.control",
        WorkflowOwnedDefinitionKind::ReadModel => "WorkflowOwnedDefinitionKind.readModel",
        WorkflowOwnedDefinitionKind::Outcome => "WorkflowOwnedDefinitionKind.outcome",
        WorkflowOwnedDefinitionKind::Error => "WorkflowOwnedDefinitionKind.error",
        WorkflowOwnedDefinitionKind::Automation => "WorkflowOwnedDefinitionKind.automation",
        WorkflowOwnedDefinitionKind::Translation => "WorkflowOwnedDefinitionKind.translation",
        WorkflowOwnedDefinitionKind::ExternalPayload => {
            "WorkflowOwnedDefinitionKind.externalPayload"
        }
    }
}

pub(crate) fn quint_workflow_owned_definition_kind(
    kind: WorkflowOwnedDefinitionKind,
) -> &'static str {
    match kind {
        WorkflowOwnedDefinitionKind::Command => "OwnedCommand",
        WorkflowOwnedDefinitionKind::Event => "OwnedEvent",
        WorkflowOwnedDefinitionKind::View => "OwnedView",
        WorkflowOwnedDefinitionKind::Control => "OwnedControl",
        WorkflowOwnedDefinitionKind::ReadModel => "OwnedReadModel",
        WorkflowOwnedDefinitionKind::Outcome => "OwnedOutcome",
        WorkflowOwnedDefinitionKind::Error => "OwnedError",
        WorkflowOwnedDefinitionKind::Automation => "OwnedAutomation",
        WorkflowOwnedDefinitionKind::Translation => "OwnedTranslation",
        WorkflowOwnedDefinitionKind::ExternalPayload => "OwnedExternalPayload",
    }
}

pub(crate) fn lean_workflow_step_relationship_name(
    relationship: WorkflowStepRelationshipName,
) -> &'static str {
    match relationship {
        WorkflowStepRelationshipName::Entry => "WorkflowStepRelationshipName.entry",
        WorkflowStepRelationshipName::Main => "WorkflowStepRelationshipName.main",
        WorkflowStepRelationshipName::Branch => "WorkflowStepRelationshipName.branch",
        WorkflowStepRelationshipName::Alternate => "WorkflowStepRelationshipName.alternate",
        WorkflowStepRelationshipName::AsyncLifecycle => {
            "WorkflowStepRelationshipName.asyncLifecycle"
        }
        WorkflowStepRelationshipName::Supporting => "WorkflowStepRelationshipName.supporting",
    }
}

pub(crate) fn quint_workflow_step_relationship_name(
    relationship: WorkflowStepRelationshipName,
) -> &'static str {
    match relationship {
        WorkflowStepRelationshipName::Entry => "StepEntry",
        WorkflowStepRelationshipName::Main => "StepMain",
        WorkflowStepRelationshipName::Branch => "StepBranch",
        WorkflowStepRelationshipName::Alternate => "StepAlternate",
        WorkflowStepRelationshipName::AsyncLifecycle => "StepAsyncLifecycle",
        WorkflowStepRelationshipName::Supporting => "StepSupporting",
    }
}

pub(crate) fn lean_workflow_entry_lifecycle_state_name(
    state: WorkflowEntryLifecycleStateName,
) -> &'static str {
    match state {
        WorkflowEntryLifecycleStateName::FreshUninitialized => {
            "WorkflowEntryLifecycleStateName.freshUninitialized"
        }
        WorkflowEntryLifecycleStateName::InitializedUnauthenticated => {
            "WorkflowEntryLifecycleStateName.initializedUnauthenticated"
        }
        WorkflowEntryLifecycleStateName::InitializedAuthenticated => {
            "WorkflowEntryLifecycleStateName.initializedAuthenticated"
        }
        WorkflowEntryLifecycleStateName::PartiallyConfigured => {
            "WorkflowEntryLifecycleStateName.partiallyConfigured"
        }
        WorkflowEntryLifecycleStateName::FullyConfigured => {
            "WorkflowEntryLifecycleStateName.fullyConfigured"
        }
    }
}

pub(crate) fn quint_workflow_entry_lifecycle_state_name(
    state: WorkflowEntryLifecycleStateName,
) -> &'static str {
    match state {
        WorkflowEntryLifecycleStateName::FreshUninitialized => "FreshUninitialized",
        WorkflowEntryLifecycleStateName::InitializedUnauthenticated => "InitializedUnauthenticated",
        WorkflowEntryLifecycleStateName::InitializedAuthenticated => "InitializedAuthenticated",
        WorkflowEntryLifecycleStateName::PartiallyConfigured => "PartiallyConfigured",
        WorkflowEntryLifecycleStateName::FullyConfigured => "FullyConfigured",
    }
}
