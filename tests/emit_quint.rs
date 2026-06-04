#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::digest::{WorkflowArtifactDigestInput, artifact_digest, slice_artifact_digest};
    use emc::core::emit::quint::{emit_slice_module, emit_workflow_module};
    use emc::core::types::{
        CommandErrorName, CommandName, OutcomeLabelName, PayloadContractName, SliceKindName,
        TransitionTriggerName, WorkflowCommandErrorRecord, WorkflowCommandErrorRecords,
        WorkflowModuleData, WorkflowOutcomeRecord, WorkflowOutcomeRecords,
        WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord,
        WorkflowOwnedDefinitionRecords, WorkflowSliceDetail, WorkflowSliceDetails,
        WorkflowStepRelationshipName, WorkflowTransitionEndpoint, WorkflowTransitionKind,
        WorkflowTransitionRecord, WorkflowTransitionRecords,
    };
    use emc::io::dto::{
        parse_model_description, parse_model_name, parse_quint_module_name, parse_slice_slug,
        parse_workflow_slug,
    };

    #[test]
    fn quint_workflow_module_represents_business_workflow_fields() -> Result<(), Box<dyn Error>> {
        let workflow_name = parse_model_name("Open ticket")?;
        let workflow_description = parse_model_description("Actor opens a repair ticket.")?;
        let workflow_slug = parse_workflow_slug("open-ticket")?;
        let workflow_slice_details = vec![
            WorkflowSliceDetail::new_with_relationship(
                parse_slice_slug("capture-ticket")?,
                parse_model_name("Capture ticket")?,
                SliceKindName::try_new("state_view".to_owned())?,
                parse_model_description("Actor enters repair ticket details.")?,
                WorkflowStepRelationshipName::try_new("entry".to_owned())?,
            ),
            WorkflowSliceDetail::new_with_relationship(
                parse_slice_slug("review-ticket")?,
                parse_model_name("Review ticket")?,
                SliceKindName::try_new("state_view".to_owned())?,
                parse_model_description("Actor reviews repair ticket details.")?,
                WorkflowStepRelationshipName::try_new("main".to_owned())?,
            ),
        ];
        let workflow_transitions = vec![WorkflowTransitionRecord::new_with_payload_contract(
            WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
            WorkflowTransitionEndpoint::try_new("review-ticket".to_owned())?,
            WorkflowTransitionKind::try_new("external_trigger".to_owned())?,
            TransitionTriggerName::try_new("callback_received".to_owned())?,
            PayloadContractName::try_new("CallbackReceivedPayload".to_owned())?,
        )];
        let workflow_outcomes = WorkflowOutcomeRecords::from_records([WorkflowOutcomeRecord::new(
            WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
            OutcomeLabelName::try_new("ticket_captured".to_owned())?,
            true,
        )]);
        let workflow_command_errors =
            WorkflowCommandErrorRecords::from_records([WorkflowCommandErrorRecord::new(
                WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
                CommandName::try_new("CaptureTicket".to_owned())?,
                CommandErrorName::try_new("DuplicateTicket".to_owned())?,
            )]);
        let workflow_owned_definitions =
            WorkflowOwnedDefinitionRecords::from_records([WorkflowOwnedDefinitionRecord::new(
                WorkflowTransitionEndpoint::try_new("capture-ticket".to_owned())?,
                WorkflowOwnedDefinitionKind::try_new("external_payload".to_owned())?,
                WorkflowOwnedDefinitionName::try_new("CallbackReceivedPayload".to_owned())?,
            )]);
        let module = emit_workflow_module(
            parse_quint_module_name("OpenTicket")?,
            WorkflowModuleData::new(
                workflow_name.clone(),
                workflow_description.clone(),
                workflow_slug.clone(),
                artifact_digest(WorkflowArtifactDigestInput {
                    workflow_name,
                    workflow_slug,
                    workflow_description,
                    workflow_slice_details: WorkflowSliceDetails::from_details(
                        workflow_slice_details.clone(),
                    ),
                    workflow_transitions: WorkflowTransitionRecords::from_records(
                        workflow_transitions.clone(),
                    ),
                    workflow_outcomes: workflow_outcomes.clone(),
                    workflow_command_errors: workflow_command_errors.clone(),
                    workflow_owned_definitions: workflow_owned_definitions.clone(),
                    workflow_transition_evidences: Default::default(),
                    workflow_requires_entry_lifecycle_coverage: false,
                    workflow_entry_lifecycle_states: Default::default(),
                }),
            )
            .with_slice_details(WorkflowSliceDetails::from_details(workflow_slice_details))
            .with_transitions(WorkflowTransitionRecords::from_records(
                workflow_transitions,
            ))
            .with_outcomes(workflow_outcomes)
            .with_command_errors(workflow_command_errors)
            .with_owned_definitions(workflow_owned_definitions),
        );
        let quint = module.as_ref();

        assert!(quint.contains("module OpenTicket"));
        assert!(
            quint.contains(
                "// EMC-DIGEST: workflow:name=Open ticket;slug=open-ticket;description=Actor opens a repair ticket.;slices=capture-ticket|Capture ticket|state_view|Actor enters repair ticket details.|entry,review-ticket|Review ticket|state_view|Actor reviews repair ticket details.|main;transitions=capture-ticket->review-ticket:external_trigger:callback_received::CallbackReceivedPayload"
            )
        );
        assert!(quint.contains("val workflowName = \"Open ticket\""));
        assert!(quint.contains("val workflowSlug = \"open-ticket\""));
        assert!(quint.contains("val workflowDescription = \"Actor opens a repair ticket.\""));
        assert!(quint.contains(
            "type WorkflowSliceDetail = { slug: str, name: str, kind: str, description: str }"
        ));
        assert!(quint.contains(
            "type WorkflowTransition = { source: str, target: str, kind: str, trigger: str, rationale: str, payloadContract: str }"
        ));
        assert!(quint.contains(
            "type WorkflowOutcome = { sourceSlice: str, label: str, externallyRelevant: bool }"
        ));
        assert!(quint.contains(
            "type WorkflowCommandError = { sourceSlice: str, commandName: str, errorName: str }"
        ));
        assert!(quint.contains(
            "type WorkflowOwnedDefinition = { sourceSlice: str, definitionKind: str, definitionName: str, definitionStream: str, sourceProvenance: str }"
        ));
        assert!(quint.contains(
            "type WorkflowEntryLifecycleState = { state: str, step: str, evidence: str }"
        ));
        assert!(
            quint
                .contains("val workflowSlices: List[str] = [\"capture-ticket\",\"review-ticket\"]")
        );
        assert!(
            quint.contains(
                "val workflowSliceDetails: List[WorkflowSliceDetail] = [{ slug: \"capture-ticket\", name: \"Capture ticket\", kind: \"state_view\", description: \"Actor enters repair ticket details.\" },{ slug: \"review-ticket\", name: \"Review ticket\", kind: \"state_view\", description: \"Actor reviews repair ticket details.\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"external_trigger\", trigger: \"callback_received\", rationale: \"\", payloadContract: \"CallbackReceivedPayload\" }]"
            )
        );
        assert!(quint.contains(
            "val workflowOutcomes: List[WorkflowOutcome] = [{ sourceSlice: \"capture-ticket\", label: \"ticket_captured\", externallyRelevant: true }]"
        ));
        assert!(quint.contains(
            "val workflowCommandErrors: List[WorkflowCommandError] = [{ sourceSlice: \"capture-ticket\", commandName: \"CaptureTicket\", errorName: \"DuplicateTicket\" }]"
        ));
        assert!(quint.contains("val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: \"external_payload\", definitionName: \"CallbackReceivedPayload\", definitionStream: \"\", sourceProvenance: \"\" }]"));
        assert!(quint.contains("val workflowRequiresEntryLifecycleCoverage = false"));
        assert!(
            quint.contains(
                "val workflowEntryLifecycleStates: List[WorkflowEntryLifecycleState] = []"
            )
        );
        assert!(quint.contains("val workflowIdentityStable"));
        assert!(quint.contains("val workflowSlicesHaveDetails ="));
        assert!(quint.contains("val workflowSliceDetailsComplete = workflowSlicesHaveDetails"));
        assert!(quint.contains(
            "val workflowTransitionsStructured = workflowTransitions.select(transition => transition.source != \"\" and transition.target != \"\" and transition.kind != \"\" and transition.trigger != \"\").length() == workflowTransitions.length()"
        ));
        assert!(
            quint.contains("val workflowExitTargets: List[str] = []"),
            "Quint workflow artifacts must explicitly model workflow-exit targets for transition target resolution"
        );
        assert!(
            quint.contains("val requiredEntryLifecycleStates: List[str] = [\"fresh_uninitialized\",\"initialized_unauthenticated\",\"initialized_authenticated\",\"partially_configured\",\"fully_configured\"]"),
            "Quint workflow artifacts must encode the required application-entry lifecycle states"
        );
        assert!(
            quint.contains("val allowedWorkflowStepRelationships: List[str] = [\"entry\",\"main\",\"branch\",\"alternate\",\"async_lifecycle\",\"supporting\"]"),
            "Quint workflow artifacts must model the allowed workflow step relationship inventory"
        );
        assert!(
            quint.contains("type WorkflowStepRelationship = { step: str, relationship: str }"),
            "Quint workflow artifacts must represent workflow step relationships separately from rendering concerns"
        );
        assert!(
            quint.contains("val workflowStepRelationships: List[WorkflowStepRelationship] ="),
            "Quint workflow artifacts must emit workflow step relationships"
        );
        assert!(
            quint.contains(
                "val workflowStepRelationships: List[WorkflowStepRelationship] = [{ step: \"capture-ticket\", relationship: \"entry\" },{ step: \"review-ticket\", relationship: \"main\" }]"
            ),
            "Quint workflow artifacts must emit concrete workflow step relationships"
        );
        assert!(
            quint.contains("val workflowStepRelationshipsAreAllowed = workflowStepRelationships.select(step => workflowStepRelationshipIsAllowed(step)).length() == workflowStepRelationships.length()"),
            "Quint workflow artifacts must expose allowed step relationships as an invariant"
        );
        assert!(
            quint.contains("val workflowStepSlugsAreUnique = workflowSlices.select(step => workflowSlices.select(other => other == step).length() == 1).length() == workflowSlices.length()"),
            "Quint workflow artifacts must verify workflow step slugs are unique"
        );
        assert!(
            quint.contains("val workflowHasExactlyOneEntryStep = workflowStepRelationships.select(step => step.relationship == \"entry\").length() == 1"),
            "Quint workflow artifacts must verify the composition has exactly one entry step"
        );
        assert!(
            quint.contains("val workflowMainStepsHaveIncomingReachability = workflowStepRelationships.select(step => workflowMainStepHasIncomingTransition(step)).length() == workflowStepRelationships.length()"),
            "Quint workflow artifacts must verify main workflow steps have incoming reachability"
        );
        assert!(
            quint.contains("def workflowReachableStepsAfterFuel(fuel, reachable) ="),
            "Quint workflow artifacts must compute workflow reachability from the entry step"
        );
        assert!(
            quint.contains("val workflowReachableStepsFromEntry = workflowReachableStepsAfterFuel(2, workflowEntrySteps)"),
            "Quint workflow artifacts must bound reachability traversal by the emitted workflow size"
        );
        assert!(
            quint.contains("def workflowStepIsReachableFromEntry(step) = step.relationship == \"supporting\" or workflowReachableStepsFromEntry.select(reachableStep => reachableStep == step.step).length() > 0"),
            "Quint workflow artifacts must exempt only supporting steps from required entry reachability"
        );
        assert!(
            quint.contains("val workflowNonSupportingStepsReachableFromEntry = workflowStepRelationships.select(step => workflowStepIsReachableFromEntry(step)).length() == workflowStepRelationships.length()"),
            "Quint workflow artifacts must expose non-supporting workflow reachability as an invariant"
        );
        assert!(
            quint.contains("def workflowBranchOrAlternateStepHasTriggerOrRationale(step) = (step.relationship != \"branch\" and step.relationship != \"alternate\") or workflowTransitions.select(transition => transition.target == step.step and (transition.trigger != \"\" or transition.rationale != \"\")).length() > 0"),
            "Quint workflow artifacts must define branch and alternate step trigger/rationale obligations"
        );
        assert!(
            quint.contains("val workflowBranchAndAlternateStepsHaveTriggerOrRationale = workflowStepRelationships.select(step => workflowBranchOrAlternateStepHasTriggerOrRationale(step)).length() == workflowStepRelationships.length()"),
            "Quint workflow artifacts must verify branch and alternate steps explain why they are reached"
        );
        assert!(
            quint.contains("val workflowEntryLifecycleStatesCoverRequiredStates = not(workflowRequiresEntryLifecycleCoverage) or requiredEntryLifecycleStates.select(state => workflowEntryLifecycleStateCovered(state)).length() == requiredEntryLifecycleStates.length()"),
            "Quint workflow artifacts must expose application-entry lifecycle coverage as an invariant"
        );
        assert!(
            quint.contains(
                "val workflowTransitionSourcesResolve = workflowTransitions.select(transition => workflowSlices.select(step => step == transition.source).length() > 0).length() == workflowTransitions.length()"
            ),
            "Quint workflow artifacts must verify transition sources are composed workflow steps"
        );
        assert!(
            quint.contains(
                "val workflowTransitionTargetsResolve = workflowTransitions.select(transition => workflowSlices.select(step => step == transition.target).length() > 0 or workflowExitTargets.select(exitTarget => exitTarget == transition.target).length() > 0).length() == workflowTransitions.length()"
            ),
            "Quint workflow artifacts must verify transition targets are composed steps or explicit workflow exits"
        );
        assert!(
            quint.contains(
                "def workflowTransitionKindIsModeled(transition) = transition.kind == \"navigation\" or transition.kind == \"command\" or transition.kind == \"event\" or transition.kind == \"external_trigger\" or transition.kind == \"outcome\" or workflowExitTargets.select(exitTarget => exitTarget == transition.target).length() > 0"
            ),
            "Quint workflow artifacts must model the legal workflow transition kinds, including explicit workflow exits"
        );
        assert!(
            quint.contains(
                "def workflowTransitionExitHasRationale(transition) = workflowExitTargets.select(exitTarget => exitTarget == transition.target).length() == 0 or transition.rationale != \"\""
            ),
            "Quint workflow artifacts must require workflow exits to explain why the exit is reached"
        );
        assert!(
            quint.contains(
                "val workflowTransitionsHaveModeledKinds = workflowTransitions.select(transition => workflowTransitionKindIsModeled(transition)).length() == workflowTransitions.length()"
            ),
            "Quint workflow artifacts must expose transition-kind legality as an invariant"
        );
        assert!(
            quint.contains(
                "val workflowExitsNameTargetsAndRationale = workflowTransitions.select(transition => workflowTransitionExitHasRationale(transition)).length() == workflowTransitions.length()"
            ),
            "Quint workflow artifacts must expose workflow-exit rationale as an invariant"
        );
        assert!(quint.contains(
            "def workflowOutcomeHandledByTransition(outcome) = not(outcome.externallyRelevant) or workflowTransitions.select(transition => transition.source == outcome.sourceSlice and transition.kind == \"outcome\" and transition.trigger == outcome.label).length() > 0"
        ));
        assert!(quint.contains(
            "val workflowExternallyRelevantOutcomesHandled = workflowOutcomes.select(outcome => workflowOutcomeHandledByTransition(outcome)).length() == workflowOutcomes.length()"
        ));
        assert!(quint.contains(
            "def workflowOutcomeSourceResolves(outcome) = workflowSlices.select(step => step == outcome.sourceSlice).length() > 0"
        ));
        assert!(quint.contains(
            "val workflowOutcomesSourceResolve = workflowOutcomes.select(outcome => workflowOutcomeSourceResolves(outcome)).length() == workflowOutcomes.length()"
        ));
        assert!(quint.contains(
            "def workflowCommandErrorSourceResolves(error) = workflowSlices.select(step => step == error.sourceSlice).length() > 0"
        ));
        assert!(quint.contains(
            "val workflowCommandErrorsSourceResolve = workflowCommandErrors.select(error => workflowCommandErrorSourceResolves(error)).length() == workflowCommandErrors.length()"
        ));
        assert!(quint.contains(
            "val workflowTransitionsDoNotUseCommandErrorsAsOutcomes = workflowTransitions.select(transition => transition.kind != \"outcome\" or workflowCommandErrors.select(error => error.sourceSlice == transition.source and error.errorName == transition.trigger).length() == 0).length() == workflowTransitions.length()"
        ));
        assert!(quint.contains(
            "def workflowNonEventDefinitionOwnedOnce(definition) = definition.definitionKind == \"event\" or workflowOwnedDefinitions.select(other => other.definitionKind == definition.definitionKind and other.definitionName == definition.definitionName).length() == 1"
        ));
        assert!(quint.contains(
            "val workflowNonEventDefinitionsAreUniquelyOwned = workflowOwnedDefinitions.select(definition => workflowNonEventDefinitionOwnedOnce(definition)).length() == workflowOwnedDefinitions.length()"
        ));
        assert!(quint.contains(
            "def workflowEventDefinitionHasIdentity(definition) = definition.definitionKind != \"event\" or (definition.definitionStream != \"\" and definition.sourceProvenance != \"\")"
        ));
        assert!(quint.contains(
            "def workflowSharedEventDefinitionMatches(left, right) = left.definitionKind != \"event\" or right.definitionKind != \"event\" or left.definitionName != right.definitionName or (left.definitionStream == right.definitionStream and left.sourceProvenance == right.sourceProvenance)"
        ));
        assert!(quint.contains(
            "val workflowSharedEventDefinitionsHaveIdenticalIdentity = workflowOwnedDefinitions.select(definition => workflowEventDefinitionHasIdentity(definition)).length() == workflowOwnedDefinitions.length() and workflowOwnedDefinitions.select(definition => workflowOwnedDefinitions.select(other => workflowSharedEventDefinitionMatches(definition, other)).length() == workflowOwnedDefinitions.length()).length() == workflowOwnedDefinitions.length()"
        ));
        assert!(quint.contains(
            "def workflowOwnsDefinition(sourceSlice, definitionKind, definitionName) = workflowOwnedDefinitions.select(definition => definition.sourceSlice == sourceSlice and definition.definitionKind == definitionKind and definition.definitionName == definitionName).length() > 0"
        ));
        assert!(quint.contains(
            "def workflowCommandTransitionTargetsOwnedCommand(transition) = transition.kind != \"command\" or workflowOwnsDefinition(transition.target, \"command\", transition.trigger)"
        ));
        assert!(quint.contains(
            "def workflowCommandTransitionSourceOwnsControl(transition) = transition.kind != \"command\" or workflowOwnsDefinition(transition.source, \"control\", transition.trigger)"
        ));
        assert!(quint.contains(
            "val workflowCommandTransitionsResolveControlsAndCommands = workflowTransitions.select(transition => workflowCommandTransitionSourceOwnsControl(transition) and workflowCommandTransitionTargetsOwnedCommand(transition)).length() == workflowTransitions.length()"
        ));
        assert!(quint.contains(
            "def workflowEventTransitionIsSharedByEndpoints(transition) = transition.kind != \"event\" or (workflowOwnsDefinition(transition.source, \"event\", transition.trigger) and workflowOwnsDefinition(transition.target, \"event\", transition.trigger))"
        ));
        assert!(quint.contains(
            "val workflowEventTransitionsAreSharedByEndpointSlices = workflowTransitions.select(transition => workflowEventTransitionIsSharedByEndpoints(transition)).length() == workflowTransitions.length()"
        ));
        assert!(quint.contains(
            "def workflowNavigationTransitionSourceOwnsControl(transition) = transition.kind != \"navigation\" or workflowOwnsDefinition(transition.source, \"control\", transition.trigger)"
        ));
        assert!(quint.contains(
            "def workflowNavigationTransitionTargetsOwnedView(transition) = transition.kind != \"navigation\" or workflowOwnsDefinition(transition.target, \"view\", transition.trigger)"
        ));
        assert!(quint.contains(
            "val workflowNavigationTransitionsResolveControlsAndViews = workflowTransitions.select(transition => workflowNavigationTransitionSourceOwnsControl(transition) and workflowNavigationTransitionTargetsOwnedView(transition)).length() == workflowTransitions.length()"
        ));
        assert!(quint.contains(
            "def workflowExternalTriggerDeclaresPayloadContract(transition) = transition.kind != \"external_trigger\" or (transition.payloadContract != \"\" and workflowOwnsDefinition(transition.source, \"external_payload\", transition.payloadContract))"
        ));
        assert!(quint.contains(
            "val workflowExternalTriggersDeclarePayloadContracts = workflowTransitions.select(transition => workflowExternalTriggerDeclaresPayloadContract(transition)).length() == workflowTransitions.length()"
        ));
        assert!(
            !quint.contains("all { transition <- workflowTransitions }"),
            "Quint transition invariant must be a pure list expression, not an action all block"
        );
        assert!(
            !quint.contains("length(workflowTransitions) == length(workflowTransitions)"),
            "Quint transition invariant must not be a tautological length self-comparison"
        );
        assert!(quint.contains("var modelState: int"));
        assert!(quint.contains("action init = modelState' = 0"));
        assert!(quint.contains("action step = modelState' = modelState"));

        Ok(())
    }

    #[test]
    fn quint_slice_module_exposes_verification_entrypoints() -> Result<(), Box<dyn Error>> {
        let slice_name = parse_model_name("Capture ticket")?;
        let slice_description = parse_model_description("Actor enters repair ticket details.")?;
        let slice_slug = parse_slice_slug("capture-ticket")?;
        let slice_kind = SliceKindName::try_new("state_view".to_owned())?;
        let module = emit_slice_module(
            parse_quint_module_name("CaptureTicket")?,
            slice_name.clone(),
            slice_description.clone(),
            slice_slug.clone(),
            slice_kind.clone(),
            slice_artifact_digest(slice_name, slice_slug, slice_kind, slice_description),
        );
        let quint = module.as_ref();

        assert!(quint.contains("module CaptureTicket"));
        assert!(
            quint.contains(
                "// EMC-DIGEST: slice:name=Capture ticket;slug=capture-ticket;kind=state_view;description=Actor enters repair ticket details."
            )
        );
        assert!(quint.contains(
            "type EventModelScenario = { name: str, givenSteps: List[str], whenSteps: List[str], thenSteps: List[str], readStreams: List[str], writtenStreams: List[str], contractKind: str, coveredDefinition: str, errorReferences: List[str] }"
        ));
        assert!(quint.contains(
            "type BitLevelDataFlow = { datum: str, source: str, transformationSemantics: str, target: str, bitEncoding: str }"
        ));
        assert!(quint.contains(
            "type CommandInput = { name: str, sourceKind: str, sourceDescription: str, provenanceChain: List[str] }"
        ));
        assert!(quint.contains(
            "type CommandErrorDefinition = { name: str, scenarioName: str, recoveryKind: str }"
        ));
        assert!(quint.contains(
            "type CommandDefinition = { name: str, inputs: List[CommandInput], emittedEvents: List[str], observedStreams: List[str], errors: List[CommandErrorDefinition], singleton: bool, repeatBehavior: str }"
        ));
        assert!(quint.contains(
            "type OutcomeDefinition = { label: str, eventSet: List[str], externallyRelevant: bool }"
        ));
        assert!(quint.contains("type StreamDefinition = { name: str }"));
        assert!(quint.contains(
            "type EventAttribute = { name: str, sourceKind: str, sourceName: str, sourceField: str, provenanceDescription: str }"
        ));
        assert!(
            quint.contains("type ExternalPayloadField = { name: str, provenanceDescription: str, bitEncoding: str }\n  type ExternalPayloadDefinition = { name: str, fields: List[ExternalPayloadField] }")
        );
        assert!(quint.contains(
            "type EventDefinition = { name: str, stream: str, attributes: List[EventAttribute], observed: bool, shared: bool }"
        ));
        assert!(quint.contains(
            "type ReadModelField = { name: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, absenceEvent: str, derivationScenarioName: str, absenceScenarioName: str, provenanceDescription: str }"
        ));
        assert!(
            quint
                .contains("type ReadModelDefinition = { name: str, fields: List[ReadModelField], transitive: bool, relationshipFields: List[str], transitiveRule: str, exampleScenarioName: str }")
        );
        assert!(quint.contains(
            "type ViewField = { name: str, sourceKind: str, sourceReadModel: str, sourceField: str, sketchToken: str, provenanceDescription: str, bitEncoding: str }"
        ));
        assert!(quint.contains(
            "type ControlInputProvision = { name: str, sourceKind: str, sourceDescription: str, sketchToken: str, visibleToActor: bool, decisionField: bool }"
        ));
        assert!(quint.contains(
            "type NavigationTarget = { targetType: str, targetName: str, externalWorkflowName: str, externalSystemName: str, handoffContract: str }"
        ));
        assert!(quint.contains(
            "type ControlDefinition = { name: str, commandName: str, inputs: List[ControlInputProvision], handledErrors: List[str], recoveryBehavior: str, sketchToken: str, navigation: NavigationTarget }"
        ));
        assert!(quint.contains(
            "type ViewDefinition = { name: str, readModels: List[str], fields: List[ViewField], controls: List[ControlDefinition], sketchTokens: List[str], localStates: List[str] }"
        ));
        assert!(quint.contains(
            "type AutomationDefinition = { name: str, triggerName: str, commandName: str, handledErrors: List[str], reactionDescription: str }"
        ));
        assert!(quint.contains(
            "type TranslationDefinition = { name: str, externalEventName: str, payloadContractName: str, commandName: str }"
        ));
        assert!(quint.contains(
            "type BoardElement = { name: str, kind: str, lane: str, declaredName: str, mainPath: bool }"
        ));
        assert!(quint.contains(
            "type BoardConnection = { source: str, sourceKind: str, target: str, targetKind: str }"
        ));
        assert!(quint.contains("val sliceCommands: List[str] = []"));
        assert!(quint.contains("val sliceCommandDefinitions: List[CommandDefinition] = []"));
        assert!(quint.contains("val sliceAutomations: List[AutomationDefinition] = []"));
        assert!(quint.contains("val sliceTranslations: List[TranslationDefinition] = []"));
        assert!(
            quint.contains("val canonicalBoardLanes: List[str] = [\"ux\",\"actions\",\"events\"]")
        );
        assert!(quint.contains("val sliceBoardElements: List[BoardElement] = []"));
        assert!(quint.contains("val sliceBoardConnections: List[BoardConnection] = []"));
        assert!(quint.contains("val sliceReferencedCommands: List[str] = []"));
        assert!(quint.contains("val sliceOutcomeDefinitions: List[OutcomeDefinition] = []"));
        assert!(quint.contains(
            "val allowedCommandInputSourceKinds: List[str] = [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]"
        ));
        assert!(quint.contains(
            "val allowedRecoveryKinds: List[str] = [\"retry\",\"stay_on_screen\",\"navigation\",\"explicit_recovery_action\"]"
        ));
        assert!(quint.contains(
            "val allowedSingletonRepeatBehaviors: List[str] = [\"already_exists_error\",\"idempotent\"]"
        ));
        assert!(quint.contains("val sliceEvents: List[str] = []"));
        assert!(quint.contains("val sliceStreams: List[StreamDefinition] = []"));
        assert!(quint.contains("val sliceExternalPayloads: List[ExternalPayloadDefinition] = []"));
        assert!(quint.contains("val sliceEventDefinitions: List[EventDefinition] = []"));
        assert!(quint.contains(
            "val allowedEventAttributeSourceKinds: List[str] = [\"command_input\",\"external_payload\",\"generated\",\"session\",\"constant\",\"derivation\"]"
        ));
        assert!(quint.contains("val sliceReadModels: List[str] = []"));
        assert!(quint.contains("val sliceReadModelDefinitions: List[ReadModelDefinition] = []"));
        assert!(quint.contains(
            "val allowedReadModelFieldSourceKinds: List[str] = [\"event_attribute\",\"derivation\",\"absence_default\"]"
        ));
        assert!(quint.contains("val sliceViews: List[str] = []"));
        assert!(quint.contains("val sliceViewDefinitions: List[ViewDefinition] = []"));
        assert!(quint.contains("val allowedViewFieldSourceKinds: List[str] = [\"read_model\"]"));
        assert!(quint.contains(
            "val allowedControlInputSourceKinds: List[str] = [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]"
        ));
        assert!(quint.contains(
            "val allowedNavigationTargetTypes: List[str] = [\"modeled_view\",\"local_view_state\",\"external_system\",\"external_workflow\"]"
        ));
        assert!(quint.contains("val sliceAcceptanceScenarios: List[EventModelScenario] = []"));
        assert!(quint.contains("val sliceContractScenarios: List[EventModelScenario] = []"));
        assert!(quint.contains("val sliceBitLevelDataFlows: List[BitLevelDataFlow] = []"));
        assert!(quint.contains(
            "val sliceAcceptanceScenariosHaveGwt = sliceAcceptanceScenarios.select(scenario => scenario.name != \"\" and scenario.givenSteps.length() > 0 and scenario.whenSteps.length() > 0 and scenario.thenSteps.length() > 0).length() == sliceAcceptanceScenarios.length()"
        ));
        assert!(quint.contains(
            "val sliceContractScenariosHaveGwt = sliceContractScenarios.select(scenario => scenario.name != \"\" and scenario.givenSteps.length() > 0 and scenario.whenSteps.length() > 0 and scenario.thenSteps.length() > 0).length() == sliceContractScenarios.length()"
        ));
        assert!(quint.contains(
            "val sliceScenariosHaveGwt = sliceAcceptanceScenariosHaveGwt and sliceContractScenariosHaveGwt"
        ));
        assert!(quint.contains(
            "val sliceScenarioNamesAreUnique = sliceAcceptanceScenarios.select(scenario => sliceAcceptanceScenarios.select(other => other.name == scenario.name).length() + sliceContractScenarios.select(other => other.name == scenario.name).length() == 1).length() == sliceAcceptanceScenarios.length() and sliceContractScenarios.select(scenario => sliceAcceptanceScenarios.select(other => other.name == scenario.name).length() + sliceContractScenarios.select(other => other.name == scenario.name).length() == 1).length() == sliceContractScenarios.length()"
        ));
        assert!(quint.contains(
            "def scenarioStreamResolves(streamName) = sliceStreams.select(stream => stream.name == streamName).length() > 0"
        ));
        assert!(quint.contains(
            "def scenarioStreamsResolve(scenario) = scenario.readStreams.select(streamName => scenarioStreamResolves(streamName)).length() == scenario.readStreams.length() and scenario.writtenStreams.select(streamName => scenarioStreamResolves(streamName)).length() == scenario.writtenStreams.length()"
        ));
        assert!(quint.contains(
            "def stateChangeScenarioNamesStreams(scenario) = sliceKind != \"state_change\" or (scenario.readStreams.length() > 0 and scenario.writtenStreams.length() > 0)"
        ));
        assert!(quint.contains(
            "val sliceAcceptanceScenarioStreamsResolve = sliceAcceptanceScenarios.select(scenario => scenarioStreamsResolve(scenario)).length() == sliceAcceptanceScenarios.length()"
        ));
        assert!(quint.contains(
            "val sliceContractScenarioStreamsResolve = sliceContractScenarios.select(scenario => scenarioStreamsResolve(scenario)).length() == sliceContractScenarios.length()"
        ));
        assert!(quint.contains(
            "val sliceScenarioStreamsResolve = sliceAcceptanceScenarioStreamsResolve and sliceContractScenarioStreamsResolve"
        ));
        assert!(quint.contains(
            "val stateChangeAcceptanceScenariosNameStreams = sliceAcceptanceScenarios.select(scenario => stateChangeScenarioNamesStreams(scenario)).length() == sliceAcceptanceScenarios.length()"
        ));
        assert!(quint.contains(
            "val stateChangeContractScenariosNameStreams = sliceContractScenarios.select(scenario => stateChangeScenarioNamesStreams(scenario)).length() == sliceContractScenarios.length()"
        ));
        assert!(quint.contains(
            "val stateChangeScenariosNameStreams = stateChangeAcceptanceScenariosNameStreams and stateChangeContractScenariosNameStreams"
        ));
        assert!(quint.contains(
            "val acceptanceScenariosAreUserFacing = sliceAcceptanceScenarios.select(scenario => scenario.contractKind == \"\" and scenario.coveredDefinition == \"\").length() == sliceAcceptanceScenarios.length()"
        ));
        assert!(quint.contains(
            "def scenarioCoversContract(contractKind, definitionName, scenario) = scenario.contractKind == contractKind and scenario.coveredDefinition == definitionName"
        ));
        assert!(quint.contains(
            "def readModelHasProjectorContract(readModel) = sliceContractScenarios.select(scenario => scenarioCoversContract(\"projector\", readModel.name, scenario)).length() > 0"
        ));
        assert!(quint.contains(
            "val stateViewReadModelsHaveProjectorContracts = sliceKind != \"state_view\" or sliceReadModelDefinitions.select(readModel => readModelHasProjectorContract(readModel)).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "def contractScenarioTargetsKnownDefinition(scenario) = (scenario.contractKind == \"projector\" and (sliceReadModels.select(readModel => readModel == scenario.coveredDefinition).length() > 0 or sliceReadModelDefinitions.select(readModel => readModel.name == scenario.coveredDefinition).length() > 0)) or (scenario.contractKind == \"command\" and (sliceCommands.select(command => command == scenario.coveredDefinition).length() > 0 or sliceCommandDefinitions.select(command => command.name == scenario.coveredDefinition).length() > 0)) or (scenario.contractKind == \"automation\" and sliceAutomations.select(automation => automation.name == scenario.coveredDefinition).length() > 0) or (scenario.contractKind == \"translation\" and sliceTranslations.select(translation => translation.name == scenario.coveredDefinition).length() > 0) or (scenario.contractKind == \"derivation\" and scenario.coveredDefinition != \"\" and sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelField.sourceKind == \"derivation\" and readModelField.derivationScenarioName == scenario.name).length() > 0).length() > 0) or (scenario.contractKind == \"absence\" and scenario.coveredDefinition != \"\" and sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelField.sourceKind == \"absence_default\" and readModelField.absenceScenarioName == scenario.name).length() > 0).length() > 0) or (scenario.contractKind == \"transitive\" and sliceReadModelDefinitions.select(readModel => readModel.transitive and readModel.name == scenario.coveredDefinition and readModel.exampleScenarioName == scenario.name).length() > 0)"
        ));
        assert!(quint.contains(
            "val contractScenariosTargetKnownDefinitions = sliceContractScenarios.select(scenario => contractScenarioTargetsKnownDefinition(scenario)).length() == sliceContractScenarios.length()"
        ));
        assert!(quint.contains(
            "val commandInputsHaveAllowedSources = sliceCommandDefinitions.select(command => command.inputs.select(input => allowedCommandInputSourceKinds.select(sourceKind => sourceKind == input.sourceKind).length() > 0).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "val commandInputsHaveProvenance = sliceCommandDefinitions.select(command => command.inputs.select(input => input.name != \"\" and input.sourceKind != \"\" and input.sourceDescription != \"\" and input.provenanceChain.length() > 0).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "def commandInputEventStreamSourceResolves(command, input) = input.sourceKind != \"event_stream_state\" or (command.observedStreams.length() > 0 and command.observedStreams.select(streamName => scenarioStreamResolves(streamName)).length() == command.observedStreams.length())"
        ));
        assert!(quint.contains(
            "val commandInputsSourcedFromEventStreamsResolve = sliceCommandDefinitions.select(command => command.inputs.select(input => commandInputEventStreamSourceResolves(command, input)).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "def bitLevelFlowCoversTarget(target, datum) = sliceBitLevelDataFlows.select(flow => flow.target == target and flow.datum == datum and flow.source != \"\" and flow.transformationSemantics != \"\" and flow.bitEncoding != \"\").length() > 0"
        ));
        assert!(quint.contains(
            "def commandInputHasBitLevelFlow(command, input) = bitLevelFlowCoversTarget(command.name, input.name)"
        ));
        assert!(quint.contains(
            "val commandErrorsAreDeclared = sliceCommandDefinitions.select(command => command.errors.select(error => error.name != \"\" and error.scenarioName != \"\" and error.recoveryKind != \"\").length() == command.errors.length()).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "val commandErrorsHaveAllowedRecovery = sliceCommandDefinitions.select(command => command.errors.select(error => allowedRecoveryKinds.select(recoveryKind => recoveryKind == error.recoveryKind).length() > 0).length() == command.errors.length()).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "def scenarioNameIsModeled(scenarioName) = sliceAcceptanceScenarios.select(scenario => scenario.name == scenarioName).length() > 0 or sliceContractScenarios.select(scenario => scenario.name == scenarioName).length() > 0"
        ));
        assert!(quint.contains(
            "def commandErrorHasScenarioCoverage(command, error) = sliceContractScenarios.select(scenario => scenario.name == error.scenarioName and scenario.contractKind == \"command\" and scenario.coveredDefinition == command.name and scenario.errorReferences.select(errorName => errorName == error.name).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "val commandErrorsHaveScenarioCoverage = sliceCommandDefinitions.select(command => command.errors.select(error => commandErrorHasScenarioCoverage(command, error)).length() == command.errors.length()).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "def scenarioErrorReferenceIsDeclared(scenario, errorName) = scenario.contractKind != \"command\" or sliceCommandDefinitions.select(command => command.name == scenario.coveredDefinition and command.errors.select(error => error.name == errorName).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "def scenarioErrorReferencesAreDeclaredForScenario(scenario) = scenario.errorReferences.select(errorName => scenarioErrorReferenceIsDeclared(scenario, errorName)).length() == scenario.errorReferences.length()"
        ));
        assert!(quint.contains(
            "val scenarioErrorReferencesAreDeclared = sliceContractScenarios.select(scenario => scenarioErrorReferencesAreDeclaredForScenario(scenario)).length() == sliceContractScenarios.length()"
        ));
        assert!(quint.contains(
            "def singletonCommandDeclaresRepeatBehavior(command) = not(command.singleton) or allowedSingletonRepeatBehaviors.select(repeatBehavior => repeatBehavior == command.repeatBehavior).length() > 0"
        ));
        assert!(quint.contains(
            "val singletonCommandsDeclareRepeatBehavior = sliceCommandDefinitions.select(command => singletonCommandDeclaresRepeatBehavior(command)).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "def automationHasTrigger(automation) = automation.name != \"\" and automation.triggerName != \"\" and automation.reactionDescription != \"\""
        ));
        assert!(quint.contains(
            "def automationIssuesKnownCommand(automation) = sliceCommands.select(command => command == automation.commandName).length() > 0 or sliceReferencedCommands.select(command => command == automation.commandName).length() > 0 or sliceCommandDefinitions.select(command => command.name == automation.commandName).length() > 0"
        ));
        assert!(quint.contains(
            "def automationHandlesCommandErrors(automation, command) = command.name != automation.commandName or command.errors.select(error => automation.handledErrors.select(handledError => handledError == error.name).length() > 0).length() == command.errors.length()"
        ));
        assert!(quint.contains(
            "val automationSlicesDeclareTriggers = sliceKind != \"automation\" or (sliceAutomations.length() > 0 and sliceAutomations.select(automation => automationHasTrigger(automation)).length() == sliceAutomations.length())"
        ));
        assert!(quint.contains(
            "val automationSlicesRepresentOneReaction = sliceKind != \"automation\" or sliceAutomations.length() == 1"
        ));
        assert!(quint.contains(
            "val automationsIssueKnownCommands = sliceAutomations.select(automation => automationIssuesKnownCommand(automation)).length() == sliceAutomations.length()"
        ));
        assert!(quint.contains(
            "val automationsHandleCommandErrors = sliceAutomations.select(automation => sliceCommandDefinitions.select(command => automationHandlesCommandErrors(automation, command)).length() == sliceCommandDefinitions.length()).length() == sliceAutomations.length()"
        ));
        assert!(quint.contains(
            "def translationHasExternalContract(translation) = translation.name != \"\" and translation.externalEventName != \"\" and translation.payloadContractName != \"\" and sliceExternalPayloads.select(payload => payload.name == translation.payloadContractName).length() > 0"
        ));
        assert!(quint.contains(
            "def translationTargetsKnownCommand(translation) = sliceCommands.select(command => command == translation.commandName).length() > 0 or sliceReferencedCommands.select(command => command == translation.commandName).length() > 0 or sliceCommandDefinitions.select(command => command.name == translation.commandName).length() > 0"
        ));
        assert!(quint.contains(
            "def translationReferencesObservedExternalEvent(translation) = sliceEventDefinitions.select(event => event.name == translation.externalEventName and event.observed).length() > 0"
        ));
        assert!(quint.contains(
            "val translationSlicesDeclareExternalContracts = sliceKind != \"translation\" or (sliceTranslations.length() > 0 and sliceTranslations.select(translation => translationHasExternalContract(translation)).length() == sliceTranslations.length())"
        ));
        assert!(quint.contains(
            "val translationsTargetKnownCommands = sliceTranslations.select(translation => translationTargetsKnownCommand(translation)).length() == sliceTranslations.length()"
        ));
        assert!(quint.contains(
            "val translationsReferenceObservedExternalEvents = sliceTranslations.select(translation => translationReferencesObservedExternalEvent(translation)).length() == sliceTranslations.length()"
        ));
        assert!(quint.contains(
            "def boardElementLaneMatchesKind(element) = (element.kind == \"view\" and element.lane == \"ux\") or (element.kind == \"automation\" and element.lane == \"ux\") or (element.kind == \"external_event\" and element.lane == \"ux\") or (element.kind == \"command\" and element.lane == \"actions\") or (element.kind == \"read_model\" and element.lane == \"actions\") or (element.kind == \"event\" and element.lane == \"events\")"
        ));
        assert!(quint.contains(
            "def boardElementReferencesDeclaration(element) = (element.kind == \"view\" and (sliceViews.select(viewName => viewName == element.declaredName).length() > 0 or sliceViewDefinitions.select(view => view.name == element.declaredName).length() > 0)) or (element.kind == \"automation\" and sliceAutomations.select(automation => automation.name == element.declaredName).length() > 0) or (element.kind == \"external_event\" and sliceEventDefinitions.select(event => event.name == element.declaredName and event.observed).length() > 0) or (element.kind == \"command\" and (sliceCommands.select(command => command == element.declaredName).length() > 0 or sliceReferencedCommands.select(command => command == element.declaredName).length() > 0 or sliceCommandDefinitions.select(command => command.name == element.declaredName).length() > 0)) or (element.kind == \"read_model\" and (sliceReadModels.select(readModel => readModel == element.declaredName).length() > 0 or sliceReadModelDefinitions.select(readModel => readModel.name == element.declaredName).length() > 0)) or (element.kind == \"event\" and (sliceEvents.select(eventName => eventName == element.declaredName).length() > 0 or sliceEventDefinitions.select(event => event.name == element.declaredName and (event.observed or event.shared)).length() > 0))"
        ));
        assert!(quint.contains(
            "def boardConnectionHasAllowedShape(connection) = (connection.sourceKind == \"view\" and connection.targetKind == \"command\") or (connection.sourceKind == \"automation\" and connection.targetKind == \"command\") or (connection.sourceKind == \"external_event\" and connection.targetKind == \"command\") or (connection.sourceKind == \"workflow_trigger\" and connection.targetKind == \"command\") or (connection.sourceKind == \"command\" and connection.targetKind == \"event\") or (connection.sourceKind == \"event\" and connection.targetKind == \"read_model\") or (connection.sourceKind == \"read_model\" and connection.targetKind == \"view\")"
        ));
        assert!(quint.contains(
            "def commandEventBoardEdgeMatchesEmission(connection) = connection.sourceKind != \"command\" or connection.targetKind != \"event\" or sliceCommandDefinitions.select(command => command.name == connection.source and command.emittedEvents.select(eventName => eventName == connection.target).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "def eventReadModelBoardEdgeMatchesProjection(connection) = connection.sourceKind != \"event\" or connection.targetKind != \"read_model\" or sliceReadModelDefinitions.select(readModel => readModel.name == connection.target and readModel.fields.select(readModelField => readModelField.sourceEvent == connection.source).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "def externalEventCommandBoardEdgeMatchesTranslation(connection) = connection.sourceKind != \"external_event\" or connection.targetKind != \"command\" or sliceTranslations.select(translation => translation.externalEventName == connection.source and translation.commandName == connection.target).length() > 0"
        ));
        assert!(quint.contains(
            "val externalEventTriggersMatchTranslations = sliceBoardConnections.select(connection => externalEventCommandBoardEdgeMatchesTranslation(connection)).length() == sliceBoardConnections.length()"
        ));
        assert!(quint.contains(
            "def externalEventDoesNotUpdateReadModel(connection) = connection.sourceKind != \"event\" or connection.targetKind != \"read_model\" or sliceEventDefinitions.select(event => event.name == connection.source and event.observed).length() == 0"
        ));
        assert!(quint.contains(
            "val externalEventsDoNotUpdateReadModels = sliceBoardConnections.select(connection => externalEventDoesNotUpdateReadModel(connection)).length() == sliceBoardConnections.length()"
        ));
        assert!(quint.contains(
            "def viewCommandBoardEdgeMatchesControl(connection) = connection.sourceKind != \"view\" or connection.targetKind != \"command\" or sliceViewDefinitions.select(view => view.name == connection.source and view.controls.select(control => control.commandName == connection.target).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "val boardLanesAreCanonical = canonicalBoardLanes == [\"ux\",\"actions\",\"events\"]"
        ));
        assert!(quint.contains(
            "val boardElementsUseCanonicalLanes = sliceBoardElements.select(element => canonicalBoardLanes.select(lane => lane == element.lane).length() > 0 and boardElementLaneMatchesKind(element)).length() == sliceBoardElements.length()"
        ));
        assert!(quint.contains(
            "val boardElementsReferenceDeclarations = sliceBoardElements.select(element => boardElementReferencesDeclaration(element)).length() == sliceBoardElements.length()"
        ));
        assert!(quint.contains(
            "val boardConnectionsHaveCausalSemantics = sliceBoardConnections.select(connection => boardConnectionHasAllowedShape(connection) and commandEventBoardEdgeMatchesEmission(connection) and eventReadModelBoardEdgeMatchesProjection(connection) and externalEventCommandBoardEdgeMatchesTranslation(connection) and externalEventDoesNotUpdateReadModel(connection) and viewCommandBoardEdgeMatchesControl(connection)).length() == sliceBoardConnections.length()"
        ));
        assert!(quint.contains(
            "val readModelsDoNotFeedCommands = sliceBoardConnections.select(connection => connection.sourceKind != \"read_model\" or connection.targetKind != \"command\").length() == sliceBoardConnections.length()"
        ));
        assert!(quint.contains(
            "def readModelViewConnectionHasIncomingEventUpdate(connection) = connection.sourceKind != \"read_model\" or connection.targetKind != \"view\" or sliceBoardConnections.select(incoming => incoming.target == connection.source and incoming.targetKind == \"read_model\" and incoming.sourceKind == \"event\").length() > 0"
        ));
        assert!(quint.contains(
            "val readModelsFeedingViewsHaveIncomingEventUpdates = sliceBoardConnections.select(connection => readModelViewConnectionHasIncomingEventUpdate(connection)).length() == sliceBoardConnections.length()"
        ));
        assert!(quint.contains(
            "val commandsHaveIncomingTriggers = sliceBoardElements.select(element => element.kind != \"command\" or sliceBoardConnections.select(connection => connection.target == element.name and connection.targetKind == \"command\" and (connection.sourceKind == \"view\" or connection.sourceKind == \"automation\" or connection.sourceKind == \"external_event\" or connection.sourceKind == \"workflow_trigger\")).length() > 0).length() == sliceBoardElements.length()"
        ));
        assert!(quint.contains(
            "val mainPathBoardHasNoDisconnectedIslands = sliceBoardElements.select(element => not(element.mainPath) or sliceBoardConnections.select(connection => connection.source == element.name or connection.target == element.name).length() > 0).length() == sliceBoardElements.length()"
        ));
        assert!(quint.contains(
            "val outcomeLabelsAreUnique = sliceOutcomeDefinitions.select(outcome => sliceOutcomeDefinitions.select(other => other.label == outcome.label).length() == 1).length() == sliceOutcomeDefinitions.length()"
        ));
        assert!(quint.contains(
            "val outcomeEventSetsAreNonEmpty = sliceOutcomeDefinitions.select(outcome => outcome.eventSet.length() > 0).length() == sliceOutcomeDefinitions.length()"
        ));
        assert!(quint.contains(
            "val outcomeEventSetsAreDistinct = sliceOutcomeDefinitions.select(outcome => sliceOutcomeDefinitions.select(other => outcome.label == other.label or not(sameOutcomeEventSet(outcome, other))).length() == sliceOutcomeDefinitions.length()).length() == sliceOutcomeDefinitions.length()"
        ));
        assert!(quint.contains(
            "val outcomeEventsAreKnownToSlice = sliceOutcomeDefinitions.select(outcome => outcome.eventSet.select(eventName => eventIsKnownToSlice(eventName)).length() == outcome.eventSet.length()).length() == sliceOutcomeDefinitions.length()"
        ));
        assert!(quint.contains(
            "val eventsReferenceKnownStreams = sliceEventDefinitions.select(event => sliceStreams.select(stream => stream.name == event.stream).length() > 0).length() == sliceEventDefinitions.length()"
        ));
        assert!(quint.contains(
            "def commandEmittedEventIsKnown(eventName) = sliceEvents.select(event => event == eventName).length() > 0 or sliceEventDefinitions.select(event => event.name == eventName).length() > 0"
        ));
        assert!(quint.contains(
            "def eventProducedByCommand(event) = event.observed or event.shared or sliceCommandDefinitions.select(command => command.emittedEvents.select(eventName => eventName == event.name).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "val commandEmittedEventsAreKnown = sliceCommandDefinitions.select(command => command.emittedEvents.select(eventName => commandEmittedEventIsKnown(eventName)).length() == command.emittedEvents.length()).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "val locallyEmittedEventsAreProducedByCommands = sliceEventDefinitions.select(event => eventProducedByCommand(event)).length() == sliceEventDefinitions.length()"
        ));
        assert!(quint.contains(
            "val eventAttributesHaveAllowedSources = sliceEventDefinitions.select(event => event.attributes.select(attribute => allowedEventAttributeSourceKinds.select(sourceKind => sourceKind == attribute.sourceKind).length() > 0).length() == event.attributes.length()).length() == sliceEventDefinitions.length()"
        ));
        assert!(quint.contains(
            "val eventAttributesHaveProvenance = sliceEventDefinitions.select(event => event.attributes.select(attribute => attribute.name != \"\" and attribute.sourceKind != \"\" and attribute.sourceName != \"\" and attribute.provenanceDescription != \"\").length() == event.attributes.length()).length() == sliceEventDefinitions.length()"
        ));
        assert!(quint.contains(
            "def externalPayloadFieldHasProvenance(payloadField) = payloadField.name != \"\" and payloadField.provenanceDescription != \"\" and payloadField.bitEncoding != \"\""
        ));
        assert!(quint.contains(
            "val externalPayloadFieldsHaveProvenance = sliceExternalPayloads.select(payload => payload.name != \"\" and payload.fields.select(payloadField => externalPayloadFieldHasProvenance(payloadField)).length() == payload.fields.length()).length() == sliceExternalPayloads.length()"
        ));
        assert!(quint.contains(
            "def commandInputReferencesAttributeSource(event, attribute, command) = command.emittedEvents.select(eventName => eventName == event.name).length() > 0 and command.inputs.select(input => input.name == attribute.sourceName).length() > 0"
        ));
        assert!(quint.contains(
            "def externalPayloadFieldIsDeclared(attribute) = sliceExternalPayloads.select(payload => payload.name == attribute.sourceName and payload.fields.select(payloadField => payloadField.name == attribute.sourceField).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "def eventAttributeSourceIsComplete(event, attribute) = (attribute.sourceKind == \"command_input\" and attribute.sourceName != \"\" and attribute.sourceField != \"\" and sliceCommandDefinitions.select(command => commandInputReferencesAttributeSource(event, attribute, command)).length() > 0) or (attribute.sourceKind == \"external_payload\" and attribute.sourceName != \"\" and attribute.sourceField != \"\" and externalPayloadFieldIsDeclared(attribute)) or (attribute.sourceKind == \"generated\" and attribute.sourceName != \"\") or (attribute.sourceKind == \"session\" and attribute.sourceName != \"\") or (attribute.sourceKind == \"constant\" and attribute.sourceField != \"\") or (attribute.sourceKind == \"derivation\" and attribute.sourceName != \"\" and attribute.sourceField != \"\")"
        ));
        assert!(quint.contains(
            "val eventAttributeSourcesAreComplete = sliceEventDefinitions.select(event => event.attributes.select(attribute => eventAttributeSourceIsComplete(event, attribute)).length() == event.attributes.length()).length() == sliceEventDefinitions.length()"
        ));
        assert!(quint.contains(
            "def eventAttributeHasBitLevelFlow(event, attribute) = bitLevelFlowCoversTarget(event.name, attribute.name)"
        ));
        assert!(quint.contains(
            "val readModelFieldsHaveAllowedSources = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => allowedReadModelFieldSourceKinds.select(sourceKind => sourceKind == readModelField.sourceKind).length() > 0).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "val readModelFieldsHaveProvenance = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelField.name != \"\" and readModelField.sourceKind != \"\" and readModelField.provenanceDescription != \"\").length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "def readModelFieldSourceIsComplete(readModelField) = (readModelField.sourceKind == \"event_attribute\" and readModelField.sourceEvent != \"\" and readModelField.sourceAttribute != \"\") or (readModelField.sourceKind == \"derivation\" and readModelField.derivationRule != \"\") or (readModelField.sourceKind == \"absence_default\" and readModelField.absenceEvent != \"\")"
        ));
        assert!(quint.contains(
            "val readModelFieldSourcesAreComplete = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldSourceIsComplete(readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "def eventAttributeIsDeclared(eventName, attributeName) = sliceEventDefinitions.select(event => event.name == eventName and event.attributes.select(attribute => attribute.name == attributeName).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "def readModelFieldEventAttributeSourceResolves(readModelField) = readModelField.sourceKind != \"event_attribute\" or eventAttributeIsDeclared(readModelField.sourceEvent, readModelField.sourceAttribute)"
        ));
        assert!(quint.contains(
            "val readModelFieldEventAttributeSourcesResolve = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldEventAttributeSourceResolves(readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "def readModelFieldDerivationScenarioIsCovered(readModelField) = readModelField.sourceKind != \"derivation\" or (readModelField.derivationScenarioName != \"\" and scenarioNameIsModeled(readModelField.derivationScenarioName))"
        ));
        assert!(quint.contains(
            "def readModelFieldAbsenceScenarioIsCovered(readModelField) = readModelField.sourceKind != \"absence_default\" or (readModelField.absenceScenarioName != \"\" and scenarioNameIsModeled(readModelField.absenceScenarioName))"
        ));
        assert!(quint.contains(
            "val derivedReadModelFieldsHaveScenarioCoverage = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldDerivationScenarioIsCovered(readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "val absenceReadModelFieldsHaveScenarioCoverage = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldAbsenceScenarioIsCovered(readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "def transitiveReadModelHasSemantics(readModel) = not(readModel.transitive) or (readModel.relationshipFields.length() > 0 and readModel.transitiveRule != \"\" and readModel.exampleScenarioName != \"\" and scenarioNameIsModeled(readModel.exampleScenarioName))"
        ));
        assert!(quint.contains(
            "val transitiveReadModelsHaveSemantics = sliceReadModelDefinitions.select(readModel => transitiveReadModelHasSemantics(readModel)).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "def readModelFieldHasBitLevelFlow(readModel, readModelField) = bitLevelFlowCoversTarget(readModel.name, readModelField.name)"
        ));
        assert!(quint.contains(
            "val viewFieldsHaveAllowedSources = sliceViewDefinitions.select(view => view.fields.select(viewField => allowedViewFieldSourceKinds.select(sourceKind => sourceKind == viewField.sourceKind).length() > 0).length() == view.fields.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewFieldsHaveProvenance = sliceViewDefinitions.select(view => view.fields.select(viewField => viewField.name != \"\" and viewField.sourceKind != \"\" and viewField.provenanceDescription != \"\" and viewField.bitEncoding != \"\").length() == view.fields.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewFieldSourcesAreComplete = sliceViewDefinitions.select(view => view.fields.select(viewField => viewField.sourceKind == \"read_model\" and viewField.sourceReadModel != \"\" and viewField.sourceField != \"\" and viewField.sketchToken != \"\").length() == view.fields.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewFieldsSourceFromUsedReadModels = sliceViewDefinitions.select(view => view.fields.select(viewField => view.readModels.select(readModel => readModel == viewField.sourceReadModel).length() > 0 and sliceReadModels.select(readModel => readModel == viewField.sourceReadModel).length() > 0).length() == view.fields.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "def viewFieldAppearsInSketch(view, viewField) = viewField.sketchToken != \"\" and view.sketchTokens.select(sketchToken => sketchToken == viewField.sketchToken).length() > 0"
        ));
        assert!(
            quint.contains("def viewHasInformationSketch(view) = view.sketchTokens.length() > 0")
        );
        assert!(quint.contains(
            "val viewsHaveInformationSketches = sliceViewDefinitions.select(view => viewHasInformationSketch(view)).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewFieldsAppearInSketch = sliceViewDefinitions.select(view => view.fields.select(viewField => viewFieldAppearsInSketch(view, viewField)).length() == view.fields.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "def sketchTokenMapsToModeledElement(view, token) = view.fields.select(viewField => viewField.sketchToken == token).length() > 0 or view.controls.select(control => control.sketchToken == token or control.inputs.select(input => input.sourceKind == \"actor\" and input.sketchToken == token).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "val viewSketchTokensMapToModeledElements = sliceViewDefinitions.select(view => view.sketchTokens.select(sketchToken => sketchTokenMapsToModeledElement(view, sketchToken)).length() == view.sketchTokens.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "def readModelFieldIsDeclared(readModelName, fieldName) = sliceReadModelDefinitions.select(readModel => readModel.name == readModelName and readModel.fields.select(readModelField => readModelField.name == fieldName).length() > 0).length() > 0"
        ));
        assert!(quint.contains(
            "def viewFieldSourceReadModelFieldResolves(viewField) = viewField.sourceKind != \"read_model\" or readModelFieldIsDeclared(viewField.sourceReadModel, viewField.sourceField)"
        ));
        assert!(quint.contains(
            "val viewFieldReadModelFieldSourcesResolve = sliceViewDefinitions.select(view => view.fields.select(viewField => viewFieldSourceReadModelFieldResolves(viewField)).length() == view.fields.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "def viewFieldHasBitLevelFlow(view, viewField) = bitLevelFlowCoversTarget(view.name, viewField.name)"
        ));
        assert!(quint.contains(
            "val commandInputDataFlowsAreComplete = sliceCommandDefinitions.select(command => command.inputs.select(input => commandInputHasBitLevelFlow(command, input)).length() == command.inputs.length()).length() == sliceCommandDefinitions.length()"
        ));
        assert!(quint.contains(
            "val eventAttributeDataFlowsAreComplete = sliceEventDefinitions.select(event => event.attributes.select(attribute => eventAttributeHasBitLevelFlow(event, attribute)).length() == event.attributes.length()).length() == sliceEventDefinitions.length()"
        ));
        assert!(quint.contains(
            "val readModelFieldDataFlowsAreComplete = sliceReadModelDefinitions.select(readModel => readModel.fields.select(readModelField => readModelFieldHasBitLevelFlow(readModel, readModelField)).length() == readModel.fields.length()).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewFieldDataFlowsAreComplete = sliceViewDefinitions.select(view => view.fields.select(viewField => viewFieldHasBitLevelFlow(view, viewField)).length() == view.fields.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val modeledDataFlowsAreBitComplete = commandInputDataFlowsAreComplete and eventAttributeDataFlowsAreComplete and readModelFieldDataFlowsAreComplete and viewFieldDataFlowsAreComplete"
        ));
        assert!(quint.contains(
            "val viewControlsHaveSketchTokens = sliceViewDefinitions.select(view => view.controls.select(control => control.name != \"\" and control.commandName != \"\" and control.sketchToken != \"\").length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "def controlAppearsInSketch(view, control) = control.sketchToken != \"\" and view.sketchTokens.select(sketchToken => sketchToken == control.sketchToken).length() > 0"
        ));
        assert!(quint.contains(
            "val viewControlsAppearInSketch = sliceViewDefinitions.select(view => view.controls.select(control => controlAppearsInSketch(view, control)).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewControlsReferenceKnownCommands = sliceViewDefinitions.select(view => view.controls.select(control => sliceCommands.select(command => command == control.commandName).length() > 0 or sliceReferencedCommands.select(command => command == control.commandName).length() > 0 or sliceCommandDefinitions.select(command => command.name == control.commandName).length() > 0).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "def controlProvidesCommandInput(control, input) = control.inputs.select(providedInput => providedInput.name == input.name).length() > 0"
        ));
        assert!(quint.contains(
            "def controlProvidesEveryCommandInput(control, command) = command.name != control.commandName or command.inputs.select(input => controlProvidesCommandInput(control, input)).length() == command.inputs.length()"
        ));
        assert!(quint.contains(
            "val viewControlsProvideCommandInputs = sliceViewDefinitions.select(view => view.controls.select(control => sliceCommandDefinitions.select(command => controlProvidesEveryCommandInput(control, command)).length() == sliceCommandDefinitions.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewControlInputsHaveAllowedSources = sliceViewDefinitions.select(view => view.controls.select(control => control.inputs.select(input => allowedControlInputSourceKinds.select(sourceKind => sourceKind == input.sourceKind).length() > 0).length() == control.inputs.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewControlInputsHaveProvenance = sliceViewDefinitions.select(view => view.controls.select(control => control.inputs.select(input => input.name != \"\" and input.sourceKind != \"\" and input.sourceDescription != \"\").length() == control.inputs.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewControlInputVisibilityIsModeled = sliceViewDefinitions.select(view => view.controls.select(control => control.inputs.select(input => (input.sourceKind != \"actor\" or input.sketchToken != \"\" or input.visibleToActor) and (not(input.decisionField) or input.sketchToken != \"\" or input.visibleToActor)).length() == control.inputs.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewControlsHandleCommandErrors = sliceViewDefinitions.select(view => view.controls.select(control => sliceCommandDefinitions.select(command => command.name != control.commandName or command.errors.select(error => control.handledErrors.select(handledError => handledError == error.name).length() > 0 and control.recoveryBehavior != \"\").length() == command.errors.length()).length() == sliceCommandDefinitions.length()).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "def controlRecoveryBehaviorIsModeled(control) = control.handledErrors.length() == 0 or allowedRecoveryKinds.select(recoveryKind => recoveryKind == control.recoveryBehavior).length() > 0"
        ));
        assert!(quint.contains(
            "val viewControlRecoveryBehaviorIsModeled = sliceViewDefinitions.select(view => view.controls.select(control => controlRecoveryBehaviorIsModeled(control)).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val stateViewSlicesDoNotOwnCommands = sliceKind != \"state_view\" or (sliceCommands.length() == 0 and sliceCommandDefinitions.length() == 0)"
        ));
        assert!(quint.contains(
            "val stateViewSlicesOwnViews = sliceKind != \"state_view\" or (sliceViews.length() > 0 or sliceViewDefinitions.length() > 0)"
        ));
        assert!(quint.contains(
            "val stateViewSlicesOwnReadModels = sliceKind != \"state_view\" or (sliceReadModels.length() > 0 or sliceReadModelDefinitions.length() > 0)"
        ));
        assert!(quint.contains(
            "def readModelOwnsProjectionPath(readModel) = readModel.fields.length() > 0 and readModel.fields.select(readModelField => readModelFieldSourceIsComplete(readModelField)).length() == readModel.fields.length()"
        ));
        assert!(quint.contains(
            "val stateViewSlicesOwnProjectionPaths = sliceKind != \"state_view\" or sliceReadModelDefinitions.select(readModel => readModelOwnsProjectionPath(readModel)).length() == sliceReadModelDefinitions.length()"
        ));
        assert!(quint.contains(
            "val stateChangeSlicesOwnCommands = sliceKind != \"state_change\" or (sliceCommands.length() > 0 or sliceCommandDefinitions.length() > 0)"
        ));
        assert!(quint.contains(
            "val stateChangeSlicesOwnEvents = sliceKind != \"state_change\" or (sliceEvents.length() > 0 or sliceEventDefinitions.length() > 0)"
        ));
        assert!(quint.contains(
            "val stateChangeSlicesOwnOutcomes = sliceKind != \"state_change\" or sliceOutcomeDefinitions.length() > 0"
        ));
        assert!(quint.contains(
            "val stateChangeSlicesOwnErrors = sliceKind != \"state_change\" or commandErrorsAreDeclared"
        ));
        assert!(quint.contains(
            "val stateChangeSlicesDoNotOwnReadModelsOrViews = sliceKind != \"state_change\" or (sliceReadModels.length() == 0 and sliceReadModelDefinitions.length() == 0 and sliceViews.length() == 0 and sliceViewDefinitions.length() == 0)"
        ));
        assert!(quint.contains(
            "val stateChangeSlicesDoNotOwnAutomationsOrTranslations = sliceKind != \"state_change\" or (sliceAutomations.length() == 0 and sliceTranslations.length() == 0)"
        ));
        assert!(quint.contains(
            "val stateChangeSlicesDoNotOwnControlsOrSketches = sliceKind != \"state_change\" or sliceViewDefinitions.select(view => view.controls.length() == 0 and view.sketchTokens.length() == 0).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val translationSlicesDoNotOwnViews = sliceKind != \"translation\" or (sliceViews.length() == 0 and sliceViewDefinitions.length() == 0)"
        ));
        assert!(quint.contains(
            "def navigationTargetTypeIsModeled(target) = target.targetType == \"\" or allowedNavigationTargetTypes.select(targetType => targetType == target.targetType).length() > 0"
        ));
        assert!(quint.contains(
            "def navigationTargetIsComplete(view, target) = (target.targetType == \"\" and target.targetName == \"\" and target.externalWorkflowName == \"\" and target.externalSystemName == \"\" and target.handoffContract == \"\") or (target.targetType == \"modeled_view\" and target.targetName != \"\" and sliceViews.select(viewName => viewName == target.targetName).length() > 0) or (target.targetType == \"local_view_state\" and target.targetName != \"\" and view.localStates.select(localState => localState == target.targetName).length() > 0) or (target.targetType == \"external_workflow\" and target.externalWorkflowName != \"\") or (target.targetType == \"external_system\" and target.externalSystemName != \"\" and target.handoffContract != \"\")"
        ));
        assert!(quint.contains(
            "val viewControlNavigationTypesAreModeled = sliceViewDefinitions.select(view => view.controls.select(control => navigationTargetTypeIsModeled(control.navigation)).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains(
            "val viewControlNavigationTargetsAreComplete = sliceViewDefinitions.select(view => view.controls.select(control => navigationTargetIsComplete(view, control.navigation)).length() == view.controls.length()).length() == sliceViewDefinitions.length()"
        ));
        assert!(quint.contains("val sliceIdentityStable = sliceName == \"Capture ticket\""));
        assert!(
            quint.contains(
                "val sliceHasLocallyEmittedEvent = sliceEvents.length() > 0 or sliceEventDefinitions.select(event => not(event.observed) and not(event.shared)).length() > 0"
            ),
            "Quint slice artifacts must count locally emitted formal event definitions"
        );
        assert!(
            quint.contains(
                "val sliceStateChangeRequiresEvent = sliceKind != \"state_change\" or sliceHasLocallyEmittedEvent"
            ),
            "Quint slice artifacts must expose the state-change event obligation"
        );
        assert!(
            quint.contains(
                "val sliceBitLevelDataFlowsStructured = sliceBitLevelDataFlows.select(flow => flow.datum != \"\" and flow.source != \"\" and flow.transformationSemantics != \"\" and flow.target != \"\" and flow.bitEncoding != \"\").length() == sliceBitLevelDataFlows.length()"
            ),
            "Quint slice artifacts must verify represented data-flow rows include source, transformation/projection, target, and bit encoding fields"
        );
        assert!(
            quint.contains(
                "val modeledDataFlowsAreBitComplete = commandInputDataFlowsAreComplete and eventAttributeDataFlowsAreComplete and readModelFieldDataFlowsAreComplete and viewFieldDataFlowsAreComplete"
            ),
            "Quint slice artifacts must verify modeled data has bit-level flow coverage"
        );
        assert!(quint.contains("var modelState: int"));
        assert!(quint.contains("action init = modelState' = 0"));
        assert!(quint.contains("action step = modelState' = modelState"));

        Ok(())
    }
}
