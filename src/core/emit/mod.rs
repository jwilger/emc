// Copyright 2026 John Wilger

use crate::core::types::WorkflowTransitionKind;

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
