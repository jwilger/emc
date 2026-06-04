use crate::core::connection::{WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::effect::{Effect, EffectPlan};
use crate::core::formal_slice_facts::{
    NewAutomationDefinition, NewBitLevelDataFlow, NewBoardConnection, NewBoardElement,
    NewCommandDefinition, NewEventDefinition, NewExternalPayloadDefinition, NewOutcomeDefinition,
    NewReadModelDefinition, NewSliceScenario, NewTranslationDefinition, NewViewDefinition,
};
use crate::core::gherkin::{
    GherkinSuite, list_gherkin_features, run_all_gherkin_suites, run_gherkin_suite,
};
use crate::core::project::{ProjectName, init_project};
use crate::core::review_gate::review_gate;
use crate::core::slice::{NewSlice, SliceKind};
use crate::core::types::{
    ModelDescription, ModelName, ReviewTimestamp, ReviewerId, SliceSlug,
    WorkflowCommandErrorRecord, WorkflowEntryLifecycleStateRecord, WorkflowOutcomeRecord,
    WorkflowOwnedDefinitionRecord, WorkflowSlug, WorkflowTransitionEvidenceRecord,
};
use crate::core::workflow::NewWorkflow;

pub fn add_slice(slice: NewSlice) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddSliceFromWorkflow(slice)])
}

pub fn add_slice_scenario(scenario: NewSliceScenario) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddSliceScenarioFromSlice(scenario)])
}

pub fn add_automation_definition(automation: NewAutomationDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddAutomationDefinitionFromSlice(automation)])
}

pub fn add_translation_definition(translation: NewTranslationDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddTranslationDefinitionFromSlice(translation)])
}

pub fn add_command_definition(command: NewCommandDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddCommandDefinitionFromSlice(command)])
}

pub fn add_event_definition(event: NewEventDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddEventDefinitionFromSlice(event)])
}

pub fn add_external_payload_definition(
    external_payload: NewExternalPayloadDefinition,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddExternalPayloadDefinitionFromSlice(
        external_payload,
    )])
}

pub fn add_outcome_definition(outcome: NewOutcomeDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddOutcomeDefinitionFromSlice(outcome)])
}

pub fn add_read_model_definition(read_model: NewReadModelDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddReadModelDefinitionFromSlice(read_model)])
}

pub fn add_view_definition(view: NewViewDefinition) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddViewDefinitionFromSlice(view)])
}

pub fn add_bit_level_data_flow(data_flow: NewBitLevelDataFlow) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddBitLevelDataFlowFromSlice(data_flow)])
}

pub fn add_board_element(element: NewBoardElement) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddBoardElementFromSlice(element)])
}

pub fn add_board_connection(connection: NewBoardConnection) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddBoardConnectionFromSlice(connection)])
}

pub fn add_workflow(workflow: NewWorkflow) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowFromIndex(workflow)])
}

pub fn add_workflow_outcome(
    workflow_slug: WorkflowSlug,
    outcome: WorkflowOutcomeRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowOutcomeFromWorkflow(
        workflow_slug,
        outcome,
    )])
}

pub fn add_workflow_command_error(
    workflow_slug: WorkflowSlug,
    error: WorkflowCommandErrorRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowCommandErrorFromWorkflow(
        workflow_slug,
        error,
    )])
}

pub fn add_workflow_owned_definition(
    workflow_slug: WorkflowSlug,
    definition: WorkflowOwnedDefinitionRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowOwnedDefinitionFromWorkflow(
        workflow_slug,
        definition,
    )])
}

pub fn add_workflow_transition_evidence(
    workflow_slug: WorkflowSlug,
    evidence: WorkflowTransitionEvidenceRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowTransitionEvidenceFromWorkflow(
        workflow_slug,
        evidence,
    )])
}

pub fn require_workflow_entry_lifecycle_coverage(workflow_slug: WorkflowSlug) -> EffectPlan {
    EffectPlan::new(vec![
        Effect::RequireWorkflowEntryLifecycleCoverageFromWorkflow(workflow_slug),
    ])
}

pub fn add_workflow_entry_lifecycle_state(
    workflow_slug: WorkflowSlug,
    coverage: WorkflowEntryLifecycleStateRecord,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::AddWorkflowEntryLifecycleStateFromWorkflow(
        workflow_slug,
        coverage,
    )])
}

pub fn check_project() -> EffectPlan {
    EffectPlan::new(vec![Effect::CheckCurrentProject])
}

pub fn connect_workflow(connection: WorkflowConnection) -> EffectPlan {
    EffectPlan::new(vec![Effect::ConnectWorkflowFromWorkflow(connection)])
}

pub fn remove_transition(removal: WorkflowTransitionRemoval) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveTransitionFromWorkflow(removal)])
}

pub fn remove_workflow(slug: WorkflowSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveWorkflowFromIndex(slug)])
}

pub fn gherkin_list(suite: GherkinSuite) -> EffectPlan {
    list_gherkin_features(suite)
}

pub fn gherkin_run(suite: GherkinSuite) -> EffectPlan {
    run_gherkin_suite(suite)
}

pub fn gherkin_run_all() -> EffectPlan {
    run_all_gherkin_suites()
}

pub fn init(name: ProjectName) -> EffectPlan {
    init_project(name)
}

pub fn list_workflows() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListWorkflowsFromIndex])
}

pub fn list_slices() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListSlicesFromIndex])
}

pub fn list_transitions() -> EffectPlan {
    EffectPlan::new(vec![Effect::ListTransitionsFromIndex])
}

pub fn review_gate_for_workflow(slug: WorkflowSlug) -> EffectPlan {
    review_gate(slug)
}

pub fn record_clean_review(
    slug: WorkflowSlug,
    reviewer: ReviewerId,
    reviewed_at: ReviewTimestamp,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::RecordCleanReviewFromWorkflow(
        slug,
        reviewer,
        reviewed_at,
    )])
}

pub fn show_workflow(slug: WorkflowSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::ShowWorkflowFromWorkflow(slug)])
}

pub fn show_slice(slug: SliceSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::ShowSliceFromSlice(slug)])
}

pub fn update_workflow_description(
    slug: WorkflowSlug,
    description: ModelDescription,
) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(
        slug,
        description,
    )])
}

pub fn update_workflow_name(slug: WorkflowSlug, name: ModelName) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateWorkflowNameFromIndexAndWorkflow(
        slug, name,
    )])
}

pub fn update_slice_description(slug: SliceSlug, description: ModelDescription) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceDescriptionFromWorkflow(
        slug,
        description,
    )])
}

pub fn update_slice_kind(slug: SliceSlug, kind: SliceKind) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceKindFromWorkflow(slug, kind)])
}

pub fn update_slice_name(slug: SliceSlug, name: ModelName) -> EffectPlan {
    EffectPlan::new(vec![Effect::UpdateSliceNameFromWorkflow(slug, name)])
}

pub fn remove_slice(slug: SliceSlug) -> EffectPlan {
    EffectPlan::new(vec![Effect::RemoveSliceFromWorkflow(slug)])
}

pub fn verify() -> EffectPlan {
    EffectPlan::new(vec![Effect::VerifyProjectFromIndex])
}
