use crate::core::effect::{ArtifactDigest, FileContents};
use crate::core::types::{
    ModelDescription, ModelName, QuintModuleName, SliceKindName, SliceSlug,
    WorkflowCommandErrorRecord, WorkflowCommandErrorRecords, WorkflowEntryLifecycleStateRecord,
    WorkflowEntryLifecycleStateRecords, WorkflowModuleData, WorkflowOutcomeRecord,
    WorkflowOutcomeRecords, WorkflowOwnedDefinitionRecord, WorkflowOwnedDefinitionRecords,
    WorkflowSliceDetail, WorkflowTransitionEvidenceRecord, WorkflowTransitionEvidenceRecords,
    WorkflowTransitionRecord, WorkflowTransitionRecords,
};

pub fn emit_workflow_module(
    module_name: QuintModuleName,
    workflow_module: WorkflowModuleData,
) -> FileContents {
    let slice_list = slice_list(workflow_module.workflow_slice_details().as_slice());
    let slice_detail_list = slice_detail_list(workflow_module.workflow_slice_details().as_slice());
    let workflow_slice_count = workflow_module.workflow_slice_details().as_slice().len();
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
    let transition_list = transition_list(workflow_module.workflow_transitions().clone());
    let workflow_requires_entry_lifecycle_coverage =
        workflow_module.workflow_requires_entry_lifecycle_coverage();
    file_contents(format!(
        r#"module {module_name} {{
  // EMC-DIGEST: {digest}
  type WorkflowSliceDetail = {{ slug: str, name: str, kind: str, description: str }}
  type WorkflowStepRelationship = {{ step: str, relationship: str }}
  type WorkflowTransition = {{ source: str, target: str, kind: str, trigger: str, rationale: str, payloadContract: str }}
  type WorkflowOutcome = {{ sourceSlice: str, label: str, externallyRelevant: bool }}
  type WorkflowCommandError = {{ sourceSlice: str, commandName: str, errorName: str }}
  type WorkflowOwnedDefinition = {{ sourceSlice: str, definitionKind: str, definitionName: str }}
  type WorkflowTransitionEvidence = {{ source: str, target: str, kind: str, trigger: str, sourceEvidence: str, targetEvidence: str }}
  type WorkflowEntryLifecycleState = {{ state: str, step: str, evidence: str }}
  val workflowName = {workflow_name_json}
  val workflowSlug = {workflow_slug_json}
  val workflowDescription = {workflow_description_json}
  val workflowSlices: List[str] = {slice_list}
  val workflowSliceDetails: List[WorkflowSliceDetail] = {slice_detail_list}
  val allowedWorkflowStepRelationships: List[str] = ["entry","main","branch","alternate","async_lifecycle","supporting"]
  val workflowStepRelationships: List[WorkflowStepRelationship] = {workflow_step_relationship_list}
  val workflowTransitions: List[WorkflowTransition] = {transition_list}
  val workflowOutcomes: List[WorkflowOutcome] = {workflow_outcome_list}
  val workflowCommandErrors: List[WorkflowCommandError] = {workflow_command_error_list}
  val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = {workflow_owned_definition_list}
  val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = {workflow_transition_evidence_list}
  val workflowRequiresEntryLifecycleCoverage = {workflow_requires_entry_lifecycle_coverage}
  val workflowEntryLifecycleStates: List[WorkflowEntryLifecycleState] = {workflow_entry_lifecycle_state_list}
  val workflowExitTargets: List[str] = {workflow_exit_target_list}
  val requiredEntryLifecycleStates: List[str] = ["fresh_uninitialized","initialized_unauthenticated","initialized_authenticated","partially_configured","fully_configured"]
  val workflowIdentityStable = workflowName == {workflow_name_json}
  val workflowSlicesHaveDetails = length(workflowSlices) == length(workflowSliceDetails)
  val workflowSliceDetailsComplete = workflowSlicesHaveDetails
  val workflowTransitionsStructured = workflowTransitions.select(transition => transition.source != "" and transition.target != "" and transition.kind != "" and transition.trigger != "").length() == workflowTransitions.length()
  val workflowTransitionSourcesResolve = workflowTransitions.select(transition => workflowSlices.select(step => step == transition.source).length() > 0).length() == workflowTransitions.length()
  val workflowTransitionTargetsResolve = workflowTransitions.select(transition => workflowSlices.select(step => step == transition.target).length() > 0 or workflowExitTargets.select(exitTarget => exitTarget == transition.target).length() > 0).length() == workflowTransitions.length()
  def workflowStepRelationshipIsAllowed(step) = workflowSlices.select(workflowSlice => workflowSlice == step.step).length() > 0 and allowedWorkflowStepRelationships.select(relationship => relationship == step.relationship).length() > 0
  val workflowStepRelationshipsAreAllowed = workflowStepRelationships.select(step => workflowStepRelationshipIsAllowed(step)).length() == workflowStepRelationships.length()
  val workflowStepSlugsAreUnique = workflowSlices.select(step => workflowSlices.select(other => other == step).length() == 1).length() == workflowSlices.length()
  val workflowHasExactlyOneEntryStep = workflowStepRelationships.select(step => step.relationship == "entry").length() == 1
  def workflowMainStepHasIncomingTransition(step) = step.relationship != "main" or workflowTransitions.select(transition => transition.target == step.step).length() > 0
  val workflowMainStepsHaveIncomingReachability = workflowStepRelationships.select(step => workflowMainStepHasIncomingTransition(step)).length() == workflowStepRelationships.length()
  val workflowEntrySteps: List[str] = workflowStepRelationships.select(step => step.relationship == "entry").foldl([], (entrySteps, step) => entrySteps.append(step.step))
  def workflowTargetsFromReachable(reachable) = workflowTransitions.select(transition => reachable.select(step => step == transition.source).length() > 0 and workflowSlices.select(step => step == transition.target).length() > 0).foldl([], (targets, transition) => targets.append(transition.target))
  def workflowReachableStepsAfterFuel(fuel, reachable) = range(0, fuel).foldl(reachable, (currentReachable, _) => currentReachable.concat(workflowTargetsFromReachable(currentReachable)))
  val workflowReachableStepsFromEntry = workflowReachableStepsAfterFuel({workflow_slice_count}, workflowEntrySteps)
  def workflowStepIsReachableFromEntry(step) = step.relationship == "supporting" or workflowReachableStepsFromEntry.select(reachableStep => reachableStep == step.step).length() > 0
  val workflowNonSupportingStepsReachableFromEntry = workflowStepRelationships.select(step => workflowStepIsReachableFromEntry(step)).length() == workflowStepRelationships.length()
  def workflowBranchOrAlternateStepHasTriggerOrRationale(step) = (step.relationship != "branch" and step.relationship != "alternate") or workflowTransitions.select(transition => transition.target == step.step and (transition.trigger != "" or transition.rationale != "")).length() > 0
  val workflowBranchAndAlternateStepsHaveTriggerOrRationale = workflowStepRelationships.select(step => workflowBranchOrAlternateStepHasTriggerOrRationale(step)).length() == workflowStepRelationships.length()
  def workflowTransitionKindIsModeled(transition) = transition.kind == "navigation" or transition.kind == "command" or transition.kind == "event" or transition.kind == "external_trigger" or transition.kind == "outcome" or workflowExitTargets.select(exitTarget => exitTarget == transition.target).length() > 0
  def workflowTransitionExitHasRationale(transition) = workflowExitTargets.select(exitTarget => exitTarget == transition.target).length() == 0 or transition.rationale != ""
  val workflowTransitionsHaveModeledKinds = workflowTransitions.select(transition => workflowTransitionKindIsModeled(transition)).length() == workflowTransitions.length()
  val workflowExitsNameTargetsAndRationale = workflowTransitions.select(transition => workflowTransitionExitHasRationale(transition)).length() == workflowTransitions.length()
  def workflowOutcomeHandledByTransition(outcome) = not(outcome.externallyRelevant) or workflowTransitions.select(transition => transition.source == outcome.sourceSlice and transition.kind == "outcome" and transition.trigger == outcome.label).length() > 0
  val workflowExternallyRelevantOutcomesHandled = workflowOutcomes.select(outcome => workflowOutcomeHandledByTransition(outcome)).length() == workflowOutcomes.length()
  def workflowOutcomeSourceResolves(outcome) = workflowSlices.select(step => step == outcome.sourceSlice).length() > 0
  val workflowOutcomesSourceResolve = workflowOutcomes.select(outcome => workflowOutcomeSourceResolves(outcome)).length() == workflowOutcomes.length()
  def workflowCommandErrorSourceResolves(error) = workflowSlices.select(step => step == error.sourceSlice).length() > 0
  val workflowCommandErrorsSourceResolve = workflowCommandErrors.select(error => workflowCommandErrorSourceResolves(error)).length() == workflowCommandErrors.length()
  val workflowTransitionsDoNotUseCommandErrorsAsOutcomes = workflowTransitions.select(transition => transition.kind != "outcome" or workflowCommandErrors.select(error => error.sourceSlice == transition.source and error.errorName == transition.trigger).length() == 0).length() == workflowTransitions.length()
  def workflowNonEventDefinitionOwnedOnce(definition) = definition.definitionKind == "event" or workflowOwnedDefinitions.select(other => other.definitionKind == definition.definitionKind and other.definitionName == definition.definitionName).length() == 1
  val workflowNonEventDefinitionsAreUniquelyOwned = workflowOwnedDefinitions.select(definition => workflowNonEventDefinitionOwnedOnce(definition)).length() == workflowOwnedDefinitions.length()
  def workflowOwnsDefinition(sourceSlice, definitionKind, definitionName) = workflowOwnedDefinitions.select(definition => definition.sourceSlice == sourceSlice and definition.definitionKind == definitionKind and definition.definitionName == definitionName).length() > 0
  def workflowCommandTransitionTargetsOwnedCommand(transition) = transition.kind != "command" or workflowOwnsDefinition(transition.target, "command", transition.trigger)
  def workflowCommandTransitionSourceOwnsControl(transition) = transition.kind != "command" or workflowOwnsDefinition(transition.source, "control", transition.trigger)
  val workflowCommandTransitionsResolveControlsAndCommands = workflowTransitions.select(transition => workflowCommandTransitionSourceOwnsControl(transition) and workflowCommandTransitionTargetsOwnedCommand(transition)).length() == workflowTransitions.length()
  def workflowEventTransitionIsSharedByEndpoints(transition) = transition.kind != "event" or (workflowOwnsDefinition(transition.source, "event", transition.trigger) and workflowOwnsDefinition(transition.target, "event", transition.trigger))
  val workflowEventTransitionsAreSharedByEndpointSlices = workflowTransitions.select(transition => workflowEventTransitionIsSharedByEndpoints(transition)).length() == workflowTransitions.length()
  def workflowNavigationTransitionSourceOwnsControl(transition) = transition.kind != "navigation" or workflowOwnsDefinition(transition.source, "control", transition.trigger)
  def workflowNavigationTransitionTargetsOwnedView(transition) = transition.kind != "navigation" or workflowOwnsDefinition(transition.target, "view", transition.trigger)
  val workflowNavigationTransitionsResolveControlsAndViews = workflowTransitions.select(transition => workflowNavigationTransitionSourceOwnsControl(transition) and workflowNavigationTransitionTargetsOwnedView(transition)).length() == workflowTransitions.length()
  def workflowExternalTriggerDeclaresPayloadContract(transition) = transition.kind != "external_trigger" or transition.payloadContract != ""
  val workflowExternalTriggersDeclarePayloadContracts = workflowTransitions.select(transition => workflowExternalTriggerDeclaresPayloadContract(transition)).length() == workflowTransitions.length()
  def workflowTransitionRequiresEvidence(transition) = transition.kind == "event" or transition.kind == "command" or transition.kind == "navigation"
  def workflowTransitionEvidenceMatches(transition, evidence) = evidence.source == transition.source and evidence.target == transition.target and evidence.kind == transition.kind and evidence.trigger == transition.trigger and evidence.sourceEvidence != "" and evidence.targetEvidence != ""
  def workflowTransitionHasRequiredEvidence(transition) = not(workflowTransitionRequiresEvidence(transition)) or workflowTransitionEvidences.select(evidence => workflowTransitionEvidenceMatches(transition, evidence)).length() > 0
  val workflowTransitionsHaveRequiredEvidence = workflowTransitions.select(transition => workflowTransitionHasRequiredEvidence(transition)).length() == workflowTransitions.length()
  def workflowEntryLifecycleStateCovered(state) = workflowEntryLifecycleStates.select(coverage => coverage.state == state and workflowSlices.select(step => step == coverage.step).length() > 0 and coverage.evidence != "").length() > 0
  val workflowEntryLifecycleStatesCoverRequiredStates = not(workflowRequiresEntryLifecycleCoverage) or requiredEntryLifecycleStates.select(state => workflowEntryLifecycleStateCovered(state)).length() == requiredEntryLifecycleStates.length()
  var modelState: int
  action init = modelState' = 0
  action step = modelState' = modelState
}}
"#,
        module_name = module_name.as_ref(),
        digest = workflow_module.digest().as_ref(),
        workflow_name_json = quoted(workflow_module.workflow_name().as_ref()),
        workflow_slug_json = quoted(workflow_module.workflow_slug().as_ref()),
        workflow_description_json = quoted(workflow_module.workflow_description().as_ref()),
        slice_list = slice_list,
        slice_detail_list = slice_detail_list,
        workflow_slice_count = workflow_slice_count,
        workflow_step_relationship_list = workflow_step_relationship_list,
        transition_list = transition_list,
        workflow_outcome_list = workflow_outcome_list,
        workflow_command_error_list = workflow_command_error_list,
        workflow_owned_definition_list = workflow_owned_definition_list,
        workflow_transition_evidence_list = workflow_transition_evidence_list,
        workflow_requires_entry_lifecycle_coverage = workflow_requires_entry_lifecycle_coverage,
        workflow_entry_lifecycle_state_list = workflow_entry_lifecycle_state_list,
        workflow_exit_target_list = workflow_exit_target_list,
    ))
}

pub fn emit_slice_module(
    module_name: QuintModuleName,
    slice_name: ModelName,
    slice_description: ModelDescription,
    slice_slug: SliceSlug,
    slice_kind: SliceKindName,
    digest: ArtifactDigest,
) -> FileContents {
    let contents = format!(
        "module {module_name} {{\n  // EMC-DIGEST: {digest}\n  // EMC generated Quint business slice model.\n  type EventModelScenario = {{ name: str, givenSteps: List[str], whenSteps: List[str], thenSteps: List[str] }}\n  type BitLevelDataFlow = {{ datum: str, source: str, transformationSemantics: str, target: str, bitEncoding: str }}\n  type CommandInput = {{ name: str, sourceKind: str, sourceDescription: str, provenanceChain: List[str] }}\n  type CommandErrorDefinition = {{ name: str, scenarioName: str, recoveryKind: str }}\n  type CommandDefinition = {{ name: str, inputs: List[CommandInput], emittedEvents: List[str], observedStreams: List[str], errors: List[CommandErrorDefinition], singleton: bool, repeatBehavior: str }}\n  type OutcomeDefinition = {{ label: str, eventSet: List[str], externallyRelevant: bool }}\n  type StreamDefinition = {{ name: str }}\n  type EventAttribute = {{ name: str, sourceKind: str, sourceName: str, sourceField: str, provenanceDescription: str }}\n  type EventDefinition = {{ name: str, stream: str, attributes: List[EventAttribute], observed: bool, shared: bool }}\n  type ReadModelField = {{ name: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, absenceEvent: str, provenanceDescription: str }}\n  type ReadModelDefinition = {{ name: str, fields: List[ReadModelField] }}\n  type ViewField = {{ name: str, sourceKind: str, sourceReadModel: str, sourceField: str, sketchToken: str, provenanceDescription: str, bitEncoding: str }}\n  type ControlInputProvision = {{ name: str, sourceKind: str, sourceDescription: str, sketchToken: str, visibleToActor: bool, decisionField: bool }}\n  type ControlDefinition = {{ name: str, commandName: str, inputs: List[ControlInputProvision], handledErrors: List[str], recoveryBehavior: str, sketchToken: str }}\n  type ViewDefinition = {{ name: str, readModels: List[str], fields: List[ViewField], controls: List[ControlDefinition], sketchTokens: List[str] }}\n  val sliceName = {slice_name_json}\n  val sliceSlug = {slice_slug_json}\n  val sliceKind = {slice_kind_json}\n  val sliceDescription = {slice_description_json}\n  val sliceCommands: List[str] = []\n  val sliceCommandDefinitions: List[CommandDefinition] = []\n  val sliceReferencedCommands: List[str] = []\n  val sliceOutcomeDefinitions: List[OutcomeDefinition] = []\n  val allowedCommandInputSourceKinds: List[str] = [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]\n  val allowedRecoveryKinds: List[str] = [\"retry\",\"stay_on_screen\",\"navigation\",\"explicit_recovery_action\"]\n  val allowedSingletonRepeatBehaviors: List[str] = [\"already_exists_error\",\"idempotent\"]\n  val sliceEvents: List[str] = []\n  val sliceStreams: List[StreamDefinition] = []\n  val sliceEventDefinitions: List[EventDefinition] = []\n  val allowedEventAttributeSourceKinds: List[str] = [\"command_input\",\"external_payload\",\"generated\",\"session\",\"constant\",\"derivation\"]\n  val sliceReadModels: List[str] = []\n  val sliceReadModelDefinitions: List[ReadModelDefinition] = []\n  val allowedReadModelFieldSourceKinds: List[str] = [\"event_attribute\",\"derivation\",\"absence_default\"]\n  val sliceViews: List[str] = []\n  val sliceViewDefinitions: List[ViewDefinition] = []\n  val allowedViewFieldSourceKinds: List[str] = [\"read_model\"]\n  val allowedControlInputSourceKinds: List[str] = [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]\n  val sliceAcceptanceScenarios: List[EventModelScenario] = []\n  val sliceContractScenarios: List[EventModelScenario] = []\n  val sliceBitLevelDataFlows: List[BitLevelDataFlow] = []\n  val sliceAcceptanceScenariosHaveGwt = sliceAcceptanceScenarios.select(scenario => scenario.name != \"\" and scenario.givenSteps.length() > 0 and scenario.whenSteps.length() > 0 and scenario.thenSteps.length() > 0).length() == sliceAcceptanceScenarios.length()\n  val sliceContractScenariosHaveGwt = sliceContractScenarios.select(scenario => scenario.name != \"\" and scenario.givenSteps.length() > 0 and scenario.whenSteps.length() > 0 and scenario.thenSteps.length() > 0).length() == sliceContractScenarios.length()\n  val sliceScenariosHaveGwt = sliceAcceptanceScenariosHaveGwt and sliceContractScenariosHaveGwt\n  val sliceScenarioNamesAreUnique = sliceAcceptanceScenarios.select(scenario => sliceAcceptanceScenarios.select(other => other.name == scenario.name).length() + sliceContractScenarios.select(other => other.name == scenario.name).length() == 1).length() == sliceAcceptanceScenarios.length() and sliceContractScenarios.select(scenario => sliceAcceptanceScenarios.select(other => other.name == scenario.name).length() + sliceContractScenarios.select(other => other.name == scenario.name).length() == 1).length() == sliceContractScenarios.length()\n  val commandInputsHaveAllowedSources = sliceCommandDefinitions.select(command => command.inputs.select(input => allowedCommandInputSourceKinds.select(sourceKind => sourceKind == input.sourceKind).length() > 0).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()\n  val commandInputsHaveProvenance = sliceCommandDefinitions.select(command => command.inputs.select(input => input.name != \"\" and input.sourceKind != \"\" and input.sourceDescription != \"\" and input.provenanceChain.length() > 0).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()\n  val commandErrorsAreDeclared = sliceCommandDefinitions.select(command => command.errors.select(error => error.name != \"\" and error.scenarioName != \"\" and error.recoveryKind != \"\").length() == command.errors.length()).length() == sliceCommandDefinitions.length()\n  val commandErrorsHaveAllowedRecovery = sliceCommandDefinitions.select(command => command.errors.select(error => allowedRecoveryKinds.select(recoveryKind => recoveryKind == error.recoveryKind).length() > 0).length() == command.errors.length()).length() == sliceCommandDefinitions.length()\n  def sameOutcomeEventSet(left, right) = left.eventSet.select(eventName => right.eventSet.select(otherEventName => otherEventName == eventName).length() > 0).length() == left.eventSet.length() and right.eventSet.select(eventName => left.eventSet.select(otherEventName => otherEventName == eventName).length() > 0).length() == right.eventSet.length()\n  def eventIsKnownToSlice(eventName) = sliceEvents.select(event => event == eventName).length() > 0 or sliceEventDefinitions.select(event => event.name == eventName and (event.observed or event.shared)).length() > 0\n  val outcomeLabelsAreUnique = sliceOutcomeDefinitions.select(outcome => sliceOutcomeDefinitions.select(other => other.label == outcome.label).length() == 1).length() == sliceOutcomeDefinitions.length()\n  val outcomeEventSetsAreNonEmpty = sliceOutcomeDefinitions.select(outcome => outcome.eventSet.length() > 0).length() == sliceOutcomeDefinitions.length()\n  val outcomeEventSetsAreDistinct = sliceOutcomeDefinitions.select(outcome => sliceOutcomeDefinitions.select(other => outcome.label == other.label or not(sameOutcomeEventSet(outcome, other))).length() == sliceOutcomeDefinitions.length()).length() == sliceOutcomeDefinitions.length()\n  val outcomeEventsAreKnownToSlice = sliceOutcomeDefinitions.select(outcome => outcome.eventSet.select(eventName => eventIsKnownToSlice(eventName)).length() == outcome.eventSet.length()).length() == sliceOutcomeDefinitions.length()\n  val eventsReferenceKnownStreams = sliceEventDefinitions.select(event => sliceStreams.select(stream => stream.name == event.stream).length() > 0).length() == sliceEventDefinitions.length()\n  val eventAttributesHaveAllowedSources = sliceEventDefinitions.select(event => event.attributes.select(attribute => allowedEventAttributeSourceKinds.select(sourceKind => sourceKind == attribute.sourceKind).length() > 0).length() == event.attributes.length()).length() == sliceEventDefinitions.length()\n  val eventAttributesHaveProvenance = sliceEventDefinitions.select(event => event.attributes.select(attribute => attribute.name != \"\" and attribute.sourceKind != \"\" and attribute.sourceName != \"\" and attribute.provenanceDescription != \"\").length() == event.attributes.length()).length() == sliceEventDefinitions.length()\n  val readModelFieldsHaveAllowedSources = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => allowedReadModelFieldSourceKinds.select(sourceKind => sourceKind == readModelField.sourceKind).length() > 0).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()\n  val readModelFieldsHaveProvenance = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelField.name != \"\" and readModelField.sourceKind != \"\" and readModelField.provenanceDescription != \"\").length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()\n  val readModelFieldSourcesAreComplete = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => (readModelField.sourceKind == \"event_attribute\" and readModelField.sourceEvent != \"\" and readModelField.sourceAttribute != \"\") or (readModelField.sourceKind == \"derivation\" and readModelField.derivationRule != \"\") or (readModelField.sourceKind == \"absence_default\" and readModelField.absenceEvent != \"\")).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()\n  val viewFieldsHaveAllowedSources = sliceViewDefinitions.select(view => view.fields.select(viewField => allowedViewFieldSourceKinds.select(sourceKind => sourceKind == viewField.sourceKind).length() > 0).length() == view.fields.length()).length() == sliceViewDefinitions.length()\n  val viewFieldsHaveProvenance = sliceViewDefinitions.select(view => view.fields.select(viewField => viewField.name != \"\" and viewField.sourceKind != \"\" and viewField.provenanceDescription != \"\" and viewField.bitEncoding != \"\").length() == view.fields.length()).length() == sliceViewDefinitions.length()\n  val viewFieldSourcesAreComplete = sliceViewDefinitions.select(view => view.fields.select(viewField => viewField.sourceKind == \"read_model\" and viewField.sourceReadModel != \"\" and viewField.sourceField != \"\" and viewField.sketchToken != \"\").length() == view.fields.length()).length() == sliceViewDefinitions.length()\n  val viewFieldsSourceFromUsedReadModels = sliceViewDefinitions.select(view => view.fields.select(viewField => view.readModels.select(readModel => readModel == viewField.sourceReadModel).length() > 0 and sliceReadModels.select(readModel => readModel == viewField.sourceReadModel).length() > 0).length() == view.fields.length()).length() == sliceViewDefinitions.length()\n  val viewControlsHaveSketchTokens = sliceViewDefinitions.select(view => view.controls.select(control => control.name != \"\" and control.commandName != \"\" and control.sketchToken != \"\").length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val viewControlsReferenceKnownCommands = sliceViewDefinitions.select(view => view.controls.select(control => sliceCommands.select(command => command == control.commandName).length() > 0 or sliceReferencedCommands.select(command => command == control.commandName).length() > 0 or sliceCommandDefinitions.select(command => command.name == control.commandName).length() > 0).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val viewControlInputsHaveAllowedSources = sliceViewDefinitions.select(view => view.controls.select(control => control.inputs.select(input => allowedControlInputSourceKinds.select(sourceKind => sourceKind == input.sourceKind).length() > 0).length() == control.inputs.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val viewControlInputsHaveProvenance = sliceViewDefinitions.select(view => view.controls.select(control => control.inputs.select(input => input.name != \"\" and input.sourceKind != \"\" and input.sourceDescription != \"\").length() == control.inputs.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val viewControlInputVisibilityIsModeled = sliceViewDefinitions.select(view => view.controls.select(control => control.inputs.select(input => (input.sourceKind != \"actor\" or input.sketchToken != \"\" or input.visibleToActor) and (not(input.decisionField) or input.sketchToken != \"\" or input.visibleToActor)).length() == control.inputs.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val viewControlsHandleCommandErrors = sliceViewDefinitions.select(view => view.controls.select(control => sliceCommandDefinitions.select(command => command.name != control.commandName or command.errors.select(error => control.handledErrors.select(handledError => handledError == error.name).length() > 0 and control.recoveryBehavior != \"\").length() == command.errors.length()).length() == sliceCommandDefinitions.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val sliceIdentityStable = sliceName == {slice_name_json}\n  val sliceStateChangeRequiresEvent = sliceKind != \"state_change\" or length(sliceEvents) > 0\n  val sliceBitLevelDataFlowsStructured = sliceBitLevelDataFlows.select(flow => flow.datum != \"\" and flow.source != \"\" and flow.transformationSemantics != \"\" and flow.target != \"\" and flow.bitEncoding != \"\").length() == sliceBitLevelDataFlows.length()\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}}\n",
        module_name = module_name.as_ref(),
        digest = digest.as_ref(),
        slice_name_json = quoted(slice_name.as_ref()),
        slice_slug_json = quoted(slice_slug.as_ref()),
        slice_kind_json = quoted(slice_kind.as_ref()),
        slice_description_json = quoted(slice_description.as_ref()),
    );
    let contents = contents
        .replace(
            "type ControlDefinition = { name: str, commandName: str, inputs: List[ControlInputProvision], handledErrors: List[str], recoveryBehavior: str, sketchToken: str }\n  type ViewDefinition = { name: str, readModels: List[str], fields: List[ViewField], controls: List[ControlDefinition], sketchTokens: List[str] }",
            "type NavigationTarget = { targetType: str, targetName: str, externalWorkflowName: str, externalSystemName: str, handoffContract: str }\n  type ControlDefinition = { name: str, commandName: str, inputs: List[ControlInputProvision], handledErrors: List[str], recoveryBehavior: str, sketchToken: str, navigation: NavigationTarget }\n  type ViewDefinition = { name: str, readModels: List[str], fields: List[ViewField], controls: List[ControlDefinition], sketchTokens: List[str], localStates: List[str] }",
        )
        .replace(
            "val sliceCommands: List[str] = []",
            "type AutomationDefinition = { name: str, triggerName: str, commandName: str, handledErrors: List[str], reactionDescription: str }\n  type TranslationDefinition = { name: str, externalEventName: str, payloadContractName: str, commandName: str }\n  type BoardElement = { name: str, kind: str, lane: str, declaredName: str, mainPath: bool }\n  type BoardConnection = { source: str, sourceKind: str, target: str, targetKind: str }\n  val sliceCommands: List[str] = []",
        )
        .replace(
            "val sliceReferencedCommands: List[str] = []",
            "val sliceAutomations: List[AutomationDefinition] = []\n  val sliceTranslations: List[TranslationDefinition] = []\n  val canonicalBoardLanes: List[str] = [\"ux\",\"actions\",\"events\"]\n  val sliceBoardElements: List[BoardElement] = []\n  val sliceBoardConnections: List[BoardConnection] = []\n  val sliceReferencedCommands: List[str] = []",
        )
        .replace(
            "type EventModelScenario = { name: str, givenSteps: List[str], whenSteps: List[str], thenSteps: List[str] }",
            "type EventModelScenario = { name: str, givenSteps: List[str], whenSteps: List[str], thenSteps: List[str], readStreams: List[str], writtenStreams: List[str], contractKind: str, coveredDefinition: str, errorReferences: List[str] }",
        )
        .replace(
            "type EventDefinition = { name: str, stream: str, attributes: List[EventAttribute], observed: bool, shared: bool }",
            "type ExternalPayloadField = { name: str, provenanceDescription: str, bitEncoding: str }\n  type ExternalPayloadDefinition = { name: str, fields: List[ExternalPayloadField] }\n  type EventDefinition = { name: str, stream: str, attributes: List[EventAttribute], observed: bool, shared: bool }",
        )
        .replace(
            "type ReadModelField = { name: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, absenceEvent: str, provenanceDescription: str }",
            "type ReadModelField = { name: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, absenceEvent: str, derivationScenarioName: str, absenceScenarioName: str, provenanceDescription: str }",
        )
        .replace(
            "type ReadModelDefinition = { name: str, fields: List[ReadModelField] }",
            "type ReadModelDefinition = { name: str, fields: List[ReadModelField], transitive: bool, relationshipFields: List[str], transitiveRule: str, exampleScenarioName: str }",
        )
        .replace(
            "val sliceEventDefinitions: List[EventDefinition] = []",
            "val sliceExternalPayloads: List[ExternalPayloadDefinition] = []\n  val sliceEventDefinitions: List[EventDefinition] = []",
        )
        .replace(
            "val sliceAcceptanceScenarios: List[EventModelScenario] = []",
            "val allowedNavigationTargetTypes: List[str] = [\"modeled_view\",\"local_view_state\",\"external_system\",\"external_workflow\"]\n  val sliceAcceptanceScenarios: List[EventModelScenario] = []",
        )
        .replace(
            "val eventAttributesHaveAllowedSources = sliceEventDefinitions.select(event => event.attributes.select(attribute => allowedEventAttributeSourceKinds.select(sourceKind => sourceKind == attribute.sourceKind).length() > 0).length() == event.attributes.length()).length() == sliceEventDefinitions.length()",
            "def commandEmittedEventIsKnown(eventName) = sliceEvents.select(event => event == eventName).length() > 0 or sliceEventDefinitions.select(event => event.name == eventName).length() > 0\n  def eventProducedByCommand(event) = event.observed or event.shared or sliceCommandDefinitions.select(command => command.emittedEvents.select(eventName => eventName == event.name).length() > 0).length() > 0\n  val commandEmittedEventsAreKnown = sliceCommandDefinitions.select(command => command.emittedEvents.select(eventName => commandEmittedEventIsKnown(eventName)).length() == command.emittedEvents.length()).length() == sliceCommandDefinitions.length()\n  val locallyEmittedEventsAreProducedByCommands = sliceEventDefinitions.select(event => eventProducedByCommand(event)).length() == sliceEventDefinitions.length()\n  val eventAttributesHaveAllowedSources = sliceEventDefinitions.select(event => event.attributes.select(attribute => allowedEventAttributeSourceKinds.select(sourceKind => sourceKind == attribute.sourceKind).length() > 0).length() == event.attributes.length()).length() == sliceEventDefinitions.length()",
        )
        .replace(
            "val commandInputsHaveAllowedSources = sliceCommandDefinitions.select(command => command.inputs.select(input => allowedCommandInputSourceKinds.select(sourceKind => sourceKind == input.sourceKind).length() > 0).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()",
            "def scenarioStreamResolves(streamName) = sliceStreams.select(stream => stream.name == streamName).length() > 0\n  def scenarioStreamsResolve(scenario) = scenario.readStreams.select(streamName => scenarioStreamResolves(streamName)).length() == scenario.readStreams.length() and scenario.writtenStreams.select(streamName => scenarioStreamResolves(streamName)).length() == scenario.writtenStreams.length()\n  def stateChangeScenarioNamesStreams(scenario) = sliceKind != \"state_change\" or (scenario.readStreams.length() > 0 and scenario.writtenStreams.length() > 0)\n  val sliceAcceptanceScenarioStreamsResolve = sliceAcceptanceScenarios.select(scenario => scenarioStreamsResolve(scenario)).length() == sliceAcceptanceScenarios.length()\n  val sliceContractScenarioStreamsResolve = sliceContractScenarios.select(scenario => scenarioStreamsResolve(scenario)).length() == sliceContractScenarios.length()\n  val sliceScenarioStreamsResolve = sliceAcceptanceScenarioStreamsResolve and sliceContractScenarioStreamsResolve\n  val stateChangeAcceptanceScenariosNameStreams = sliceAcceptanceScenarios.select(scenario => stateChangeScenarioNamesStreams(scenario)).length() == sliceAcceptanceScenarios.length()\n  val stateChangeContractScenariosNameStreams = sliceContractScenarios.select(scenario => stateChangeScenarioNamesStreams(scenario)).length() == sliceContractScenarios.length()\n  val stateChangeScenariosNameStreams = stateChangeAcceptanceScenariosNameStreams and stateChangeContractScenariosNameStreams\n  val acceptanceScenariosAreUserFacing = sliceAcceptanceScenarios.select(scenario => scenario.contractKind == \"\" and scenario.coveredDefinition == \"\").length() == sliceAcceptanceScenarios.length()\n  def scenarioCoversContract(contractKind, definitionName, scenario) = scenario.contractKind == contractKind and scenario.coveredDefinition == definitionName\n  def readModelHasProjectorContract(readModel) = sliceContractScenarios.select(scenario => scenarioCoversContract(\"projector\", readModel.name, scenario)).length() > 0\n  val stateViewReadModelsHaveProjectorContracts = sliceKind != \"state_view\" or sliceReadModelDefinitions.select(readModel => readModelHasProjectorContract(readModel)).length() == sliceReadModelDefinitions.length()\n  def contractScenarioTargetsKnownDefinition(scenario) = (scenario.contractKind == \"projector\" and (sliceReadModels.select(readModel => readModel == scenario.coveredDefinition).length() > 0 or sliceReadModelDefinitions.select(readModel => readModel.name == scenario.coveredDefinition).length() > 0)) or (scenario.contractKind == \"command\" and (sliceCommands.select(command => command == scenario.coveredDefinition).length() > 0 or sliceCommandDefinitions.select(command => command.name == scenario.coveredDefinition).length() > 0)) or (scenario.contractKind == \"automation\" and sliceAutomations.select(automation => automation.name == scenario.coveredDefinition).length() > 0) or (scenario.contractKind == \"translation\" and sliceTranslations.select(translation => translation.name == scenario.coveredDefinition).length() > 0) or (scenario.contractKind == \"derivation\" and scenario.coveredDefinition != \"\" and sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelField.sourceKind == \"derivation\" and readModelField.derivationScenarioName == scenario.name).length() > 0).length() > 0) or (scenario.contractKind == \"absence\" and scenario.coveredDefinition != \"\" and sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelField.sourceKind == \"absence_default\" and readModelField.absenceScenarioName == scenario.name).length() > 0).length() > 0) or (scenario.contractKind == \"transitive\" and sliceReadModelDefinitions.select(readModel => readModel.transitive and readModel.name == scenario.coveredDefinition and readModel.exampleScenarioName == scenario.name).length() > 0)\n  val contractScenariosTargetKnownDefinitions = sliceContractScenarios.select(scenario => contractScenarioTargetsKnownDefinition(scenario)).length() == sliceContractScenarios.length()\n  val commandInputsHaveAllowedSources = sliceCommandDefinitions.select(command => command.inputs.select(input => allowedCommandInputSourceKinds.select(sourceKind => sourceKind == input.sourceKind).length() > 0).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()",
        )
        .replace(
            "val commandInputsHaveProvenance = sliceCommandDefinitions.select(command => command.inputs.select(input => input.name != \"\" and input.sourceKind != \"\" and input.sourceDescription != \"\" and input.provenanceChain.length() > 0).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()",
            "val commandInputsHaveProvenance = sliceCommandDefinitions.select(command => command.inputs.select(input => input.name != \"\" and input.sourceKind != \"\" and input.sourceDescription != \"\" and input.provenanceChain.length() > 0).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()\n  def commandInputEventStreamSourceResolves(command, input) = input.sourceKind != \"event_stream_state\" or (command.observedStreams.length() > 0 and command.observedStreams.select(streamName => scenarioStreamResolves(streamName)).length() == command.observedStreams.length())\n  val commandInputsSourcedFromEventStreamsResolve = sliceCommandDefinitions.select(command => command.inputs.select(input => commandInputEventStreamSourceResolves(command, input)).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()\n  def bitLevelFlowCoversTarget(target, datum) = sliceBitLevelDataFlows.select(flow => flow.target == target and flow.datum == datum and flow.source != \"\" and flow.transformationSemantics != \"\" and flow.bitEncoding != \"\").length() > 0\n  def commandInputHasBitLevelFlow(command, input) = bitLevelFlowCoversTarget(command.name, input.name)",
        )
        .replace(
            "val readModelFieldsHaveAllowedSources = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => allowedReadModelFieldSourceKinds.select(sourceKind => sourceKind == readModelField.sourceKind).length() > 0).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()",
            "def commandInputReferencesAttributeSource(event, attribute, command) = command.emittedEvents.select(eventName => eventName == event.name).length() > 0 and command.inputs.select(input => input.name == attribute.sourceName).length() > 0\n  def externalPayloadFieldHasProvenance(payloadField) = payloadField.name != \"\" and payloadField.provenanceDescription != \"\" and payloadField.bitEncoding != \"\"\n  val externalPayloadFieldsHaveProvenance = sliceExternalPayloads.select(payload => payload.name != \"\" and payload.fields.select(payloadField => externalPayloadFieldHasProvenance(payloadField)).length() == payload.fields.length()).length() == sliceExternalPayloads.length()\n  def externalPayloadFieldIsDeclared(attribute) = sliceExternalPayloads.select(payload => payload.name == attribute.sourceName and payload.fields.select(payloadField => payloadField.name == attribute.sourceField).length() > 0).length() > 0\n  def eventAttributeSourceIsComplete(event, attribute) = (attribute.sourceKind == \"command_input\" and attribute.sourceName != \"\" and attribute.sourceField != \"\" and sliceCommandDefinitions.select(command => commandInputReferencesAttributeSource(event, attribute, command)).length() > 0) or (attribute.sourceKind == \"external_payload\" and attribute.sourceName != \"\" and attribute.sourceField != \"\" and externalPayloadFieldIsDeclared(attribute)) or (attribute.sourceKind == \"generated\" and attribute.sourceName != \"\") or (attribute.sourceKind == \"session\" and attribute.sourceName != \"\") or (attribute.sourceKind == \"constant\" and attribute.sourceField != \"\") or (attribute.sourceKind == \"derivation\" and attribute.sourceName != \"\" and attribute.sourceField != \"\")\n  val eventAttributeSourcesAreComplete = sliceEventDefinitions.select(event => event.attributes.select(attribute => eventAttributeSourceIsComplete(event, attribute)).length() == event.attributes.length()).length() == sliceEventDefinitions.length()\n  def eventAttributeHasBitLevelFlow(event, attribute) = bitLevelFlowCoversTarget(event.name, attribute.name)\n  val readModelFieldsHaveAllowedSources = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => allowedReadModelFieldSourceKinds.select(sourceKind => sourceKind == readModelField.sourceKind).length() > 0).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()",
        )
        .replace(
            "val commandErrorsHaveAllowedRecovery = sliceCommandDefinitions.select(command => command.errors.select(error => allowedRecoveryKinds.select(recoveryKind => recoveryKind == error.recoveryKind).length() > 0).length() == command.errors.length()).length() == sliceCommandDefinitions.length()",
            "val commandErrorsHaveAllowedRecovery = sliceCommandDefinitions.select(command => command.errors.select(error => allowedRecoveryKinds.select(recoveryKind => recoveryKind == error.recoveryKind).length() > 0).length() == command.errors.length()).length() == sliceCommandDefinitions.length()\n  def scenarioNameIsModeled(scenarioName) = sliceAcceptanceScenarios.select(scenario => scenario.name == scenarioName).length() > 0 or sliceContractScenarios.select(scenario => scenario.name == scenarioName).length() > 0\n  def commandErrorHasScenarioCoverage(command, error) = sliceContractScenarios.select(scenario => scenario.name == error.scenarioName and scenario.contractKind == \"command\" and scenario.coveredDefinition == command.name and scenario.errorReferences.select(errorName => errorName == error.name).length() > 0).length() > 0\n  val commandErrorsHaveScenarioCoverage = sliceCommandDefinitions.select(command => command.errors.select(error => commandErrorHasScenarioCoverage(command, error)).length() == command.errors.length()).length() == sliceCommandDefinitions.length()\n  def scenarioErrorReferenceIsDeclared(scenario, errorName) = scenario.contractKind != \"command\" or sliceCommandDefinitions.select(command => command.name == scenario.coveredDefinition and command.errors.select(error => error.name == errorName).length() > 0).length() > 0\n  def scenarioErrorReferencesAreDeclaredForScenario(scenario) = scenario.errorReferences.select(errorName => scenarioErrorReferenceIsDeclared(scenario, errorName)).length() == scenario.errorReferences.length()\n  val scenarioErrorReferencesAreDeclared = sliceContractScenarios.select(scenario => scenarioErrorReferencesAreDeclaredForScenario(scenario)).length() == sliceContractScenarios.length()
  def singletonCommandDeclaresRepeatBehavior(command) = not(command.singleton) or allowedSingletonRepeatBehaviors.select(repeatBehavior => repeatBehavior == command.repeatBehavior).length() > 0
  val singletonCommandsDeclareRepeatBehavior = sliceCommandDefinitions.select(command => singletonCommandDeclaresRepeatBehavior(command)).length() == sliceCommandDefinitions.length()",
        )
        .replace(
            "def sameOutcomeEventSet(left, right) = left.eventSet.select(eventName => right.eventSet.select(otherEventName => otherEventName == eventName).length() > 0).length() == left.eventSet.length() and right.eventSet.select(eventName => left.eventSet.select(otherEventName => otherEventName == eventName).length() > 0).length() == right.eventSet.length()",
            "def automationHasTrigger(automation) = automation.name != \"\" and automation.triggerName != \"\" and automation.reactionDescription != \"\"\n  def automationIssuesKnownCommand(automation) = sliceCommands.select(command => command == automation.commandName).length() > 0 or sliceReferencedCommands.select(command => command == automation.commandName).length() > 0 or sliceCommandDefinitions.select(command => command.name == automation.commandName).length() > 0\n  def automationHandlesCommandErrors(automation, command) = command.name != automation.commandName or command.errors.select(error => automation.handledErrors.select(handledError => handledError == error.name).length() > 0).length() == command.errors.length()\n  val automationSlicesDeclareTriggers = sliceKind != \"automation\" or (sliceAutomations.length() > 0 and sliceAutomations.select(automation => automationHasTrigger(automation)).length() == sliceAutomations.length())\n  val automationsIssueKnownCommands = sliceAutomations.select(automation => automationIssuesKnownCommand(automation)).length() == sliceAutomations.length()\n  val automationsHandleCommandErrors = sliceAutomations.select(automation => sliceCommandDefinitions.select(command => automationHandlesCommandErrors(automation, command)).length() == sliceCommandDefinitions.length()).length() == sliceAutomations.length()\n  def translationHasExternalContract(translation) = translation.name != \"\" and translation.externalEventName != \"\" and translation.payloadContractName != \"\" and sliceExternalPayloads.select(payload => payload.name == translation.payloadContractName).length() > 0\n  def translationTargetsKnownCommand(translation) = sliceCommands.select(command => command == translation.commandName).length() > 0 or sliceReferencedCommands.select(command => command == translation.commandName).length() > 0 or sliceCommandDefinitions.select(command => command.name == translation.commandName).length() > 0\n  val translationSlicesDeclareExternalContracts = sliceKind != \"translation\" or (sliceTranslations.length() > 0 and sliceTranslations.select(translation => translationHasExternalContract(translation)).length() == sliceTranslations.length())\n  val translationsTargetKnownCommands = sliceTranslations.select(translation => translationTargetsKnownCommand(translation)).length() == sliceTranslations.length()\n  def boardElementLaneMatchesKind(element) = (element.kind == \"view\" and element.lane == \"ux\") or (element.kind == \"automation\" and element.lane == \"ux\") or (element.kind == \"external_event\" and element.lane == \"ux\") or (element.kind == \"command\" and element.lane == \"actions\") or (element.kind == \"read_model\" and element.lane == \"actions\") or (element.kind == \"event\" and element.lane == \"events\")\n  def boardElementReferencesDeclaration(element) = (element.kind == \"view\" and (sliceViews.select(viewName => viewName == element.declaredName).length() > 0 or sliceViewDefinitions.select(view => view.name == element.declaredName).length() > 0)) or (element.kind == \"automation\" and sliceAutomations.select(automation => automation.name == element.declaredName).length() > 0) or (element.kind == \"external_event\" and sliceEventDefinitions.select(event => event.name == element.declaredName and event.observed).length() > 0) or (element.kind == \"command\" and (sliceCommands.select(command => command == element.declaredName).length() > 0 or sliceReferencedCommands.select(command => command == element.declaredName).length() > 0 or sliceCommandDefinitions.select(command => command.name == element.declaredName).length() > 0)) or (element.kind == \"read_model\" and (sliceReadModels.select(readModel => readModel == element.declaredName).length() > 0 or sliceReadModelDefinitions.select(readModel => readModel.name == element.declaredName).length() > 0)) or (element.kind == \"event\" and (sliceEvents.select(eventName => eventName == element.declaredName).length() > 0 or sliceEventDefinitions.select(event => event.name == element.declaredName and (event.observed or event.shared)).length() > 0))\n  def boardConnectionHasAllowedShape(connection) = (connection.sourceKind == \"view\" and connection.targetKind == \"command\") or (connection.sourceKind == \"automation\" and connection.targetKind == \"command\") or (connection.sourceKind == \"external_event\" and connection.targetKind == \"command\") or (connection.sourceKind == \"workflow_trigger\" and connection.targetKind == \"command\") or (connection.sourceKind == \"command\" and connection.targetKind == \"event\") or (connection.sourceKind == \"event\" and connection.targetKind == \"read_model\") or (connection.sourceKind == \"read_model\" and connection.targetKind == \"view\")\n  def commandEventBoardEdgeMatchesEmission(connection) = connection.sourceKind != \"command\" or connection.targetKind != \"event\" or sliceCommandDefinitions.select(command => command.name == connection.source and command.emittedEvents.select(eventName => eventName == connection.target).length() > 0).length() > 0\n  def eventReadModelBoardEdgeMatchesProjection(connection) = connection.sourceKind != \"event\" or connection.targetKind != \"read_model\" or sliceReadModelDefinitions.select(readModel => readModel.name == connection.target and readModel.fields.select(readModelField => readModelField.sourceEvent == connection.source).length() > 0).length() > 0\n  def externalEventDoesNotUpdateReadModel(connection) = connection.sourceKind != \"event\" or connection.targetKind != \"read_model\" or sliceEventDefinitions.select(event => event.name == connection.source and event.observed).length() == 0\n  val externalEventsDoNotUpdateReadModels = sliceBoardConnections.select(connection => externalEventDoesNotUpdateReadModel(connection)).length() == sliceBoardConnections.length()\n  def viewCommandBoardEdgeMatchesControl(connection) = connection.sourceKind != \"view\" or connection.targetKind != \"command\" or sliceViewDefinitions.select(view => view.name == connection.source and view.controls.select(control => control.commandName == connection.target).length() > 0).length() > 0\n  val boardLanesAreCanonical = canonicalBoardLanes == [\"ux\",\"actions\",\"events\"]\n  val boardElementsUseCanonicalLanes = sliceBoardElements.select(element => canonicalBoardLanes.select(lane => lane == element.lane).length() > 0 and boardElementLaneMatchesKind(element)).length() == sliceBoardElements.length()\n  val boardElementsReferenceDeclarations = sliceBoardElements.select(element => boardElementReferencesDeclaration(element)).length() == sliceBoardElements.length()\n  val boardConnectionsHaveCausalSemantics = sliceBoardConnections.select(connection => boardConnectionHasAllowedShape(connection) and commandEventBoardEdgeMatchesEmission(connection) and eventReadModelBoardEdgeMatchesProjection(connection) and externalEventDoesNotUpdateReadModel(connection) and viewCommandBoardEdgeMatchesControl(connection)).length() == sliceBoardConnections.length()\n  val readModelsDoNotFeedCommands = sliceBoardConnections.select(connection => connection.sourceKind != \"read_model\" or connection.targetKind != \"command\").length() == sliceBoardConnections.length()\n  def readModelViewConnectionHasIncomingEventUpdate(connection) = connection.sourceKind != \"read_model\" or connection.targetKind != \"view\" or sliceBoardConnections.select(incoming => incoming.target == connection.source and incoming.targetKind == \"read_model\" and incoming.sourceKind == \"event\").length() > 0\n  val readModelsFeedingViewsHaveIncomingEventUpdates = sliceBoardConnections.select(connection => readModelViewConnectionHasIncomingEventUpdate(connection)).length() == sliceBoardConnections.length()\n  val commandsHaveIncomingTriggers = sliceBoardElements.select(element => element.kind != \"command\" or sliceBoardConnections.select(connection => connection.target == element.name and connection.targetKind == \"command\" and (connection.sourceKind == \"view\" or connection.sourceKind == \"automation\" or connection.sourceKind == \"external_event\" or connection.sourceKind == \"workflow_trigger\")).length() > 0).length() == sliceBoardElements.length()\n  val mainPathBoardHasNoDisconnectedIslands = sliceBoardElements.select(element => not(element.mainPath) or sliceBoardConnections.select(connection => connection.source == element.name or connection.target == element.name).length() > 0).length() == sliceBoardElements.length()\n  def sameOutcomeEventSet(left, right) = left.eventSet.select(eventName => right.eventSet.select(otherEventName => otherEventName == eventName).length() > 0).length() == left.eventSet.length() and right.eventSet.select(eventName => left.eventSet.select(otherEventName => otherEventName == eventName).length() > 0).length() == right.eventSet.length()",
        )
        .replace(
            "val sliceIdentityStable",
            "val stateViewSlicesDoNotOwnCommands = sliceKind != \"state_view\" or (sliceCommands.length() == 0 and sliceCommandDefinitions.length() == 0)\n  val stateViewSlicesOwnViews = sliceKind != \"state_view\" or (sliceViews.length() > 0 or sliceViewDefinitions.length() > 0)\n  val stateViewSlicesOwnReadModels = sliceKind != \"state_view\" or (sliceReadModels.length() > 0 or sliceReadModelDefinitions.length() > 0)\n  val stateChangeSlicesOwnCommands = sliceKind != \"state_change\" or (sliceCommands.length() > 0 or sliceCommandDefinitions.length() > 0)\n  val stateChangeSlicesDoNotOwnReadModelsOrViews = sliceKind != \"state_change\" or (sliceReadModels.length() == 0 and sliceReadModelDefinitions.length() == 0 and sliceViews.length() == 0 and sliceViewDefinitions.length() == 0)\n  val stateChangeSlicesDoNotOwnAutomationsOrTranslations = sliceKind != \"state_change\" or (sliceAutomations.length() == 0 and sliceTranslations.length() == 0)\n  val translationSlicesDoNotOwnViews = sliceKind != \"translation\" or (sliceViews.length() == 0 and sliceViewDefinitions.length() == 0)\n  def navigationTargetTypeIsModeled(target) = target.targetType == \"\" or allowedNavigationTargetTypes.select(targetType => targetType == target.targetType).length() > 0\n  def navigationTargetIsComplete(view, target) = (target.targetType == \"\" and target.targetName == \"\" and target.externalWorkflowName == \"\" and target.externalSystemName == \"\" and target.handoffContract == \"\") or (target.targetType == \"modeled_view\" and target.targetName != \"\" and sliceViews.select(viewName => viewName == target.targetName).length() > 0) or (target.targetType == \"local_view_state\" and target.targetName != \"\" and view.localStates.select(localState => localState == target.targetName).length() > 0) or (target.targetType == \"external_workflow\" and target.externalWorkflowName != \"\") or (target.targetType == \"external_system\" and target.externalSystemName != \"\" and target.handoffContract != \"\")\n  val viewControlNavigationTypesAreModeled = sliceViewDefinitions.select(view => view.controls.select(control => navigationTargetTypeIsModeled(control.navigation)).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val viewControlNavigationTargetsAreComplete = sliceViewDefinitions.select(view => view.controls.select(control => navigationTargetIsComplete(view, control.navigation)).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val sliceIdentityStable",
        )
        .replace(
            "val viewControlInputsHaveAllowedSources = sliceViewDefinitions.select(view => view.controls.select(control => control.inputs.select(input => allowedControlInputSourceKinds.select(sourceKind => sourceKind == input.sourceKind).length() > 0).length() == control.inputs.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()",
            "def controlAppearsInSketch(view, control) = control.sketchToken != \"\" and view.sketchTokens.select(sketchToken => sketchToken == control.sketchToken).length() > 0\n  val viewControlsAppearInSketch = sliceViewDefinitions.select(view => view.controls.select(control => controlAppearsInSketch(view, control)).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  def controlProvidesCommandInput(control, input) = control.inputs.select(providedInput => providedInput.name == input.name).length() > 0\n  def controlProvidesEveryCommandInput(control, command) = command.name != control.commandName or command.inputs.select(input => controlProvidesCommandInput(control, input)).length() == command.inputs.length()\n  val viewControlsProvideCommandInputs = sliceViewDefinitions.select(view => view.controls.select(control => sliceCommandDefinitions.select(command => controlProvidesEveryCommandInput(control, command)).length() == sliceCommandDefinitions.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  val viewControlInputsHaveAllowedSources = sliceViewDefinitions.select(view => view.controls.select(control => control.inputs.select(input => allowedControlInputSourceKinds.select(sourceKind => sourceKind == input.sourceKind).length() > 0).length() == control.inputs.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()",
        );
    let contents = contents.replace(
        "val viewControlsHandleCommandErrors = sliceViewDefinitions.select(view => view.controls.select(control => sliceCommandDefinitions.select(command => command.name != control.commandName or command.errors.select(error => control.handledErrors.select(handledError => handledError == error.name).length() > 0 and control.recoveryBehavior != \"\").length() == command.errors.length()).length() == sliceCommandDefinitions.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()",
        "val viewControlsHandleCommandErrors = sliceViewDefinitions.select(view => view.controls.select(control => sliceCommandDefinitions.select(command => command.name != control.commandName or command.errors.select(error => control.handledErrors.select(handledError => handledError == error.name).length() > 0 and control.recoveryBehavior != \"\").length() == command.errors.length()).length() == sliceCommandDefinitions.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()\n  def controlRecoveryBehaviorIsModeled(control) = control.handledErrors.length() == 0 or allowedRecoveryKinds.select(recoveryKind => recoveryKind == control.recoveryBehavior).length() > 0\n  val viewControlRecoveryBehaviorIsModeled = sliceViewDefinitions.select(view => view.controls.select(control => controlRecoveryBehaviorIsModeled(control)).length() == view.controls.length()).length() == sliceViewDefinitions.length()",
    );
    let contents = contents.replace(
        "val viewFieldsHaveAllowedSources = sliceViewDefinitions.select(view => view.fields.select(viewField => allowedViewFieldSourceKinds.select(sourceKind => sourceKind == viewField.sourceKind).length() > 0).length() == view.fields.length()).length() == sliceViewDefinitions.length()",
        "def eventAttributeIsDeclared(eventName, attributeName) = sliceEventDefinitions.select(event => event.name == eventName and event.attributes.select(attribute => attribute.name == attributeName).length() > 0).length() > 0\n  def readModelFieldEventAttributeSourceResolves(readModelField) = readModelField.sourceKind != \"event_attribute\" or eventAttributeIsDeclared(readModelField.sourceEvent, readModelField.sourceAttribute)\n  val readModelFieldEventAttributeSourcesResolve = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldEventAttributeSourceResolves(readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()\n  def readModelFieldDerivationScenarioIsCovered(readModelField) = readModelField.sourceKind != \"derivation\" or (readModelField.derivationScenarioName != \"\" and scenarioNameIsModeled(readModelField.derivationScenarioName))\n  def readModelFieldAbsenceScenarioIsCovered(readModelField) = readModelField.sourceKind != \"absence_default\" or (readModelField.absenceScenarioName != \"\" and scenarioNameIsModeled(readModelField.absenceScenarioName))\n  val derivedReadModelFieldsHaveScenarioCoverage = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldDerivationScenarioIsCovered(readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()\n  val absenceReadModelFieldsHaveScenarioCoverage = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldAbsenceScenarioIsCovered(readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()\n  def transitiveReadModelHasSemantics(readModel) = not(readModel.transitive) or (readModel.relationshipFields.length() > 0 and readModel.transitiveRule != \"\" and readModel.exampleScenarioName != \"\" and scenarioNameIsModeled(readModel.exampleScenarioName))\n  val transitiveReadModelsHaveSemantics = sliceReadModelDefinitions.select(readModel => transitiveReadModelHasSemantics(readModel)).length() == sliceReadModelDefinitions.length()\n  def readModelFieldHasBitLevelFlow(readModel, readModelField) = bitLevelFlowCoversTarget(readModel.name, readModelField.name)\n  val viewFieldsHaveAllowedSources = sliceViewDefinitions.select(view => view.fields.select(viewField => allowedViewFieldSourceKinds.select(sourceKind => sourceKind == viewField.sourceKind).length() > 0).length() == view.fields.length()).length() == sliceViewDefinitions.length()",
    );
    let contents = contents.replace(
        "val viewFieldsSourceFromUsedReadModels = sliceViewDefinitions.select(view => view.fields.select(viewField => view.readModels.select(readModel => readModel == viewField.sourceReadModel).length() > 0 and sliceReadModels.select(readModel => readModel == viewField.sourceReadModel).length() > 0).length() == view.fields.length()).length() == sliceViewDefinitions.length()",
        "val viewFieldsSourceFromUsedReadModels = sliceViewDefinitions.select(view => view.fields.select(viewField => view.readModels.select(readModel => readModel == viewField.sourceReadModel).length() > 0 and sliceReadModels.select(readModel => readModel == viewField.sourceReadModel).length() > 0).length() == view.fields.length()).length() == sliceViewDefinitions.length()\n  def viewFieldAppearsInSketch(view, viewField) = viewField.sketchToken != \"\" and view.sketchTokens.select(sketchToken => sketchToken == viewField.sketchToken).length() > 0\n  def viewHasInformationSketch(view) = view.sketchTokens.length() > 0\n  val viewsHaveInformationSketches = sliceViewDefinitions.select(view => viewHasInformationSketch(view)).length() == sliceViewDefinitions.length()\n  val viewFieldsAppearInSketch = sliceViewDefinitions.select(view => view.fields.select(viewField => viewFieldAppearsInSketch(view, viewField)).length() == view.fields.length()).length() == sliceViewDefinitions.length()\n  def sketchTokenMapsToModeledElement(view, token) = view.fields.select(viewField => viewField.sketchToken == token).length() > 0 or view.controls.select(control => control.sketchToken == token or control.inputs.select(input => input.sourceKind == \"actor\" and input.sketchToken == token).length() > 0).length() > 0\n  val viewSketchTokensMapToModeledElements = sliceViewDefinitions.select(view => view.sketchTokens.select(sketchToken => sketchTokenMapsToModeledElement(view, sketchToken)).length() == view.sketchTokens.length()).length() == sliceViewDefinitions.length()\n  def readModelFieldIsDeclared(readModelName, fieldName) = sliceReadModelDefinitions.select(readModel => readModel.name == readModelName and readModel.fields.select(readModelField => readModelField.name == fieldName).length() > 0).length() > 0\n  def viewFieldSourceReadModelFieldResolves(viewField) = viewField.sourceKind != \"read_model\" or readModelFieldIsDeclared(viewField.sourceReadModel, viewField.sourceField)\n  val viewFieldReadModelFieldSourcesResolve = sliceViewDefinitions.select(view => view.fields.select(viewField => viewFieldSourceReadModelFieldResolves(viewField)).length() == view.fields.length()).length() == sliceViewDefinitions.length()\n  def viewFieldHasBitLevelFlow(view, viewField) = bitLevelFlowCoversTarget(view.name, viewField.name)\n  val commandInputDataFlowsAreComplete = sliceCommandDefinitions.select(command => command.inputs.select(input => commandInputHasBitLevelFlow(command, input)).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()\n  val eventAttributeDataFlowsAreComplete = sliceEventDefinitions.select(event => event.attributes.select(attribute => eventAttributeHasBitLevelFlow(event, attribute)).length() == event.attributes.length()).length() == sliceEventDefinitions.length()\n  val readModelFieldDataFlowsAreComplete = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldHasBitLevelFlow(readModel, readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()\n  val viewFieldDataFlowsAreComplete = sliceViewDefinitions.select(view => view.fields.select(viewField => viewFieldHasBitLevelFlow(view, viewField)).length() == view.fields.length()).length() == sliceViewDefinitions.length()\n  val modeledDataFlowsAreBitComplete = commandInputDataFlowsAreComplete and eventAttributeDataFlowsAreComplete and readModelFieldDataFlowsAreComplete and viewFieldDataFlowsAreComplete",
    );
    file_contents(contents)
}

fn file_contents(value: impl Into<String>) -> FileContents {
    FileContents::try_new(value.into()).unwrap_or_else(|error| {
        unreachable!("EMC generated Quint file contents must be valid: {error}");
    })
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        unreachable!("EMC generated Quint string literal must be valid: {error}");
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
                    "{{ slug: {}, name: {}, kind: {}, description: {} }}",
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

fn workflow_step_relationship_list(workflow_slice_details: &[WorkflowSliceDetail]) -> String {
    format!(
        "[{}]",
        workflow_slice_details
            .iter()
            .map(|slice| {
                format!(
                    "{{ step: {}, relationship: {} }}",
                    quoted(slice.slug().as_ref()),
                    quoted(slice.relationship().as_ref())
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

fn workflow_exit_target_list(workflow_transitions: &[WorkflowTransitionRecord]) -> String {
    format!(
        "[{}]",
        workflow_transitions
            .iter()
            .filter(|transition| transition.kind().as_ref().starts_with("workflow_exit:"))
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
        "{{ sourceSlice: {}, label: {}, externallyRelevant: {} }}",
        quoted(outcome.source_slice().as_ref()),
        quoted(outcome.label().as_ref()),
        outcome.externally_relevant(),
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
        "{{ sourceSlice: {}, commandName: {}, errorName: {} }}",
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
        "{{ sourceSlice: {}, definitionKind: {}, definitionName: {} }}",
        quoted(definition.source_slice().as_ref()),
        quoted(definition.definition_kind().as_ref()),
        quoted(definition.definition_name().as_ref()),
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
        "{{ source: {}, target: {}, kind: {}, trigger: {}, sourceEvidence: {}, targetEvidence: {} }}",
        quoted(evidence.source().as_ref()),
        quoted(evidence.target().as_ref()),
        quoted(evidence.kind().as_ref()),
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
        "{{ state: {}, step: {}, evidence: {} }}",
        quoted(coverage.state().as_ref()),
        quoted(coverage.step().as_ref()),
        quoted(coverage.evidence().as_ref()),
    )
}

fn transition_record(transition: &WorkflowTransitionRecord) -> String {
    format!(
        "{{ source: {}, target: {}, kind: {}, trigger: {}, rationale: {}, payloadContract: {} }}",
        quoted(transition.source().as_ref()),
        quoted(transition.target().as_ref()),
        quoted(transition.kind().as_ref()),
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
