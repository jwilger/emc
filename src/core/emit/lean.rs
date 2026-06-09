// Copyright 2026 John Wilger

use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::emit::{
    lean_slice_kind_name, lean_workflow_entry_lifecycle_state_name,
    lean_workflow_owned_definition_kind, lean_workflow_step_relationship_name,
    lean_workflow_transition_kind,
};
use crate::core::types::{
    BoardLaneId, CommandErrorRecoveryKind, CommandInputSourceKind, EventAttributeSourceKind,
    LeanModuleName, ModelDescription, ModelName, NavigationTargetType, ReadModelFieldSourceKind,
    SingletonRepeatBehavior, SliceKindName, SliceSlug, ViewFieldSourceKind,
    WorkflowCommandErrorRecord, WorkflowCommandErrorRecords, WorkflowEntryLifecycleStateName,
    WorkflowEntryLifecycleStateRecord, WorkflowEntryLifecycleStateRecords, WorkflowModuleData,
    WorkflowOutcomeRecord, WorkflowOutcomeRecords, WorkflowOwnedDefinitionRecord,
    WorkflowOwnedDefinitionRecords, WorkflowSliceDetail, WorkflowTransitionEvidenceRecord,
    WorkflowTransitionEvidenceRecords, WorkflowTransitionRecord, WorkflowTransitionRecords,
};

#[cfg(test)]
#[path = "lean_tests.rs"]
mod external_tests;

pub(crate) fn emit_workflow_module(
    module_name: LeanModuleName,
    workflow_module: WorkflowModuleData,
) -> FileContents {
    let slice_list = slice_list(workflow_module.workflow_slice_details().as_slice());
    let slice_detail_list = slice_detail_list(workflow_module.workflow_slice_details().as_slice());
    let slice_module_list = slice_module_list(workflow_module.workflow_slice_details().as_slice());
    let workflow_step_relationship_list =
        workflow_step_relationship_list(workflow_module.workflow_slice_details().as_slice());
    let workflow_exit_target_list =
        workflow_exit_target_list(workflow_module.workflow_transitions().as_slice());
    let workflow_outcome_list = workflow_outcome_list(workflow_module.workflow_outcomes());
    let workflow_command_error_list =
        workflow_command_error_list(workflow_module.workflow_command_errors());
    let workflow_owned_definition_list =
        workflow_owned_definition_list(workflow_module.workflow_owned_definitions());
    let workflow_transition_evidence_list =
        workflow_transition_evidence_list(workflow_module.workflow_transition_evidences());
    let workflow_entry_lifecycle_state_list =
        workflow_entry_lifecycle_state_list(workflow_module.workflow_entry_lifecycle_states());
    let required_entry_lifecycle_state_list = required_entry_lifecycle_state_list();
    let transition_list = transition_list(workflow_module.workflow_transitions().clone());
    let workflow_requires_entry_lifecycle_coverage =
        workflow_module.workflow_requires_entry_lifecycle_coverage();
    file_contents(format!(
        r#"namespace {module_name}

-- EMC-DIGEST: {digest}
-- EMC generated Lean4 business workflow model.
def workflowName := {workflow_name_json}

def workflowSlug := {workflow_slug_json}

def workflowDescription := {workflow_description_json}

structure WorkflowSlice where
  slug : String

def workflowSlices : List WorkflowSlice := {slice_list}

def workflowSliceSlugs : List String := workflowSlices.map (fun slice => slice.slug)

inductive SliceKindName where
  | stateView
  | stateChange
  | translation
  | automation
deriving BEq, DecidableEq, Repr

structure WorkflowSliceDetail where
  slug : String
  name : String
  kind : SliceKindName
  description : String

structure WorkflowSliceModule where
  slice : String
  formalModule : String

inductive WorkflowTransitionKind where
  | command
  | event
  | navigation
  | externalTrigger
  | outcome
  | workflowExitCommand
  | workflowExitEvent
  | workflowExitNavigation
  | workflowExitExternalTrigger
  | workflowExitOutcome
deriving BEq, DecidableEq, Repr

inductive WorkflowOwnedDefinitionKind where
  | command
  | event
  | view
  | control
  | readModel
  | outcome
  | error
  | automation
  | translation
  | externalPayload
deriving BEq, DecidableEq, Repr

inductive WorkflowStepRelationshipName where
  | entry
  | main
  | branch
  | alternate
  | asyncLifecycle
  | supporting
deriving BEq, DecidableEq, Repr

inductive WorkflowEntryLifecycleStateName where
  | freshUninitialized
  | initializedUnauthenticated
  | initializedAuthenticated
  | partiallyConfigured
  | fullyConfigured
deriving BEq, DecidableEq, Repr

def workflowSliceDetails : List WorkflowSliceDetail := {slice_detail_list}

def workflowSliceModules : List WorkflowSliceModule := {slice_module_list}

structure WorkflowTransition where
  source : String
  target : String
  kind : WorkflowTransitionKind
  trigger : String
  rationale : String
  payloadContract : String

structure WorkflowOutcome where
  sourceSlice : String
  label : String
  externallyRelevant : Bool

structure WorkflowCommandError where
  sourceSlice : String
  commandName : String
  errorName : String

structure WorkflowOwnedDefinition where
  sourceSlice : String
  definitionKind : WorkflowOwnedDefinitionKind
  definitionName : String
  definitionStream : String
  sourceProvenance : String
  eventParticipation : String
  viewRole : String

structure WorkflowTransitionEvidence where
  source : String
  target : String
  kind : WorkflowTransitionKind
  trigger : String
  sourceEvidence : String
  targetEvidence : String

structure WorkflowEntryLifecycleState where
  state : WorkflowEntryLifecycleStateName
  step : String
  evidence : String

def workflowTransitions : List WorkflowTransition := {transition_list}

def workflowOutcomes : List WorkflowOutcome := {workflow_outcome_list}

def workflowCommandErrors : List WorkflowCommandError := {workflow_command_error_list}

def workflowOwnedDefinitions : List WorkflowOwnedDefinition := {workflow_owned_definition_list}

def workflowTransitionEvidences : List WorkflowTransitionEvidence := {workflow_transition_evidence_list}

def workflowRequiresEntryLifecycleCoverage : Bool := {workflow_requires_entry_lifecycle_coverage}

def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := {workflow_entry_lifecycle_state_list}

def workflowExitTargets : List String := {workflow_exit_target_list}

def requiredEntryLifecycleStates : List WorkflowEntryLifecycleStateName := {required_entry_lifecycle_state_list}

structure WorkflowStepRelationship where
  step : String
  relationship : WorkflowStepRelationshipName

def workflowStepRelationships : List WorkflowStepRelationship := {workflow_step_relationship_list}

def workflowStepRelationshipIsAllowed (step : WorkflowStepRelationship) : Bool := workflowSliceSlugs.contains step.step

def workflowStepRelationshipsAreAllowed : Bool := workflowStepRelationships.all workflowStepRelationshipIsAllowed

def workflowStepSlugCount (slug : String) : Nat := (workflowSliceSlugs.filter (fun step => step == slug)).length

def workflowStepSlugsAreUnique : Bool := workflowSliceSlugs.all (fun step => workflowStepSlugCount step == 1)

def workflowEntryStepCount : Nat := (workflowStepRelationships.filter (fun step => step.relationship == WorkflowStepRelationshipName.entry)).length

def workflowHasExactlyOneEntryStep : Bool := workflowEntryStepCount == 1

def workflowMainStepHasIncomingTransition (step : WorkflowStepRelationship) : Bool := step.relationship != WorkflowStepRelationshipName.main || workflowTransitions.any (fun transition => transition.target == step.step)

def workflowMainStepsHaveIncomingReachability : Bool := workflowStepRelationships.all workflowMainStepHasIncomingTransition

def workflowEntrySteps : List String := (workflowStepRelationships.filter (fun step => step.relationship == WorkflowStepRelationshipName.entry)).map (fun step => step.step)

def workflowTargetsFromReachable (reachable : List String) : List String := (workflowTransitions.filter (fun transition => reachable.contains transition.source && workflowSliceSlugs.contains transition.target)).map (fun transition => transition.target)

def workflowReachableStepsAfterFuel : Nat -> List String -> List String
  | Nat.zero, reachable => reachable
  | Nat.succ fuel, reachable => workflowReachableStepsAfterFuel fuel (reachable ++ workflowTargetsFromReachable reachable)

def workflowReachableStepsFromEntry : List String := workflowReachableStepsAfterFuel workflowSlices.length workflowEntrySteps

def workflowStepIsReachableFromEntry (step : WorkflowStepRelationship) : Bool := step.relationship == WorkflowStepRelationshipName.supporting || step.relationship == WorkflowStepRelationshipName.asyncLifecycle || workflowReachableStepsFromEntry.contains step.step

def workflowNonSupportingStepsReachableFromEntry : Bool := workflowStepRelationships.all workflowStepIsReachableFromEntry

def workflowBranchOrAlternateStepHasTriggerOrRationale (step : WorkflowStepRelationship) : Bool := (step.relationship != WorkflowStepRelationshipName.branch && step.relationship != WorkflowStepRelationshipName.alternate) || workflowTransitions.any (fun transition => transition.target == step.step && (transition.trigger.isEmpty == false || transition.rationale.isEmpty == false))

def workflowBranchAndAlternateStepsHaveTriggerOrRationale : Bool := workflowStepRelationships.all workflowBranchOrAlternateStepHasTriggerOrRationale

def workflowTransitionKindIsModeled (transition : WorkflowTransition) : Bool := transition.kind == WorkflowTransitionKind.navigation || transition.kind == WorkflowTransitionKind.command || transition.kind == WorkflowTransitionKind.event || transition.kind == WorkflowTransitionKind.externalTrigger || transition.kind == WorkflowTransitionKind.outcome || transition.kind == WorkflowTransitionKind.workflowExitNavigation || transition.kind == WorkflowTransitionKind.workflowExitCommand || transition.kind == WorkflowTransitionKind.workflowExitEvent || transition.kind == WorkflowTransitionKind.workflowExitExternalTrigger || transition.kind == WorkflowTransitionKind.workflowExitOutcome

def workflowTransitionExitHasRationale (transition : WorkflowTransition) : Bool := workflowExitTargets.contains transition.target == false || transition.rationale.isEmpty == false

def workflowTransitionsHaveModeledKinds : Bool := workflowTransitions.all workflowTransitionKindIsModeled

def workflowExitsNameTargetsAndRationale : Bool := workflowTransitions.all workflowTransitionExitHasRationale

def workflowOutcomeHandledByTransition (outcome : WorkflowOutcome) : Bool := outcome.externallyRelevant == false || workflowTransitions.any (fun transition => transition.source == outcome.sourceSlice && transition.kind == WorkflowTransitionKind.outcome && transition.trigger == outcome.label)

def workflowExternallyRelevantOutcomesHandled : Bool := workflowOutcomes.all workflowOutcomeHandledByTransition

def workflowOutcomeSourceResolves (outcome : WorkflowOutcome) : Bool := workflowSliceSlugs.contains outcome.sourceSlice

def workflowOutcomesSourceResolve : Bool := workflowOutcomes.all workflowOutcomeSourceResolves

def workflowCommandErrorSourceResolves (error : WorkflowCommandError) : Bool := workflowSliceSlugs.contains error.sourceSlice

def workflowCommandErrorsSourceResolve : Bool := workflowCommandErrors.all workflowCommandErrorSourceResolves

def workflowTransitionIsNotCommandErrorOutcome (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.outcome || workflowCommandErrors.any (fun error => error.sourceSlice == transition.source && error.errorName == transition.trigger) == false

def workflowTransitionsDoNotUseCommandErrorsAsOutcomes : Bool := workflowTransitions.all workflowTransitionIsNotCommandErrorOutcome

def workflowNonEventDefinitionOwnedOnce (definition : WorkflowOwnedDefinition) : Bool := definition.definitionKind == WorkflowOwnedDefinitionKind.event || (workflowOwnedDefinitions.filter (fun other => other.definitionKind == definition.definitionKind && other.definitionName == definition.definitionName)).length == 1

def workflowNonEventDefinitionsAreUniquelyOwned : Bool := workflowOwnedDefinitions.all workflowNonEventDefinitionOwnedOnce

def workflowEventDefinitionHasIdentity (definition : WorkflowOwnedDefinition) : Bool := definition.definitionKind != WorkflowOwnedDefinitionKind.event || (definition.definitionStream.isEmpty == false && definition.sourceProvenance.isEmpty == false)

def workflowSharedEventDefinitionMatches (left : WorkflowOwnedDefinition) (right : WorkflowOwnedDefinition) : Bool := left.definitionKind != WorkflowOwnedDefinitionKind.event || right.definitionKind != WorkflowOwnedDefinitionKind.event || left.definitionName != right.definitionName || (left.definitionStream == right.definitionStream && left.sourceProvenance == right.sourceProvenance)

def workflowSharedEventDefinitionsHaveIdenticalIdentity : Bool := workflowOwnedDefinitions.all workflowEventDefinitionHasIdentity && workflowOwnedDefinitions.all (fun definition => workflowOwnedDefinitions.all (workflowSharedEventDefinitionMatches definition))

def workflowOnlyEventsMayBeSharedAcrossSlices : Bool := workflowNonEventDefinitionsAreUniquelyOwned && workflowSharedEventDefinitionsHaveIdenticalIdentity

def workflowOwnsDefinition (sourceSlice : String) (definitionKind : WorkflowOwnedDefinitionKind) (definitionName : String) : Bool := workflowOwnedDefinitions.any (fun definition => definition.sourceSlice == sourceSlice && definition.definitionKind == definitionKind && definition.definitionName == definitionName)

def workflowSliceHasKind (slice : String) (kind : SliceKindName) : Bool := workflowSliceDetails.any (fun detail => detail.slug == slice && detail.kind == kind)

def workflowEventParticipationIsModeled (definition : WorkflowOwnedDefinition) : Bool := definition.eventParticipation == "emitted" || definition.eventParticipation == "observed"

def workflowEventDefinitionParticipates (sourceSlice : String) (eventName : String) : Bool := workflowOwnedDefinitions.any (fun definition => definition.sourceSlice == sourceSlice && definition.definitionKind == WorkflowOwnedDefinitionKind.event && definition.definitionName == eventName && workflowEventParticipationIsModeled definition)

def workflowViewRoleIsEntry (definition : WorkflowOwnedDefinition) : Bool := definition.viewRole == "entry"

def workflowOwnsEntryView (sourceSlice : String) (viewName : String) : Bool := workflowOwnedDefinitions.any (fun definition => definition.sourceSlice == sourceSlice && definition.definitionKind == WorkflowOwnedDefinitionKind.view && definition.definitionName == viewName && workflowViewRoleIsEntry definition)

def workflowCommandTransitionTargetsOwnedCommand (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.command || workflowOwnsDefinition transition.target WorkflowOwnedDefinitionKind.command transition.trigger

def workflowCommandTransitionsTargetOwnedCommands : Bool := workflowTransitions.all workflowCommandTransitionTargetsOwnedCommand

def workflowCommandTransitionSourceOwnsControl (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.command || workflowOwnsDefinition transition.source WorkflowOwnedDefinitionKind.control transition.trigger

def workflowCommandTransitionsSourceOwnedControls : Bool := workflowTransitions.all workflowCommandTransitionSourceOwnsControl

def workflowCommandTransitionsResolveControlsAndCommands : Bool := workflowTransitions.all (fun transition => workflowCommandTransitionSourceOwnsControl transition && workflowCommandTransitionTargetsOwnedCommand transition)

def workflowStateViewCommandTransitionTargetsStateChange (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.command || workflowSliceHasKind transition.source SliceKindName.stateView == false || workflowSliceHasKind transition.target SliceKindName.stateChange

def workflowStateViewCommandTransitionsTargetStateChanges : Bool := workflowTransitions.all workflowStateViewCommandTransitionTargetsStateChange

def workflowEventTransitionIsSharedByEndpoints (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.event || (workflowOwnsDefinition transition.source WorkflowOwnedDefinitionKind.event transition.trigger && workflowOwnsDefinition transition.target WorkflowOwnedDefinitionKind.event transition.trigger)

def workflowEventTransitionsAreSharedByEndpointSlices : Bool := workflowTransitions.all workflowEventTransitionIsSharedByEndpoints

def workflowEventTransitionSourceParticipates (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.event || workflowEventDefinitionParticipates transition.source transition.trigger

def workflowEventTransitionTargetParticipates (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.event || workflowEventDefinitionParticipates transition.target transition.trigger

def workflowEventTransitionsHaveParticipatingEndpointEvents : Bool := workflowTransitions.all (fun transition => workflowEventTransitionSourceParticipates transition && workflowEventTransitionTargetParticipates transition)

def workflowNavigationTransitionSourceOwnsControl (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.navigation || workflowOwnsDefinition transition.source WorkflowOwnedDefinitionKind.control transition.trigger

def workflowNavigationTransitionTargetsOwnedView (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.navigation || workflowOwnsDefinition transition.target WorkflowOwnedDefinitionKind.view transition.trigger

def workflowNavigationTransitionTargetsEntryView (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.navigation || workflowOwnsEntryView transition.target transition.trigger

def workflowNavigationTransitionsResolveControlsAndViews : Bool := workflowTransitions.all (fun transition => workflowNavigationTransitionSourceOwnsControl transition && workflowNavigationTransitionTargetsOwnedView transition)

def workflowNavigationTransitionsResolveToEntryViews : Bool := workflowTransitions.all workflowNavigationTransitionTargetsEntryView

def workflowExternalTriggerDeclaresPayloadContract (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.externalTrigger || (transition.payloadContract.isEmpty == false && workflowOwnsDefinition transition.source WorkflowOwnedDefinitionKind.externalPayload transition.payloadContract)

def workflowExternalTriggersDeclarePayloadContracts : Bool := workflowTransitions.all workflowExternalTriggerDeclaresPayloadContract

def workflowExternalTriggerPayloadContractHasProvenance (transition : WorkflowTransition) : Bool := transition.kind != WorkflowTransitionKind.externalTrigger || workflowOwnedDefinitions.any (fun definition => definition.sourceSlice == transition.source && definition.definitionKind == WorkflowOwnedDefinitionKind.externalPayload && definition.definitionName == transition.payloadContract && definition.sourceProvenance.isEmpty == false)

def workflowExternalTriggerPayloadContractsHaveProvenance : Bool := workflowTransitions.all workflowExternalTriggerPayloadContractHasProvenance

def workflowTransitionRequiresEvidence (transition : WorkflowTransition) : Bool := transition.kind == WorkflowTransitionKind.event || transition.kind == WorkflowTransitionKind.command || transition.kind == WorkflowTransitionKind.navigation

def workflowTransitionEvidenceMatches (transition : WorkflowTransition) (evidence : WorkflowTransitionEvidence) : Bool := evidence.source == transition.source && evidence.target == transition.target && evidence.kind == transition.kind && evidence.trigger == transition.trigger && evidence.sourceEvidence.isEmpty == false && evidence.targetEvidence.isEmpty == false

def workflowTransitionHasRequiredEvidence (transition : WorkflowTransition) : Bool := workflowTransitionRequiresEvidence transition == false || workflowTransitionEvidences.any (workflowTransitionEvidenceMatches transition)

def workflowTransitionsHaveRequiredEvidence : Bool := workflowTransitions.all workflowTransitionHasRequiredEvidence

def workflowEntryLifecycleStateCovered (state : WorkflowEntryLifecycleStateName) : Bool := workflowEntryLifecycleStates.any (fun coverage => coverage.state == state && workflowSliceSlugs.contains coverage.step && coverage.evidence.isEmpty == false)

def workflowEntryLifecycleStatesCoverRequiredStates : Bool := workflowRequiresEntryLifecycleCoverage == false || requiredEntryLifecycleStates.all workflowEntryLifecycleStateCovered

theorem workflowIdentityIsStable : workflowName = {workflow_name_json} := rfl

theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl

theorem workflowSlicesHaveModuleReferences : workflowSlices.length = workflowSliceModules.length := rfl

theorem workflowTransitionsAreStructured : workflowTransitions.all (fun transition => transition.source.isEmpty == false && transition.target.isEmpty == false && transition.trigger.isEmpty == false) = true := rfl

theorem workflowTransitionSourcesResolve : workflowTransitions.all (fun transition => workflowSliceSlugs.contains transition.source) = true := rfl

theorem workflowTransitionTargetsResolve : workflowTransitions.all (fun transition => workflowSliceSlugs.contains transition.target || workflowExitTargets.contains transition.target) = true := rfl

theorem workflowStepRelationshipsAreAllowedIsStable : workflowStepRelationshipsAreAllowed = true := rfl

theorem workflowStepSlugsAreUniqueIsStable : workflowStepSlugsAreUnique = true := rfl

theorem workflowHasExactlyOneEntryStepIsStable : workflowHasExactlyOneEntryStep = true := rfl

theorem workflowMainStepsHaveIncomingReachabilityIsStable : workflowMainStepsHaveIncomingReachability = true := rfl

theorem workflowNonSupportingStepsReachableFromEntryIsStable : workflowNonSupportingStepsReachableFromEntry = true := rfl

theorem workflowBranchAndAlternateStepsHaveTriggerOrRationaleIsStable : workflowBranchAndAlternateStepsHaveTriggerOrRationale = true := rfl

theorem workflowTransitionsHaveModeledKindsIsStable : workflowTransitionsHaveModeledKinds = true := rfl

theorem workflowExitsNameTargetsAndRationaleIsStable : workflowExitsNameTargetsAndRationale = true := rfl

theorem workflowExternallyRelevantOutcomesHandledIsStable : workflowExternallyRelevantOutcomesHandled = true := rfl

theorem workflowOutcomesSourceResolveIsStable : workflowOutcomesSourceResolve = true := rfl

theorem workflowCommandErrorsSourceResolveIsStable : workflowCommandErrorsSourceResolve = true := rfl

theorem workflowTransitionsDoNotUseCommandErrorsAsOutcomesIsStable : workflowTransitionsDoNotUseCommandErrorsAsOutcomes = true := rfl

theorem workflowNonEventDefinitionsAreUniquelyOwnedIsStable : workflowNonEventDefinitionsAreUniquelyOwned = true := rfl

theorem workflowSharedEventDefinitionsHaveIdenticalIdentityIsStable : workflowSharedEventDefinitionsHaveIdenticalIdentity = true := rfl

theorem workflowOnlyEventsMayBeSharedAcrossSlicesIsStable : workflowOnlyEventsMayBeSharedAcrossSlices = true := rfl

theorem workflowCommandTransitionsTargetOwnedCommandsIsStable : workflowCommandTransitionsTargetOwnedCommands = true := rfl

theorem workflowCommandTransitionsSourceOwnedControlsIsStable : workflowCommandTransitionsSourceOwnedControls = true := rfl

theorem workflowCommandTransitionsResolveControlsAndCommandsIsStable : workflowCommandTransitionsResolveControlsAndCommands = true := rfl

theorem workflowStateViewCommandTransitionsTargetStateChangesIsStable : workflowStateViewCommandTransitionsTargetStateChanges = true := rfl

theorem workflowEventTransitionsAreSharedByEndpointSlicesIsStable : workflowEventTransitionsAreSharedByEndpointSlices = true := rfl

theorem workflowEventTransitionsHaveParticipatingEndpointEventsIsStable : workflowEventTransitionsHaveParticipatingEndpointEvents = true := rfl

theorem workflowNavigationTransitionsResolveControlsAndViewsIsStable : workflowNavigationTransitionsResolveControlsAndViews = true := rfl

theorem workflowNavigationTransitionsResolveToEntryViewsIsStable : workflowNavigationTransitionsResolveToEntryViews = true := rfl

theorem workflowExternalTriggersDeclarePayloadContractsIsStable : workflowExternalTriggersDeclarePayloadContracts = true := rfl

theorem workflowExternalTriggerPayloadContractsHaveProvenanceIsStable : workflowExternalTriggerPayloadContractsHaveProvenance = true := rfl

theorem workflowTransitionsHaveRequiredEvidenceIsStable : workflowTransitionsHaveRequiredEvidence = true := rfl

theorem workflowEntryLifecycleStatesCoverRequiredStatesIsStable : workflowEntryLifecycleStatesCoverRequiredStates = true := rfl

end {module_name}
"#,
        module_name = module_name.as_ref(),
        digest = workflow_module.digest().as_ref(),
        workflow_name_json = quoted(workflow_module.workflow_name().as_ref()),
        workflow_slug_json = quoted(workflow_module.workflow_slug().as_ref()),
        workflow_description_json = quoted(workflow_module.workflow_description().as_ref()),
        slice_list = slice_list,
        slice_detail_list = slice_detail_list,
        slice_module_list = slice_module_list,
        workflow_step_relationship_list = workflow_step_relationship_list,
        transition_list = transition_list,
        workflow_outcome_list = workflow_outcome_list,
        workflow_command_error_list = workflow_command_error_list,
        workflow_owned_definition_list = workflow_owned_definition_list,
        workflow_transition_evidence_list = workflow_transition_evidence_list,
        workflow_requires_entry_lifecycle_coverage = workflow_requires_entry_lifecycle_coverage,
        workflow_entry_lifecycle_state_list = workflow_entry_lifecycle_state_list,
        required_entry_lifecycle_state_list = required_entry_lifecycle_state_list,
        workflow_exit_target_list = workflow_exit_target_list,
    ))
}

pub(crate) fn emit_slice_module(
    module_name: LeanModuleName,
    slice_name: ModelName,
    slice_description: ModelDescription,
    slice_slug: SliceSlug,
    slice_kind: SliceKindName,
    digest: ArtifactDigest,
) -> FileContents {
    let allowed_command_input_source_kind_list = command_input_source_kind_list();
    let allowed_recovery_kind_list = command_error_recovery_kind_list();
    let allowed_singleton_repeat_behavior_list = singleton_repeat_behavior_list();
    let allowed_navigation_target_type_list = navigation_target_type_list();
    let stored_event_fact_source_kind_list = event_attribute_source_kind_list();
    let allowed_read_model_field_source_kind_list = read_model_field_source_kind_list();
    let allowed_view_field_source_kind_list = view_field_source_kind_list();
    let canonical_board_lane_list = board_lane_id_list();
    let contents = format!(
        "namespace {module_name}\n\n-- EMC-DIGEST: {digest}\n-- EMC generated Lean4 business slice model.\ndef sliceName := {slice_name_json}\n\ndef sliceSlug := {slice_slug_json}\n\ninductive SliceKindName where\n  | stateView\n  | stateChange\n  | translation\n  | automation\nderiving BEq, DecidableEq, Repr\n\ndef sliceKind : SliceKindName := {slice_kind}\n\ndef sliceDescription := {slice_description_json}\n\nstructure EventModelScenario where\n  name : String\n  givenSteps : List String\n  whenSteps : List String\n  thenSteps : List String\n\nstructure BitLevelDataFlow where\n  datum : String\n  sourceKind : String\n  source : String\n  transformationSemantics : String\n  target : String\n  bitEncoding : String\n\nstructure CommandInput where\n  name : String\n  sourceKind : String\n  sourceDescription : String\n  provenanceChain : List String\n  eventStreamSourceEvent : String\n  eventStreamSourceAttribute : String\n  externalPayloadSourceName : String\n  externalPayloadSourceField : String\n  generatedSourceName : String\n  generatedSourceField : String\n  sessionSourceName : String\n  sessionSourceField : String\n  invocationArgumentSourceName : String\n  invocationArgumentSourceField : String\n\nstructure CommandErrorDefinition where\n  name : String\n  scenarioName : String\n  recoveryKind : String\n\nstructure CommandDefinition where\n  name : String\n  inputs : List CommandInput\n  emittedEvents : List String\n  observedStreams : List String\n  errors : List CommandErrorDefinition\n  singleton : Bool\n  repeatBehavior : String\n\nstructure OutcomeDefinition where\n  label : String\n  eventSet : List String\n  externallyRelevant : Bool\n\nstructure StreamDefinition where\n  name : String\n\nstructure EventAttribute where\n  name : String\n  sourceKind : String\n  sourceName : String\n  sourceField : String\n  generatedSourceKind : String\n  provenanceDescription : String\n\nstructure EventDefinition where\n  name : String\n  stream : String\n  attributes : List EventAttribute\n  observed : Bool\n  shared : Bool\n\nstructure ReadModelField where\n  name : String\n  sourceKind : String\n  sourceEvent : String\n  sourceAttribute : String\n  derivationRule : String\n  absenceEvent : String\n  provenanceDescription : String\n\nstructure ReadModelDefinition where\n  name : String\n  fields : List ReadModelField\n\nstructure ViewField where\n  name : String\n  sourceKind : String\n  sourceReadModel : String\n  sourceField : String\n  sketchToken : String\n  provenanceDescription : String\n  bitEncoding : String\n\nstructure ControlInputProvision where\n  name : String\n  sourceKind : String\n  sourceDescription : String\n  sketchToken : String\n  visibleToActor : Bool\n  decisionField : Bool\n\nstructure ControlDefinition where\n  name : String\n  commandName : String\n  inputs : List ControlInputProvision\n  handledErrors : List String\n  recoveryBehavior : String\n  sketchToken : String\n\nstructure ViewDefinition where\n  name : String\n  readModels : List String\n  fields : List ViewField\n  controls : List ControlDefinition\n  sketchTokens : List String\n\ndef sliceCommands : List String := []\n\ndef sliceCommandDefinitions : List CommandDefinition := []\n\ndef sliceReferencedCommands : List String := []\n\ndef sliceOutcomeDefinitions : List OutcomeDefinition := []\n\ndef allowedCommandInputSourceKinds : List String := [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]\n\ndef allowedRecoveryKinds : List String := [\"retry\",\"stay_on_screen\",\"navigation\",\"explicit_recovery_action\"]\n\ndef allowedSingletonRepeatBehaviors : List String := [\"already_exists_error\",\"idempotent\"]\n\ndef sliceEvents : List String := []\n\ndef sliceStreams : List StreamDefinition := []\n\ndef sliceEventDefinitions : List EventDefinition := []\n\ndef storedEventFactSourceKinds : List String := [\"command_input\",\"external_payload\",\"generated\",\"session\",\"derivation\"]\n\ndef allowedEventAttributeSourceKinds : List String := storedEventFactSourceKinds\n\ndef sliceReadModels : List String := []\n\ndef sliceReadModelDefinitions : List ReadModelDefinition := []\n\ndef allowedReadModelFieldSourceKinds : List String := [\"event_attribute\",\"derivation\",\"absence_default\"]\n\ndef sliceViews : List String := []\n\ndef sliceViewDefinitions : List ViewDefinition := []\n\ndef allowedViewFieldSourceKinds : List String := [\"read_model\"]\n\ndef allowedControlInputSourceKinds : List String := [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]\n\ndef sliceAcceptanceScenarios : List EventModelScenario := []\n\ndef sliceContractScenarios : List EventModelScenario := []\n\ndef sliceBitLevelDataFlows : List BitLevelDataFlow := []\n\ndef scenarioHasGwt (scenario : EventModelScenario) : Bool := scenario.name.isEmpty == false && scenario.givenSteps.isEmpty == false && scenario.whenSteps.isEmpty == false && scenario.thenSteps.isEmpty == false\n\ndef sliceScenariosHaveGwt : Bool := sliceAcceptanceScenarios.all scenarioHasGwt && sliceContractScenarios.all scenarioHasGwt\n\ndef scenarioNameCount (name : String) (scenarios : List EventModelScenario) : Nat := (scenarios.filter (fun scenario => scenario.name == name)).length\n\ndef scenarioNamesAreUnique (scenarios : List EventModelScenario) : Bool := scenarios.all (fun scenario => scenarioNameCount scenario.name scenarios == 1)\n\ndef sliceScenarioNamesAreUnique : Bool := scenarioNamesAreUnique (sliceAcceptanceScenarios ++ sliceContractScenarios)\n\ndef stringNameCount (name : String) (names : List String) : Nat := (names.filter (fun other => other == name)).length\n\ndef definitionNamesAreUnique (names : List String) : Bool := names.all (fun name => stringNameCount name names == 1)\n\ndef sliceOwnedCommandNames : List String := sliceCommandDefinitions.map (fun command => command.name)\n\ndef sliceOwnedEventNames : List String := sliceEventDefinitions.map (fun event => event.name)\n\ndef sliceOwnedStreamNames : List String := sliceStreams.map (fun stream => stream.name)\n\ndef sliceOwnedExternalPayloadNames : List String := sliceExternalPayloads.map (fun payload => payload.name)\n\ndef sliceOwnedReadModelNames : List String := sliceReadModelDefinitions.map (fun readModel => readModel.name)\n\ndef sliceOwnedViewNames : List String := sliceViewDefinitions.map (fun view => view.name)\n\ndef sliceOwnedAutomationNames : List String := sliceAutomations.map (fun automation => automation.name)\n\ndef sliceOwnedTranslationNames : List String := sliceTranslations.map (fun translation => translation.name)\n\ndef sliceOwnedControlNames : List String := sliceViewDefinitions.flatMap (fun view => view.controls.map (fun control => control.name))\n\ndef sliceNamedDefinitionsAreUniquelyOwned : Bool := definitionNamesAreUnique sliceCommands && definitionNamesAreUnique sliceOwnedCommandNames && definitionNamesAreUnique sliceEvents && definitionNamesAreUnique sliceOwnedEventNames && definitionNamesAreUnique sliceOwnedStreamNames && definitionNamesAreUnique sliceOwnedExternalPayloadNames && definitionNamesAreUnique sliceReadModels && definitionNamesAreUnique sliceOwnedReadModelNames && definitionNamesAreUnique sliceViews && definitionNamesAreUnique sliceOwnedViewNames && definitionNamesAreUnique sliceOwnedAutomationNames && definitionNamesAreUnique sliceOwnedTranslationNames && definitionNamesAreUnique sliceOwnedControlNames\n\ndef commandInputHasAllowedSource (input : CommandInput) : Bool := allowedCommandInputSourceKinds.contains input.sourceKind\n\ndef commandInputHasProvenance (input : CommandInput) : Bool := input.name.isEmpty == false && input.sourceKind.isEmpty == false && input.sourceDescription.isEmpty == false && input.provenanceChain.isEmpty == false\n\ndef commandInputsHaveAllowedSources : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasAllowedSource)\n\ndef commandInputsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasProvenance)\n\ndef commandErrorHasDeclaration (error : CommandErrorDefinition) : Bool := error.name.isEmpty == false && error.scenarioName.isEmpty == false && error.recoveryKind.isEmpty == false\n\ndef commandErrorHasAllowedRecovery (error : CommandErrorDefinition) : Bool := allowedRecoveryKinds.contains error.recoveryKind\n\ndef commandErrorsAreDeclared : Bool := sliceCommandDefinitions.all (fun command => command.errors.all commandErrorHasDeclaration)\n\ndef commandErrorsHaveAllowedRecovery : Bool := sliceCommandDefinitions.all (fun command => command.errors.all commandErrorHasAllowedRecovery)\n\ndef outcomeLabelCount (label : String) : Nat := (sliceOutcomeDefinitions.filter (fun outcome => outcome.label == label)).length\n\ndef outcomeLabelsAreUnique : Bool := sliceOutcomeDefinitions.all (fun outcome => outcomeLabelCount outcome.label == 1)\n\ndef outcomeEventSetsAreNonEmpty : Bool := sliceOutcomeDefinitions.all (fun outcome => outcome.eventSet.isEmpty == false)\n\ndef sameOutcomeEventSet (left : OutcomeDefinition) (right : OutcomeDefinition) : Bool := left.eventSet.all (fun eventName => right.eventSet.contains eventName) && right.eventSet.all (fun eventName => left.eventSet.contains eventName)\n\ndef eventIsKnownToSlice (eventName : String) : Bool := sliceEvents.contains eventName || sliceEventDefinitions.any (fun event => event.name == eventName && (event.observed || event.shared))\n\ndef outcomeEventSetsAreDistinct : Bool := sliceOutcomeDefinitions.all (fun outcome => sliceOutcomeDefinitions.all (fun other => outcome.label == other.label || sameOutcomeEventSet outcome other == false))\n\ndef outcomeEventsAreKnownToSlice : Bool := sliceOutcomeDefinitions.all (fun outcome => outcome.eventSet.all eventIsKnownToSlice)\n\ndef eventReferencesKnownStream (event : EventDefinition) : Bool := sliceStreams.any (fun stream => stream.name == event.stream)\n\ndef eventAttributeHasAllowedSource (eventAttribute : EventAttribute) : Bool := allowedEventAttributeSourceKinds.contains eventAttribute.sourceKind\n\ndef eventAttributeHasProvenance (eventAttribute : EventAttribute) : Bool := eventAttribute.name.isEmpty == false && eventAttribute.sourceKind.isEmpty == false && eventAttribute.sourceName.isEmpty == false && eventAttribute.provenanceDescription.isEmpty == false\n\ndef eventsReferenceKnownStreams : Bool := sliceEventDefinitions.all eventReferencesKnownStream\n\ndef eventAttributesHaveAllowedSources : Bool := sliceEventDefinitions.all (fun event => event.attributes.all eventAttributeHasAllowedSource)\n\ndef eventAttributesHaveProvenance : Bool := sliceEventDefinitions.all (fun event => event.attributes.all eventAttributeHasProvenance)\n\ndef readModelFieldHasAllowedSource (field : ReadModelField) : Bool := allowedReadModelFieldSourceKinds.contains field.sourceKind\n\ndef readModelFieldHasProvenance (field : ReadModelField) : Bool := field.name.isEmpty == false && field.sourceKind.isEmpty == false && field.provenanceDescription.isEmpty == false\n\ndef readModelFieldSourceIsComplete (field : ReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && field.sourceEvent.isEmpty == false && field.sourceAttribute.isEmpty == false) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false)\n\ndef readModelFieldsHaveAllowedSources : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldHasAllowedSource)\n\ndef readModelFieldsHaveProvenance : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldHasProvenance)\n\ndef readModelFieldSourcesAreComplete : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldSourceIsComplete)\n\ndef viewFieldHasAllowedSource (field : ViewField) : Bool := allowedViewFieldSourceKinds.contains field.sourceKind\n\ndef viewFieldHasProvenance (field : ViewField) : Bool := field.name.isEmpty == false && field.sourceKind.isEmpty == false && field.provenanceDescription.isEmpty == false && field.bitEncoding.isEmpty == false\n\ndef viewFieldSourceIsComplete (field : ViewField) : Bool := field.sourceKind == \"read_model\" && field.sourceReadModel.isEmpty == false && field.sourceField.isEmpty == false && field.sketchToken.isEmpty == false\n\ndef viewFieldSourceReadModelIsUsed (view : ViewDefinition) (field : ViewField) : Bool := view.readModels.contains field.sourceReadModel && sliceReadModels.contains field.sourceReadModel\n\ndef viewFieldsHaveAllowedSources : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldHasAllowedSource)\n\ndef viewFieldsHaveProvenance : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldHasProvenance)\n\ndef viewFieldSourcesAreComplete : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldSourceIsComplete)\n\ndef viewFieldsSourceFromUsedReadModels : Bool := sliceViewDefinitions.all (fun view => view.fields.all (viewFieldSourceReadModelIsUsed view))\n\ndef controlInputHasAllowedSource (input : ControlInputProvision) : Bool := allowedControlInputSourceKinds.contains input.sourceKind\n\ndef controlInputHasProvenance (input : ControlInputProvision) : Bool := input.name.isEmpty == false && input.sourceKind.isEmpty == false && input.sourceDescription.isEmpty == false\n\ndef controlInputVisibilityIsModeled (input : ControlInputProvision) : Bool := (input.sourceKind != \"actor\" || input.sketchToken.isEmpty == false || input.visibleToActor) && (input.decisionField == false || input.sketchToken.isEmpty == false || input.visibleToActor)\n\ndef controlHasSketchToken (control : ControlDefinition) : Bool := control.name.isEmpty == false && control.commandName.isEmpty == false && control.sketchToken.isEmpty == false\n\ndef controlReferencesKnownCommand (control : ControlDefinition) : Bool := sliceCommands.contains control.commandName || sliceReferencedCommands.contains control.commandName || sliceCommandDefinitions.any (fun command => command.name == control.commandName)\n\ndef commandErrorsHandledByControl (control : ControlDefinition) (command : CommandDefinition) : Bool := command.name != control.commandName || command.errors.all (fun error => control.handledErrors.contains error.name && control.recoveryBehavior.isEmpty == false)\n\ndef viewControlsHaveSketchTokens : Bool := sliceViewDefinitions.all (fun view => view.controls.all controlHasSketchToken)\n\ndef viewControlsReferenceKnownCommands : Bool := sliceViewDefinitions.all (fun view => view.controls.all controlReferencesKnownCommand)\n\ndef viewControlInputsHaveAllowedSources : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasAllowedSource))\n\ndef viewControlInputsHaveProvenance : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasProvenance))\n\ndef viewControlInputVisibilityIsModeled : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputVisibilityIsModeled))\n\ndef viewControlsHandleCommandErrors : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => sliceCommandDefinitions.all (commandErrorsHandledByControl control)))\n\ndef sliceHasLocallyEmittedEvent : Bool := sliceEvents.isEmpty == false || sliceEventDefinitions.any (fun event => event.observed == false && event.shared == false)

def sliceStateChangeRequiresEvent : Prop := sliceKind = SliceKindName.stateChange -> sliceHasLocallyEmittedEvent\n\ntheorem sliceIdentityIsStable : sliceName = {slice_name_json} := rfl\n\ntheorem sliceBitLevelDataFlowsAreStructured : sliceBitLevelDataFlows.all (fun flow => flow.datum.isEmpty == false && flow.sourceKind.isEmpty == false && flow.source.isEmpty == false && flow.transformationSemantics.isEmpty == false && flow.target.isEmpty == false && flow.bitEncoding.isEmpty == false) = true := rfl\n\ntheorem sliceScenariosHaveGwtIsStable : sliceScenariosHaveGwt = true := rfl\n\ntheorem sliceScenarioNamesAreUniqueIsStable : sliceScenarioNamesAreUnique = true := rfl\n\ntheorem sliceNamedDefinitionsAreUniquelyOwnedIsStable : sliceNamedDefinitionsAreUniquelyOwned = true := by\n  native_decide\n\ntheorem commandInputsHaveAllowedSourcesIsStable : commandInputsHaveAllowedSources = true := rfl\n\ntheorem commandInputsHaveProvenanceIsStable : commandInputsHaveProvenance = true := rfl\n\ntheorem commandErrorsAreDeclaredIsStable : commandErrorsAreDeclared = true := rfl\n\ntheorem commandErrorsHaveAllowedRecoveryIsStable : commandErrorsHaveAllowedRecovery = true := rfl\n\ntheorem outcomeLabelsAreUniqueIsStable : outcomeLabelsAreUnique = true := rfl\n\ntheorem outcomeEventSetsAreNonEmptyIsStable : outcomeEventSetsAreNonEmpty = true := rfl\n\ntheorem outcomeEventSetsAreDistinctIsStable : outcomeEventSetsAreDistinct = true := rfl\n\ntheorem outcomeEventsAreKnownToSliceIsStable : outcomeEventsAreKnownToSlice = true := rfl\n\ntheorem eventsReferenceKnownStreamsIsStable : eventsReferenceKnownStreams = true := rfl\n\ntheorem eventAttributesHaveAllowedSourcesIsStable : eventAttributesHaveAllowedSources = true := rfl\n\ntheorem eventAttributesHaveProvenanceIsStable : eventAttributesHaveProvenance = true := rfl\n\ntheorem readModelFieldsHaveAllowedSourcesIsStable : readModelFieldsHaveAllowedSources = true := rfl\n\ntheorem readModelFieldsHaveProvenanceIsStable : readModelFieldsHaveProvenance = true := rfl\n\ntheorem readModelFieldSourcesAreCompleteIsStable : readModelFieldSourcesAreComplete = true := rfl\n\ntheorem viewFieldsHaveAllowedSourcesIsStable : viewFieldsHaveAllowedSources = true := rfl\n\ntheorem viewFieldsHaveProvenanceIsStable : viewFieldsHaveProvenance = true := rfl\n\ntheorem viewFieldSourcesAreCompleteIsStable : viewFieldSourcesAreComplete = true := rfl\n\ntheorem viewFieldsSourceFromUsedReadModelsIsStable : viewFieldsSourceFromUsedReadModels = true := rfl\n\ntheorem viewControlsHaveSketchTokensIsStable : viewControlsHaveSketchTokens = true := rfl\n\ntheorem viewControlsReferenceKnownCommandsIsStable : viewControlsReferenceKnownCommands = true := rfl\n\ntheorem viewControlInputsHaveAllowedSourcesIsStable : viewControlInputsHaveAllowedSources = true := rfl\n\ntheorem viewControlInputsHaveProvenanceIsStable : viewControlInputsHaveProvenance = true := rfl\n\ntheorem viewControlInputVisibilityIsModeledIsStable : viewControlInputVisibilityIsModeled = true := rfl\n\ntheorem viewControlsHandleCommandErrorsIsStable : viewControlsHandleCommandErrors = true := rfl\n\nend {module_name}\n",
        module_name = module_name.as_ref(),
        digest = digest.as_ref(),
        slice_name_json = quoted(slice_name.as_ref()),
        slice_slug_json = quoted(slice_slug.as_ref()),
        slice_kind = lean_slice_kind_name(slice_kind),
        slice_description_json = quoted(slice_description.as_ref()),
    );
    let contents = contents
        .replace(
            "structure ControlDefinition where\n  name : String\n  commandName : String\n  inputs : List ControlInputProvision\n  handledErrors : List String\n  recoveryBehavior : String\n  sketchToken : String",
            "structure NavigationTarget where\n  targetType : String\n  targetName : String\n  externalWorkflowName : String\n  externalSystemName : String\n  handoffContract : String\n\nstructure ControlDefinition where\n  name : String\n  commandName : String\n  inputs : List ControlInputProvision\n  handledErrors : List String\n  recoveryBehavior : String\n  sketchToken : String\n  navigation : NavigationTarget",
        )
        .replace(
            "structure EventModelScenario where\n  name : String\n  givenSteps : List String\n  whenSteps : List String\n  thenSteps : List String",
            "structure EventModelScenario where\n  name : String\n  givenSteps : List String\n  whenSteps : List String\n  thenSteps : List String\n  readStreams : List String\n  writtenStreams : List String\n  contractKind : String\n  coveredDefinition : String\n  errorReferences : List String",
        )
        .replace(
            "structure ViewDefinition where\n  name : String\n  readModels : List String\n  fields : List ViewField\n  controls : List ControlDefinition\n  sketchTokens : List String",
            "structure ViewDefinition where\n  name : String\n  readModels : List String\n  fields : List ViewField\n  controls : List ControlDefinition\n  sketchTokens : List String\n  localStates : List String\n  filters : List String",
        )
        .replace(
            "def sliceCommands : List String := []",
            "structure AutomationDefinition where\n  name : String\n  triggerName : String\n  commandName : String\n  handledErrors : List String\n  reactionDescription : String\n\nstructure TranslationDefinition where\n  name : String\n  externalEventName : String\n  payloadContractName : String\n  commandName : String\n\nstructure BoardElement where\n  name : String\n  kind : String\n  lane : String\n  declaredName : String\n  mainPath : Bool\n\nstructure BoardConnection where\n  source : String\n  sourceKind : String\n  target : String\n  targetKind : String\n\ndef sliceCommands : List String := []",
        )
        .replace(
            "def sliceReferencedCommands : List String := []",
            "def sliceAutomations : List AutomationDefinition := []\n\ndef sliceTranslations : List TranslationDefinition := []\n\ndef canonicalBoardLanes : List String := [\"ux\",\"actions\",\"events\"]\n\ndef sliceBoardElements : List BoardElement := []\n\ndef sliceBoardConnections : List BoardConnection := []\n\ndef sliceReferencedCommands : List String := []",
        )
        .replace(
            "structure EventDefinition where",
            "structure ExternalPayloadField where\n  name : String\n  provenanceDescription : String\n  bitEncoding : String\n\nstructure ExternalPayloadDefinition where\n  name : String\n  fields : List ExternalPayloadField\n\nstructure EventDefinition where",
        )
        .replace(
            "structure ReadModelField where\n  name : String\n  sourceKind : String\n  sourceEvent : String\n  sourceAttribute : String\n  derivationRule : String\n  absenceEvent : String\n  provenanceDescription : String",
            "structure ReadModelField where\n  name : String\n  sourceKind : String\n  sourceEvent : String\n  sourceAttribute : String\n  derivationRule : String\n  derivationSourceFields : List String\n  absenceEvent : String\n  derivationScenarioName : String\n  absenceScenarioName : String\n  provenanceDescription : String",
        )
        .replace(
            "structure ReadModelDefinition where\n  name : String\n  fields : List ReadModelField",
            "structure ReadModelDefinition where\n  name : String\n  fields : List ReadModelField\n  transitive : Bool\n  relationshipFields : List String\n  transitiveRule : String\n  exampleScenarioName : String",
        )
        .replace(
            "def readModelFieldSourceIsComplete (field : ReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && field.sourceEvent.isEmpty == false && field.sourceAttribute.isEmpty == false) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false)",
            "def readModelFieldSourceIsComplete (field : ReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && field.sourceEvent.isEmpty == false && field.sourceAttribute.isEmpty == false) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false && field.derivationSourceFields.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false)",
        )
        .replace(
            "def sliceEventDefinitions : List EventDefinition := []",
            "def sliceExternalPayloads : List ExternalPayloadDefinition := []\n\ndef sliceEventDefinitions : List EventDefinition := []",
        )
        .replace(
            "def sliceAcceptanceScenarios : List EventModelScenario := []",
            "def allowedNavigationTargetTypes : List String := [\"modeled_view\",\"local_view_state\",\"external_system\",\"external_workflow\"]\n\ndef sliceAcceptanceScenarios : List EventModelScenario := []",
        )
        .replace(
            "def eventsReferenceKnownStreams : Bool := sliceEventDefinitions.all eventReferencesKnownStream",
            "def commandEmittedEventIsKnown (eventName : String) : Bool := sliceEvents.contains eventName || sliceEventDefinitions.any (fun event => event.name == eventName)\n\ndef eventProducedByCommand (event : EventDefinition) : Bool := event.observed || event.shared || sliceCommandDefinitions.any (fun command => command.emittedEvents.contains event.name)\n\ndef commandInputReferencesAttributeSource (event : EventDefinition) (eventAttribute : EventAttribute) (command : CommandDefinition) : Bool := command.emittedEvents.contains event.name && command.inputs.any (fun input => input.name == eventAttribute.sourceName)\n\ndef externalPayloadFieldsHaveProvenance : Bool := sliceExternalPayloads.all (fun payload => payload.name.isEmpty == false && payload.fields.all externalPayloadFieldHasProvenance)\n\ndef externalPayloadFieldHasBitLevelFlow (payload : ExternalPayloadDefinition) (field : ExternalPayloadField) : Bool := bitLevelFlowCoversTarget payload.name field.name\n\ndef externalPayloadFieldIsDeclared (eventAttribute : EventAttribute) : Bool := sliceExternalPayloads.any (fun payload => payload.name == eventAttribute.sourceName && payload.fields.any (fun field => field.name == eventAttribute.sourceField))\n\ndef eventAttributeSourceIsComplete (event : EventDefinition) (eventAttribute : EventAttribute) : Bool := (eventAttribute.sourceKind == \"command_input\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false && sliceCommandDefinitions.any (commandInputReferencesAttributeSource event eventAttribute)) || (eventAttribute.sourceKind == \"external_payload\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false && externalPayloadFieldIsDeclared eventAttribute) || (eventAttribute.sourceKind == \"generated\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.generatedSourceKind.isEmpty == false) || (eventAttribute.sourceKind == \"session\" && eventAttribute.sourceName.isEmpty == false) || (eventAttribute.sourceKind == \"derivation\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false)\n\ndef eventAttributeTracesToStoredFactSource (eventAttribute : EventAttribute) : Bool := storedEventFactSourceKinds.contains eventAttribute.sourceKind\n\ndef eventsReferenceKnownStreams : Bool := sliceEventDefinitions.all eventReferencesKnownStream",
        )
        .replace(
            "def commandInputHasAllowedSource (input : CommandInput) : Bool := allowedCommandInputSourceKinds.contains input.sourceKind",
            "def scenarioStreamResolves (streamName : String) : Bool := sliceStreams.any (fun stream => stream.name == streamName)\n\ndef scenarioStreamsResolve (scenario : EventModelScenario) : Bool := scenario.readStreams.all scenarioStreamResolves && scenario.writtenStreams.all scenarioStreamResolves\n\ndef stateChangeScenarioNamesStreams (scenario : EventModelScenario) : Bool := sliceKind != SliceKindName.stateChange || (scenario.readStreams.isEmpty == false && scenario.writtenStreams.isEmpty == false)\n\ndef sliceScenarioStreamsResolve : Bool := (sliceAcceptanceScenarios ++ sliceContractScenarios).all scenarioStreamsResolve\n\ndef stateChangeScenariosNameStreams : Bool := (sliceAcceptanceScenarios ++ sliceContractScenarios).all stateChangeScenarioNamesStreams\n\ndef acceptanceScenariosAreUserFacing : Bool := sliceAcceptanceScenarios.all (fun scenario => scenario.contractKind.isEmpty && scenario.coveredDefinition.isEmpty)\n\ndef scenarioCoversContract (contractKind : String) (definitionName : String) (scenario : EventModelScenario) : Bool := scenario.contractKind == contractKind && scenario.coveredDefinition == definitionName\n\ndef readModelHasProjectorContract (readModel : ReadModelDefinition) : Bool := sliceContractScenarios.any (scenarioCoversContract \"projector\" readModel.name)\n\ndef stateViewReadModelsHaveProjectorContracts : Bool := sliceKind != SliceKindName.stateView || sliceReadModelDefinitions.all readModelHasProjectorContract\n\ndef contractScenarioTargetsKnownDefinition (scenario : EventModelScenario) : Bool := (scenario.contractKind == \"projector\" && (sliceReadModels.contains scenario.coveredDefinition || sliceReadModelDefinitions.any (fun readModel => readModel.name == scenario.coveredDefinition))) || (scenario.contractKind == \"command\" && (sliceCommands.contains scenario.coveredDefinition || sliceCommandDefinitions.any (fun command => command.name == scenario.coveredDefinition))) || (scenario.contractKind == \"automation\" && sliceAutomations.any (fun automation => automation.name == scenario.coveredDefinition)) || (scenario.contractKind == \"translation\" && sliceTranslations.any (fun translation => translation.name == scenario.coveredDefinition)) || (scenario.contractKind == \"derivation\" && scenario.coveredDefinition.isEmpty == false && sliceReadModelDefinitions.any (fun readModel => readModel.fields.any (fun field => field.sourceKind == \"derivation\" && field.derivationScenarioName == scenario.name))) || (scenario.contractKind == \"absence\" && scenario.coveredDefinition.isEmpty == false && sliceReadModelDefinitions.any (fun readModel => readModel.fields.any (fun field => field.sourceKind == \"absence_default\" && field.absenceScenarioName == scenario.name))) || (scenario.contractKind == \"transitive\" && sliceReadModelDefinitions.any (fun readModel => readModel.transitive && readModel.name == scenario.coveredDefinition && readModel.exampleScenarioName == scenario.name))\n\ndef contractScenariosTargetKnownDefinitions : Bool := sliceContractScenarios.all contractScenarioTargetsKnownDefinition\n\ndef commandHasContractScenario (command : CommandDefinition) : Bool := sliceContractScenarios.any (scenarioCoversContract \"command\" command.name)\n\ndef automationHasContractScenario (automation : AutomationDefinition) : Bool := sliceContractScenarios.any (scenarioCoversContract \"automation\" automation.name)\n\ndef translationHasContractScenario (translation : TranslationDefinition) : Bool := sliceContractScenarios.any (scenarioCoversContract \"translation\" translation.name)\n\ndef derivationFieldHasContractScenario (field : ReadModelField) : Bool := field.sourceKind != \"derivation\" || sliceContractScenarios.any (fun scenario => scenario.contractKind == \"derivation\" && scenario.coveredDefinition.isEmpty == false && scenario.name == field.derivationScenarioName)\n\ndef contractScenariosCoverModeledContracts : Bool := sliceCommandDefinitions.all commandHasContractScenario && sliceAutomations.all automationHasContractScenario && sliceTranslations.all translationHasContractScenario && sliceReadModelDefinitions.all (fun readModel => readModel.fields.all derivationFieldHasContractScenario)\n\ndef commandInputHasAllowedSource (input : CommandInput) : Bool := allowedCommandInputSourceKinds.contains input.sourceKind",
        )
        .replace(
            "def eventAttributesHaveAllowedSources : Bool := sliceEventDefinitions.all (fun event => event.attributes.all eventAttributeHasAllowedSource)",
            "def commandEmittedEventsAreKnown : Bool := sliceCommandDefinitions.all (fun command => command.emittedEvents.all commandEmittedEventIsKnown)\n\ndef locallyEmittedEventsAreProducedByCommands : Bool := sliceEventDefinitions.all eventProducedByCommand\n\ndef eventAttributesHaveAllowedSources : Bool := sliceEventDefinitions.all (fun event => event.attributes.all eventAttributeHasAllowedSource)",
        )
        .replace(
            "def commandErrorsHaveAllowedRecovery : Bool := sliceCommandDefinitions.all (fun command => command.errors.all commandErrorHasAllowedRecovery)",
            "def commandErrorsHaveAllowedRecovery : Bool := sliceCommandDefinitions.all (fun command => command.errors.all commandErrorHasAllowedRecovery)\n\ndef scenarioNameIsModeled (scenarioName : String) : Bool := (sliceAcceptanceScenarios ++ sliceContractScenarios).any (fun scenario => scenario.name == scenarioName)\n\ndef commandErrorHasScenarioCoverage (command : CommandDefinition) (error : CommandErrorDefinition) : Bool := sliceContractScenarios.any (fun scenario => scenario.name == error.scenarioName && scenario.contractKind == \"command\" && scenario.coveredDefinition == command.name && scenario.errorReferences.contains error.name)\n\ndef commandErrorsHaveScenarioCoverage : Bool := sliceCommandDefinitions.all (fun command => command.errors.all (commandErrorHasScenarioCoverage command))\n\ndef scenarioErrorReferenceIsDeclared (scenario : EventModelScenario) (errorName : String) : Bool := scenario.contractKind != \"command\" || sliceCommandDefinitions.any (fun command => command.name == scenario.coveredDefinition && command.errors.any (fun error => error.name == errorName))\n\ndef scenarioErrorReferencesAreDeclaredForScenario (scenario : EventModelScenario) : Bool := scenario.errorReferences.all (scenarioErrorReferenceIsDeclared scenario)\n\ndef scenarioErrorReferencesAreDeclared : Bool := sliceContractScenarios.all scenarioErrorReferencesAreDeclaredForScenario

def singletonCommandDeclaresRepeatBehavior (command : CommandDefinition) : Bool := command.singleton == false || allowedSingletonRepeatBehaviors.contains command.repeatBehavior

def singletonCommandsDeclareRepeatBehavior : Bool := sliceCommandDefinitions.all singletonCommandDeclaresRepeatBehavior",
        )
        .replace(
            "def commandInputsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasProvenance)",
            "def commandInputsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasProvenance)\n\ndef commandInputTracesToInvocationSource (input : CommandInput) : Bool := allowedCommandInputSourceKinds.contains input.sourceKind && input.provenanceChain.isEmpty == false\n\ndef commandInputsTraceToInvocationSources : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputTracesToInvocationSource)\n\ndef commandInputEventStreamSourceResolves (command : CommandDefinition) (input : CommandInput) : Bool := input.sourceKind != \"event_stream_state\" || (command.observedStreams.isEmpty == false && command.observedStreams.all scenarioStreamResolves && input.eventStreamSourceEvent.isEmpty == false && input.eventStreamSourceAttribute.isEmpty == false && sliceEventDefinitions.any (fun event => event.name == input.eventStreamSourceEvent && event.attributes.any (fun eventAttribute => eventAttribute.name == input.eventStreamSourceAttribute)))\n\ndef commandInputExternalPayloadSourceResolves (input : CommandInput) : Bool := input.sourceKind != \"external_payload\" || (input.externalPayloadSourceName.isEmpty == false && input.externalPayloadSourceField.isEmpty == false && sliceExternalPayloads.any (fun payload => payload.name == input.externalPayloadSourceName && payload.fields.any (fun payloadField => payloadField.name == input.externalPayloadSourceField)))\n\ndef commandInputGeneratedSourceHasCoordinates (input : CommandInput) : Bool := input.sourceKind != \"generated\" || (input.generatedSourceName.isEmpty == false && input.generatedSourceField.isEmpty == false)\n\ndef commandInputSessionSourceHasCoordinates (input : CommandInput) : Bool := input.sourceKind != \"session\" || (input.sessionSourceName.isEmpty == false && input.sessionSourceField.isEmpty == false)\n\ndef commandInputInvocationArgumentSourceHasCoordinates (input : CommandInput) : Bool := input.sourceKind != \"invocation_argument\" || (input.invocationArgumentSourceName.isEmpty == false && input.invocationArgumentSourceField.isEmpty == false)\n\ndef commandInputsSourcedFromEventStreamsResolve : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all (commandInputEventStreamSourceResolves command))\n\ndef commandInputsSourcedFromExternalPayloadsResolve : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputExternalPayloadSourceResolves)\n\ndef commandInputsSourcedFromGeneratedValuesHaveCoordinates : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputGeneratedSourceHasCoordinates)\n\ndef commandInputsSourcedFromSessionValuesHaveCoordinates : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputSessionSourceHasCoordinates)\n\ndef commandInputsSourcedFromInvocationArgumentsHaveCoordinates : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputInvocationArgumentSourceHasCoordinates)\n\ndef bitLevelFlowCoversTarget (target : String) (datum : String) : Bool := sliceBitLevelDataFlows.any (fun flow => flow.target == target && flow.datum == datum && flow.sourceKind.isEmpty == false && flow.source.isEmpty == false && flow.transformationSemantics.isEmpty == false && flow.bitEncoding.isEmpty == false)\n\ndef commandInputHasBitLevelFlow (command : CommandDefinition) (input : CommandInput) : Bool := bitLevelFlowCoversTarget command.name input.name",
        )
        .replace(
            "def readModelFieldHasAllowedSource (field : ReadModelField) : Bool := allowedReadModelFieldSourceKinds.contains field.sourceKind",
            "def eventAttributeSourcesAreComplete : Bool := sliceEventDefinitions.all (fun event => event.attributes.all (eventAttributeSourceIsComplete event))\n\ndef storedEventFactsTraceToOriginalSources : Bool := sliceEventDefinitions.all (fun event => event.attributes.all eventAttributeTracesToStoredFactSource)\n\ndef eventAttributeHasBitLevelFlow (event : EventDefinition) (eventAttribute : EventAttribute) : Bool := bitLevelFlowCoversTarget event.name eventAttribute.name\n\ndef readModelFieldHasAllowedSource (field : ReadModelField) : Bool := allowedReadModelFieldSourceKinds.contains field.sourceKind",
        )
        .replace(
            "def outcomeLabelCount (label : String) : Nat := (sliceOutcomeDefinitions.filter (fun outcome => outcome.label == label)).length",
            "def automationHasTrigger (automation : AutomationDefinition) : Bool := automation.name.isEmpty == false && automation.triggerName.isEmpty == false && automation.reactionDescription.isEmpty == false\n\ndef automationIssuesKnownCommand (automation : AutomationDefinition) : Bool := sliceCommands.contains automation.commandName || sliceReferencedCommands.contains automation.commandName || sliceCommandDefinitions.any (fun command => command.name == automation.commandName)\n\ndef automationHandlesCommandErrors (automation : AutomationDefinition) (command : CommandDefinition) : Bool := command.name != automation.commandName || command.errors.all (fun error => automation.handledErrors.contains error.name)\n\ndef automationSlicesDeclareTriggers : Bool := sliceKind != SliceKindName.automation || (sliceAutomations.isEmpty == false && sliceAutomations.all automationHasTrigger)\n\ndef automationSlicesRepresentOneReaction : Bool := sliceKind != SliceKindName.automation || sliceAutomations.length == 1\n\ndef automationsIssueKnownCommands : Bool := sliceAutomations.all automationIssuesKnownCommand\n\ndef automationsHandleCommandErrors : Bool := sliceAutomations.all (fun automation => sliceCommandDefinitions.all (automationHandlesCommandErrors automation))\n\ndef externalPayloadFieldHasProvenance (field : ExternalPayloadField) : Bool := field.name.isEmpty == false && field.provenanceDescription.isEmpty == false && field.bitEncoding.isEmpty == false\n\ndef translationHasExternalContract (translation : TranslationDefinition) : Bool := translation.name.isEmpty == false && translation.externalEventName.isEmpty == false && translation.payloadContractName.isEmpty == false && sliceExternalPayloads.any (fun payload => payload.name == translation.payloadContractName)\n\ndef externalBoundaryHasPayloadContractAndFieldProvenance (translation : TranslationDefinition) : Bool := translationHasExternalContract translation && sliceExternalPayloads.any (fun payload => payload.name == translation.payloadContractName && payload.fields.isEmpty == false && payload.fields.all externalPayloadFieldHasProvenance)\n\ndef externalBoundariesHavePayloadContractsAndFieldProvenance : Bool := sliceTranslations.all externalBoundaryHasPayloadContractAndFieldProvenance\n\ndef translationTargetsKnownCommand (translation : TranslationDefinition) : Bool := sliceCommands.contains translation.commandName || sliceReferencedCommands.contains translation.commandName || sliceCommandDefinitions.any (fun command => command.name == translation.commandName)\n\ndef translationReferencesObservedExternalEvent (translation : TranslationDefinition) : Bool := sliceEventDefinitions.any (fun event => event.name == translation.externalEventName && event.observed)\n\ndef translationSlicesDeclareExternalContracts : Bool := sliceKind != SliceKindName.translation || (sliceTranslations.isEmpty == false && sliceTranslations.all translationHasExternalContract)\n\ndef translationsTargetKnownCommands : Bool := sliceTranslations.all translationTargetsKnownCommand\n\ndef translationsReferenceObservedExternalEvents : Bool := sliceTranslations.all translationReferencesObservedExternalEvent\n\ndef boardElementLaneMatchesKind (element : BoardElement) : Bool := (element.kind == \"view\" && element.lane == \"ux\") || (element.kind == \"automation\" && element.lane == \"ux\") || (element.kind == \"external_event\" && element.lane == \"ux\") || (element.kind == \"command\" && element.lane == \"actions\") || (element.kind == \"read_model\" && element.lane == \"actions\") || (element.kind == \"event\" && element.lane == \"events\")\n\ndef boardElementReferencesDeclaration (element : BoardElement) : Bool := (element.kind == \"view\" && (sliceViews.contains element.declaredName || sliceViewDefinitions.any (fun view => view.name == element.declaredName))) || (element.kind == \"automation\" && sliceAutomations.any (fun automation => automation.name == element.declaredName)) || (element.kind == \"external_event\" && sliceEventDefinitions.any (fun event => event.name == element.declaredName && event.observed)) || (element.kind == \"command\" && (sliceCommands.contains element.declaredName || sliceReferencedCommands.contains element.declaredName || sliceCommandDefinitions.any (fun command => command.name == element.declaredName))) || (element.kind == \"read_model\" && (sliceReadModels.contains element.declaredName || sliceReadModelDefinitions.any (fun readModel => readModel.name == element.declaredName))) || (element.kind == \"event\" && (sliceEvents.contains element.declaredName || sliceEventDefinitions.any (fun event => event.name == element.declaredName && (event.observed || event.shared))))\n\ndef automationBoardElementIsDeclaredAutomation (element : BoardElement) : Bool := element.kind != \"automation\" || sliceAutomations.any (fun automation => automation.name == element.declaredName)\n\ndef automationBoardElementsAreDeclaredAutomations : Bool := sliceBoardElements.all automationBoardElementIsDeclaredAutomation\n\ndef externalBoardElementIsObservedEvent (element : BoardElement) : Bool := element.kind != \"external_event\" || sliceEventDefinitions.any (fun event => event.name == element.declaredName && event.observed)\n\ndef externalBoardElementsAreObservedEvents : Bool := sliceBoardElements.all externalBoardElementIsObservedEvent\n\ndef boardConnectionHasAllowedShape (connection : BoardConnection) : Bool := (connection.sourceKind == \"view\" && connection.targetKind == \"command\") || (connection.sourceKind == \"automation\" && connection.targetKind == \"command\") || (connection.sourceKind == \"external_event\" && connection.targetKind == \"command\") || (connection.sourceKind == \"workflow_trigger\" && connection.targetKind == \"command\") || (connection.sourceKind == \"command\" && connection.targetKind == \"event\") || (connection.sourceKind == \"event\" && connection.targetKind == \"read_model\") || (connection.sourceKind == \"read_model\" && connection.targetKind == \"view\")\n\ndef commandEventBoardEdgeMatchesEmission (connection : BoardConnection) : Bool := connection.sourceKind != \"command\" || connection.targetKind != \"event\" || sliceCommandDefinitions.any (fun command => command.name == connection.source && command.emittedEvents.contains connection.target)\n\ndef commandEventBoardEdgesMatchEmissions : Bool := sliceBoardConnections.all commandEventBoardEdgeMatchesEmission\n\ndef eventReadModelBoardEdgeMatchesProjection (connection : BoardConnection) : Bool := connection.sourceKind != \"event\" || connection.targetKind != \"read_model\" || sliceReadModelDefinitions.any (fun readModel => readModel.name == connection.target && readModel.fields.any (fun field => field.sourceEvent == connection.source))\n\ndef eventReadModelBoardEdgesMatchProjectionSources : Bool := sliceBoardConnections.all eventReadModelBoardEdgeMatchesProjection\n\ndef externalEventCommandBoardEdgeMatchesTranslation (connection : BoardConnection) : Bool := connection.sourceKind != \"external_event\" || connection.targetKind != \"command\" || sliceTranslations.any (fun translation => translation.externalEventName == connection.source && translation.commandName == connection.target)\n\ndef externalEventTriggersMatchTranslations : Bool := sliceBoardConnections.all externalEventCommandBoardEdgeMatchesTranslation\n\ndef externalEventDoesNotUpdateReadModel (connection : BoardConnection) : Bool := connection.sourceKind != \"event\" || connection.targetKind != \"read_model\" || sliceEventDefinitions.any (fun event => event.name == connection.source && event.observed) == false\n\ndef externalEventsDoNotUpdateReadModels : Bool := sliceBoardConnections.all externalEventDoesNotUpdateReadModel\n\ndef viewCommandBoardEdgeMatchesControl (connection : BoardConnection) : Bool := connection.sourceKind != \"view\" || connection.targetKind != \"command\" || sliceViewDefinitions.any (fun view => view.name == connection.source && view.controls.any (fun control => control.commandName == connection.target))\n\ndef viewCommandBoardEdgesMatchControls : Bool := sliceBoardConnections.all viewCommandBoardEdgeMatchesControl\n\ndef boardLanesAreCanonical : Bool := canonicalBoardLanes == [\"ux\",\"actions\",\"events\"]\n\ndef boardElementsUseCanonicalLanes : Bool := sliceBoardElements.all (fun element => canonicalBoardLanes.contains element.lane && boardElementLaneMatchesKind element)\n\ndef boardElementsReferenceDeclarations : Bool := sliceBoardElements.all boardElementReferencesDeclaration\n\ndef boardConnectionsHaveCausalSemantics : Bool := sliceBoardConnections.all (fun connection => boardConnectionHasAllowedShape connection && commandEventBoardEdgeMatchesEmission connection && eventReadModelBoardEdgeMatchesProjection connection && externalEventCommandBoardEdgeMatchesTranslation connection && externalEventDoesNotUpdateReadModel connection && viewCommandBoardEdgeMatchesControl connection)\n\ndef readModelViewConnectionHasIncomingEventUpdate (connection : BoardConnection) : Bool := connection.sourceKind != \"read_model\" || connection.targetKind != \"view\" || sliceBoardConnections.any (fun incoming => incoming.target == connection.source && incoming.targetKind == \"read_model\" && incoming.sourceKind == \"event\")\n\ndef readModelsFeedingViewsHaveIncomingEventUpdates : Bool := sliceBoardConnections.all readModelViewConnectionHasIncomingEventUpdate\n\ndef commandsHaveIncomingTriggers : Bool := sliceBoardElements.all (fun element => element.kind != \"command\" || sliceBoardConnections.any (fun connection => connection.target == element.name && connection.targetKind == \"command\" && (connection.sourceKind == \"view\" || connection.sourceKind == \"automation\" || connection.sourceKind == \"external_event\" || connection.sourceKind == \"workflow_trigger\")))\n\ndef mainPathBoardHasNoDisconnectedIslands : Bool := sliceBoardElements.all (fun element => element.mainPath == false || sliceBoardConnections.any (fun connection => connection.source == element.name || connection.target == element.name))\n\ndef outcomeLabelCount (label : String) : Nat := (sliceOutcomeDefinitions.filter (fun outcome => outcome.label == label)).length",
        )
        .replace(
            "def readModelFieldsHaveAllowedSources : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldHasAllowedSource)",
            "def eventAttributeIsDeclared (eventName : String) (attributeName : String) : Bool := sliceEventDefinitions.any (fun event => event.name == eventName && event.attributes.any (fun eventAttribute => eventAttribute.name == attributeName))\n\ndef readModelFieldEventAttributeSourceResolves (field : ReadModelField) : Bool := field.sourceKind != \"event_attribute\" || eventAttributeIsDeclared field.sourceEvent field.sourceAttribute\n\ndef readModelFieldDerivationScenarioIsCovered (field : ReadModelField) : Bool := field.sourceKind != \"derivation\" || (field.derivationScenarioName.isEmpty == false && scenarioNameIsModeled field.derivationScenarioName)\n\ndef readModelFieldAbsenceScenarioIsCovered (field : ReadModelField) : Bool := field.sourceKind != \"absence_default\" || (field.absenceScenarioName.isEmpty == false && scenarioNameIsModeled field.absenceScenarioName)\n\ndef readModelFieldsHaveAllowedSources : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldHasAllowedSource)",
        )
        .replace(
            "def viewFieldHasAllowedSource (field : ViewField) : Bool := allowedViewFieldSourceKinds.contains field.sourceKind",
            "def readModelFieldEventAttributeSourcesResolve : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldEventAttributeSourceResolves)\n\ndef derivedReadModelFieldsHaveScenarioCoverage : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldDerivationScenarioIsCovered)\n\ndef absenceReadModelFieldsHaveScenarioCoverage : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldAbsenceScenarioIsCovered)\n\ndef transitiveReadModelHasSemantics (readModel : ReadModelDefinition) : Bool := readModel.transitive == false || (readModel.relationshipFields.isEmpty == false && readModel.transitiveRule.isEmpty == false && readModel.exampleScenarioName.isEmpty == false && scenarioNameIsModeled readModel.exampleScenarioName)\n\ndef transitiveReadModelsHaveSemantics : Bool := sliceReadModelDefinitions.all transitiveReadModelHasSemantics\n\ndef readModelFieldHasBitLevelFlow (readModel : ReadModelDefinition) (field : ReadModelField) : Bool := bitLevelFlowCoversTarget readModel.name field.name\n\ndef viewFieldHasAllowedSource (field : ViewField) : Bool := allowedViewFieldSourceKinds.contains field.sourceKind",
        )
        .replace(
            "def viewFieldsSourceFromUsedReadModels : Bool := sliceViewDefinitions.all (fun view => view.fields.all (viewFieldSourceReadModelIsUsed view))",
            "def viewFieldsSourceFromUsedReadModels : Bool := sliceViewDefinitions.all (fun view => view.fields.all (viewFieldSourceReadModelIsUsed view))\n\ndef viewFieldAppearsInSketch (view : ViewDefinition) (field : ViewField) : Bool := field.sketchToken.isEmpty == false && view.sketchTokens.contains field.sketchToken\n\ndef viewHasInformationSketch (view : ViewDefinition) : Bool := view.sketchTokens.isEmpty == false\n\ndef viewsHaveInformationSketches : Bool := sliceViewDefinitions.all viewHasInformationSketch\n\ndef viewFieldsAppearInSketch : Bool := sliceViewDefinitions.all (fun view => view.fields.all (viewFieldAppearsInSketch view))\n\ndef sketchTokenMapsToModeledElement (view : ViewDefinition) (token : String) : Bool := view.fields.any (fun field => field.sketchToken == token) || view.controls.any (fun control => control.sketchToken == token || control.inputs.any (fun input => input.sourceKind == \"actor\" && input.sketchToken == token))\n\ndef viewSketchTokensMapToModeledElements : Bool := sliceViewDefinitions.all (fun view => view.sketchTokens.all (sketchTokenMapsToModeledElement view))\n\ndef readModelFieldIsDeclared (readModelName : String) (fieldName : String) : Bool := sliceReadModelDefinitions.any (fun readModel => readModel.name == readModelName && readModel.fields.any (fun readModelField => readModelField.name == fieldName))\n\ndef viewFieldSourceReadModelFieldResolves (field : ViewField) : Bool := field.sourceKind != \"read_model\" || readModelFieldIsDeclared field.sourceReadModel field.sourceField\n\ndef readModelFieldHasOriginalProvenance (field : ReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && readModelFieldEventAttributeSourceResolves field) || field.sourceKind == \"derivation\" || field.sourceKind == \"absence_default\"\n\ndef viewFieldTracesToOriginalProvenance (field : ViewField) : Bool := field.sourceKind == \"read_model\" && sliceReadModelDefinitions.any (fun readModel => readModel.name == field.sourceReadModel && readModel.fields.any (fun readModelField => readModelField.name == field.sourceField && readModelFieldHasOriginalProvenance readModelField))\n\ndef viewFieldReadModelFieldSourcesResolve : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldSourceReadModelFieldResolves)\n\ndef displayedDataTraceToOriginalProvenance : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldTracesToOriginalProvenance)\n\ndef viewFieldHasBitLevelFlow (view : ViewDefinition) (field : ViewField) : Bool := bitLevelFlowCoversTarget view.name field.name\n\ndef commandInputDataFlowsAreComplete : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all (commandInputHasBitLevelFlow command))\n\ndef eventAttributeDataFlowsAreComplete : Bool := sliceEventDefinitions.all (fun event => event.attributes.all (eventAttributeHasBitLevelFlow event))\n\ndef readModelFieldDataFlowsAreComplete : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all (readModelFieldHasBitLevelFlow readModel))\n\ndef viewFieldDataFlowsAreComplete : Bool := sliceViewDefinitions.all (fun view => view.fields.all (viewFieldHasBitLevelFlow view))\n\ndef externalPayloadFieldDataFlowsAreComplete : Bool := sliceExternalPayloads.all (fun payload => payload.fields.all (externalPayloadFieldHasBitLevelFlow payload))\n\ndef modeledDataFlowsAreBitComplete : Bool := commandInputDataFlowsAreComplete && eventAttributeDataFlowsAreComplete && readModelFieldDataFlowsAreComplete && viewFieldDataFlowsAreComplete && externalPayloadFieldDataFlowsAreComplete",
        )
        .replace(
            "def viewControlsHaveSketchTokens : Bool := sliceViewDefinitions.all (fun view => view.controls.all controlHasSketchToken)",
            "def navigationTargetTypeIsModeled (target : NavigationTarget) : Bool := target.targetType.isEmpty || allowedNavigationTargetTypes.contains target.targetType\n\ndef navigationTargetHasPayload (target : NavigationTarget) : Bool := target.targetName.isEmpty == false || target.externalWorkflowName.isEmpty == false || target.externalSystemName.isEmpty == false || target.handoffContract.isEmpty == false\n\ndef navigationControlDeclaresType (target : NavigationTarget) : Bool := navigationTargetHasPayload target == false || target.targetType.isEmpty == false\n\ndef navigationModeledViewTargetsExistingView (target : NavigationTarget) : Bool := target.targetType != \"modeled_view\" || (target.targetName.isEmpty == false && sliceViews.contains target.targetName)\n\ndef localViewStateNavigationTargetResolves (view : ViewDefinition) (target : NavigationTarget) : Bool := target.targetType != \"local_view_state\" || (target.targetName.isEmpty == false && (view.localStates.contains target.targetName || view.filters.contains target.targetName))\n\ndef navigationExternalWorkflowTargetsNamed (target : NavigationTarget) : Bool := target.targetType != \"external_workflow\" || target.externalWorkflowName.isEmpty == false\n\ndef navigationExternalSystemTargetsHaveContracts (target : NavigationTarget) : Bool := target.targetType != \"external_system\" || (target.externalSystemName.isEmpty == false && target.handoffContract.isEmpty == false)\n\ndef navigationTargetIsComplete (view : ViewDefinition) (target : NavigationTarget) : Bool := (target.targetType.isEmpty && target.targetName.isEmpty && target.externalWorkflowName.isEmpty && target.externalSystemName.isEmpty && target.handoffContract.isEmpty) || (target.targetType == \"modeled_view\" && target.targetName.isEmpty == false && sliceViews.contains target.targetName) || (target.targetType == \"local_view_state\" && localViewStateNavigationTargetResolves view target) || (target.targetType == \"external_workflow\" && navigationExternalWorkflowTargetsNamed target) || (target.targetType == \"external_system\" && navigationExternalSystemTargetsHaveContracts target)\n\ndef viewControlsHaveSketchTokens : Bool := sliceViewDefinitions.all (fun view => view.controls.all controlHasSketchToken)",
        )
        .replace(
            "def commandErrorsHandledByControl (control : ControlDefinition) (command : CommandDefinition) : Bool := command.name != control.commandName || command.errors.all (fun error => control.handledErrors.contains error.name && control.recoveryBehavior.isEmpty == false)",
            "def controlProvidesCommandInput (control : ControlDefinition) (input : CommandInput) : Bool := control.inputs.any (fun providedInput => providedInput.name == input.name)\n\ndef controlProvidesEveryCommandInput (control : ControlDefinition) (command : CommandDefinition) : Bool := command.name != control.commandName || command.inputs.all (controlProvidesCommandInput control)\n\ndef commandErrorsHandledByControl (control : ControlDefinition) (command : CommandDefinition) : Bool := command.name != control.commandName || command.errors.all (fun error => control.handledErrors.contains error.name && control.recoveryBehavior.isEmpty == false)\n\ndef controlRecoveryBehaviorIsModeled (control : ControlDefinition) : Bool := control.handledErrors.isEmpty || allowedRecoveryKinds.contains control.recoveryBehavior",
        )
        .replace(
            "def viewControlInputsHaveAllowedSources : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasAllowedSource))",
            "def controlAppearsInSketch (view : ViewDefinition) (control : ControlDefinition) : Bool := control.sketchToken.isEmpty == false && view.sketchTokens.contains control.sketchToken\n\ndef viewControlsAppearInSketch : Bool := sliceViewDefinitions.all (fun view => view.controls.all (controlAppearsInSketch view))\n\ndef viewControlsProvideCommandInputs : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => sliceCommandDefinitions.all (controlProvidesEveryCommandInput control)))\n\ndef viewControlInputsHaveAllowedSources : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasAllowedSource))",
        )
        .replace(
            "def sliceHasLocallyEmittedEvent : Bool := sliceEvents.isEmpty == false || sliceEventDefinitions.any (fun event => event.observed == false && event.shared == false)

def sliceStateChangeRequiresEvent : Prop := sliceKind = SliceKindName.stateChange -> sliceHasLocallyEmittedEvent",
            "def viewControlRecoveryBehaviorIsModeled : Bool := sliceViewDefinitions.all (fun view => view.controls.all controlRecoveryBehaviorIsModeled)\n\ndef stateViewSlicesDoNotOwnCommands : Bool := sliceKind != SliceKindName.stateView || (sliceCommands.isEmpty && sliceCommandDefinitions.isEmpty)\n\ndef stateViewSlicesOwnViews : Bool := sliceKind != SliceKindName.stateView || (sliceViews.isEmpty == false || sliceViewDefinitions.isEmpty == false)\n\ndef stateViewSlicesOwnReadModels : Bool := sliceKind != SliceKindName.stateView || (sliceReadModels.isEmpty == false || sliceReadModelDefinitions.isEmpty == false)\n\ndef readModelOwnsProjectionPath (readModel : ReadModelDefinition) : Bool := readModel.fields.isEmpty == false && readModel.fields.all readModelFieldSourceIsComplete\n\ndef stateViewSlicesOwnProjectionPaths : Bool := sliceKind != SliceKindName.stateView || sliceReadModelDefinitions.all readModelOwnsProjectionPath\n\ndef stateViewSlicesRepresentSingleViewProjectionBoundary : Bool := sliceKind != SliceKindName.stateView || (sliceViewDefinitions.length == 1 && sliceReadModelDefinitions.isEmpty == false)\n\ndef stateChangeSlicesOwnCommands : Bool := sliceKind != SliceKindName.stateChange || (sliceCommands.isEmpty == false || sliceCommandDefinitions.isEmpty == false)\n\ndef stateChangeSlicesOwnEvents : Bool := sliceKind != SliceKindName.stateChange || (sliceEvents.isEmpty == false || sliceEventDefinitions.isEmpty == false)\n\ndef stateChangeSlicesOwnOutcomes : Bool := sliceKind != SliceKindName.stateChange || sliceOutcomeDefinitions.isEmpty == false\n\ndef stateChangeSlicesOwnErrors : Bool := sliceKind != SliceKindName.stateChange || commandErrorsAreDeclared\n\ndef stateChangeSlicesDoNotOwnReadModelsOrViews : Bool := sliceKind != SliceKindName.stateChange || (sliceReadModels.isEmpty && sliceReadModelDefinitions.isEmpty && sliceViews.isEmpty && sliceViewDefinitions.isEmpty)\n\ndef stateChangeSlicesDoNotOwnAutomationsOrTranslations : Bool := sliceKind != SliceKindName.stateChange || (sliceAutomations.isEmpty && sliceTranslations.isEmpty)\n\ndef stateChangeSlicesDoNotOwnControlsOrSketches : Bool := sliceKind != SliceKindName.stateChange || sliceViewDefinitions.all (fun view => view.controls.isEmpty && view.sketchTokens.isEmpty)\n\ndef translationSlicesDoNotOwnViews : Bool := sliceKind != SliceKindName.translation || (sliceViews.isEmpty && sliceViewDefinitions.isEmpty)\n\ndef recognizedSliceKind : Bool := true\n\ndef sliceRepresentsOneCoherentModelUnit : Bool := recognizedSliceKind && stateViewSlicesDoNotOwnCommands && stateViewSlicesOwnViews && stateViewSlicesOwnReadModels && stateViewSlicesOwnProjectionPaths && stateChangeSlicesOwnCommands && stateChangeSlicesOwnEvents && stateChangeSlicesOwnOutcomes && stateChangeSlicesOwnErrors && stateChangeSlicesDoNotOwnReadModelsOrViews && stateChangeSlicesDoNotOwnAutomationsOrTranslations && stateChangeSlicesDoNotOwnControlsOrSketches && translationSlicesDeclareExternalContracts && externalBoundariesHavePayloadContractsAndFieldProvenance && translationsTargetKnownCommands && translationsReferenceObservedExternalEvents && translationSlicesDoNotOwnViews && automationSlicesDeclareTriggers && automationSlicesRepresentOneReaction && automationsIssueKnownCommands && automationsHandleCommandErrors\n\ndef stateChangeSlicesRepresentSingleCommandBoundary : Bool := sliceKind != SliceKindName.stateChange || sliceCommandDefinitions.length == 1\n\ndef sliceRepresentsSmallestUsefulBehaviorBoundary : Bool := sliceRepresentsOneCoherentModelUnit && stateViewSlicesRepresentSingleViewProjectionBoundary && stateChangeSlicesRepresentSingleCommandBoundary && automationSlicesRepresentOneReaction && translationSlicesDeclareExternalContracts\n\ndef viewControlNavigationTypesAreModeled : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => navigationTargetTypeIsModeled control.navigation))\n\ndef viewControlNavigationTypesAreDeclared : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => navigationControlDeclaresType control.navigation))\n\ndef viewControlModeledViewNavigationTargetsResolve : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => navigationModeledViewTargetsExistingView control.navigation))\n\ndef viewControlExternalWorkflowNavigationTargetsNamed : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => navigationExternalWorkflowTargetsNamed control.navigation))\n\ndef viewControlExternalSystemNavigationTargetsHaveContracts : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => navigationExternalSystemTargetsHaveContracts control.navigation))\n\ndef viewControlNavigationTargetsAreComplete : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => navigationTargetIsComplete view control.navigation))\n\ndef sliceHasLocallyEmittedEvent : Bool := sliceEvents.isEmpty == false || sliceEventDefinitions.any (fun event => event.observed == false && event.shared == false)

def sliceStateChangeRequiresEvent : Prop := sliceKind = SliceKindName.stateChange -> sliceHasLocallyEmittedEvent",
        )
        .replace(
            "theorem viewControlsHandleCommandErrorsIsStable : viewControlsHandleCommandErrors = true := rfl\n\nend",
            "theorem viewControlsHandleCommandErrorsIsStable : viewControlsHandleCommandErrors = true := rfl\n\ntheorem viewControlRecoveryBehaviorIsModeledIsStable : viewControlRecoveryBehaviorIsModeled = true := rfl\n\ntheorem stateViewSlicesDoNotOwnCommandsIsStable : stateViewSlicesDoNotOwnCommands = true := rfl\n\ntheorem stateViewSlicesOwnViewsIsStable : stateViewSlicesOwnViews = true := rfl\n\ntheorem stateViewSlicesOwnReadModelsIsStable : stateViewSlicesOwnReadModels = true := rfl\n\ntheorem stateViewSlicesOwnProjectionPathsIsStable : stateViewSlicesOwnProjectionPaths = true := rfl\n\ntheorem stateViewSlicesRepresentSingleViewProjectionBoundaryIsStable : stateViewSlicesRepresentSingleViewProjectionBoundary = true := by\n  native_decide\n\ntheorem stateChangeSlicesOwnCommandsIsStable : stateChangeSlicesOwnCommands = true := rfl\n\ntheorem stateChangeSlicesOwnEventsIsStable : stateChangeSlicesOwnEvents = true := by\n  native_decide\n\ntheorem stateChangeSlicesOwnOutcomesIsStable : stateChangeSlicesOwnOutcomes = true := by\n  native_decide\n\ntheorem stateChangeSlicesOwnErrorsIsStable : stateChangeSlicesOwnErrors = true := by\n  native_decide\n\ntheorem stateChangeSlicesDoNotOwnReadModelsOrViewsIsStable : stateChangeSlicesDoNotOwnReadModelsOrViews = true := rfl\n\ntheorem stateChangeSlicesDoNotOwnAutomationsOrTranslationsIsStable : stateChangeSlicesDoNotOwnAutomationsOrTranslations = true := rfl\n\ntheorem stateChangeSlicesDoNotOwnControlsOrSketchesIsStable : stateChangeSlicesDoNotOwnControlsOrSketches = true := rfl\n\ntheorem translationSlicesDoNotOwnViewsIsStable : translationSlicesDoNotOwnViews = true := rfl\n\ntheorem sliceRepresentsOneCoherentModelUnitIsStable : sliceRepresentsOneCoherentModelUnit = true := by\n  native_decide\n\ntheorem stateChangeSlicesRepresentSingleCommandBoundaryIsStable : stateChangeSlicesRepresentSingleCommandBoundary = true := by\n  native_decide\n\ntheorem sliceRepresentsSmallestUsefulBehaviorBoundaryIsStable : sliceRepresentsSmallestUsefulBehaviorBoundary = true := by\n  native_decide\n\ntheorem viewControlNavigationTypesAreModeledIsStable : viewControlNavigationTypesAreModeled = true := rfl\n\ntheorem viewControlNavigationTypesAreDeclaredIsStable : viewControlNavigationTypesAreDeclared = true := rfl\n\ntheorem viewControlModeledViewNavigationTargetsResolveIsStable : viewControlModeledViewNavigationTargetsResolve = true := rfl\n\ntheorem viewControlExternalWorkflowNavigationTargetsNamedIsStable : viewControlExternalWorkflowNavigationTargetsNamed = true := rfl\n\ntheorem viewControlExternalSystemNavigationTargetsHaveContractsIsStable : viewControlExternalSystemNavigationTargetsHaveContracts = true := rfl\n\ntheorem viewControlNavigationTargetsAreCompleteIsStable : viewControlNavigationTargetsAreComplete = true := rfl\n\nend",
        );
    let contents = contents
        .replace(
            "theorem sliceBitLevelDataFlowsAreStructured : sliceBitLevelDataFlows.all (fun flow => flow.datum.isEmpty == false && flow.sourceKind.isEmpty == false && flow.source.isEmpty == false && flow.transformationSemantics.isEmpty == false && flow.target.isEmpty == false && flow.bitEncoding.isEmpty == false) = true := rfl",
            "theorem sliceStateChangeRequiresEventIsStable : sliceStateChangeRequiresEvent := by\n  simp [sliceStateChangeRequiresEvent, sliceHasLocallyEmittedEvent, sliceKind, sliceEvents, sliceEventDefinitions]\n\ntheorem sliceBitLevelDataFlowsAreStructured : sliceBitLevelDataFlows.all (fun flow => flow.datum.isEmpty == false && flow.sourceKind.isEmpty == false && flow.source.isEmpty == false && flow.transformationSemantics.isEmpty == false && flow.target.isEmpty == false && flow.bitEncoding.isEmpty == false) = true := rfl",
        )
        .replace(
            "theorem eventAttributesHaveAllowedSourcesIsStable : eventAttributesHaveAllowedSources = true := rfl",
            "theorem commandEmittedEventsAreKnownIsStable : commandEmittedEventsAreKnown = true := rfl\n\ntheorem locallyEmittedEventsAreProducedByCommandsIsStable : locallyEmittedEventsAreProducedByCommands = true := rfl\n\ntheorem externalPayloadFieldsHaveProvenanceIsStable : externalPayloadFieldsHaveProvenance = true := rfl\n\ntheorem eventAttributesHaveAllowedSourcesIsStable : eventAttributesHaveAllowedSources = true := rfl",
        )
        .replace(
            "theorem commandInputsHaveAllowedSourcesIsStable : commandInputsHaveAllowedSources = true := rfl",
            "theorem sliceScenarioStreamsResolveIsStable : sliceScenarioStreamsResolve = true := rfl\n\ntheorem stateChangeScenariosNameStreamsIsStable : stateChangeScenariosNameStreams = true := by\n  native_decide\n\ntheorem acceptanceScenariosAreUserFacingIsStable : acceptanceScenariosAreUserFacing = true := rfl\n\ntheorem stateViewReadModelsHaveProjectorContractsIsStable : stateViewReadModelsHaveProjectorContracts = true := rfl\n\ntheorem contractScenariosTargetKnownDefinitionsIsStable : contractScenariosTargetKnownDefinitions = true := rfl\n\ntheorem contractScenariosCoverModeledContractsIsStable : contractScenariosCoverModeledContracts = true := rfl\n\ntheorem commandInputsHaveAllowedSourcesIsStable : commandInputsHaveAllowedSources = true := rfl",
        )
        .replace(
            "theorem viewControlInputsHaveAllowedSourcesIsStable : viewControlInputsHaveAllowedSources = true := rfl",
            "theorem viewControlsAppearInSketchIsStable : viewControlsAppearInSketch = true := rfl\n\ntheorem viewControlsProvideCommandInputsIsStable : viewControlsProvideCommandInputs = true := rfl\n\ntheorem viewControlInputsHaveAllowedSourcesIsStable : viewControlInputsHaveAllowedSources = true := rfl",
        )
        .replace(
            "theorem viewFieldsSourceFromUsedReadModelsIsStable : viewFieldsSourceFromUsedReadModels = true := rfl",
            "theorem viewFieldsSourceFromUsedReadModelsIsStable : viewFieldsSourceFromUsedReadModels = true := rfl\n\ntheorem viewsHaveInformationSketchesIsStable : viewsHaveInformationSketches = true := rfl\n\ntheorem viewFieldsAppearInSketchIsStable : viewFieldsAppearInSketch = true := rfl\n\ntheorem viewSketchTokensMapToModeledElementsIsStable : viewSketchTokensMapToModeledElements = true := rfl",
        )
        .replace(
            "theorem sliceScenariosHaveGwtIsStable : sliceScenariosHaveGwt = true := rfl",
            "theorem modeledDataFlowsAreBitCompleteIsStable : modeledDataFlowsAreBitComplete = true := rfl\n\ntheorem sliceScenariosHaveGwtIsStable : sliceScenariosHaveGwt = true := rfl",
        )
        .replace(
            "theorem commandInputsHaveProvenanceIsStable : commandInputsHaveProvenance = true := rfl",
            "theorem commandInputsHaveProvenanceIsStable : commandInputsHaveProvenance = true := rfl\n\ntheorem commandInputsTraceToInvocationSourcesIsStable : commandInputsTraceToInvocationSources = true := rfl\n\ntheorem commandInputsSourcedFromEventStreamsResolveIsStable : commandInputsSourcedFromEventStreamsResolve = true := rfl\n\ntheorem commandInputsSourcedFromExternalPayloadsResolveIsStable : commandInputsSourcedFromExternalPayloadsResolve = true := rfl\n\ntheorem commandInputsSourcedFromGeneratedValuesHaveCoordinatesIsStable : commandInputsSourcedFromGeneratedValuesHaveCoordinates = true := rfl\n\ntheorem commandInputsSourcedFromSessionValuesHaveCoordinatesIsStable : commandInputsSourcedFromSessionValuesHaveCoordinates = true := rfl\n\ntheorem commandInputsSourcedFromInvocationArgumentsHaveCoordinatesIsStable : commandInputsSourcedFromInvocationArgumentsHaveCoordinates = true := rfl",
        )
        .replace(
            "theorem commandErrorsHaveAllowedRecoveryIsStable : commandErrorsHaveAllowedRecovery = true := rfl",
            "theorem commandErrorsHaveAllowedRecoveryIsStable : commandErrorsHaveAllowedRecovery = true := rfl\n\ntheorem commandErrorsHaveScenarioCoverageIsStable : commandErrorsHaveScenarioCoverage = true := rfl\n\ntheorem scenarioErrorReferencesAreDeclaredIsStable : scenarioErrorReferencesAreDeclared = true := rfl

theorem singletonCommandsDeclareRepeatBehaviorIsStable : singletonCommandsDeclareRepeatBehavior = true := rfl",
        )
        .replace(
            "theorem outcomeLabelsAreUniqueIsStable : outcomeLabelsAreUnique = true := rfl",
            "theorem automationSlicesDeclareTriggersIsStable : automationSlicesDeclareTriggers = true := rfl\n\ntheorem automationSlicesRepresentOneReactionIsStable : automationSlicesRepresentOneReaction = true := rfl\n\ntheorem automationsIssueKnownCommandsIsStable : automationsIssueKnownCommands = true := rfl\n\ntheorem automationsHandleCommandErrorsIsStable : automationsHandleCommandErrors = true := rfl\n\ntheorem translationSlicesDeclareExternalContractsIsStable : translationSlicesDeclareExternalContracts = true := rfl\n\ntheorem externalBoundariesHavePayloadContractsAndFieldProvenanceIsStable : externalBoundariesHavePayloadContractsAndFieldProvenance = true := rfl\n\ntheorem translationsTargetKnownCommandsIsStable : translationsTargetKnownCommands = true := rfl\n\ntheorem translationsReferenceObservedExternalEventsIsStable : translationsReferenceObservedExternalEvents = true := rfl\n\ntheorem boardLanesAreCanonicalIsStable : boardLanesAreCanonical = true := rfl\n\ntheorem boardElementsUseCanonicalLanesIsStable : boardElementsUseCanonicalLanes = true := rfl\n\ntheorem boardElementsReferenceDeclarationsIsStable : boardElementsReferenceDeclarations = true := rfl\n\ntheorem automationBoardElementsAreDeclaredAutomationsIsStable : automationBoardElementsAreDeclaredAutomations = true := rfl\n\ntheorem externalBoardElementsAreObservedEventsIsStable : externalBoardElementsAreObservedEvents = true := rfl\n\ntheorem commandEventBoardEdgesMatchEmissionsIsStable : commandEventBoardEdgesMatchEmissions = true := rfl\n\ntheorem eventReadModelBoardEdgesMatchProjectionSourcesIsStable : eventReadModelBoardEdgesMatchProjectionSources = true := rfl\n\ntheorem viewCommandBoardEdgesMatchControlsIsStable : viewCommandBoardEdgesMatchControls = true := rfl\n\ntheorem boardConnectionsHaveCausalSemanticsIsStable : boardConnectionsHaveCausalSemantics = true := rfl\n\ntheorem externalEventTriggersMatchTranslationsIsStable : externalEventTriggersMatchTranslations = true := rfl\n\ntheorem externalEventsDoNotUpdateReadModelsIsStable : externalEventsDoNotUpdateReadModels = true := rfl\n\ntheorem readModelsFeedingViewsHaveIncomingEventUpdatesIsStable : readModelsFeedingViewsHaveIncomingEventUpdates = true := rfl\n\ntheorem commandsHaveIncomingTriggersIsStable : commandsHaveIncomingTriggers = true := rfl\n\ntheorem mainPathBoardHasNoDisconnectedIslandsIsStable : mainPathBoardHasNoDisconnectedIslands = true := rfl\n\ntheorem outcomeLabelsAreUniqueIsStable : outcomeLabelsAreUnique = true := rfl",
        )
        .replace(
            "theorem readModelFieldsHaveAllowedSourcesIsStable : readModelFieldsHaveAllowedSources = true := rfl",
            "theorem eventAttributeSourcesAreCompleteIsStable : eventAttributeSourcesAreComplete = true := rfl\n\ntheorem storedEventFactsTraceToOriginalSourcesIsStable : storedEventFactsTraceToOriginalSources = true := rfl\n\ntheorem readModelFieldsHaveAllowedSourcesIsStable : readModelFieldsHaveAllowedSources = true := rfl",
        );
    let contents = contents.replace(
        "theorem viewFieldsHaveAllowedSourcesIsStable : viewFieldsHaveAllowedSources = true := rfl",
        "theorem readModelFieldEventAttributeSourcesResolveIsStable : readModelFieldEventAttributeSourcesResolve = true := rfl\n\ntheorem derivedReadModelFieldsHaveScenarioCoverageIsStable : derivedReadModelFieldsHaveScenarioCoverage = true := rfl\n\ntheorem absenceReadModelFieldsHaveScenarioCoverageIsStable : absenceReadModelFieldsHaveScenarioCoverage = true := rfl\n\ntheorem transitiveReadModelsHaveSemanticsIsStable : transitiveReadModelsHaveSemantics = true := rfl\n\ntheorem viewFieldReadModelFieldSourcesResolveIsStable : viewFieldReadModelFieldSourcesResolve = true := rfl\n\ntheorem displayedDataTraceToOriginalProvenanceIsStable : displayedDataTraceToOriginalProvenance = true := rfl\n\ntheorem viewFieldsHaveAllowedSourcesIsStable : viewFieldsHaveAllowedSources = true := rfl",
    );
    let contents = contents
        .replace(
            "def commandInputHasProvenance (input : CommandInput) : Bool := input.name.isEmpty == false && input.sourceKind.isEmpty == false && input.sourceDescription.isEmpty == false && input.provenanceChain.isEmpty == false\n\ndef commandInputsHaveAllowedSources",
            "def commandInputHasProvenance (input : CommandInput) : Bool := input.name.isEmpty == false && input.sourceKind.isEmpty == false && input.sourceDescription.isEmpty == false && input.provenanceChain.isEmpty == false\n\ndef commandInputSessionInputHasDescription (input : CommandInput) : Bool := input.sourceKind != \"session\" || input.sourceDescription.isEmpty == false\n\ndef commandHasIssuingControl (command : CommandDefinition) : Bool := sliceViewDefinitions.any (fun view => view.controls.any (fun control => control.commandName == command.name))\n\ndef commandInputWithoutIssuingControlHasProvenance (command : CommandDefinition) (input : CommandInput) : Bool := commandHasIssuingControl command || commandInputHasProvenance input\n\ndef commandInputsHaveAllowedSources",
        )
        .replace(
            "def commandInputsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasProvenance)\n\ndef commandErrorHasDeclaration",
            "def commandInputsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasProvenance)\n\ndef commandInputsWithoutIssuingControlsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all (commandInputWithoutIssuingControlHasProvenance command))\n\ndef commandSessionInputsHaveDescriptions : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputSessionInputHasDescription)\n\ndef commandErrorHasDeclaration",
        )
        .replace(
            "theorem commandInputsHaveProvenanceIsStable : commandInputsHaveProvenance = true := rfl\n\ntheorem commandErrorsAreDeclaredIsStable",
            "theorem commandInputsHaveProvenanceIsStable : commandInputsHaveProvenance = true := rfl\n\ntheorem commandInputsWithoutIssuingControlsHaveProvenanceIsStable : commandInputsWithoutIssuingControlsHaveProvenance = true := rfl\n\ntheorem commandSessionInputsHaveDescriptionsIsStable : commandSessionInputsHaveDescriptions = true := rfl\n\ntheorem commandErrorsAreDeclaredIsStable",
        )
        .replace(
            "def commandInputsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasProvenance)\n\ndef commandInputTracesToInvocationSource",
            "def commandInputsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasProvenance)\n\ndef commandInputsWithoutIssuingControlsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all (commandInputWithoutIssuingControlHasProvenance command))\n\ndef commandSessionInputsHaveDescriptions : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputSessionInputHasDescription)\n\ndef commandInputTracesToInvocationSource",
        )
        .replace(
            "theorem commandInputsHaveProvenanceIsStable : commandInputsHaveProvenance = true := rfl\n\ntheorem commandInputsTraceToInvocationSourcesIsStable",
            "theorem commandInputsHaveProvenanceIsStable : commandInputsHaveProvenance = true := rfl\n\ntheorem commandInputsWithoutIssuingControlsHaveProvenanceIsStable : commandInputsWithoutIssuingControlsHaveProvenance = true := rfl\n\ntheorem commandSessionInputsHaveDescriptionsIsStable : commandSessionInputsHaveDescriptions = true := rfl\n\ntheorem commandInputsTraceToInvocationSourcesIsStable",
        )
        .replace(
            "def controlInputVisibilityIsModeled (input : ControlInputProvision) : Bool := (input.sourceKind != \"actor\" || input.sketchToken.isEmpty == false || input.visibleToActor) && (input.decisionField == false || input.sketchToken.isEmpty == false || input.visibleToActor)\n\ndef controlHasSketchToken",
            "def controlInputVisibilityIsModeled (input : ControlInputProvision) : Bool := (input.sourceKind != \"actor\" || input.sketchToken.isEmpty == false || input.visibleToActor) && (input.decisionField == false || input.sketchToken.isEmpty == false || input.visibleToActor)\n\ndef controlInputDecisionFieldIsVisible (input : ControlInputProvision) : Bool := input.decisionField == false || input.sketchToken.isEmpty == false || input.visibleToActor\n\ndef controlHasSketchToken",
        )
        .replace(
            "def controlInputHasProvenance (input : ControlInputProvision) : Bool := input.name.isEmpty == false && input.sourceKind.isEmpty == false && input.sourceDescription.isEmpty == false\n\ndef controlInputVisibilityIsModeled",
            "def controlInputHasProvenance (input : ControlInputProvision) : Bool := input.name.isEmpty == false && input.sourceKind.isEmpty == false && input.sourceDescription.isEmpty == false\n\ndef controlInputHasDescription (input : ControlInputProvision) : Bool := input.sourceDescription.isEmpty == false\n\ndef controlInputSessionInputHasDescription (input : ControlInputProvision) : Bool := input.sourceKind != \"session\" || input.sourceDescription.isEmpty == false\n\ndef controlInputVisibilityIsModeled",
        )
        .replace(
            "def controlInputDecisionFieldIsVisible (input : ControlInputProvision) : Bool := input.decisionField == false || input.sketchToken.isEmpty == false || input.visibleToActor\n\ndef controlHasSketchToken",
            "def controlInputDecisionFieldIsVisible (input : ControlInputProvision) : Bool := input.decisionField == false || input.sketchToken.isEmpty == false || input.visibleToActor\n\ndef controlInputActorInputIsVisible (input : ControlInputProvision) : Bool := input.sourceKind != \"actor\" || input.sketchToken.isEmpty == false || input.visibleToActor\n\ndef controlHasSketchToken",
        )
        .replace(
            "def viewControlInputVisibilityIsModeled : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputVisibilityIsModeled))\n\ndef viewControlsHandleCommandErrors",
            "def viewControlInputVisibilityIsModeled : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputVisibilityIsModeled))\n\ndef viewControlDecisionFieldsAreVisible : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputDecisionFieldIsVisible))\n\ndef viewControlsHandleCommandErrors",
        )
        .replace(
            "def viewControlInputsHaveProvenance : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasProvenance))\n\ndef viewControlInputVisibilityIsModeled",
            "def viewControlInputsHaveProvenance : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasProvenance))\n\ndef viewControlInputsHaveDescriptions : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasDescription))\n\ndef viewControlSessionInputsHaveDescriptions : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputSessionInputHasDescription))\n\ndef viewControlInputVisibilityIsModeled",
        )
        .replace(
            "def viewControlDecisionFieldsAreVisible : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputDecisionFieldIsVisible))\n\ndef viewControlsHandleCommandErrors",
            "def viewControlDecisionFieldsAreVisible : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputDecisionFieldIsVisible))\n\ndef viewControlActorInputsAreVisible : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputActorInputIsVisible))\n\ndef viewControlsHandleCommandErrors",
        )
        .replace(
            "theorem viewControlInputVisibilityIsModeledIsStable : viewControlInputVisibilityIsModeled = true := rfl\n\ntheorem viewControlsHandleCommandErrorsIsStable",
            "theorem viewControlInputVisibilityIsModeledIsStable : viewControlInputVisibilityIsModeled = true := rfl\n\ntheorem viewControlDecisionFieldsAreVisibleIsStable : viewControlDecisionFieldsAreVisible = true := rfl\n\ntheorem viewControlsHandleCommandErrorsIsStable",
        );
    let contents = contents.replace(
        "theorem viewControlDecisionFieldsAreVisibleIsStable : viewControlDecisionFieldsAreVisible = true := rfl\n\ntheorem viewControlsHandleCommandErrorsIsStable",
        "theorem viewControlDecisionFieldsAreVisibleIsStable : viewControlDecisionFieldsAreVisible = true := rfl\n\ntheorem viewControlActorInputsAreVisibleIsStable : viewControlActorInputsAreVisible = true := rfl\n\ntheorem viewControlsHandleCommandErrorsIsStable",
    );
    let contents = contents.replace(
        "theorem viewControlInputsHaveProvenanceIsStable : viewControlInputsHaveProvenance = true := rfl\n\ntheorem viewControlInputVisibilityIsModeledIsStable",
        "theorem viewControlInputsHaveProvenanceIsStable : viewControlInputsHaveProvenance = true := rfl\n\ntheorem viewControlInputsHaveDescriptionsIsStable : viewControlInputsHaveDescriptions = true := rfl\n\ntheorem viewControlSessionInputsHaveDescriptionsIsStable : viewControlSessionInputsHaveDescriptions = true := rfl\n\ntheorem viewControlInputVisibilityIsModeledIsStable",
    );
    let contents = contents
        .replace(
            "def allowedCommandInputSourceKinds : List String := [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]",
            &format!(
                "def allowedCommandInputSourceKinds : List String := {allowed_command_input_source_kind_list}"
            ),
        )
        .replace(
            "def allowedRecoveryKinds : List String := [\"retry\",\"stay_on_screen\",\"navigation\",\"explicit_recovery_action\"]",
            &format!(
                "def allowedRecoveryKinds : List String := {allowed_recovery_kind_list}"
            ),
        )
        .replace(
            "def allowedSingletonRepeatBehaviors : List String := [\"already_exists_error\",\"idempotent\"]",
            &format!(
                "def allowedSingletonRepeatBehaviors : List String := {allowed_singleton_repeat_behavior_list}"
            ),
        )
        .replace(
            "def storedEventFactSourceKinds : List String := [\"command_input\",\"external_payload\",\"generated\",\"session\",\"derivation\"]",
            &format!(
                "def storedEventFactSourceKinds : List String := {stored_event_fact_source_kind_list}"
            ),
        )
        .replace(
            "def allowedReadModelFieldSourceKinds : List String := [\"event_attribute\",\"derivation\",\"absence_default\"]",
            &format!(
                "def allowedReadModelFieldSourceKinds : List String := {allowed_read_model_field_source_kind_list}"
            ),
        )
        .replace(
            "def allowedViewFieldSourceKinds : List String := [\"read_model\"]",
            &format!(
                "def allowedViewFieldSourceKinds : List String := {allowed_view_field_source_kind_list}"
            ),
        )
        .replace(
            "def allowedNavigationTargetTypes : List String := [\"modeled_view\",\"local_view_state\",\"external_system\",\"external_workflow\"]",
            &format!(
                "def allowedNavigationTargetTypes : List String := {allowed_navigation_target_type_list}"
            ),
        )
        .replace(
            "def canonicalBoardLanes : List String := [\"ux\",\"actions\",\"events\"]",
            &format!("def canonicalBoardLanes : List String := {canonical_board_lane_list}"),
        )
        .replace(
            "def boardLanesAreCanonical : Bool := canonicalBoardLanes == [\"ux\",\"actions\",\"events\"]",
            &format!(
                "def boardLanesAreCanonical : Bool := canonicalBoardLanes == {canonical_board_lane_list}"
            ),
        )
        .replace(
            "def allowedControlInputSourceKinds : List String := [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]",
            &format!(
                "def allowedControlInputSourceKinds : List String := {allowed_command_input_source_kind_list}"
            ),
        );
    let contents = contents
        .replace(
            "structure CommandDefinition where\n  name : String\n  inputs : List CommandInput\n  emittedEvents : List String\n  observedStreams : List String\n  errors : List CommandErrorDefinition\n  singleton : Bool\n  repeatBehavior : String\n\nstructure OutcomeDefinition where",
            "structure SliceEventReference where\n  name : String\n\nstructure SliceStreamReference where\n  name : String\n\nstructure CommandDefinition where\n  name : String\n  inputs : List CommandInput\n  emittedEvents : List SliceEventReference\n  observedStreams : List SliceStreamReference\n  errors : List CommandErrorDefinition\n  singleton : Bool\n  repeatBehavior : String\n\nstructure SliceCommandReference where\n  name : String\n\nstructure OutcomeDefinition where",
        )
        .replace(
            "structure ReadModelDefinition where\n  name : String\n  fields : List ReadModelField\n  transitive : Bool\n  relationshipFields : List String\n  transitiveRule : String\n  exampleScenarioName : String\n\nstructure ViewField where",
            "structure ReadModelDefinition where\n  name : String\n  fields : List ReadModelField\n  transitive : Bool\n  relationshipFields : List String\n  transitiveRule : String\n  exampleScenarioName : String\n\nstructure SliceReadModelReference where\n  name : String\n\nstructure ViewField where",
        )
        .replace(
            "structure ViewDefinition where\n  name : String\n  readModels : List String\n  fields : List ViewField\n  controls : List ControlDefinition\n  sketchTokens : List String\n  localStates : List String\n  filters : List String\n\nstructure AutomationDefinition where",
            "structure ViewDefinition where\n  name : String\n  readModels : List String\n  fields : List ViewField\n  controls : List ControlDefinition\n  sketchTokens : List String\n  localStates : List String\n  filters : List String\n\nstructure SliceViewReference where\n  name : String\n\nstructure AutomationDefinition where",
        )
        .replace(
            "def sliceCommands : List String := []\n\ndef sliceCommandDefinitions : List CommandDefinition := []",
            "def sliceCommands : List SliceCommandReference := []\n\ndef sliceCommandNames : List String := sliceCommands.map (fun commandRef => commandRef.name)\n\ndef sliceCommandDefinitions : List CommandDefinition := []",
        )
        .replace(
            "def sliceReferencedCommands : List String := []\n\ndef sliceOutcomeDefinitions : List OutcomeDefinition := []",
            "def sliceReferencedCommands : List SliceCommandReference := []\n\ndef sliceReferencedCommandNames : List String := sliceReferencedCommands.map (fun commandRef => commandRef.name)\n\ndef sliceOutcomeDefinitions : List OutcomeDefinition := []",
        )
        .replace(
            "def sliceEvents : List String := []\n\ndef sliceStreams : List StreamDefinition := []",
            "def sliceEvents : List SliceEventReference := []\n\ndef sliceEventNames : List String := sliceEvents.map (fun eventRef => eventRef.name)\n\ndef sliceStreams : List StreamDefinition := []",
        )
        .replace(
            "def sliceReadModels : List String := []\n\ndef sliceReadModelDefinitions : List ReadModelDefinition := []",
            "def sliceReadModels : List SliceReadModelReference := []\n\ndef sliceReadModelNames : List String := sliceReadModels.map (fun readModelRef => readModelRef.name)\n\ndef sliceReadModelDefinitions : List ReadModelDefinition := []",
        )
        .replace(
            "def sliceViews : List String := []\n\ndef sliceViewDefinitions : List ViewDefinition := []",
            "def sliceViews : List SliceViewReference := []\n\ndef sliceViewNames : List String := sliceViews.map (fun viewRef => viewRef.name)\n\ndef sliceViewDefinitions : List ViewDefinition := []",
        )
        .replace(
            "definitionNamesAreUnique sliceCommands",
            "definitionNamesAreUnique sliceCommandNames",
        )
        .replace(
            "definitionNamesAreUnique sliceEvents",
            "definitionNamesAreUnique sliceEventNames",
        )
        .replace(
            "definitionNamesAreUnique sliceReadModels",
            "definitionNamesAreUnique sliceReadModelNames",
        )
        .replace(
            "definitionNamesAreUnique sliceViews",
            "definitionNamesAreUnique sliceViewNames",
        )
        .replace("sliceCommands.contains ", "sliceCommandNames.contains ")
        .replace(
            "sliceReferencedCommands.contains ",
            "sliceReferencedCommandNames.contains ",
        )
        .replace(
            "sliceReadModels.contains ",
            "sliceReadModelNames.contains ",
        )
        .replace("sliceViews.contains ", "sliceViewNames.contains ")
        .replace(
            "def automationHasTrigger (automation : AutomationDefinition) : Bool :=",
            "def commandEmittedEventNames (command : CommandDefinition) : List String := command.emittedEvents.map (fun eventRef => eventRef.name)\n\ndef automationHasTrigger (automation : AutomationDefinition) : Bool :=",
        )
        .replace(
            "def commandInputEventStreamSourceResolves (command : CommandDefinition) (input : CommandInput) : Bool := input.sourceKind != \"event_stream_state\" || (command.observedStreams.isEmpty == false && command.observedStreams.all scenarioStreamResolves && input.eventStreamSourceEvent.isEmpty == false && input.eventStreamSourceAttribute.isEmpty == false && sliceEventDefinitions.any (fun event => event.name == input.eventStreamSourceEvent && event.attributes.any (fun eventAttribute => eventAttribute.name == input.eventStreamSourceAttribute)))",
            "def commandObservedStreamNames (command : CommandDefinition) : List String := command.observedStreams.map (fun streamRef => streamRef.name)\n\ndef commandInputEventStreamSourceResolves (command : CommandDefinition) (input : CommandInput) : Bool := input.sourceKind != \"event_stream_state\" || ((commandObservedStreamNames command).isEmpty == false && (commandObservedStreamNames command).all scenarioStreamResolves && input.eventStreamSourceEvent.isEmpty == false && input.eventStreamSourceAttribute.isEmpty == false && sliceEventDefinitions.any (fun event => event.name == input.eventStreamSourceEvent && event.attributes.any (fun eventAttribute => eventAttribute.name == input.eventStreamSourceAttribute)))",
        )
        .replace(
            "command.emittedEvents.contains ",
            "(commandEmittedEventNames command).contains ",
        )
        .replace(
            "command.emittedEvents.all commandEmittedEventIsKnown",
            "(commandEmittedEventNames command).all commandEmittedEventIsKnown",
        )
        .replace(
            "def eventIsKnownToSlice (eventName : String) : Bool := sliceEvents.contains eventName || sliceEventDefinitions.any (fun event => event.name == eventName && (event.observed || event.shared))",
            "def eventIsKnownToSlice (eventName : String) : Bool := sliceEventNames.contains eventName || sliceEventDefinitions.any (fun event => event.name == eventName && (event.observed || event.shared))",
        )
        .replace(
            "sliceEvents.contains element.declaredName",
            "sliceEventNames.contains element.declaredName",
        )
        .replace(
            "def commandEmittedEventIsKnown (eventName : String) : Bool := sliceEvents.contains eventName || sliceEventDefinitions.any (fun event => event.name == eventName)",
            "def commandEmittedEventIsKnown (eventName : String) : Bool := sliceEventNames.contains eventName || sliceEventDefinitions.any (fun event => event.name == eventName)",
        );
    file_contents(contents)
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

fn command_input_source_kind_list() -> String {
    format!(
        "[{}]",
        CommandInputSourceKind::ALLOWED
            .iter()
            .map(|source_kind| quoted(source_kind.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn command_error_recovery_kind_list() -> String {
    format!(
        "[{}]",
        CommandErrorRecoveryKind::ALLOWED
            .iter()
            .map(|recovery_kind| quoted(recovery_kind.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn singleton_repeat_behavior_list() -> String {
    format!(
        "[{}]",
        SingletonRepeatBehavior::ALLOWED
            .iter()
            .map(|repeat_behavior| quoted(repeat_behavior.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn navigation_target_type_list() -> String {
    format!(
        "[{}]",
        NavigationTargetType::ALLOWED
            .iter()
            .map(|target_type| quoted(target_type.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn event_attribute_source_kind_list() -> String {
    format!(
        "[{}]",
        EventAttributeSourceKind::ALLOWED
            .iter()
            .map(|source_kind| quoted(source_kind.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn board_lane_id_list() -> String {
    format!(
        "[{}]",
        BoardLaneId::CANONICAL
            .iter()
            .map(|lane| quoted(lane.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn read_model_field_source_kind_list() -> String {
    format!(
        "[{}]",
        ReadModelFieldSourceKind::ALLOWED
            .iter()
            .map(|source_kind| quoted(source_kind.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn view_field_source_kind_list() -> String {
    format!(
        "[{}]",
        ViewFieldSourceKind::ALLOWED
            .iter()
            .map(|source_kind| quoted(source_kind.as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn slice_list(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    format!(
        "[{}]",
        workflow_slice_details
            .iter()
            .map(|slice| format!("{{ slug := {} }}", quoted(slice.slug().as_ref())))
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
                    "{{ slug := {}, name := {}, kind := {}, description := {} }}",
                    quoted(slice.slug().as_ref()),
                    quoted(slice.name().as_ref()),
                    lean_slice_kind_name(*slice.kind()),
                    quoted(slice.description().as_ref())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn slice_module_list(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    format!(
        "[{}]",
        workflow_slice_details
            .iter()
            .map(|slice| {
                format!(
                    "{{ slice := {}, formalModule := {} }}",
                    quoted(slice.slug().as_ref()),
                    quoted(&module_name_from_raw(slice.name().as_ref()))
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn workflow_step_relationship_list(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    format!(
        "[{}]",
        workflow_slice_details
            .iter()
            .map(|slice| {
                format!(
                    "{{ step := {}, relationship := {} }}",
                    quoted(slice.slug().as_ref()),
                    lean_workflow_step_relationship_name(*slice.relationship())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn required_entry_lifecycle_state_list() -> String {
    format!(
        "[{}]",
        WorkflowEntryLifecycleStateName::REQUIRED
            .iter()
            .map(|state| lean_workflow_entry_lifecycle_state_name(*state))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn module_name_from_raw(raw: &str) -> String {
    let mut capitalize_next = true;
    raw.chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                let next = if capitalize_next {
                    character.to_ascii_uppercase()
                } else {
                    character
                };
                capitalize_next = false;
                Some(next)
            } else {
                capitalize_next = true;
                None
            }
        })
        .collect()
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

fn workflow_exit_target_list(workflow_transitions: &[WorkflowTransitionRecord]) -> String {
    format!(
        "[{}]",
        workflow_transitions
            .iter()
            .filter(|transition| transition.kind().is_workflow_exit())
            .map(|transition| quoted(transition.target().as_ref()))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn workflow_outcome_list(workflow_outcomes: &WorkflowOutcomeRecords) -> String {
    format!(
        "[{}]",
        workflow_outcomes
            .as_slice()
            .iter()
            .map(workflow_outcome_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn workflow_outcome_record(outcome: &WorkflowOutcomeRecord) -> String {
    format!(
        "{{ sourceSlice := {}, label := {}, externallyRelevant := {} }}",
        quoted(outcome.source_slice().as_ref()),
        quoted(outcome.label().as_ref()),
        if outcome.externally_relevant() {
            "true"
        } else {
            "false"
        },
    )
}

fn workflow_command_error_list(workflow_command_errors: &WorkflowCommandErrorRecords) -> String {
    format!(
        "[{}]",
        workflow_command_errors
            .as_slice()
            .iter()
            .map(workflow_command_error_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn workflow_command_error_record(error: &WorkflowCommandErrorRecord) -> String {
    format!(
        "{{ sourceSlice := {}, commandName := {}, errorName := {} }}",
        quoted(error.source_slice().as_ref()),
        quoted(error.command_name().as_ref()),
        quoted(error.error_name().as_ref()),
    )
}

fn workflow_owned_definition_list(
    workflow_owned_definitions: &WorkflowOwnedDefinitionRecords,
) -> String {
    format!(
        "[{}]",
        workflow_owned_definitions
            .as_slice()
            .iter()
            .map(workflow_owned_definition_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn workflow_owned_definition_record(definition: &WorkflowOwnedDefinitionRecord) -> String {
    format!(
        "{{ sourceSlice := {}, definitionKind := {}, definitionName := {}, definitionStream := {}, sourceProvenance := {}, eventParticipation := {}, viewRole := {} }}",
        quoted(definition.source_slice().as_ref()),
        lean_workflow_owned_definition_kind(*definition.definition_kind()),
        quoted(definition.definition_name().as_ref()),
        quoted(
            definition
                .definition_stream()
                .map_or("", |definition_stream| definition_stream.as_ref()),
        ),
        quoted(
            definition
                .source_provenance()
                .map_or("", |source_provenance| source_provenance.as_ref()),
        ),
        quoted(
            definition
                .event_participation()
                .map_or("", |event_participation| event_participation.as_ref()),
        ),
        quoted(
            definition
                .view_role()
                .map_or("", |view_role| view_role.as_ref())
        ),
    )
}

fn workflow_transition_evidence_list(
    workflow_transition_evidences: &WorkflowTransitionEvidenceRecords,
) -> String {
    format!(
        "[{}]",
        workflow_transition_evidences
            .as_slice()
            .iter()
            .map(workflow_transition_evidence_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn workflow_transition_evidence_record(evidence: &WorkflowTransitionEvidenceRecord) -> String {
    format!(
        "{{ source := {}, target := {}, kind := {}, trigger := {}, sourceEvidence := {}, targetEvidence := {} }}",
        quoted(evidence.source().as_ref()),
        quoted(evidence.target().as_ref()),
        lean_workflow_transition_kind(*evidence.kind()),
        quoted(evidence.trigger().as_ref()),
        quoted(evidence.source_evidence().as_ref()),
        quoted(evidence.target_evidence().as_ref()),
    )
}

fn workflow_entry_lifecycle_state_list(
    workflow_entry_lifecycle_states: &WorkflowEntryLifecycleStateRecords,
) -> String {
    format!(
        "[{}]",
        workflow_entry_lifecycle_states
            .as_slice()
            .iter()
            .map(workflow_entry_lifecycle_state_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn workflow_entry_lifecycle_state_record(coverage: &WorkflowEntryLifecycleStateRecord) -> String {
    format!(
        "{{ state := {}, step := {}, evidence := {} }}",
        lean_workflow_entry_lifecycle_state_name(*coverage.state()),
        quoted(coverage.step().as_ref()),
        quoted(coverage.evidence().as_ref()),
    )
}

fn transition_record(transition: &WorkflowTransitionRecord) -> String {
    format!(
        "{{ source := {}, target := {}, kind := {}, trigger := {}, rationale := {}, payloadContract := {} }}",
        quoted(transition.source().as_ref()),
        quoted(transition.target().as_ref()),
        lean_workflow_transition_kind(*transition.kind()),
        quoted(transition.trigger().as_ref()),
        quoted(
            transition
                .rationale()
                .map_or("", |rationale| rationale.as_ref())
        ),
        quoted(
            transition
                .payload_contract()
                .map_or("", |payload_contract| payload_contract.as_ref())
        )
    )
}
