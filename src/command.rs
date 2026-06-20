// Copyright 2026 John Wilger

use crate::core::connection::{WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::effect::{
    ChosenEventId, CleanReviewEffect, Effect, EffectPlan, EventConflictId, EventConflictResolution,
    SliceCommandDefinitionRemovalEffect, SliceDescriptionUpdateEffect,
    SliceEventDefinitionRemovalEffect, SliceKindUpdateEffect, SliceNameUpdateEffect,
    SliceReadModelDefinitionRemovalEffect, SliceScenarioRemovalEffect,
    SliceViewControlRemovalEffect, SliceViewControlUpdateEffect, SliceViewDefinitionRemovalEffect,
    WorkflowCommandErrorEffect, WorkflowDescriptionUpdateEffect, WorkflowEntryLifecycleStateEffect,
    WorkflowNameUpdateEffect, WorkflowOutcomeEffect, WorkflowOwnedDefinitionEffect,
    WorkflowTransitionEvidenceEffect,
};
use crate::core::formal_slice_facts::{
    NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection, NewBoardElement,
    NewCommandDefinition, NewControlDefinition, NewEventDefinition, NewExternalPayloadDefinition,
    NewOutcomeDefinition, NewReadModelDefinition, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition,
};
use crate::core::gherkin::{
    GherkinSuite, list_gherkin_features, run_all_gherkin_suites, run_gherkin_suite,
};
use crate::core::project::{ProjectName, init_project};
use crate::core::review_gate::review_gate;
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{
    CommandName, ControlName, EventName, ModelDescription, ModelName, ReadModelName,
    ReviewTimestamp, ReviewerId, ScenarioName, SliceSlug, ViewName, WorkflowCommandErrorRecord,
    WorkflowEntryLifecycleStateRecord, WorkflowOutcomeRecord, WorkflowOwnedDefinitionRecord,
    WorkflowSlug, WorkflowTransitionEvidenceRecord,
};
use crate::core::workflow::NewWorkflow;

pub(crate) fn add_slice(slice: NewSlice) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddSliceFromWorkflow(slice)])
}

pub(crate) fn add_slice_scenario(scenario: NewSliceScenario) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddSliceScenarioFromSlice(scenario)])
}

pub(crate) fn update_slice_scenario(scenario: NewSliceScenario) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceScenarioFromSlice(scenario)])
}

pub(crate) fn remove_slice_scenario(
    slice_slug: SliceSlug,
    scenario_name: ScenarioName,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveSliceScenarioFromSlice(
        SliceScenarioRemovalEffect::new(slice_slug, scenario_name),
    )])
}

pub(crate) fn add_automation_definition(automation: NewAutomationDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddAutomationDefinitionFromSlice(automation)])
}

pub(crate) fn add_translation_definition(translation: NewTranslationDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddTranslationDefinitionFromSlice(translation)])
}

pub(crate) fn add_command_definition(command: NewCommandDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddCommandDefinitionFromSlice(command)])
}

pub(crate) fn update_command_definition(command: NewCommandDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateCommandDefinitionFromSlice(command)])
}

pub(crate) fn remove_command_definition(
    slice_slug: SliceSlug,
    command_name: CommandName,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveCommandDefinitionFromSlice(
        SliceCommandDefinitionRemovalEffect::new(slice_slug, command_name),
    )])
}

pub(crate) fn add_event_definition(event: NewEventDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddEventDefinitionFromSlice(event)])
}

pub(crate) fn update_event_definition(event: NewEventDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateEventDefinitionFromSlice(event)])
}

pub(crate) fn remove_event_definition(slice_slug: SliceSlug, event_name: EventName) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveEventDefinitionFromSlice(
        SliceEventDefinitionRemovalEffect::new(slice_slug, event_name),
    )])
}

pub(crate) fn add_external_payload_definition(
    external_payload: NewExternalPayloadDefinition,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddExternalPayloadDefinitionFromSlice(
        external_payload,
    )])
}

pub(crate) fn add_outcome_definition(outcome: NewOutcomeDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddOutcomeDefinitionFromSlice(outcome)])
}

pub(crate) fn add_read_model_definition(read_model: NewReadModelDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddReadModelDefinitionFromSlice(read_model)])
}

pub(crate) fn update_read_model_definition(read_model: NewReadModelDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateReadModelDefinitionFromSlice(read_model)])
}

pub(crate) fn remove_read_model_definition(
    slice_slug: SliceSlug,
    read_model_name: ReadModelName,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveReadModelDefinitionFromSlice(
        SliceReadModelDefinitionRemovalEffect::new(slice_slug, read_model_name),
    )])
}

pub(crate) fn add_view_definition(view: NewViewDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddViewDefinitionFromSlice(view)])
}

pub(crate) fn update_view_definition(view: NewViewDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateViewDefinitionFromSlice(view)])
}

pub(crate) fn remove_view_definition(slice_slug: SliceSlug, view_name: ViewName) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveViewDefinitionFromSlice(
        SliceViewDefinitionRemovalEffect::new(slice_slug, view_name),
    )])
}

pub(crate) fn update_control_definition(
    slice_slug: SliceSlug,
    view_name: ViewName,
    control: NewControlDefinition,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateViewControlFromSlice(
        SliceViewControlUpdateEffect::new(slice_slug, view_name, control),
    )])
}

pub(crate) fn remove_control_definition(
    slice_slug: SliceSlug,
    view_name: ViewName,
    control_name: ControlName,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveViewControlFromSlice(
        SliceViewControlRemovalEffect::new(slice_slug, view_name, control_name),
    )])
}

pub(crate) fn add_bit_level_data_flow(data_flow: NewBitLevelDataFlow) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddBitLevelDataFlowFromSlice(data_flow)])
}

pub(crate) fn add_board_element(element: NewBoardElement) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddBoardElementFromSlice(element)])
}

pub(crate) fn add_board_connection(connection: NewBoardConnection) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddBoardConnectionFromSlice(connection)])
}

pub(crate) fn add_workflow(workflow: NewWorkflow) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowFromIndex(workflow)])
}

pub(crate) fn add_workflow_outcome(
    workflow_slug: WorkflowSlug,
    outcome: WorkflowOutcomeRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowOutcomeFromWorkflow(
        WorkflowOutcomeEffect::new(workflow_slug, outcome),
    )])
}

pub(crate) fn add_workflow_command_error(
    workflow_slug: WorkflowSlug,
    error: WorkflowCommandErrorRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowCommandErrorFromWorkflow(
        WorkflowCommandErrorEffect::new(workflow_slug, error),
    )])
}

pub(crate) fn add_workflow_owned_definition(
    workflow_slug: WorkflowSlug,
    definition: WorkflowOwnedDefinitionRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowOwnedDefinitionFromWorkflow(
        WorkflowOwnedDefinitionEffect::new(workflow_slug, definition),
    )])
}

pub(crate) fn add_workflow_transition_evidence(
    workflow_slug: WorkflowSlug,
    evidence: WorkflowTransitionEvidenceRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowTransitionEvidenceFromWorkflow(
        WorkflowTransitionEvidenceEffect::new(workflow_slug, evidence),
    )])
}

pub(crate) fn require_workflow_entry_lifecycle_coverage(workflow_slug: WorkflowSlug) -> EffectPlan {
    EffectPlan::new(vec![
        Effect::RequireWorkflowEntryLifecycleCoverageFromWorkflow(workflow_slug),
    ])
}

pub(crate) fn add_workflow_entry_lifecycle_state(
    workflow_slug: WorkflowSlug,
    coverage: WorkflowEntryLifecycleStateRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowEntryLifecycleStateFromWorkflow(
        WorkflowEntryLifecycleStateEffect::new(workflow_slug, coverage),
    )])
}

pub(crate) fn check_project() -> EffectPlan {
    EffectPlan::new(vec![Effect::CheckCurrentProject])
}

pub(crate) fn connect_workflow(connection: WorkflowConnection) -> EffectPlan {
    EffectPlan::new(vec![Effect::ConnectWorkflowFromWorkflow(connection)])
}

pub(crate) fn remove_transition(removal: WorkflowTransitionRemoval) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveTransitionFromWorkflow(removal)])
}

pub(crate) fn remove_workflow(slug: WorkflowSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveWorkflowFromIndex(slug)])
}

pub(crate) fn resolve_conflict(
    conflict_id: EventConflictId,
    chosen_event_id: ChosenEventId,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::ResolveEventConflict(
        EventConflictResolution::new(conflict_id, chosen_event_id),
    )])
}

pub(crate) fn gherkin_list(suite: &GherkinSuite) -> EffectPlan {
    list_gherkin_features(suite)
}

pub(crate) fn gherkin_run(suite: &GherkinSuite) -> EffectPlan {
    run_gherkin_suite(suite)
}

pub(crate) fn gherkin_run_all() -> EffectPlan {
    run_all_gherkin_suites()
}

pub(crate) fn init(name: &ProjectName) -> EffectPlan {
    init_project(name)
}

pub(crate) fn list_workflows() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListWorkflowsFromIndex])
}

pub(crate) fn list_conflicts() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListConflictsFromEvents])
}

pub(crate) fn list_slices() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListSlicesFromIndex])
}

pub(crate) fn list_transitions() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListTransitionsFromIndex])
}

pub(crate) fn review_gate_for_workflow(slug: WorkflowSlug) -> EffectPlan {
    review_gate(slug)
}

pub(crate) fn record_clean_review(
    slug: WorkflowSlug,
    reviewer: ReviewerId,
    reviewed_at: ReviewTimestamp,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::RecordCleanReviewFromWorkflow(
        CleanReviewEffect::new(slug, reviewer, reviewed_at),
    )])
}

pub(crate) fn show_workflow(slug: WorkflowSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::ShowWorkflowFromWorkflow(slug)])
}

pub(crate) fn show_slice(slug: SliceSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::ShowSliceFromSlice(slug)])
}

pub(crate) fn update_workflow_description(
    slug: WorkflowSlug,
    description: ModelDescription,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(
        WorkflowDescriptionUpdateEffect::new(slug, description),
    )])
}

pub(crate) fn update_workflow_name(slug: WorkflowSlug, name: ModelName) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateWorkflowNameFromIndexAndWorkflow(
        WorkflowNameUpdateEffect::new(slug, name),
    )])
}

pub(crate) fn update_slice_description(
    slug: SliceSlug,
    description: ModelDescription,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceDescriptionFromWorkflow(
        SliceDescriptionUpdateEffect::new(slug, description),
    )])
}

pub(crate) fn update_slice_kind(slug: SliceSlug, kind: SliceKind) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceKindFromWorkflow(
        SliceKindUpdateEffect::new(slug, kind),
    )])
}

pub(crate) fn update_slice_name(slug: SliceSlug, name: ModelName) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceNameFromWorkflow(
        SliceNameUpdateEffect::new(slug, name),
    )])
}

pub(crate) fn remove_slice(slug: SliceSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveSliceFromWorkflow(slug)])
}

pub(crate) fn verify() -> EffectPlan {
    EffectPlan::new(vec![Effect::VerifyProjectFromIndex])
}
