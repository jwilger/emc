// Copyright 2026 John Wilger

use crate::core::types::{
    CommandInputSourceKind, SliceKindName, WorkflowEntryLifecycleStateName,
    WorkflowOwnedDefinitionKind, WorkflowStepRelationshipName, WorkflowTransitionKind,
};

pub(crate) mod lean;
pub(crate) mod quint;

pub(crate) fn lean_slice_kind_name(kind: SliceKindName) -> &'static str {
    match kind {
        SliceKindName::StateView => "SliceKindName.stateView",
        SliceKindName::StateChange => "SliceKindName.stateChange",
        SliceKindName::Translation => "SliceKindName.translation",
        SliceKindName::Automation => "SliceKindName.automation",
    }
}

pub(crate) fn quint_slice_kind_name(kind: SliceKindName) -> &'static str {
    match kind {
        SliceKindName::StateView => "SliceStateView",
        SliceKindName::StateChange => "SliceStateChange",
        SliceKindName::Translation => "SliceTranslation",
        SliceKindName::Automation => "SliceAutomation",
    }
}

pub(crate) fn lean_command_input_source_kind(kind: CommandInputSourceKind) -> &'static str {
    match kind {
        CommandInputSourceKind::Actor => "CommandInputSourceKind.actor",
        CommandInputSourceKind::Session => "CommandInputSourceKind.session",
        CommandInputSourceKind::Generated => "CommandInputSourceKind.generated",
        CommandInputSourceKind::ExternalPayload => "CommandInputSourceKind.externalPayload",
        CommandInputSourceKind::EventStreamState => "CommandInputSourceKind.eventStreamState",
        CommandInputSourceKind::InvocationArgument => "CommandInputSourceKind.invocationArgument",
    }
}

pub(crate) fn lean_model_command_input_source_kind(kind: CommandInputSourceKind) -> &'static str {
    match kind {
        CommandInputSourceKind::Actor => "ModelCommandInputSourceKind.actor",
        CommandInputSourceKind::Session => "ModelCommandInputSourceKind.session",
        CommandInputSourceKind::Generated => "ModelCommandInputSourceKind.generated",
        CommandInputSourceKind::ExternalPayload => "ModelCommandInputSourceKind.externalPayload",
        CommandInputSourceKind::EventStreamState => "ModelCommandInputSourceKind.eventStreamState",
        CommandInputSourceKind::InvocationArgument => {
            "ModelCommandInputSourceKind.invocationArgument"
        }
    }
}

pub(crate) fn quint_command_input_source_kind(kind: CommandInputSourceKind) -> &'static str {
    match kind {
        CommandInputSourceKind::Actor => "CommandInputActor",
        CommandInputSourceKind::Session => "CommandInputSession",
        CommandInputSourceKind::Generated => "CommandInputGenerated",
        CommandInputSourceKind::ExternalPayload => "CommandInputExternalPayload",
        CommandInputSourceKind::EventStreamState => "CommandInputEventStreamState",
        CommandInputSourceKind::InvocationArgument => "CommandInputInvocationArgument",
    }
}

pub(crate) fn quint_model_command_input_source_kind(kind: CommandInputSourceKind) -> &'static str {
    match kind {
        CommandInputSourceKind::Actor => "ModelCommandInputActor",
        CommandInputSourceKind::Session => "ModelCommandInputSession",
        CommandInputSourceKind::Generated => "ModelCommandInputGenerated",
        CommandInputSourceKind::ExternalPayload => "ModelCommandInputExternalPayload",
        CommandInputSourceKind::EventStreamState => "ModelCommandInputEventStreamState",
        CommandInputSourceKind::InvocationArgument => "ModelCommandInputInvocationArgument",
    }
}

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
    }
}

pub(crate) fn quint_workflow_step_relationship_name(
    relationship: WorkflowStepRelationshipName,
) -> &'static str {
    match relationship {
        WorkflowStepRelationshipName::Entry => "StepEntry",
        WorkflowStepRelationshipName::Main => "StepMain",
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
