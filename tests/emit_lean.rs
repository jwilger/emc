#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::core::digest::{WorkflowArtifactDigestInput, artifact_digest, slice_artifact_digest};
    use emc::core::emit::lean::{emit_slice_module, emit_workflow_module};
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
        parse_lean_module_name, parse_model_description, parse_model_name, parse_slice_slug,
        parse_workflow_slug,
    };

    #[test]
    fn lean_workflow_module_represents_business_workflow_fields() -> Result<(), Box<dyn Error>> {
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
            parse_lean_module_name("OpenTicket")?,
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
        let lean = module.as_ref();

        assert!(lean.contains("namespace OpenTicket"));
        assert!(
            lean.contains(
                "-- EMC-DIGEST: workflow:name=Open ticket;slug=open-ticket;description=Actor opens a repair ticket.;slices=capture-ticket|Capture ticket|state_view|Actor enters repair ticket details.|entry,review-ticket|Review ticket|state_view|Actor reviews repair ticket details.|main;transitions=capture-ticket->review-ticket:external_trigger:callback_received::CallbackReceivedPayload"
            )
        );
        assert!(lean.contains("def workflowName := \"Open ticket\""));
        assert!(lean.contains("def workflowSlug := \"open-ticket\""));
        assert!(lean.contains("def workflowDescription := \"Actor opens a repair ticket.\""));
        assert!(lean.contains(
            "def workflowSlices : List String := [\"capture-ticket\",\"review-ticket\"]"
        ));
        assert!(
            lean.contains(
                "def workflowSliceDetails : List (String × String × String × String) := [(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Actor enters repair ticket details.\"),(\"review-ticket\", \"Review ticket\", \"state_view\", \"Actor reviews repair ticket details.\")]"
            )
        );
        assert!(
            lean.contains(
                "structure WorkflowTransition where\n  source : String\n  target : String\n  kind : String\n  trigger : String\n  rationale : String\n  payloadContract : String"
            )
        );
        assert!(
            lean.contains(
                "structure WorkflowOutcome where\n  sourceSlice : String\n  label : String\n  externallyRelevant : Bool"
            ),
            "Lean workflow artifacts must represent externally relevant slice outcomes at composition scope"
        );
        assert!(
            lean.contains(
                "structure WorkflowCommandError where\n  sourceSlice : String\n  commandName : String\n  errorName : String"
            ),
            "Lean workflow artifacts must represent command-local errors so they cannot be treated as business outcomes"
        );
        assert!(
            lean.contains(
                "structure WorkflowOwnedDefinition where\n  sourceSlice : String\n  definitionKind : String\n  definitionName : String\n  definitionStream : String\n  sourceProvenance : String"
            ),
            "Lean workflow artifacts must represent cross-slice definition ownership with event identity fields"
        );
        assert!(
            lean.contains(
                "structure WorkflowEntryLifecycleState where\n  state : String\n  step : String\n  evidence : String"
            ),
            "Lean workflow artifacts must represent application-entry lifecycle coverage"
        );
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"external_trigger\", trigger := \"callback_received\", rationale := \"\", payloadContract := \"CallbackReceivedPayload\" }]"
            ),
            "Lean artifact must model transitions as named business records, not anonymous tuples"
        );
        assert!(
            lean.contains(
                "def workflowOutcomes : List WorkflowOutcome := [{ sourceSlice := \"capture-ticket\", label := \"ticket_captured\", externallyRelevant := true }]"
            ),
            "Lean workflow artifacts must carry authored slice outcomes at composition scope"
        );
        assert!(
            lean.contains(
                "def workflowCommandErrors : List WorkflowCommandError := [{ sourceSlice := \"capture-ticket\", commandName := \"CaptureTicket\", errorName := \"DuplicateTicket\" }]"
            ),
            "Lean workflow artifacts must carry command-local errors so outcome transitions cannot use them"
        );
        assert!(
            lean.contains("def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := \"external_payload\", definitionName := \"CallbackReceivedPayload\", definitionStream := \"\", sourceProvenance := \"\" }]"),
            "Lean workflow artifacts must carry the authored cross-slice ownership inventory"
        );
        assert!(
            lean.contains("def workflowRequiresEntryLifecycleCoverage : Bool := false"),
            "Lean workflow artifacts must make application-entry lifecycle coverage an explicit formal flag"
        );
        assert!(
            lean.contains(
                "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := []"
            ),
            "Lean workflow artifacts must carry authored application-entry lifecycle coverage facts"
        );
        assert!(
            lean.contains("def workflowExitTargets : List String := []"),
            "Lean workflow artifacts must explicitly model workflow-exit targets for transition target resolution"
        );
        assert!(
            lean.contains("def requiredEntryLifecycleStates : List String := [\"fresh_uninitialized\",\"initialized_unauthenticated\",\"initialized_authenticated\",\"partially_configured\",\"fully_configured\"]"),
            "Lean workflow artifacts must encode the required application-entry lifecycle states"
        );
        assert!(
            lean.contains("def allowedWorkflowStepRelationships : List String := [\"entry\",\"main\",\"branch\",\"alternate\",\"async_lifecycle\",\"supporting\"]"),
            "Lean workflow artifacts must model the allowed workflow step relationship inventory"
        );
        assert!(
            lean.contains("def workflowStepRelationships : List (String × String) :="),
            "Lean workflow artifacts must represent each workflow step relationship separately from rendering concerns"
        );
        assert!(
            lean.contains(
                "def workflowStepRelationships : List (String × String) := [(\"capture-ticket\", \"entry\"),(\"review-ticket\", \"main\")]"
            ),
            "Lean workflow artifacts must emit concrete workflow step relationships"
        );
        assert!(
            lean.contains("def workflowStepRelationshipsAreAllowed : Bool := workflowStepRelationships.all workflowStepRelationshipIsAllowed"),
            "Lean workflow artifacts must expose allowed step relationships as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowStepSlugCount (slug : String) : Nat := (workflowSlices.filter (fun step => step == slug)).length"
            ),
            "Lean workflow artifacts must count step slugs for uniqueness proofs"
        );
        assert!(
            lean.contains(
                "def workflowStepSlugsAreUnique : Bool := workflowSlices.all (fun step => workflowStepSlugCount step == 1)"
            ),
            "Lean workflow artifacts must prove workflow step slugs are unique"
        );
        assert!(
            lean.contains(
                "def workflowHasExactlyOneEntryStep : Bool := workflowEntryStepCount == 1"
            ),
            "Lean workflow artifacts must prove the composition has exactly one entry step"
        );
        assert!(
            lean.contains("def workflowMainStepsHaveIncomingReachability : Bool := workflowStepRelationships.all workflowMainStepHasIncomingTransition"),
            "Lean workflow artifacts must prove main workflow steps have incoming reachability"
        );
        assert!(
            lean.contains(
                "def workflowReachableStepsAfterFuel : Nat -> List String -> List String"
            ),
            "Lean workflow artifacts must compute workflow reachability from the entry step"
        );
        assert!(
            lean.contains(
                "def workflowReachableStepsFromEntry : List String := workflowReachableStepsAfterFuel workflowSlices.length workflowEntrySteps"
            ),
            "Lean workflow artifacts must bound reachability traversal by the composed workflow size"
        );
        assert!(
            lean.contains(
                "def workflowStepIsReachableFromEntry (step : String × String) : Bool := step.2 == \"supporting\" || workflowReachableStepsFromEntry.contains step.1"
            ),
            "Lean workflow artifacts must exempt only supporting steps from required entry reachability"
        );
        assert!(
            lean.contains(
                "def workflowNonSupportingStepsReachableFromEntry : Bool := workflowStepRelationships.all workflowStepIsReachableFromEntry"
            ),
            "Lean workflow artifacts must expose non-supporting workflow reachability as a proof obligation"
        );
        assert!(
            lean.contains("def workflowBranchOrAlternateStepHasTriggerOrRationale (step : String × String) : Bool := (step.2 != \"branch\" && step.2 != \"alternate\") || workflowTransitions.any (fun transition => transition.target == step.1 && (transition.trigger.isEmpty == false || transition.rationale.isEmpty == false))"),
            "Lean workflow artifacts must define branch and alternate step trigger/rationale obligations"
        );
        assert!(
            lean.contains("def workflowBranchAndAlternateStepsHaveTriggerOrRationale : Bool := workflowStepRelationships.all workflowBranchOrAlternateStepHasTriggerOrRationale"),
            "Lean workflow artifacts must expose branch and alternate trigger/rationale as a proof obligation"
        );
        assert!(
            lean.contains("def workflowEntryLifecycleStatesCoverRequiredStates : Bool := workflowRequiresEntryLifecycleCoverage == false || requiredEntryLifecycleStates.all workflowEntryLifecycleStateCovered"),
            "Lean workflow artifacts must expose application-entry lifecycle coverage as a proof obligation"
        );
        assert!(lean.contains("theorem workflowIdentityIsStable"));
        assert!(lean.contains(
            "theorem workflowEntryLifecycleStatesCoverRequiredStatesIsStable : workflowEntryLifecycleStatesCoverRequiredStates = true := rfl"
        ));
        assert!(
            lean.contains(
                "theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl"
            ),
            "Lean artifact must prove every modeled workflow slice has generated detail metadata"
        );
        assert!(
            lean.contains(
                "theorem workflowTransitionsAreStructured : workflowTransitions.all (fun transition => transition.source.isEmpty == false && transition.target.isEmpty == false && transition.kind.isEmpty == false && transition.trigger.isEmpty == false) = true := rfl"
            ),
            "Lean artifact must prove every business transition has source, target, kind, and trigger fields"
        );
        assert!(
            lean.contains(
                "theorem workflowTransitionSourcesResolve : workflowTransitions.all (fun transition => workflowSlices.contains transition.source) = true := rfl"
            ),
            "Lean artifact must prove transition sources are composed workflow steps"
        );
        assert!(
            lean.contains(
                "theorem workflowTransitionTargetsResolve : workflowTransitions.all (fun transition => workflowSlices.contains transition.target || workflowExitTargets.contains transition.target) = true := rfl"
            ),
            "Lean artifact must prove transition targets are composed steps or explicit workflow exits"
        );
        assert!(
            lean.contains(
                "theorem workflowStepRelationshipsAreAllowedIsStable : workflowStepRelationshipsAreAllowed = true := rfl"
            ),
            "Lean workflow artifacts must prove current workflow step relationships are modeled"
        );
        assert!(
            lean.contains(
                "theorem workflowStepSlugsAreUniqueIsStable : workflowStepSlugsAreUnique = true := rfl"
            ),
            "Lean workflow artifacts must prove current workflow step slugs are unique"
        );
        assert!(
            lean.contains(
                "theorem workflowHasExactlyOneEntryStepIsStable : workflowHasExactlyOneEntryStep = true := rfl"
            ),
            "Lean workflow artifacts must prove current workflow has exactly one entry step"
        );
        assert!(
            lean.contains(
                "theorem workflowMainStepsHaveIncomingReachabilityIsStable : workflowMainStepsHaveIncomingReachability = true := rfl"
            ),
            "Lean workflow artifacts must prove current main workflow steps are reachable"
        );
        assert!(
            lean.contains(
                "theorem workflowNonSupportingStepsReachableFromEntryIsStable : workflowNonSupportingStepsReachableFromEntry = true := rfl"
            ),
            "Lean workflow artifacts must prove current non-supporting workflow steps are reachable from entry"
        );
        assert!(
            lean.contains(
                "theorem workflowBranchAndAlternateStepsHaveTriggerOrRationaleIsStable : workflowBranchAndAlternateStepsHaveTriggerOrRationale = true := rfl"
            ),
            "Lean workflow artifacts must prove current branch and alternate steps explain why they are reached"
        );
        assert!(
            lean.contains(
                "def workflowTransitionKindIsModeled (transition : WorkflowTransition) : Bool := transition.kind == \"navigation\" || transition.kind == \"command\" || transition.kind == \"event\" || transition.kind == \"external_trigger\" || transition.kind == \"outcome\" || workflowExitTargets.contains transition.target"
            ),
            "Lean workflow artifacts must model the legal workflow transition kinds, including explicit workflow exits"
        );
        assert!(
            lean.contains(
                "def workflowTransitionExitHasRationale (transition : WorkflowTransition) : Bool := workflowExitTargets.contains transition.target == false || transition.rationale.isEmpty == false"
            ),
            "Lean workflow artifacts must require workflow exits to explain why the exit is reached"
        );
        assert!(
            lean.contains(
                "def workflowTransitionsHaveModeledKinds : Bool := workflowTransitions.all workflowTransitionKindIsModeled"
            ),
            "Lean workflow artifacts must expose transition-kind legality as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowExitsNameTargetsAndRationale : Bool := workflowTransitions.all workflowTransitionExitHasRationale"
            ),
            "Lean workflow artifacts must expose workflow-exit rationale as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowOutcomeHandledByTransition (outcome : WorkflowOutcome) : Bool := outcome.externallyRelevant == false || workflowTransitions.any (fun transition => transition.source == outcome.sourceSlice && transition.kind == \"outcome\" && transition.trigger == outcome.label)"
            ),
            "Lean workflow artifacts must define how externally relevant outcomes are handled by workflow transitions"
        );
        assert!(
            lean.contains(
                "def workflowExternallyRelevantOutcomesHandled : Bool := workflowOutcomes.all workflowOutcomeHandledByTransition"
            ),
            "Lean workflow artifacts must require workflows to handle every externally relevant outcome"
        );
        assert!(
            lean.contains(
                "def workflowOutcomeSourceResolves (outcome : WorkflowOutcome) : Bool := workflowSlices.contains outcome.sourceSlice"
            ),
            "Lean workflow artifacts must require workflow outcome facts to source from composed workflow steps"
        );
        assert!(
            lean.contains(
                "def workflowOutcomesSourceResolve : Bool := workflowOutcomes.all workflowOutcomeSourceResolves"
            ),
            "Lean workflow artifacts must expose workflow outcome source resolution as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowCommandErrorSourceResolves (error : WorkflowCommandError) : Bool := workflowSlices.contains error.sourceSlice"
            ),
            "Lean workflow artifacts must require workflow command-error facts to source from composed workflow steps"
        );
        assert!(
            lean.contains(
                "def workflowCommandErrorsSourceResolve : Bool := workflowCommandErrors.all workflowCommandErrorSourceResolves"
            ),
            "Lean workflow artifacts must expose workflow command-error source resolution as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowTransitionIsNotCommandErrorOutcome (transition : WorkflowTransition) : Bool := transition.kind != \"outcome\" || workflowCommandErrors.any (fun error => error.sourceSlice == transition.source && error.errorName == transition.trigger) == false"
            ),
            "Lean workflow artifacts must distinguish command-local errors from business outcomes"
        );
        assert!(
            lean.contains(
                "def workflowNonEventDefinitionOwnedOnce (definition : WorkflowOwnedDefinition) : Bool := definition.definitionKind == \"event\" || (workflowOwnedDefinitions.filter (fun other => other.definitionKind == definition.definitionKind && other.definitionName == definition.definitionName)).length == 1"
            ),
            "Lean workflow artifacts must define non-event definition ownership uniqueness"
        );
        assert!(
            lean.contains(
                "def workflowNonEventDefinitionsAreUniquelyOwned : Bool := workflowOwnedDefinitions.all workflowNonEventDefinitionOwnedOnce"
            ),
            "Lean workflow artifacts must expose cross-slice non-event ownership uniqueness as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowEventDefinitionHasIdentity (definition : WorkflowOwnedDefinition) : Bool := definition.definitionKind != \"event\" || (definition.definitionStream.isEmpty == false && definition.sourceProvenance.isEmpty == false)"
            ),
            "Lean workflow artifacts must require shared event definitions to carry stream and provenance identity"
        );
        assert!(
            lean.contains(
                "def workflowSharedEventDefinitionMatches (left : WorkflowOwnedDefinition) (right : WorkflowOwnedDefinition) : Bool := left.definitionKind != \"event\" || right.definitionKind != \"event\" || left.definitionName != right.definitionName || (left.definitionStream == right.definitionStream && left.sourceProvenance == right.sourceProvenance)"
            ),
            "Lean workflow artifacts must compare duplicate shared events by stream and source provenance"
        );
        assert!(
            lean.contains(
                "def workflowSharedEventDefinitionsHaveIdenticalIdentity : Bool := workflowOwnedDefinitions.all workflowEventDefinitionHasIdentity && workflowOwnedDefinitions.all (fun definition => workflowOwnedDefinitions.all (workflowSharedEventDefinitionMatches definition))"
            ),
            "Lean workflow artifacts must expose shared event identity as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowOwnsDefinition (sourceSlice : String) (definitionKind : String) (definitionName : String) : Bool := workflowOwnedDefinitions.any (fun definition => definition.sourceSlice == sourceSlice && definition.definitionKind == definitionKind && definition.definitionName == definitionName)"
            ),
            "Lean workflow artifacts must define workflow-scoped definition ownership lookup"
        );
        assert!(
            lean.contains(
                "def workflowCommandTransitionTargetsOwnedCommand (transition : WorkflowTransition) : Bool := transition.kind != \"command\" || workflowOwnsDefinition transition.target \"command\" transition.trigger"
            ),
            "Lean workflow artifacts must require command transitions to target command-owning slices"
        );
        assert!(
            lean.contains(
                "def workflowCommandTransitionsTargetOwnedCommands : Bool := workflowTransitions.all workflowCommandTransitionTargetsOwnedCommand"
            ),
            "Lean workflow artifacts must expose command transition target-command resolution as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowCommandTransitionSourceOwnsControl (transition : WorkflowTransition) : Bool := transition.kind != \"command\" || workflowOwnsDefinition transition.source \"control\" transition.trigger"
            ),
            "Lean workflow artifacts must require command transitions to come from source-owned controls"
        );
        assert!(
            lean.contains(
                "def workflowCommandTransitionsSourceOwnedControls : Bool := workflowTransitions.all workflowCommandTransitionSourceOwnsControl"
            ),
            "Lean workflow artifacts must expose command transition source-control resolution as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowCommandTransitionsResolveControlsAndCommands : Bool := workflowTransitions.all (fun transition => workflowCommandTransitionSourceOwnsControl transition && workflowCommandTransitionTargetsOwnedCommand transition)"
            ),
            "Lean workflow artifacts must expose command transition control/command resolution as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowEventTransitionIsSharedByEndpoints (transition : WorkflowTransition) : Bool := transition.kind != \"event\" || (workflowOwnsDefinition transition.source \"event\" transition.trigger && workflowOwnsDefinition transition.target \"event\" transition.trigger)"
            ),
            "Lean workflow artifacts must require event transitions to be shared by source and target slices"
        );
        assert!(
            lean.contains(
                "def workflowEventTransitionsAreSharedByEndpointSlices : Bool := workflowTransitions.all workflowEventTransitionIsSharedByEndpoints"
            ),
            "Lean workflow artifacts must expose event transition sharing as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowNavigationTransitionSourceOwnsControl (transition : WorkflowTransition) : Bool := transition.kind != \"navigation\" || workflowOwnsDefinition transition.source \"control\" transition.trigger"
            ),
            "Lean workflow artifacts must require navigation transitions to come from source-owned controls"
        );
        assert!(
            lean.contains(
                "def workflowNavigationTransitionTargetsOwnedView (transition : WorkflowTransition) : Bool := transition.kind != \"navigation\" || workflowOwnsDefinition transition.target \"view\" transition.trigger"
            ),
            "Lean workflow artifacts must require navigation transitions to resolve to target-owned views"
        );
        assert!(
            lean.contains(
                "def workflowNavigationTransitionsResolveControlsAndViews : Bool := workflowTransitions.all (fun transition => workflowNavigationTransitionSourceOwnsControl transition && workflowNavigationTransitionTargetsOwnedView transition)"
            ),
            "Lean workflow artifacts must expose navigation transition control/view resolution as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowExternalTriggerDeclaresPayloadContract (transition : WorkflowTransition) : Bool := transition.kind != \"external_trigger\" || (transition.payloadContract.isEmpty == false && workflowOwnsDefinition transition.source \"external_payload\" transition.payloadContract)"
            ),
            "Lean workflow artifacts must require external-trigger transitions to declare owned payload contracts"
        );
        assert!(
            lean.contains(
                "def workflowExternalTriggersDeclarePayloadContracts : Bool := workflowTransitions.all workflowExternalTriggerDeclaresPayloadContract"
            ),
            "Lean workflow artifacts must expose external-trigger payload contracts as a proof obligation"
        );
        assert!(
            lean.contains(
                "def workflowExternalTriggerPayloadContractHasProvenance (transition : WorkflowTransition) : Bool := transition.kind != \"external_trigger\" || workflowOwnedDefinitions.any (fun definition => definition.sourceSlice == transition.source && definition.definitionKind == \"external_payload\" && definition.definitionName == transition.payloadContract && definition.sourceProvenance.isEmpty == false)"
            ),
            "Lean workflow artifacts must require external-trigger payload contracts to carry provenance"
        );
        assert!(
            lean.contains(
                "def workflowExternalTriggerPayloadContractsHaveProvenance : Bool := workflowTransitions.all workflowExternalTriggerPayloadContractHasProvenance"
            ),
            "Lean workflow artifacts must expose external-trigger payload provenance as a proof obligation"
        );
        assert!(
            lean.contains(
                "theorem workflowExternallyRelevantOutcomesHandledIsStable : workflowExternallyRelevantOutcomesHandled = true := rfl"
            ),
            "Lean workflow artifacts must prove current externally relevant outcomes are handled"
        );
        assert!(
            lean.contains(
                "theorem workflowTransitionsHaveModeledKindsIsStable : workflowTransitionsHaveModeledKinds = true := rfl"
            ),
            "Lean workflow artifacts must prove current transitions use modeled transition kinds"
        );
        assert!(
            lean.contains(
                "theorem workflowExitsNameTargetsAndRationaleIsStable : workflowExitsNameTargetsAndRationale = true := rfl"
            ),
            "Lean workflow artifacts must prove current workflow exits name their targets and rationale"
        );
        assert!(
            lean.contains(
                "theorem workflowTransitionsDoNotUseCommandErrorsAsOutcomesIsStable : workflowTransitionsDoNotUseCommandErrorsAsOutcomes = true := rfl"
            ),
            "Lean workflow artifacts must prove current outcome transitions do not branch on command-local errors"
        );
        assert!(
            lean.contains(
                "theorem workflowNonEventDefinitionsAreUniquelyOwnedIsStable : workflowNonEventDefinitionsAreUniquelyOwned = true := rfl"
            ),
            "Lean workflow artifacts must prove non-event definitions are owned by exactly one slice"
        );
        assert!(
            lean.contains(
                "theorem workflowSharedEventDefinitionsHaveIdenticalIdentityIsStable : workflowSharedEventDefinitionsHaveIdenticalIdentity = true := rfl"
            ),
            "Lean workflow artifacts must prove current shared event definitions have identical stream and provenance identity"
        );
        assert!(
            lean.contains(
                "theorem workflowCommandTransitionsResolveControlsAndCommandsIsStable : workflowCommandTransitionsResolveControlsAndCommands = true := rfl"
            ),
            "Lean workflow artifacts must prove current command transitions resolve source controls and target commands"
        );
        assert!(
            lean.contains(
                "theorem workflowCommandTransitionsTargetOwnedCommandsIsStable : workflowCommandTransitionsTargetOwnedCommands = true := rfl"
            ),
            "Lean workflow artifacts must prove current command transitions target command-owning slices"
        );
        assert!(
            lean.contains(
                "theorem workflowCommandTransitionsSourceOwnedControlsIsStable : workflowCommandTransitionsSourceOwnedControls = true := rfl"
            ),
            "Lean workflow artifacts must prove current command transitions come from source-owned controls"
        );
        assert!(
            lean.contains(
                "theorem workflowEventTransitionsAreSharedByEndpointSlicesIsStable : workflowEventTransitionsAreSharedByEndpointSlices = true := rfl"
            ),
            "Lean workflow artifacts must prove current event transitions are shared by endpoint slices"
        );
        assert!(
            lean.contains(
                "theorem workflowNavigationTransitionsResolveControlsAndViewsIsStable : workflowNavigationTransitionsResolveControlsAndViews = true := rfl"
            ),
            "Lean workflow artifacts must prove current navigation transitions resolve source controls and target views"
        );
        assert!(
            lean.contains(
                "theorem workflowOutcomesSourceResolveIsStable : workflowOutcomesSourceResolve = true := rfl"
            ),
            "Lean workflow artifacts must prove current workflow outcome facts source from composed workflow steps"
        );
        assert!(
            lean.contains(
                "theorem workflowCommandErrorsSourceResolveIsStable : workflowCommandErrorsSourceResolve = true := rfl"
            ),
            "Lean workflow artifacts must prove current workflow command-error facts source from composed workflow steps"
        );
        assert!(
            lean.contains(
                "theorem workflowExternalTriggersDeclarePayloadContractsIsStable : workflowExternalTriggersDeclarePayloadContracts = true := rfl"
            ),
            "Lean workflow artifacts must prove current external triggers declare payload contracts"
        );
        assert!(
            lean.contains(
                "theorem workflowExternalTriggerPayloadContractsHaveProvenanceIsStable : workflowExternalTriggerPayloadContractsHaveProvenance = true := rfl"
            ),
            "Lean workflow artifacts must prove current external trigger payload contract provenance"
        );
        assert!(
            !lean.contains("transition.1.isEmpty"),
            "Lean transition structure proof must not depend on positional tuple selectors"
        );
        assert!(
            !lean.contains("workflowTransitions.length = workflowTransitions.length"),
            "Lean transition structure proof must not be a tautological length self-comparison"
        );

        Ok(())
    }

    #[test]
    fn lean_workflow_module_types_empty_lists() -> Result<(), Box<dyn Error>> {
        let workflow_name = parse_model_name("Open ticket")?;
        let workflow_description = parse_model_description("Actor opens a repair ticket.")?;
        let workflow_slug = parse_workflow_slug("open-ticket")?;
        let module = emit_workflow_module(
            parse_lean_module_name("OpenTicket")?,
            WorkflowModuleData::new(
                workflow_name.clone(),
                workflow_description.clone(),
                workflow_slug.clone(),
                artifact_digest(WorkflowArtifactDigestInput {
                    workflow_name,
                    workflow_slug,
                    workflow_description,
                    workflow_slice_details: WorkflowSliceDetails::from_details([]),
                    workflow_transitions: WorkflowTransitionRecords::from_records([]),
                    workflow_outcomes: WorkflowOutcomeRecords::from_records([]),
                    workflow_command_errors: WorkflowCommandErrorRecords::from_records([]),
                    workflow_owned_definitions: WorkflowOwnedDefinitionRecords::from_records([]),
                    workflow_transition_evidences: Default::default(),
                    workflow_requires_entry_lifecycle_coverage: false,
                    workflow_entry_lifecycle_states: Default::default(),
                }),
            )
            .with_slice_details(WorkflowSliceDetails::from_details([]))
            .with_transitions(WorkflowTransitionRecords::from_records([]))
            .with_outcomes(WorkflowOutcomeRecords::from_records([]))
            .with_command_errors(WorkflowCommandErrorRecords::from_records([])),
        );
        let lean = module.as_ref();

        assert!(lean.contains("def workflowSlices : List String := []"));
        assert!(
            lean.contains(
                "def workflowSliceDetails : List (String × String × String × String) := []"
            )
        );
        assert!(lean.contains("def workflowTransitions : List WorkflowTransition := []"));

        Ok(())
    }

    #[test]
    fn lean_slice_module_represents_business_slice_fields() -> Result<(), Box<dyn Error>> {
        let slice_name = parse_model_name("Capture ticket")?;
        let slice_description = parse_model_description("Actor enters repair ticket details.")?;
        let slice_slug = parse_slice_slug("capture-ticket")?;
        let slice_kind = SliceKindName::try_new("state_view".to_owned())?;
        let module = emit_slice_module(
            parse_lean_module_name("CaptureTicket")?,
            slice_name.clone(),
            slice_description.clone(),
            slice_slug.clone(),
            slice_kind.clone(),
            slice_artifact_digest(slice_name, slice_slug, slice_kind, slice_description),
        );
        let lean = module.as_ref();

        assert!(lean.contains("namespace CaptureTicket"));
        assert!(
            lean.contains(
                "-- EMC-DIGEST: slice:name=Capture ticket;slug=capture-ticket;kind=state_view;description=Actor enters repair ticket details."
            )
        );
        assert!(lean.contains("def sliceName := \"Capture ticket\""));
        assert!(lean.contains("def sliceSlug := \"capture-ticket\""));
        assert!(lean.contains("def sliceKind := \"state_view\""));
        assert!(lean.contains("def sliceDescription := \"Actor enters repair ticket details.\""));
        assert!(
            lean.contains(
                "structure EventModelScenario where\n  name : String\n  givenSteps : List String\n  whenSteps : List String\n  thenSteps : List String\n  readStreams : List String\n  writtenStreams : List String\n  contractKind : String\n  coveredDefinition : String\n  errorReferences : List String"
            ),
            "Lean slice artifacts must represent first-class GWT scenarios with read/write stream, contract coverage, and explicit command-error references"
        );
        assert!(
            lean.contains(
                "structure BitLevelDataFlow where\n  datum : String\n  source : String\n  transformationSemantics : String\n  target : String\n  bitEncoding : String"
            ),
            "Lean slice artifacts must reserve formal space for bit-level source, transformation/projection, target, and encoding semantics"
        );
        assert!(
            lean.contains(
                "structure CommandInput where\n  name : String\n  sourceKind : String\n  sourceDescription : String\n  provenanceChain : List String"
            ),
            "Lean slice artifacts must represent command input source-chain provenance"
        );
        assert!(
            lean.contains(
                "structure CommandErrorDefinition where\n  name : String\n  scenarioName : String\n  recoveryKind : String"
            ),
            "Lean slice artifacts must represent declared command-local errors and recovery semantics"
        );
        assert!(
            lean.contains(
                "structure CommandDefinition where\n  name : String\n  inputs : List CommandInput\n  emittedEvents : List String\n  observedStreams : List String\n  errors : List CommandErrorDefinition\n  singleton : Bool\n  repeatBehavior : String"
            ),
            "Lean slice artifacts must represent commands in terms of inputs, emitted events, stream-derived state, declared errors, and singleton repeat behavior"
        );
        assert!(
            lean.contains(
                "structure OutcomeDefinition where\n  label : String\n  eventSet : List String\n  externallyRelevant : Bool"
            ),
            "Lean slice artifacts must represent business outcomes as first-class event-backed definitions"
        );
        assert!(
            lean.contains("structure StreamDefinition where\n  name : String"),
            "Lean slice artifacts must represent known event streams"
        );
        assert!(
            lean.contains(
                "structure EventAttribute where\n  name : String\n  sourceKind : String\n  sourceName : String\n  sourceField : String\n  provenanceDescription : String"
            ),
            "Lean slice artifacts must represent event attribute source provenance"
        );
        assert!(
            lean.contains(
                "structure ExternalPayloadField where\n  name : String\n  provenanceDescription : String\n  bitEncoding : String\n\nstructure ExternalPayloadDefinition where\n  name : String\n  fields : List ExternalPayloadField"
            ),
            "Lean slice artifacts must represent external payload field provenance and bit encoding"
        );
        assert!(
            lean.contains(
                "structure EventDefinition where\n  name : String\n  stream : String\n  attributes : List EventAttribute\n  observed : Bool\n  shared : Bool"
            ),
            "Lean slice artifacts must represent events with stream references and attributes"
        );
        assert!(
            lean.contains(
                "structure ReadModelField where\n  name : String\n  sourceKind : String\n  sourceEvent : String\n  sourceAttribute : String\n  derivationRule : String\n  absenceEvent : String\n  derivationScenarioName : String\n  absenceScenarioName : String\n  provenanceDescription : String"
            ),
            "Lean slice artifacts must represent read model field source, derivation, absence, scenario coverage, and provenance"
        );
        assert!(
            lean.contains(
                "structure ReadModelDefinition where\n  name : String\n  fields : List ReadModelField\n  transitive : Bool\n  relationshipFields : List String\n  transitiveRule : String\n  exampleScenarioName : String"
            ),
            "Lean slice artifacts must represent read model definitions with modeled fields and transitive semantics"
        );
        assert!(
            lean.contains(
                "structure ViewField where\n  name : String\n  sourceKind : String\n  sourceReadModel : String\n  sourceField : String\n  sketchToken : String\n  provenanceDescription : String\n  bitEncoding : String"
            ),
            "Lean slice artifacts must represent displayed view data with source and bit-level provenance"
        );
        assert!(
            lean.contains(
                "structure ControlInputProvision where\n  name : String\n  sourceKind : String\n  sourceDescription : String\n  sketchToken : String\n  visibleToActor : Bool\n  decisionField : Bool"
            ),
            "Lean slice artifacts must represent how controls provide command inputs"
        );
        assert!(
            lean.contains(
                "structure NavigationTarget where\n  targetType : String\n  targetName : String\n  externalWorkflowName : String\n  externalSystemName : String\n  handoffContract : String"
            ),
            "Lean slice artifacts must represent typed navigation targets for controls"
        );
        assert!(
            lean.contains(
                "structure ControlDefinition where\n  name : String\n  commandName : String\n  inputs : List ControlInputProvision\n  handledErrors : List String\n  recoveryBehavior : String\n  sketchToken : String\n  navigation : NavigationTarget"
            ),
            "Lean slice artifacts must represent controls as first-class view-owned interactions"
        );
        assert!(
            lean.contains(
                "structure ViewDefinition where\n  name : String\n  readModels : List String\n  fields : List ViewField\n  controls : List ControlDefinition\n  sketchTokens : List String\n  localStates : List String"
            ),
            "Lean slice artifacts must represent view definitions as first-class formal data"
        );
        assert!(
            lean.contains(
                "structure AutomationDefinition where\n  name : String\n  triggerName : String\n  commandName : String\n  handledErrors : List String\n  reactionDescription : String"
            ),
            "Lean slice artifacts must represent automation triggers, issued commands, handled errors, and reaction semantics"
        );
        assert!(
            lean.contains(
                "structure TranslationDefinition where\n  name : String\n  externalEventName : String\n  payloadContractName : String\n  commandName : String"
            ),
            "Lean slice artifacts must represent translation external events, payload contracts, and target commands"
        );
        assert!(
            lean.contains(
                "structure BoardElement where\n  name : String\n  kind : String\n  lane : String\n  declaredName : String\n  mainPath : Bool"
            ),
            "Lean slice artifacts must represent board elements as formal causal declarations"
        );
        assert!(
            lean.contains(
                "structure BoardConnection where\n  source : String\n  sourceKind : String\n  target : String\n  targetKind : String"
            ),
            "Lean slice artifacts must represent board connections as formal causal edges"
        );
        assert!(lean.contains("def sliceCommands : List String := []"));
        assert!(lean.contains("def sliceCommandDefinitions : List CommandDefinition := []"));
        assert!(lean.contains("def sliceAutomations : List AutomationDefinition := []"));
        assert!(lean.contains("def sliceTranslations : List TranslationDefinition := []"));
        assert!(
            lean.contains(
                "def canonicalBoardLanes : List String := [\"ux\",\"actions\",\"events\"]"
            )
        );
        assert!(lean.contains("def sliceBoardElements : List BoardElement := []"));
        assert!(lean.contains("def sliceBoardConnections : List BoardConnection := []"));
        assert!(lean.contains("def sliceReferencedCommands : List String := []"));
        assert!(lean.contains("def sliceOutcomeDefinitions : List OutcomeDefinition := []"));
        assert!(
            lean.contains(
                "def allowedCommandInputSourceKinds : List String := [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]"
            ),
            "Lean slice artifacts must enumerate allowed command input source kinds without read_model"
        );
        assert!(
            lean.contains(
                "def allowedRecoveryKinds : List String := [\"retry\",\"stay_on_screen\",\"navigation\",\"explicit_recovery_action\"]"
            ),
            "Lean slice artifacts must enumerate modeled recovery behavior kinds"
        );
        assert!(
            lean.contains(
                "def allowedSingletonRepeatBehaviors : List String := [\"already_exists_error\",\"idempotent\"]"
            ),
            "Lean slice artifacts must enumerate modeled singleton repeat behaviors"
        );
        assert!(lean.contains("def sliceEvents : List String := []"));
        assert!(lean.contains("def sliceStreams : List StreamDefinition := []"));
        assert!(lean.contains("def sliceExternalPayloads : List ExternalPayloadDefinition := []"));
        assert!(lean.contains("def sliceEventDefinitions : List EventDefinition := []"));
        assert!(
            lean.contains(
                "def storedEventFactSourceKinds : List String := [\"command_input\",\"external_payload\",\"generated\",\"session\",\"derivation\"]"
            ),
            "Lean slice artifacts must enumerate the source kinds that may feed stored event facts"
        );
        assert!(
            lean.contains(
                "def allowedEventAttributeSourceKinds : List String := storedEventFactSourceKinds"
            ),
            "Lean slice artifacts must constrain event attributes to stored event fact source kinds"
        );
        assert!(lean.contains("def sliceReadModels : List String := []"));
        assert!(lean.contains("def sliceReadModelDefinitions : List ReadModelDefinition := []"));
        assert!(
            lean.contains(
                "def allowedReadModelFieldSourceKinds : List String := [\"event_attribute\",\"derivation\",\"absence_default\"]"
            ),
            "Lean slice artifacts must enumerate read model field sources without command"
        );
        assert!(lean.contains("def sliceViews : List String := []"));
        assert!(lean.contains("def sliceViewDefinitions : List ViewDefinition := []"));
        assert!(
            lean.contains("def allowedViewFieldSourceKinds : List String := [\"read_model\"]"),
            "Lean slice artifacts must constrain displayed fields to read-model sources"
        );
        assert!(
            lean.contains(
                "def allowedControlInputSourceKinds : List String := [\"actor\",\"session\",\"generated\",\"external_payload\",\"event_stream_state\",\"invocation_argument\"]"
            ),
            "Lean slice artifacts must constrain control-provided inputs to modeled invocation/input sources"
        );
        assert!(
            lean.contains(
                "def allowedNavigationTargetTypes : List String := [\"modeled_view\",\"local_view_state\",\"external_system\",\"external_workflow\"]"
            ),
            "Lean slice artifacts must enumerate allowed navigation target types"
        );
        assert!(lean.contains("def sliceAcceptanceScenarios : List EventModelScenario := []"));
        assert!(lean.contains("def sliceContractScenarios : List EventModelScenario := []"));
        assert!(lean.contains("def sliceBitLevelDataFlows : List BitLevelDataFlow := []"));
        assert!(
            lean.contains(
                "def scenarioHasGwt (scenario : EventModelScenario) : Bool := scenario.name.isEmpty == false && scenario.givenSteps.isEmpty == false && scenario.whenSteps.isEmpty == false && scenario.thenSteps.isEmpty == false"
            ),
            "Lean slice artifacts must define first-class GWT scenario completeness"
        );
        assert!(
            lean.contains(
                "def sliceScenariosHaveGwt : Bool := sliceAcceptanceScenarios.all scenarioHasGwt && sliceContractScenarios.all scenarioHasGwt"
            ),
            "Lean slice artifacts must require both acceptance and contract scenarios to carry Given/When/Then"
        );
        assert!(
            lean.contains(
                "def scenarioNameCount (name : String) (scenarios : List EventModelScenario) : Nat := (scenarios.filter (fun scenario => scenario.name == name)).length"
            ),
            "Lean slice artifacts must define scenario-name cardinality over first-class scenarios"
        );
        assert!(
            lean.contains(
                "def scenarioNamesAreUnique (scenarios : List EventModelScenario) : Bool := scenarios.all (fun scenario => scenarioNameCount scenario.name scenarios == 1)"
            ),
            "Lean slice artifacts must define scenario-name uniqueness"
        );
        assert!(
            lean.contains(
                "def sliceScenarioNamesAreUnique : Bool := scenarioNamesAreUnique (sliceAcceptanceScenarios ++ sliceContractScenarios)"
            ),
            "Lean slice artifacts must require unique scenario names across acceptance and contract scenario sets"
        );
        assert!(
            lean.contains(
                "def scenarioStreamResolves (streamName : String) : Bool := sliceStreams.any (fun stream => stream.name == streamName)"
            ),
            "Lean slice artifacts must define scenario stream resolution against modeled streams"
        );
        assert!(
            lean.contains(
                "def scenarioStreamsResolve (scenario : EventModelScenario) : Bool := scenario.readStreams.all scenarioStreamResolves && scenario.writtenStreams.all scenarioStreamResolves"
            ),
            "Lean slice artifacts must require scenario read/write streams to resolve"
        );
        assert!(
            lean.contains(
                "def stateChangeScenarioNamesStreams (scenario : EventModelScenario) : Bool := sliceKind != \"state_change\" || (scenario.readStreams.isEmpty == false && scenario.writtenStreams.isEmpty == false)"
            ),
            "Lean slice artifacts must require state-change scenarios to name read and written streams"
        );
        assert!(
            lean.contains(
                "def sliceScenarioStreamsResolve : Bool := (sliceAcceptanceScenarios ++ sliceContractScenarios).all scenarioStreamsResolve"
            ),
            "Lean slice artifacts must prove all scenario stream references resolve"
        );
        assert!(
            lean.contains(
                "def stateChangeScenariosNameStreams : Bool := (sliceAcceptanceScenarios ++ sliceContractScenarios).all stateChangeScenarioNamesStreams"
            ),
            "Lean slice artifacts must prove state-change scenarios name stream reads and writes"
        );
        assert!(
            lean.contains(
                "def acceptanceScenariosAreUserFacing : Bool := sliceAcceptanceScenarios.all (fun scenario => scenario.contractKind.isEmpty && scenario.coveredDefinition.isEmpty)"
            ),
            "Lean slice artifacts must keep acceptance scenarios user-facing instead of contract-targeted"
        );
        assert!(
            lean.contains(
                "def scenarioCoversContract (contractKind : String) (definitionName : String) (scenario : EventModelScenario) : Bool := scenario.contractKind == contractKind && scenario.coveredDefinition == definitionName"
            ),
            "Lean slice artifacts must define contract scenario coverage by kind and formal definition"
        );
        assert!(
            lean.contains(
                "def readModelHasProjectorContract (readModel : ReadModelDefinition) : Bool := sliceContractScenarios.any (scenarioCoversContract \"projector\" readModel.name)"
            ),
            "Lean slice artifacts must define projector contract coverage for read models"
        );
        assert!(
            lean.contains(
                "def stateViewReadModelsHaveProjectorContracts : Bool := sliceKind != \"state_view\" || sliceReadModelDefinitions.all readModelHasProjectorContract"
            ),
            "Lean slice artifacts must require state-view read models to have projector contract scenarios"
        );
        assert!(
            lean.contains(
                "def contractScenarioTargetsKnownDefinition (scenario : EventModelScenario) : Bool := (scenario.contractKind == \"projector\" && (sliceReadModels.contains scenario.coveredDefinition || sliceReadModelDefinitions.any (fun readModel => readModel.name == scenario.coveredDefinition))) || (scenario.contractKind == \"command\" && (sliceCommands.contains scenario.coveredDefinition || sliceCommandDefinitions.any (fun command => command.name == scenario.coveredDefinition))) || (scenario.contractKind == \"automation\" && sliceAutomations.any (fun automation => automation.name == scenario.coveredDefinition)) || (scenario.contractKind == \"translation\" && sliceTranslations.any (fun translation => translation.name == scenario.coveredDefinition)) || (scenario.contractKind == \"derivation\" && scenario.coveredDefinition.isEmpty == false && sliceReadModelDefinitions.any (fun readModel => readModel.fields.any (fun field => field.sourceKind == \"derivation\" && field.derivationScenarioName == scenario.name))) || (scenario.contractKind == \"absence\" && scenario.coveredDefinition.isEmpty == false && sliceReadModelDefinitions.any (fun readModel => readModel.fields.any (fun field => field.sourceKind == \"absence_default\" && field.absenceScenarioName == scenario.name))) || (scenario.contractKind == \"transitive\" && sliceReadModelDefinitions.any (fun readModel => readModel.transitive && readModel.name == scenario.coveredDefinition && readModel.exampleScenarioName == scenario.name))"
            ),
            "Lean slice artifacts must resolve contract scenario targets to modeled definitions"
        );
        assert!(
            lean.contains(
                "def contractScenariosTargetKnownDefinitions : Bool := sliceContractScenarios.all contractScenarioTargetsKnownDefinition"
            ),
            "Lean slice artifacts must expose contract target resolution as a proof obligation"
        );
        assert!(
            lean.contains(
                "def commandInputHasAllowedSource (input : CommandInput) : Bool := allowedCommandInputSourceKinds.contains input.sourceKind"
            ),
            "Lean slice artifacts must reject read-model command input sources by construction"
        );
        assert!(
            lean.contains(
                "def commandInputHasProvenance (input : CommandInput) : Bool := input.name.isEmpty == false && input.sourceKind.isEmpty == false && input.sourceDescription.isEmpty == false && input.provenanceChain.isEmpty == false"
            ),
            "Lean slice artifacts must require reportable command input source chains"
        );
        assert!(
            lean.contains(
                "def commandInputTracesToInvocationSource (input : CommandInput) : Bool := allowedCommandInputSourceKinds.contains input.sourceKind && input.provenanceChain.isEmpty == false"
            ),
            "Lean slice artifacts must require command inputs to trace to modeled invocation source categories"
        );
        assert!(
            lean.contains(
                "def commandInputsHaveAllowedSources : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasAllowedSource)"
            ),
            "Lean slice artifacts must prove command inputs come from modeled invocation, actor, session, generated, external, or stream-derived state"
        );
        assert!(
            lean.contains(
                "def commandInputsHaveProvenance : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputHasProvenance)"
            ),
            "Lean slice artifacts must prove command inputs have source provenance"
        );
        assert!(
            lean.contains(
                "def commandInputsTraceToInvocationSources : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all commandInputTracesToInvocationSource)"
            ),
            "Lean slice artifacts must prove every command input traces to a modeled invocation/source chain"
        );
        assert!(
            lean.contains(
                "def commandInputEventStreamSourceResolves (command : CommandDefinition) (input : CommandInput) : Bool := input.sourceKind != \"event_stream_state\" || (command.observedStreams.isEmpty == false && command.observedStreams.all scenarioStreamResolves)"
            ),
            "Lean slice artifacts must require event-stream command inputs to name observed streams"
        );
        assert!(
            lean.contains(
                "def commandInputsSourcedFromEventStreamsResolve : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all (commandInputEventStreamSourceResolves command))"
            ),
            "Lean slice artifacts must prove event-stream command input sources resolve"
        );
        assert!(
            lean.contains(
                "def bitLevelFlowCoversTarget (target : String) (datum : String) : Bool := sliceBitLevelDataFlows.any (fun flow => flow.target == target && flow.datum == datum && flow.source.isEmpty == false && flow.transformationSemantics.isEmpty == false && flow.bitEncoding.isEmpty == false)"
            ),
            "Lean slice artifacts must define bit-level data-flow coverage by target and datum"
        );
        assert!(
            lean.contains(
                "def commandInputHasBitLevelFlow (command : CommandDefinition) (input : CommandInput) : Bool := bitLevelFlowCoversTarget command.name input.name"
            ),
            "Lean slice artifacts must connect command inputs to bit-level data-flow records"
        );
        assert!(
            lean.contains(
                "def commandErrorHasDeclaration (error : CommandErrorDefinition) : Bool := error.name.isEmpty == false && error.scenarioName.isEmpty == false && error.recoveryKind.isEmpty == false"
            ),
            "Lean slice artifacts must require command-local errors to be declared with scenario coverage and recovery kind"
        );
        assert!(
            lean.contains(
                "def commandErrorHasAllowedRecovery (error : CommandErrorDefinition) : Bool := allowedRecoveryKinds.contains error.recoveryKind"
            ),
            "Lean slice artifacts must restrict command-local error recovery to modeled recovery behavior"
        );
        assert!(
            lean.contains(
                "def commandErrorsAreDeclared : Bool := sliceCommandDefinitions.all (fun command => command.errors.all commandErrorHasDeclaration)"
            ),
            "Lean slice artifacts must prove command-local errors are declared"
        );
        assert!(
            lean.contains(
                "def commandErrorsHaveAllowedRecovery : Bool := sliceCommandDefinitions.all (fun command => command.errors.all commandErrorHasAllowedRecovery)"
            ),
            "Lean slice artifacts must prove command-local errors have modeled recovery behavior"
        );
        assert!(
            lean.contains(
                "def scenarioNameIsModeled (scenarioName : String) : Bool := (sliceAcceptanceScenarios ++ sliceContractScenarios).any (fun scenario => scenario.name == scenarioName)"
            ),
            "Lean slice artifacts must resolve command-error scenario coverage against modeled scenarios"
        );
        assert!(
            lean.contains(
                "def commandErrorHasScenarioCoverage (command : CommandDefinition) (error : CommandErrorDefinition) : Bool := sliceContractScenarios.any (fun scenario => scenario.name == error.scenarioName && scenario.contractKind == \"command\" && scenario.coveredDefinition == command.name && scenario.errorReferences.contains error.name)"
            ),
            "Lean slice artifacts must require command errors to be referenced by command contract scenarios"
        );
        assert!(
            lean.contains(
                "def commandErrorsHaveScenarioCoverage : Bool := sliceCommandDefinitions.all (fun command => command.errors.all (commandErrorHasScenarioCoverage command))"
            ),
            "Lean slice artifacts must prove command-error scenario references are declared by the command"
        );
        assert!(
            lean.contains(
                "def scenarioErrorReferenceIsDeclared (scenario : EventModelScenario) (errorName : String) : Bool := scenario.contractKind != \"command\" || sliceCommandDefinitions.any (fun command => command.name == scenario.coveredDefinition && command.errors.any (fun error => error.name == errorName))"
            ),
            "Lean slice artifacts must resolve command-contract scenario error references to declared command errors"
        );
        assert!(
            lean.contains(
                "def scenarioErrorReferencesAreDeclaredForScenario (scenario : EventModelScenario) : Bool := scenario.errorReferences.all (scenarioErrorReferenceIsDeclared scenario)"
            ),
            "Lean slice artifacts must define per-scenario error-reference declaration coverage"
        );
        assert!(
            lean.contains(
                "def scenarioErrorReferencesAreDeclared : Bool := sliceContractScenarios.all scenarioErrorReferencesAreDeclaredForScenario"
            ),
            "Lean slice artifacts must expose scenario error-reference declaration as a proof obligation"
        );
        assert!(
            lean.contains(
                "def singletonCommandDeclaresRepeatBehavior (command : CommandDefinition) : Bool := command.singleton == false || allowedSingletonRepeatBehaviors.contains command.repeatBehavior"
            ),
            "Lean slice artifacts must define the singleton repeat-behavior obligation on commands"
        );
        assert!(
            lean.contains(
                "def singletonCommandsDeclareRepeatBehavior : Bool := sliceCommandDefinitions.all singletonCommandDeclaresRepeatBehavior"
            ),
            "Lean slice artifacts must expose singleton repeat behavior as a proof obligation"
        );
        assert!(
            lean.contains(
                "def automationHasTrigger (automation : AutomationDefinition) : Bool := automation.name.isEmpty == false && automation.triggerName.isEmpty == false && automation.reactionDescription.isEmpty == false"
            ),
            "Lean slice artifacts must require automations to declare triggers and coherent reaction semantics"
        );
        assert!(
            lean.contains(
                "def automationIssuesKnownCommand (automation : AutomationDefinition) : Bool := sliceCommands.contains automation.commandName || sliceReferencedCommands.contains automation.commandName || sliceCommandDefinitions.any (fun command => command.name == automation.commandName)"
            ),
            "Lean slice artifacts must require automations to issue known commands"
        );
        assert!(
            lean.contains(
                "def automationHandlesCommandErrors (automation : AutomationDefinition) (command : CommandDefinition) : Bool := command.name != automation.commandName || command.errors.all (fun error => automation.handledErrors.contains error.name)"
            ),
            "Lean slice artifacts must require automations to handle every command error they can receive"
        );
        assert!(
            lean.contains(
                "def automationSlicesDeclareTriggers : Bool := sliceKind != \"automation\" || (sliceAutomations.isEmpty == false && sliceAutomations.all automationHasTrigger)"
            ),
            "Lean slice artifacts must prove automation slices declare at least one triggered automation"
        );
        assert!(
            lean.contains(
                "def automationSlicesRepresentOneReaction : Bool := sliceKind != \"automation\" || sliceAutomations.length == 1"
            ),
            "Lean slice artifacts must require automation slices to represent one coherent reaction"
        );
        assert!(
            lean.contains(
                "def automationsIssueKnownCommands : Bool := sliceAutomations.all automationIssuesKnownCommand"
            ),
            "Lean slice artifacts must prove automations issue modeled commands"
        );
        assert!(
            lean.contains(
                "def automationsHandleCommandErrors : Bool := sliceAutomations.all (fun automation => sliceCommandDefinitions.all (automationHandlesCommandErrors automation))"
            ),
            "Lean slice artifacts must prove automations handle command errors"
        );
        assert!(
            lean.contains(
                "def translationHasExternalContract (translation : TranslationDefinition) : Bool := translation.name.isEmpty == false && translation.externalEventName.isEmpty == false && translation.payloadContractName.isEmpty == false && sliceExternalPayloads.any (fun payload => payload.name == translation.payloadContractName)"
            ),
            "Lean slice artifacts must require translations to declare external events and payload contracts"
        );
        assert!(
            lean.contains(
                "def externalBoundaryHasPayloadContractAndFieldProvenance (translation : TranslationDefinition) : Bool := translationHasExternalContract translation && sliceExternalPayloads.any (fun payload => payload.name == translation.payloadContractName && payload.fields.isEmpty == false && payload.fields.all externalPayloadFieldHasProvenance)"
            ),
            "Lean slice artifacts must require each external boundary to bind a payload contract with field-level provenance"
        );
        assert!(
            lean.contains(
                "def externalBoundariesHavePayloadContractsAndFieldProvenance : Bool := sliceTranslations.all externalBoundaryHasPayloadContractAndFieldProvenance"
            ),
            "Lean slice artifacts must expose external boundary payload provenance as a proof obligation"
        );
        assert!(
            lean.contains(
                "def translationTargetsKnownCommand (translation : TranslationDefinition) : Bool := sliceCommands.contains translation.commandName || sliceReferencedCommands.contains translation.commandName || sliceCommandDefinitions.any (fun command => command.name == translation.commandName)"
            ),
            "Lean slice artifacts must require translations to target known commands"
        );
        assert!(
            lean.contains(
                "def translationReferencesObservedExternalEvent (translation : TranslationDefinition) : Bool := sliceEventDefinitions.any (fun event => event.name == translation.externalEventName && event.observed)"
            ),
            "Lean slice artifacts must require translations to reference observed external events"
        );
        assert!(
            lean.contains(
                "def translationSlicesDeclareExternalContracts : Bool := sliceKind != \"translation\" || (sliceTranslations.isEmpty == false && sliceTranslations.all translationHasExternalContract)"
            ),
            "Lean slice artifacts must prove translation slices declare at least one external contract"
        );
        assert!(
            lean.contains(
                "def translationsTargetKnownCommands : Bool := sliceTranslations.all translationTargetsKnownCommand"
            ),
            "Lean slice artifacts must prove translation commands resolve"
        );
        assert!(
            lean.contains(
                "def translationsReferenceObservedExternalEvents : Bool := sliceTranslations.all translationReferencesObservedExternalEvent"
            ),
            "Lean slice artifacts must expose observed external event resolution as a proof obligation"
        );
        assert!(
            lean.contains(
                "def boardElementLaneMatchesKind (element : BoardElement) : Bool := (element.kind == \"view\" && element.lane == \"ux\") || (element.kind == \"automation\" && element.lane == \"ux\") || (element.kind == \"external_event\" && element.lane == \"ux\") || (element.kind == \"command\" && element.lane == \"actions\") || (element.kind == \"read_model\" && element.lane == \"actions\") || (element.kind == \"event\" && element.lane == \"events\")"
            ),
            "Lean slice artifacts must encode canonical board lane semantics"
        );
        assert!(
            lean.contains(
                "def boardElementReferencesDeclaration (element : BoardElement) : Bool := (element.kind == \"view\" && (sliceViews.contains element.declaredName || sliceViewDefinitions.any (fun view => view.name == element.declaredName))) || (element.kind == \"automation\" && sliceAutomations.any (fun automation => automation.name == element.declaredName)) || (element.kind == \"external_event\" && sliceEventDefinitions.any (fun event => event.name == element.declaredName && event.observed)) || (element.kind == \"command\" && (sliceCommands.contains element.declaredName || sliceReferencedCommands.contains element.declaredName || sliceCommandDefinitions.any (fun command => command.name == element.declaredName))) || (element.kind == \"read_model\" && (sliceReadModels.contains element.declaredName || sliceReadModelDefinitions.any (fun readModel => readModel.name == element.declaredName))) || (element.kind == \"event\" && (sliceEvents.contains element.declaredName || sliceEventDefinitions.any (fun event => event.name == element.declaredName && (event.observed || event.shared))))"
            ),
            "Lean slice artifacts must require board elements to reference real declarations"
        );
        assert!(
            lean.contains(
                "def automationBoardElementIsDeclaredAutomation (element : BoardElement) : Bool := element.kind != \"automation\" || sliceAutomations.any (fun automation => automation.name == element.declaredName)"
            ),
            "Lean slice artifacts must require automation board elements to resolve to declared automations"
        );
        assert!(
            lean.contains(
                "def automationBoardElementsAreDeclaredAutomations : Bool := sliceBoardElements.all automationBoardElementIsDeclaredAutomation"
            ),
            "Lean slice artifacts must expose automation board modeling as a proof obligation"
        );
        assert!(
            lean.contains(
                "def externalBoardElementIsObservedEvent (element : BoardElement) : Bool := element.kind != \"external_event\" || sliceEventDefinitions.any (fun event => event.name == element.declaredName && event.observed)"
            ),
            "Lean slice artifacts must require external-event board elements to resolve to observed events"
        );
        assert!(
            lean.contains(
                "def externalBoardElementsAreObservedEvents : Bool := sliceBoardElements.all externalBoardElementIsObservedEvent"
            ),
            "Lean slice artifacts must expose external-event board modeling as a proof obligation"
        );
        assert!(
            lean.contains(
                "def boardConnectionHasAllowedShape (connection : BoardConnection) : Bool := (connection.sourceKind == \"view\" && connection.targetKind == \"command\") || (connection.sourceKind == \"automation\" && connection.targetKind == \"command\") || (connection.sourceKind == \"external_event\" && connection.targetKind == \"command\") || (connection.sourceKind == \"workflow_trigger\" && connection.targetKind == \"command\") || (connection.sourceKind == \"command\" && connection.targetKind == \"event\") || (connection.sourceKind == \"event\" && connection.targetKind == \"read_model\") || (connection.sourceKind == \"read_model\" && connection.targetKind == \"view\")"
            ),
            "Lean slice artifacts must encode causal board connection shapes"
        );
        assert!(
            lean.contains(
                "def commandEventBoardEdgeMatchesEmission (connection : BoardConnection) : Bool := connection.sourceKind != \"command\" || connection.targetKind != \"event\" || sliceCommandDefinitions.any (fun command => command.name == connection.source && command.emittedEvents.contains connection.target)"
            ),
            "Lean slice artifacts must require command-to-event board edges to match command emissions"
        );
        assert!(
            lean.contains(
                "def commandEventBoardEdgesMatchEmissions : Bool := sliceBoardConnections.all commandEventBoardEdgeMatchesEmission"
            ),
            "Lean slice artifacts must expose command-to-event board emission matching as a proof obligation"
        );
        assert!(
            lean.contains(
                "def eventReadModelBoardEdgeMatchesProjection (connection : BoardConnection) : Bool := connection.sourceKind != \"event\" || connection.targetKind != \"read_model\" || sliceReadModelDefinitions.any (fun readModel => readModel.name == connection.target && readModel.fields.any (fun field => field.sourceEvent == connection.source))"
            ),
            "Lean slice artifacts must require event-to-read-model edges to match projection sources"
        );
        assert!(
            lean.contains(
                "def eventReadModelBoardEdgesMatchProjectionSources : Bool := sliceBoardConnections.all eventReadModelBoardEdgeMatchesProjection"
            ),
            "Lean slice artifacts must expose event-to-read-model board projection matching as a proof obligation"
        );
        assert!(
            lean.contains(
                "def externalEventCommandBoardEdgeMatchesTranslation (connection : BoardConnection) : Bool := connection.sourceKind != \"external_event\" || connection.targetKind != \"command\" || sliceTranslations.any (fun translation => translation.externalEventName == connection.source && translation.commandName == connection.target)"
            ),
            "Lean slice artifacts must require external-event command edges to match translations"
        );
        assert!(
            lean.contains(
                "def externalEventTriggersMatchTranslations : Bool := sliceBoardConnections.all externalEventCommandBoardEdgeMatchesTranslation"
            ),
            "Lean slice artifacts must expose external-event translation trigger matching as a proof obligation"
        );
        assert!(
            lean.contains(
                "def externalEventDoesNotUpdateReadModel (connection : BoardConnection) : Bool := connection.sourceKind != \"event\" || connection.targetKind != \"read_model\" || sliceEventDefinitions.any (fun event => event.name == connection.source && event.observed) == false"
            ),
            "Lean slice artifacts must reject direct external-event updates to read models"
        );
        assert!(
            lean.contains(
                "def externalEventsDoNotUpdateReadModels : Bool := sliceBoardConnections.all externalEventDoesNotUpdateReadModel"
            ),
            "Lean slice artifacts must expose external-event read-model isolation as a proof obligation"
        );
        assert!(
            lean.contains(
                "def viewCommandBoardEdgeMatchesControl (connection : BoardConnection) : Bool := connection.sourceKind != \"view\" || connection.targetKind != \"command\" || sliceViewDefinitions.any (fun view => view.name == connection.source && view.controls.any (fun control => control.commandName == connection.target))"
            ),
            "Lean slice artifacts must require view-to-command edges to match controls"
        );
        assert!(
            lean.contains(
                "def viewCommandBoardEdgesMatchControls : Bool := sliceBoardConnections.all viewCommandBoardEdgeMatchesControl"
            ),
            "Lean slice artifacts must expose view-to-command board control matching as a proof obligation"
        );
        assert!(
            lean.contains(
                "def boardLanesAreCanonical : Bool := canonicalBoardLanes == [\"ux\",\"actions\",\"events\"]"
            ),
            "Lean slice artifacts must prove the canonical board lanes"
        );
        assert!(
            lean.contains(
                "def boardElementsUseCanonicalLanes : Bool := sliceBoardElements.all (fun element => canonicalBoardLanes.contains element.lane && boardElementLaneMatchesKind element)"
            ),
            "Lean slice artifacts must prove board elements use canonical lanes by kind"
        );
        assert!(
            lean.contains(
                "def boardElementsReferenceDeclarations : Bool := sliceBoardElements.all boardElementReferencesDeclaration"
            ),
            "Lean slice artifacts must prove board elements reference modeled declarations"
        );
        assert!(
            lean.contains(
                "def boardConnectionsHaveCausalSemantics : Bool := sliceBoardConnections.all (fun connection => boardConnectionHasAllowedShape connection && commandEventBoardEdgeMatchesEmission connection && eventReadModelBoardEdgeMatchesProjection connection && externalEventCommandBoardEdgeMatchesTranslation connection && externalEventDoesNotUpdateReadModel connection && viewCommandBoardEdgeMatchesControl connection)"
            ),
            "Lean slice artifacts must prove board connections match causal semantics"
        );
        assert!(
            lean.contains(
                "def readModelsDoNotFeedCommands : Bool := sliceBoardConnections.all (fun connection => connection.sourceKind != \"read_model\" || connection.targetKind != \"command\")"
            ),
            "Lean slice artifacts must prove read models do not feed commands"
        );
        assert!(
            lean.contains(
                "def readModelViewConnectionHasIncomingEventUpdate (connection : BoardConnection) : Bool := connection.sourceKind != \"read_model\" || connection.targetKind != \"view\" || sliceBoardConnections.any (fun incoming => incoming.target == connection.source && incoming.targetKind == \"read_model\" && incoming.sourceKind == \"event\")"
            ),
            "Lean slice artifacts must define the read-model-to-view incoming event update obligation"
        );
        assert!(
            lean.contains(
                "def readModelsFeedingViewsHaveIncomingEventUpdates : Bool := sliceBoardConnections.all readModelViewConnectionHasIncomingEventUpdate"
            ),
            "Lean slice artifacts must prove read models feeding views have incoming event updates"
        );
        assert!(
            lean.contains(
                "def commandsHaveIncomingTriggers : Bool := sliceBoardElements.all (fun element => element.kind != \"command\" || sliceBoardConnections.any (fun connection => connection.target == element.name && connection.targetKind == \"command\" && (connection.sourceKind == \"view\" || connection.sourceKind == \"automation\" || connection.sourceKind == \"external_event\" || connection.sourceKind == \"workflow_trigger\")))"
            ),
            "Lean slice artifacts must prove commands have real incoming triggers"
        );
        assert!(
            lean.contains(
                "def mainPathBoardHasNoDisconnectedIslands : Bool := sliceBoardElements.all (fun element => element.mainPath == false || sliceBoardConnections.any (fun connection => connection.source == element.name || connection.target == element.name))"
            ),
            "Lean slice artifacts must prove main-path board elements are connected"
        );
        assert!(
            lean.contains(
                "def outcomeLabelsAreUnique : Bool := sliceOutcomeDefinitions.all (fun outcome => outcomeLabelCount outcome.label == 1)"
            ),
            "Lean slice artifacts must require unique outcome labels within a slice"
        );
        assert!(
            lean.contains(
                "def outcomeEventSetsAreNonEmpty : Bool := sliceOutcomeDefinitions.all (fun outcome => outcome.eventSet.isEmpty == false)"
            ),
            "Lean slice artifacts must require every outcome to be backed by at least one event"
        );
        assert!(
            lean.contains(
                "def outcomeEventSetsAreDistinct : Bool := sliceOutcomeDefinitions.all (fun outcome => sliceOutcomeDefinitions.all (fun other => outcome.label == other.label || sameOutcomeEventSet outcome other == false))"
            ),
            "Lean slice artifacts must reject distinct outcomes backed by the same event set regardless of order"
        );
        assert!(
            lean.contains(
                "def outcomeEventsAreKnownToSlice : Bool := sliceOutcomeDefinitions.all (fun outcome => outcome.eventSet.all eventIsKnownToSlice)"
            ),
            "Lean slice artifacts must require outcome events to be emitted or observed by the slice"
        );
        assert!(
            lean.contains(
                "def eventReferencesKnownStream (event : EventDefinition) : Bool := sliceStreams.any (fun stream => stream.name == event.stream)"
            ),
            "Lean slice artifacts must require events to reference known streams"
        );
        assert!(
            lean.contains(
                "def eventAttributeHasAllowedSource (eventAttribute : EventAttribute) : Bool := allowedEventAttributeSourceKinds.contains eventAttribute.sourceKind"
            ),
            "Lean slice artifacts must reject read-model event attribute sources"
        );
        assert!(
            lean.contains(
                "def eventAttributeHasProvenance (eventAttribute : EventAttribute) : Bool := eventAttribute.name.isEmpty == false && eventAttribute.sourceKind.isEmpty == false && eventAttribute.sourceName.isEmpty == false && eventAttribute.provenanceDescription.isEmpty == false"
            ),
            "Lean slice artifacts must require event attributes to declare source provenance"
        );
        assert!(
            lean.contains(
                "def commandEmittedEventIsKnown (eventName : String) : Bool := sliceEvents.contains eventName || sliceEventDefinitions.any (fun event => event.name == eventName)"
            ),
            "Lean slice artifacts must require command-emitted events to resolve to modeled events"
        );
        assert!(
            lean.contains(
                "def eventProducedByCommand (event : EventDefinition) : Bool := event.observed || event.shared || sliceCommandDefinitions.any (fun command => command.emittedEvents.contains event.name)"
            ),
            "Lean slice artifacts must require local events to be produced by commands unless observed or shared"
        );
        assert!(
            lean.contains(
                "def commandInputReferencesAttributeSource (event : EventDefinition) (eventAttribute : EventAttribute) (command : CommandDefinition) : Bool := command.emittedEvents.contains event.name && command.inputs.any (fun input => input.name == eventAttribute.sourceName)"
            ),
            "Lean slice artifacts must connect command-sourced event attributes to inputs of commands that emit the event"
        );
        assert!(
            lean.contains(
                "def externalPayloadFieldHasProvenance (field : ExternalPayloadField) : Bool := field.name.isEmpty == false && field.provenanceDescription.isEmpty == false && field.bitEncoding.isEmpty == false"
            ),
            "Lean slice artifacts must require external payload fields to carry provenance and bit encoding"
        );
        assert!(
            lean.contains(
                "def externalPayloadFieldsHaveProvenance : Bool := sliceExternalPayloads.all (fun payload => payload.name.isEmpty == false && payload.fields.all externalPayloadFieldHasProvenance)"
            ),
            "Lean slice artifacts must check all external payload fields for provenance and bit encoding"
        );
        assert!(
            lean.contains(
                "def externalPayloadFieldIsDeclared (eventAttribute : EventAttribute) : Bool := sliceExternalPayloads.any (fun payload => payload.name == eventAttribute.sourceName && payload.fields.any (fun field => field.name == eventAttribute.sourceField))"
            ),
            "Lean slice artifacts must connect external-sourced event attributes to declared payload fields"
        );
        assert!(
            lean.contains(
                "def eventAttributeSourceIsComplete (event : EventDefinition) (eventAttribute : EventAttribute) : Bool := (eventAttribute.sourceKind == \"command_input\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false && sliceCommandDefinitions.any (commandInputReferencesAttributeSource event eventAttribute)) || (eventAttribute.sourceKind == \"external_payload\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false && externalPayloadFieldIsDeclared eventAttribute) || (eventAttribute.sourceKind == \"generated\" && eventAttribute.sourceName.isEmpty == false) || (eventAttribute.sourceKind == \"session\" && eventAttribute.sourceName.isEmpty == false) || (eventAttribute.sourceKind == \"derivation\" && eventAttribute.sourceName.isEmpty == false && eventAttribute.sourceField.isEmpty == false)"
            ),
            "Lean slice artifacts must require event attribute source details to be complete for each source kind"
        );
        assert!(
            lean.contains(
                "def eventAttributeTracesToStoredFactSource (eventAttribute : EventAttribute) : Bool := storedEventFactSourceKinds.contains eventAttribute.sourceKind"
            ),
            "Lean slice artifacts must classify event attributes by modeled stored-fact source kinds"
        );
        assert!(
            lean.contains(
                "def eventsReferenceKnownStreams : Bool := sliceEventDefinitions.all eventReferencesKnownStream"
            ),
            "Lean slice artifacts must prove event stream references resolve"
        );
        assert!(
            lean.contains(
                "def commandEmittedEventsAreKnown : Bool := sliceCommandDefinitions.all (fun command => command.emittedEvents.all commandEmittedEventIsKnown)"
            ),
            "Lean slice artifacts must prove command-emitted event names resolve"
        );
        assert!(
            lean.contains(
                "def locallyEmittedEventsAreProducedByCommands : Bool := sliceEventDefinitions.all eventProducedByCommand"
            ),
            "Lean slice artifacts must prove local events are produced by commands unless observed or shared"
        );
        assert!(
            lean.contains(
                "def eventAttributesHaveAllowedSources : Bool := sliceEventDefinitions.all (fun event => event.attributes.all eventAttributeHasAllowedSource)"
            ),
            "Lean slice artifacts must prove event attributes do not source from read models"
        );
        assert!(
            lean.contains(
                "def eventAttributesHaveProvenance : Bool := sliceEventDefinitions.all (fun event => event.attributes.all eventAttributeHasProvenance)"
            ),
            "Lean slice artifacts must prove event attributes carry source provenance"
        );
        assert!(
            lean.contains(
                "def eventAttributeSourcesAreComplete : Bool := sliceEventDefinitions.all (fun event => event.attributes.all (eventAttributeSourceIsComplete event))"
            ),
            "Lean slice artifacts must prove event attribute sources are complete"
        );
        assert!(
            lean.contains(
                "def storedEventFactsTraceToOriginalSources : Bool := sliceEventDefinitions.all (fun event => event.attributes.all eventAttributeTracesToStoredFactSource)"
            ),
            "Lean slice artifacts must prove every stored event fact traces to an original modeled source"
        );
        assert!(
            lean.contains(
                "def eventAttributeHasBitLevelFlow (event : EventDefinition) (eventAttribute : EventAttribute) : Bool := bitLevelFlowCoversTarget event.name eventAttribute.name"
            ),
            "Lean slice artifacts must connect event attributes to bit-level data-flow records"
        );
        assert!(
            lean.contains(
                "def readModelFieldHasAllowedSource (field : ReadModelField) : Bool := allowedReadModelFieldSourceKinds.contains field.sourceKind"
            ),
            "Lean slice artifacts must reject command-sourced read model fields"
        );
        assert!(
            lean.contains(
                "def readModelFieldHasProvenance (field : ReadModelField) : Bool := field.name.isEmpty == false && field.sourceKind.isEmpty == false && field.provenanceDescription.isEmpty == false"
            ),
            "Lean slice artifacts must require read model field provenance"
        );
        assert!(
            lean.contains(
                "def readModelFieldSourceIsComplete (field : ReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && field.sourceEvent.isEmpty == false && field.sourceAttribute.isEmpty == false) || (field.sourceKind == \"derivation\" && field.derivationRule.isEmpty == false) || (field.sourceKind == \"absence_default\" && field.absenceEvent.isEmpty == false)"
            ),
            "Lean slice artifacts must require read model fields to trace to event facts, derivation, or modeled absence"
        );
        assert!(
            lean.contains(
                "def eventAttributeIsDeclared (eventName : String) (attributeName : String) : Bool := sliceEventDefinitions.any (fun event => event.name == eventName && event.attributes.any (fun eventAttribute => eventAttribute.name == attributeName))"
            ),
            "Lean slice artifacts must model declared event attributes as first-class read-model sources"
        );
        assert!(
            lean.contains(
                "def readModelFieldEventAttributeSourceResolves (field : ReadModelField) : Bool := field.sourceKind != \"event_attribute\" || eventAttributeIsDeclared field.sourceEvent field.sourceAttribute"
            ),
            "Lean slice artifacts must require event-sourced read model fields to resolve to declared event attributes"
        );
        assert!(
            lean.contains(
                "def readModelFieldDerivationScenarioIsCovered (field : ReadModelField) : Bool := field.sourceKind != \"derivation\" || (field.derivationScenarioName.isEmpty == false && scenarioNameIsModeled field.derivationScenarioName)"
            ),
            "Lean slice artifacts must require derived read model fields to name modeled scenario coverage"
        );
        assert!(
            lean.contains(
                "def readModelFieldAbsenceScenarioIsCovered (field : ReadModelField) : Bool := field.sourceKind != \"absence_default\" || (field.absenceScenarioName.isEmpty == false && scenarioNameIsModeled field.absenceScenarioName)"
            ),
            "Lean slice artifacts must require absence/default read model fields to name modeled scenario coverage"
        );
        assert!(
            lean.contains(
                "def readModelFieldsHaveAllowedSources : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldHasAllowedSource)"
            ),
            "Lean slice artifacts must prove read model fields do not source from commands"
        );
        assert!(
            lean.contains(
                "def readModelFieldsHaveProvenance : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldHasProvenance)"
            ),
            "Lean slice artifacts must prove read model fields carry provenance"
        );
        assert!(
            lean.contains(
                "def readModelFieldSourcesAreComplete : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldSourceIsComplete)"
            ),
            "Lean slice artifacts must prove read model field sources are complete"
        );
        assert!(
            lean.contains(
                "def readModelFieldEventAttributeSourcesResolve : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldEventAttributeSourceResolves)"
            ),
            "Lean slice artifacts must prove event-sourced read model fields resolve"
        );
        assert!(
            lean.contains(
                "def derivedReadModelFieldsHaveScenarioCoverage : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldDerivationScenarioIsCovered)"
            ),
            "Lean slice artifacts must prove derived read model fields have scenario coverage"
        );
        assert!(
            lean.contains(
                "def absenceReadModelFieldsHaveScenarioCoverage : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all readModelFieldAbsenceScenarioIsCovered)"
            ),
            "Lean slice artifacts must prove absence/default read model fields have scenario coverage"
        );
        assert!(
            lean.contains(
                "def transitiveReadModelHasSemantics (readModel : ReadModelDefinition) : Bool := readModel.transitive == false || (readModel.relationshipFields.isEmpty == false && readModel.transitiveRule.isEmpty == false && readModel.exampleScenarioName.isEmpty == false && scenarioNameIsModeled readModel.exampleScenarioName)"
            ),
            "Lean slice artifacts must require transitive read models to declare relationship fields, transitive rule, and modeled example scenario"
        );
        assert!(
            lean.contains(
                "def transitiveReadModelsHaveSemantics : Bool := sliceReadModelDefinitions.all transitiveReadModelHasSemantics"
            ),
            "Lean slice artifacts must prove transitive read models have complete semantics"
        );
        assert!(
            lean.contains(
                "def readModelFieldHasBitLevelFlow (readModel : ReadModelDefinition) (field : ReadModelField) : Bool := bitLevelFlowCoversTarget readModel.name field.name"
            ),
            "Lean slice artifacts must connect read model fields to bit-level data-flow records"
        );
        assert!(
            lean.contains(
                "def viewFieldHasAllowedSource (field : ViewField) : Bool := allowedViewFieldSourceKinds.contains field.sourceKind"
            ),
            "Lean slice artifacts must reject direct event-sourced displayed fields"
        );
        assert!(
            lean.contains(
                "def viewFieldHasProvenance (field : ViewField) : Bool := field.name.isEmpty == false && field.sourceKind.isEmpty == false && field.provenanceDescription.isEmpty == false && field.bitEncoding.isEmpty == false"
            ),
            "Lean slice artifacts must require displayed view fields to carry provenance and bit encoding"
        );
        assert!(
            lean.contains(
                "def viewFieldSourceIsComplete (field : ViewField) : Bool := field.sourceKind == \"read_model\" && field.sourceReadModel.isEmpty == false && field.sourceField.isEmpty == false && field.sketchToken.isEmpty == false"
            ),
            "Lean slice artifacts must require displayed fields to trace to read model fields and sketch tokens"
        );
        assert!(
            lean.contains(
                "def viewFieldSourceReadModelIsUsed (view : ViewDefinition) (field : ViewField) : Bool := view.readModels.contains field.sourceReadModel && sliceReadModels.contains field.sourceReadModel"
            ),
            "Lean slice artifacts must require view fields to source from read models used by the view"
        );
        assert!(
            lean.contains(
                "def readModelFieldIsDeclared (readModelName : String) (fieldName : String) : Bool := sliceReadModelDefinitions.any (fun readModel => readModel.name == readModelName && readModel.fields.any (fun readModelField => readModelField.name == fieldName))"
            ),
            "Lean slice artifacts must model declared read model fields as first-class view field sources"
        );
        assert!(
            lean.contains(
                "def viewFieldSourceReadModelFieldResolves (field : ViewField) : Bool := field.sourceKind != \"read_model\" || readModelFieldIsDeclared field.sourceReadModel field.sourceField"
            ),
            "Lean slice artifacts must require view fields to resolve to declared read model fields"
        );
        assert!(
            lean.contains(
                "def readModelFieldHasOriginalProvenance (field : ReadModelField) : Bool := (field.sourceKind == \"event_attribute\" && readModelFieldEventAttributeSourceResolves field) || field.sourceKind == \"derivation\" || field.sourceKind == \"absence_default\""
            ),
            "Lean slice artifacts must classify read model fields by original modeled provenance"
        );
        assert!(
            lean.contains(
                "def viewFieldTracesToOriginalProvenance (field : ViewField) : Bool := field.sourceKind == \"read_model\" && sliceReadModelDefinitions.any (fun readModel => readModel.name == field.sourceReadModel && readModel.fields.any (fun readModelField => readModelField.name == field.sourceField && readModelFieldHasOriginalProvenance readModelField))"
            ),
            "Lean slice artifacts must trace displayed fields through read model fields to original provenance"
        );
        assert!(
            lean.contains(
                "def viewFieldsHaveAllowedSources : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldHasAllowedSource)"
            ),
            "Lean slice artifacts must prove displayed fields do not source directly from events"
        );
        assert!(
            lean.contains(
                "def viewFieldsHaveProvenance : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldHasProvenance)"
            ),
            "Lean slice artifacts must prove displayed fields carry source-chain provenance"
        );
        assert!(
            lean.contains(
                "def viewFieldSourcesAreComplete : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldSourceIsComplete)"
            ),
            "Lean slice artifacts must prove displayed field sources are complete"
        );
        assert!(
            lean.contains(
                "def viewFieldsSourceFromUsedReadModels : Bool := sliceViewDefinitions.all (fun view => view.fields.all (viewFieldSourceReadModelIsUsed view))"
            ),
            "Lean slice artifacts must prove displayed fields source from read models used by the owning view"
        );
        assert!(
            lean.contains(
                "def viewFieldAppearsInSketch (view : ViewDefinition) (field : ViewField) : Bool := field.sketchToken.isEmpty == false && view.sketchTokens.contains field.sketchToken"
            ),
            "Lean slice artifacts must require displayed fields to use declared sketch tokens"
        );
        assert!(
            lean.contains(
                "def viewHasInformationSketch (view : ViewDefinition) : Bool := view.sketchTokens.isEmpty == false"
            ),
            "Lean slice artifacts must require views to carry an information sketch"
        );
        assert!(
            lean.contains(
                "def viewsHaveInformationSketches : Bool := sliceViewDefinitions.all viewHasInformationSketch"
            ),
            "Lean slice artifacts must prove every modeled view has an information sketch"
        );
        assert!(
            lean.contains(
                "def viewFieldsAppearInSketch : Bool := sliceViewDefinitions.all (fun view => view.fields.all (viewFieldAppearsInSketch view))"
            ),
            "Lean slice artifacts must prove every displayed field appears in its view sketch"
        );
        assert!(
            lean.contains(
                "def sketchTokenMapsToModeledElement (view : ViewDefinition) (token : String) : Bool := view.fields.any (fun field => field.sketchToken == token) || view.controls.any (fun control => control.sketchToken == token || control.inputs.any (fun input => input.sourceKind == \"actor\" && input.sketchToken == token))"
            ),
            "Lean slice artifacts must require sketch tokens to map to modeled fields, controls, or actor inputs"
        );
        assert!(
            lean.contains(
                "def viewSketchTokensMapToModeledElements : Bool := sliceViewDefinitions.all (fun view => view.sketchTokens.all (sketchTokenMapsToModeledElement view))"
            ),
            "Lean slice artifacts must prove every sketch token maps to a modeled element"
        );
        assert!(
            lean.contains(
                "def viewFieldReadModelFieldSourcesResolve : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldSourceReadModelFieldResolves)"
            ),
            "Lean slice artifacts must prove displayed fields resolve to declared read model fields"
        );
        assert!(
            lean.contains(
                "def displayedDataTraceToOriginalProvenance : Bool := sliceViewDefinitions.all (fun view => view.fields.all viewFieldTracesToOriginalProvenance)"
            ),
            "Lean slice artifacts must prove displayed data traces transitively to original provenance"
        );
        assert!(
            lean.contains(
                "def viewFieldHasBitLevelFlow (view : ViewDefinition) (field : ViewField) : Bool := bitLevelFlowCoversTarget view.name field.name"
            ),
            "Lean slice artifacts must connect displayed fields to bit-level data-flow records"
        );
        assert!(
            lean.contains(
                "def commandInputDataFlowsAreComplete : Bool := sliceCommandDefinitions.all (fun command => command.inputs.all (commandInputHasBitLevelFlow command))"
            ),
            "Lean slice artifacts must prove every command input has bit-level flow semantics"
        );
        assert!(
            lean.contains(
                "def eventAttributeDataFlowsAreComplete : Bool := sliceEventDefinitions.all (fun event => event.attributes.all (eventAttributeHasBitLevelFlow event))"
            ),
            "Lean slice artifacts must prove every event attribute has bit-level flow semantics"
        );
        assert!(
            lean.contains(
                "def readModelFieldDataFlowsAreComplete : Bool := sliceReadModelDefinitions.all (fun readModel => readModel.fields.all (readModelFieldHasBitLevelFlow readModel))"
            ),
            "Lean slice artifacts must prove every read model field has bit-level flow semantics"
        );
        assert!(
            lean.contains(
                "def viewFieldDataFlowsAreComplete : Bool := sliceViewDefinitions.all (fun view => view.fields.all (viewFieldHasBitLevelFlow view))"
            ),
            "Lean slice artifacts must prove every displayed field has bit-level flow semantics"
        );
        assert!(
            lean.contains(
                "def modeledDataFlowsAreBitComplete : Bool := commandInputDataFlowsAreComplete && eventAttributeDataFlowsAreComplete && readModelFieldDataFlowsAreComplete && viewFieldDataFlowsAreComplete"
            ),
            "Lean slice artifacts must aggregate bit-level information-completeness obligations for modeled data"
        );
        assert!(
            lean.contains(
                "def controlInputHasAllowedSource (input : ControlInputProvision) : Bool := allowedControlInputSourceKinds.contains input.sourceKind"
            ),
            "Lean slice artifacts must constrain control-provided command inputs"
        );
        assert!(
            lean.contains(
                "def controlInputHasProvenance (input : ControlInputProvision) : Bool := input.name.isEmpty == false && input.sourceKind.isEmpty == false && input.sourceDescription.isEmpty == false"
            ),
            "Lean slice artifacts must require control input provenance and descriptions"
        );
        assert!(
            lean.contains(
                "def controlInputVisibilityIsModeled (input : ControlInputProvision) : Bool := (input.sourceKind != \"actor\" || input.sketchToken.isEmpty == false || input.visibleToActor) && (input.decisionField == false || input.sketchToken.isEmpty == false || input.visibleToActor)"
            ),
            "Lean slice artifacts must require actor and decision inputs to be visible in the sketch"
        );
        assert!(
            lean.contains(
                "def controlHasSketchToken (control : ControlDefinition) : Bool := control.name.isEmpty == false && control.commandName.isEmpty == false && control.sketchToken.isEmpty == false"
            ),
            "Lean slice artifacts must require controls to appear in the information sketch"
        );
        assert!(
            lean.contains(
                "def controlReferencesKnownCommand (control : ControlDefinition) : Bool := sliceCommands.contains control.commandName || sliceReferencedCommands.contains control.commandName || sliceCommandDefinitions.any (fun command => command.name == control.commandName)"
            ),
            "Lean slice artifacts must require controls to reference known commands"
        );
        assert!(
            lean.contains(
                "def commandErrorsHandledByControl (control : ControlDefinition) (command : CommandDefinition) : Bool := command.name != control.commandName || command.errors.all (fun error => control.handledErrors.contains error.name && control.recoveryBehavior.isEmpty == false)"
            ),
            "Lean slice artifacts must require controls to handle every returned command error with recovery behavior"
        );
        assert!(
            lean.contains(
                "def controlRecoveryBehaviorIsModeled (control : ControlDefinition) : Bool := control.handledErrors.isEmpty || allowedRecoveryKinds.contains control.recoveryBehavior"
            ),
            "Lean slice artifacts must require controls that handle errors to use modeled recovery behavior"
        );
        assert!(
            lean.contains(
                "def navigationTargetTypeIsModeled (target : NavigationTarget) : Bool := target.targetType.isEmpty || allowedNavigationTargetTypes.contains target.targetType"
            ),
            "Lean slice artifacts must require navigation targets to use modeled target types"
        );
        assert!(
            lean.contains(
                "def navigationTargetIsComplete (view : ViewDefinition) (target : NavigationTarget) : Bool := (target.targetType.isEmpty && target.targetName.isEmpty && target.externalWorkflowName.isEmpty && target.externalSystemName.isEmpty && target.handoffContract.isEmpty) || (target.targetType == \"modeled_view\" && target.targetName.isEmpty == false && sliceViews.contains target.targetName) || (target.targetType == \"local_view_state\" && target.targetName.isEmpty == false && view.localStates.contains target.targetName) || (target.targetType == \"external_workflow\" && target.externalWorkflowName.isEmpty == false) || (target.targetType == \"external_system\" && target.externalSystemName.isEmpty == false && target.handoffContract.isEmpty == false)"
            ),
            "Lean slice artifacts must require navigation targets to be complete for their target type"
        );
        assert!(
            lean.contains(
                "def viewControlNavigationTypesAreModeled : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => navigationTargetTypeIsModeled control.navigation))"
            ),
            "Lean slice artifacts must prove control navigation target types are modeled"
        );
        assert!(
            lean.contains(
                "def viewControlNavigationTargetsAreComplete : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => navigationTargetIsComplete view control.navigation))"
            ),
            "Lean slice artifacts must prove control navigation targets are complete"
        );
        assert!(
            lean.contains(
                "def viewControlsHaveSketchTokens : Bool := sliceViewDefinitions.all (fun view => view.controls.all controlHasSketchToken)"
            ),
            "Lean slice artifacts must prove every modeled control appears in a sketch"
        );
        assert!(
            lean.contains(
                "def controlAppearsInSketch (view : ViewDefinition) (control : ControlDefinition) : Bool := control.sketchToken.isEmpty == false && view.sketchTokens.contains control.sketchToken"
            ),
            "Lean slice artifacts must require controls to use declared sketch tokens"
        );
        assert!(
            lean.contains(
                "def viewControlsAppearInSketch : Bool := sliceViewDefinitions.all (fun view => view.controls.all (controlAppearsInSketch view))"
            ),
            "Lean slice artifacts must prove every modeled control appears in its view sketch"
        );
        assert!(
            lean.contains(
                "def viewControlsReferenceKnownCommands : Bool := sliceViewDefinitions.all (fun view => view.controls.all controlReferencesKnownCommand)"
            ),
            "Lean slice artifacts must prove every control references a known command"
        );
        assert!(
            lean.contains(
                "def controlProvidesCommandInput (control : ControlDefinition) (input : CommandInput) : Bool := control.inputs.any (fun providedInput => providedInput.name == input.name)"
            ),
            "Lean slice artifacts must determine whether a control provides a command input"
        );
        assert!(
            lean.contains(
                "def controlProvidesEveryCommandInput (control : ControlDefinition) (command : CommandDefinition) : Bool := command.name != control.commandName || command.inputs.all (controlProvidesCommandInput control)"
            ),
            "Lean slice artifacts must require controls to provide every input required by the invoked command"
        );
        assert!(
            lean.contains(
                "def viewControlsProvideCommandInputs : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => sliceCommandDefinitions.all (controlProvidesEveryCommandInput control)))"
            ),
            "Lean slice artifacts must prove controls provide every invoked command input"
        );
        assert!(
            lean.contains(
                "def viewControlInputsHaveAllowedSources : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasAllowedSource))"
            ),
            "Lean slice artifacts must prove control inputs use allowed sources"
        );
        assert!(
            lean.contains(
                "def viewControlInputsHaveProvenance : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputHasProvenance))"
            ),
            "Lean slice artifacts must prove control inputs declare provenance"
        );
        assert!(
            lean.contains(
                "def viewControlInputVisibilityIsModeled : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => control.inputs.all controlInputVisibilityIsModeled))"
            ),
            "Lean slice artifacts must prove actor-provided and decision inputs are visible"
        );
        assert!(
            lean.contains(
                "def viewControlsHandleCommandErrors : Bool := sliceViewDefinitions.all (fun view => view.controls.all (fun control => sliceCommandDefinitions.all (commandErrorsHandledByControl control)))"
            ),
            "Lean slice artifacts must prove controls handle returned command errors"
        );
        assert!(
            lean.contains(
                "def viewControlRecoveryBehaviorIsModeled : Bool := sliceViewDefinitions.all (fun view => view.controls.all controlRecoveryBehaviorIsModeled)"
            ),
            "Lean slice artifacts must prove control recovery behavior is modeled"
        );
        assert!(
            lean.contains(
                "def stateViewSlicesDoNotOwnCommands : Bool := sliceKind != \"state_view\" || (sliceCommands.isEmpty && sliceCommandDefinitions.isEmpty)"
            ),
            "Lean slice artifacts must prove state-view slices do not own state-changing commands"
        );
        assert!(
            lean.contains(
                "def stateViewSlicesOwnViews : Bool := sliceKind != \"state_view\" || (sliceViews.isEmpty == false || sliceViewDefinitions.isEmpty == false)"
            ),
            "Lean slice artifacts must prove state-view slices own at least one view"
        );
        assert!(
            lean.contains(
                "def stateViewSlicesOwnReadModels : Bool := sliceKind != \"state_view\" || (sliceReadModels.isEmpty == false || sliceReadModelDefinitions.isEmpty == false)"
            ),
            "Lean slice artifacts must prove state-view slices own read models"
        );
        assert!(
            lean.contains(
                "def readModelOwnsProjectionPath (readModel : ReadModelDefinition) : Bool := readModel.fields.isEmpty == false && readModel.fields.all readModelFieldSourceIsComplete"
            ),
            "Lean slice artifacts must define state-view projection paths as complete read-model field sources"
        );
        assert!(
            lean.contains(
                "def stateViewSlicesOwnProjectionPaths : Bool := sliceKind != \"state_view\" || sliceReadModelDefinitions.all readModelOwnsProjectionPath"
            ),
            "Lean slice artifacts must prove state-view slices own projection paths"
        );
        assert!(
            lean.contains(
                "def stateChangeSlicesOwnCommands : Bool := sliceKind != \"state_change\" || (sliceCommands.isEmpty == false || sliceCommandDefinitions.isEmpty == false)"
            ),
            "Lean slice artifacts must prove state-change slices own commands"
        );
        assert!(
            lean.contains(
                "def stateChangeSlicesOwnEvents : Bool := sliceKind != \"state_change\" || (sliceEvents.isEmpty == false || sliceEventDefinitions.isEmpty == false)"
            ),
            "Lean slice artifacts must prove state-change slices own emitted event facts"
        );
        assert!(
            lean.contains(
                "def stateChangeSlicesOwnOutcomes : Bool := sliceKind != \"state_change\" || sliceOutcomeDefinitions.isEmpty == false"
            ),
            "Lean slice artifacts must prove state-change slices own outcome facts"
        );
        assert!(
            lean.contains(
                "def stateChangeSlicesOwnErrors : Bool := sliceKind != \"state_change\" || commandErrorsAreDeclared"
            ),
            "Lean slice artifacts must prove state-change command-local errors are owned by command definitions"
        );
        assert!(
            lean.contains(
                "def stateChangeSlicesDoNotOwnReadModelsOrViews : Bool := sliceKind != \"state_change\" || (sliceReadModels.isEmpty && sliceReadModelDefinitions.isEmpty && sliceViews.isEmpty && sliceViewDefinitions.isEmpty)"
            ),
            "Lean slice artifacts must prove state-change slices do not own read models or views"
        );
        assert!(
            lean.contains(
                "def stateChangeSlicesDoNotOwnAutomationsOrTranslations : Bool := sliceKind != \"state_change\" || (sliceAutomations.isEmpty && sliceTranslations.isEmpty)"
            ),
            "Lean slice artifacts must prove state-change slices do not own automations or translations"
        );
        assert!(
            lean.contains(
                "def stateChangeSlicesDoNotOwnControlsOrSketches : Bool := sliceKind != \"state_change\" || sliceViewDefinitions.all (fun view => view.controls.isEmpty && view.sketchTokens.isEmpty)"
            ),
            "Lean slice artifacts must prove state-change slices do not own controls or wireframe/sketch tokens"
        );
        assert!(
            lean.contains(
                "def translationSlicesDoNotOwnViews : Bool := sliceKind != \"translation\" || (sliceViews.isEmpty && sliceViewDefinitions.isEmpty)"
            ),
            "Lean slice artifacts must prove translation slices do not own screens"
        );
        assert!(
            lean.contains(
                "def sliceHasLocallyEmittedEvent : Bool := sliceEvents.isEmpty == false || sliceEventDefinitions.any (fun event => event.observed == false && event.shared == false)"
            ),
            "Lean slice artifacts must count locally emitted formal event definitions"
        );
        assert!(
            lean.contains(
                "def sliceStateChangeRequiresEvent : Prop := sliceKind = \"state_change\" -> sliceHasLocallyEmittedEvent"
            ),
            "Lean slice artifacts must state the local event-emission proof obligation for state changes"
        );
        assert!(
            lean.contains(
                "theorem sliceStateChangeRequiresEventIsStable : sliceStateChangeRequiresEvent := by\n  simp [sliceStateChangeRequiresEvent, sliceHasLocallyEmittedEvent, sliceKind, sliceEvents, sliceEventDefinitions]"
            ),
            "Lean slice artifacts must prove current state-change slices emit at least one event"
        );
        assert!(
            lean.contains(
                "theorem sliceBitLevelDataFlowsAreStructured : sliceBitLevelDataFlows.all (fun flow => flow.datum.isEmpty == false && flow.source.isEmpty == false && flow.transformationSemantics.isEmpty == false && flow.target.isEmpty == false && flow.bitEncoding.isEmpty == false) = true := rfl"
            ),
            "Lean slice artifacts must prove represented data-flow rows include source, transformation/projection, target, and bit encoding fields"
        );
        assert!(
            lean.contains(
                "theorem modeledDataFlowsAreBitCompleteIsStable : modeledDataFlowsAreBitComplete = true := rfl"
            ),
            "Lean slice artifacts must prove current modeled data has bit-level flow coverage"
        );
        assert!(
            lean.contains(
                "theorem sliceScenariosHaveGwtIsStable : sliceScenariosHaveGwt = true := rfl"
            ),
            "Lean slice artifacts must prove current first-class scenarios satisfy GWT completeness"
        );
        assert!(
            lean.contains(
                "theorem sliceScenarioNamesAreUniqueIsStable : sliceScenarioNamesAreUnique = true := rfl"
            ),
            "Lean slice artifacts must prove current first-class scenario names are unique"
        );
        assert!(
            lean.contains(
                "theorem sliceScenarioStreamsResolveIsStable : sliceScenarioStreamsResolve = true := rfl"
            ),
            "Lean slice artifacts must prove current scenario stream references resolve"
        );
        assert!(
            lean.contains(
                "theorem stateChangeScenariosNameStreamsIsStable : stateChangeScenariosNameStreams = true := rfl"
            ),
            "Lean slice artifacts must prove current state-change scenarios name read and written streams"
        );
        assert!(
            lean.contains(
                "theorem acceptanceScenariosAreUserFacingIsStable : acceptanceScenariosAreUserFacing = true := rfl"
            ),
            "Lean slice artifacts must prove acceptance scenarios remain user-facing"
        );
        assert!(
            lean.contains(
                "theorem stateViewReadModelsHaveProjectorContractsIsStable : stateViewReadModelsHaveProjectorContracts = true := rfl"
            ),
            "Lean slice artifacts must prove state-view read models have projector contract scenarios"
        );
        assert!(
            lean.contains(
                "theorem contractScenariosTargetKnownDefinitionsIsStable : contractScenariosTargetKnownDefinitions = true := rfl"
            ),
            "Lean slice artifacts must prove current contract scenarios target known definitions"
        );
        assert!(
            lean.contains(
                "theorem commandInputsHaveAllowedSourcesIsStable : commandInputsHaveAllowedSources = true := rfl"
            ),
            "Lean slice artifacts must prove command inputs do not source from read models"
        );
        assert!(
            lean.contains(
                "theorem commandInputsHaveProvenanceIsStable : commandInputsHaveProvenance = true := rfl"
            ),
            "Lean slice artifacts must prove command inputs carry reportable provenance"
        );
        assert!(
            lean.contains(
                "theorem commandInputsTraceToInvocationSourcesIsStable : commandInputsTraceToInvocationSources = true := rfl"
            ),
            "Lean slice artifacts must prove current command inputs trace to modeled invocation sources"
        );
        assert!(
            lean.contains(
                "theorem commandInputsSourcedFromEventStreamsResolveIsStable : commandInputsSourcedFromEventStreamsResolve = true := rfl"
            ),
            "Lean slice artifacts must prove current event-stream command input sources resolve"
        );
        assert!(
            lean.contains(
                "theorem commandErrorsAreDeclaredIsStable : commandErrorsAreDeclared = true := rfl"
            ),
            "Lean slice artifacts must prove current command-local errors are declared"
        );
        assert!(
            lean.contains(
                "theorem commandErrorsHaveAllowedRecoveryIsStable : commandErrorsHaveAllowedRecovery = true := rfl"
            ),
            "Lean slice artifacts must prove current command-local errors have modeled recovery"
        );
        assert!(
            lean.contains(
                "theorem commandErrorsHaveScenarioCoverageIsStable : commandErrorsHaveScenarioCoverage = true := rfl"
            ),
            "Lean slice artifacts must prove current command-local errors have scenario coverage"
        );
        assert!(
            lean.contains(
                "theorem scenarioErrorReferencesAreDeclaredIsStable : scenarioErrorReferencesAreDeclared = true := rfl"
            ),
            "Lean slice artifacts must prove current scenario error references are declared command errors"
        );
        assert!(
            lean.contains(
                "theorem singletonCommandsDeclareRepeatBehaviorIsStable : singletonCommandsDeclareRepeatBehavior = true := rfl"
            ),
            "Lean slice artifacts must prove current singleton state changes declare repeat behavior"
        );
        assert!(
            lean.contains(
                "theorem automationSlicesDeclareTriggersIsStable : automationSlicesDeclareTriggers = true := rfl"
            ),
            "Lean slice artifacts must prove current automation slices declare triggers"
        );
        assert!(
            lean.contains(
                "theorem automationSlicesRepresentOneReactionIsStable : automationSlicesRepresentOneReaction = true := rfl"
            ),
            "Lean slice artifacts must prove current automation slices represent one coherent reaction"
        );
        assert!(
            lean.contains(
                "theorem automationsIssueKnownCommandsIsStable : automationsIssueKnownCommands = true := rfl"
            ),
            "Lean slice artifacts must prove current automations issue known commands"
        );
        assert!(
            lean.contains(
                "theorem automationsHandleCommandErrorsIsStable : automationsHandleCommandErrors = true := rfl"
            ),
            "Lean slice artifacts must prove current automations handle command errors"
        );
        assert!(
            lean.contains(
                "theorem translationSlicesDeclareExternalContractsIsStable : translationSlicesDeclareExternalContracts = true := rfl"
            ),
            "Lean slice artifacts must prove current translation slices declare external contracts"
        );
        assert!(
            lean.contains(
                "theorem externalBoundariesHavePayloadContractsAndFieldProvenanceIsStable : externalBoundariesHavePayloadContractsAndFieldProvenance = true := rfl"
            ),
            "Lean slice artifacts must prove current external boundaries bind payload contracts with field-level provenance"
        );
        assert!(
            lean.contains(
                "theorem translationsTargetKnownCommandsIsStable : translationsTargetKnownCommands = true := rfl"
            ),
            "Lean slice artifacts must prove current translations target known commands"
        );
        assert!(
            lean.contains(
                "theorem translationsReferenceObservedExternalEventsIsStable : translationsReferenceObservedExternalEvents = true := rfl"
            ),
            "Lean slice artifacts must prove current translations reference observed external events"
        );
        assert!(
            lean.contains(
                "theorem boardLanesAreCanonicalIsStable : boardLanesAreCanonical = true := rfl"
            ),
            "Lean slice artifacts must prove current board lane inventory is canonical"
        );
        assert!(
            lean.contains("theorem boardElementsUseCanonicalLanesIsStable : boardElementsUseCanonicalLanes = true := rfl"),
            "Lean slice artifacts must prove current board elements use canonical lanes"
        );
        assert!(
            lean.contains("theorem boardElementsReferenceDeclarationsIsStable : boardElementsReferenceDeclarations = true := rfl"),
            "Lean slice artifacts must prove current board elements reference declarations"
        );
        assert!(
            lean.contains("theorem automationBoardElementsAreDeclaredAutomationsIsStable : automationBoardElementsAreDeclaredAutomations = true := rfl"),
            "Lean slice artifacts must prove current automation board elements resolve to declared automations"
        );
        assert!(
            lean.contains("theorem externalBoardElementsAreObservedEventsIsStable : externalBoardElementsAreObservedEvents = true := rfl"),
            "Lean slice artifacts must prove current external-event board elements resolve to observed events"
        );
        assert!(
            lean.contains("theorem commandEventBoardEdgesMatchEmissionsIsStable : commandEventBoardEdgesMatchEmissions = true := rfl"),
            "Lean slice artifacts must prove current command-to-event board edges match command emissions"
        );
        assert!(
            lean.contains("theorem eventReadModelBoardEdgesMatchProjectionSourcesIsStable : eventReadModelBoardEdgesMatchProjectionSources = true := rfl"),
            "Lean slice artifacts must prove current event-to-read-model board edges match projection sources"
        );
        assert!(
            lean.contains("theorem viewCommandBoardEdgesMatchControlsIsStable : viewCommandBoardEdgesMatchControls = true := rfl"),
            "Lean slice artifacts must prove current view-to-command board edges match controls"
        );
        assert!(
            lean.contains("theorem boardConnectionsHaveCausalSemanticsIsStable : boardConnectionsHaveCausalSemantics = true := rfl"),
            "Lean slice artifacts must prove current board connections have causal semantics"
        );
        assert!(
            lean.contains("theorem externalEventTriggersMatchTranslationsIsStable : externalEventTriggersMatchTranslations = true := rfl"),
            "Lean slice artifacts must prove current external-event command triggers match translations"
        );
        assert!(
            lean.contains("theorem externalEventsDoNotUpdateReadModelsIsStable : externalEventsDoNotUpdateReadModels = true := rfl"),
            "Lean slice artifacts must prove current external events do not update read models"
        );
        assert!(
            lean.contains("theorem readModelsDoNotFeedCommandsIsStable : readModelsDoNotFeedCommands = true := rfl"),
            "Lean slice artifacts must prove current boards do not feed commands from read models"
        );
        assert!(
            lean.contains("theorem readModelsFeedingViewsHaveIncomingEventUpdatesIsStable : readModelsFeedingViewsHaveIncomingEventUpdates = true := rfl"),
            "Lean slice artifacts must prove current read models feeding views have incoming event updates"
        );
        assert!(
            lean.contains("theorem commandsHaveIncomingTriggersIsStable : commandsHaveIncomingTriggers = true := rfl"),
            "Lean slice artifacts must prove current commands have incoming triggers"
        );
        assert!(
            lean.contains("theorem mainPathBoardHasNoDisconnectedIslandsIsStable : mainPathBoardHasNoDisconnectedIslands = true := rfl"),
            "Lean slice artifacts must prove current main-path boards have no disconnected islands"
        );
        assert!(
            lean.contains(
                "theorem outcomeLabelsAreUniqueIsStable : outcomeLabelsAreUnique = true := rfl"
            ),
            "Lean slice artifacts must prove current outcome labels are unique"
        );
        assert!(
            lean.contains(
                "theorem outcomeEventSetsAreNonEmptyIsStable : outcomeEventSetsAreNonEmpty = true := rfl"
            ),
            "Lean slice artifacts must prove current outcomes have non-empty event sets"
        );
        assert!(
            lean.contains(
                "theorem outcomeEventSetsAreDistinctIsStable : outcomeEventSetsAreDistinct = true := rfl"
            ),
            "Lean slice artifacts must prove current outcome event sets are distinct"
        );
        assert!(
            lean.contains(
                "theorem outcomeEventsAreKnownToSliceIsStable : outcomeEventsAreKnownToSlice = true := rfl"
            ),
            "Lean slice artifacts must prove current outcome events resolve to emitted or observed events"
        );
        assert!(
            lean.contains(
                "theorem eventsReferenceKnownStreamsIsStable : eventsReferenceKnownStreams = true := rfl"
            ),
            "Lean slice artifacts must prove current events reference known streams"
        );
        assert!(
            lean.contains(
                "theorem commandEmittedEventsAreKnownIsStable : commandEmittedEventsAreKnown = true := rfl"
            ),
            "Lean slice artifacts must prove current command-emitted events resolve"
        );
        assert!(
            lean.contains(
                "theorem locallyEmittedEventsAreProducedByCommandsIsStable : locallyEmittedEventsAreProducedByCommands = true := rfl"
            ),
            "Lean slice artifacts must prove current local events are command-produced unless observed or shared"
        );
        assert!(
            lean.contains(
                "theorem externalPayloadFieldsHaveProvenanceIsStable : externalPayloadFieldsHaveProvenance = true := rfl"
            ),
            "Lean slice artifacts must prove current external payload fields carry provenance and bit encoding"
        );
        assert!(
            lean.contains(
                "theorem eventAttributesHaveAllowedSourcesIsStable : eventAttributesHaveAllowedSources = true := rfl"
            ),
            "Lean slice artifacts must prove current event attributes do not source from read models"
        );
        assert!(
            lean.contains(
                "theorem eventAttributesHaveProvenanceIsStable : eventAttributesHaveProvenance = true := rfl"
            ),
            "Lean slice artifacts must prove current event attributes carry source provenance"
        );
        assert!(
            lean.contains(
                "theorem eventAttributeSourcesAreCompleteIsStable : eventAttributeSourcesAreComplete = true := rfl"
            ),
            "Lean slice artifacts must prove current event attribute source details are complete"
        );
        assert!(
            lean.contains(
                "theorem storedEventFactsTraceToOriginalSourcesIsStable : storedEventFactsTraceToOriginalSources = true := rfl"
            ),
            "Lean slice artifacts must prove current stored event facts trace to original modeled sources"
        );
        assert!(
            lean.contains(
                "theorem readModelFieldsHaveAllowedSourcesIsStable : readModelFieldsHaveAllowedSources = true := rfl"
            ),
            "Lean slice artifacts must prove current read model fields do not source from commands"
        );
        assert!(
            lean.contains(
                "theorem readModelFieldsHaveProvenanceIsStable : readModelFieldsHaveProvenance = true := rfl"
            ),
            "Lean slice artifacts must prove current read model fields carry provenance"
        );
        assert!(
            lean.contains(
                "theorem readModelFieldSourcesAreCompleteIsStable : readModelFieldSourcesAreComplete = true := rfl"
            ),
            "Lean slice artifacts must prove current read model fields trace to event facts, derivation, or absence"
        );
        assert!(
            lean.contains(
                "theorem readModelFieldEventAttributeSourcesResolveIsStable : readModelFieldEventAttributeSourcesResolve = true := rfl"
            ),
            "Lean slice artifacts must prove current event-sourced read model fields resolve to declared event attributes"
        );
        assert!(
            lean.contains(
                "theorem derivedReadModelFieldsHaveScenarioCoverageIsStable : derivedReadModelFieldsHaveScenarioCoverage = true := rfl"
            ),
            "Lean slice artifacts must prove current derived read model fields have scenario coverage"
        );
        assert!(
            lean.contains(
                "theorem absenceReadModelFieldsHaveScenarioCoverageIsStable : absenceReadModelFieldsHaveScenarioCoverage = true := rfl"
            ),
            "Lean slice artifacts must prove current absence/default read model fields have scenario coverage"
        );
        assert!(
            lean.contains(
                "theorem transitiveReadModelsHaveSemanticsIsStable : transitiveReadModelsHaveSemantics = true := rfl"
            ),
            "Lean slice artifacts must prove current transitive read models have complete semantics"
        );
        assert!(
            lean.contains(
                "theorem viewFieldsHaveAllowedSourcesIsStable : viewFieldsHaveAllowedSources = true := rfl"
            ),
            "Lean slice artifacts must prove current view fields source from read models"
        );
        assert!(
            lean.contains(
                "theorem viewFieldsHaveProvenanceIsStable : viewFieldsHaveProvenance = true := rfl"
            ),
            "Lean slice artifacts must prove current view fields carry provenance"
        );
        assert!(
            lean.contains(
                "theorem viewFieldSourcesAreCompleteIsStable : viewFieldSourcesAreComplete = true := rfl"
            ),
            "Lean slice artifacts must prove current view fields have complete source chains"
        );
        assert!(
            lean.contains(
                "theorem viewFieldReadModelFieldSourcesResolveIsStable : viewFieldReadModelFieldSourcesResolve = true := rfl"
            ),
            "Lean slice artifacts must prove current view fields resolve to declared read model fields"
        );
        assert!(
            lean.contains(
                "theorem displayedDataTraceToOriginalProvenanceIsStable : displayedDataTraceToOriginalProvenance = true := rfl"
            ),
            "Lean slice artifacts must prove current displayed data traces transitively to original provenance"
        );
        assert!(
            lean.contains(
                "theorem viewFieldsSourceFromUsedReadModelsIsStable : viewFieldsSourceFromUsedReadModels = true := rfl"
            ),
            "Lean slice artifacts must prove current view fields source from read models used by each view"
        );
        assert!(
            lean.contains(
                "theorem viewsHaveInformationSketchesIsStable : viewsHaveInformationSketches = true := rfl"
            ),
            "Lean slice artifacts must prove current views have information sketches"
        );
        assert!(
            lean.contains(
                "theorem viewFieldsAppearInSketchIsStable : viewFieldsAppearInSketch = true := rfl"
            ),
            "Lean slice artifacts must prove current displayed fields appear in their view sketches"
        );
        assert!(
            lean.contains(
                "theorem viewSketchTokensMapToModeledElementsIsStable : viewSketchTokensMapToModeledElements = true := rfl"
            ),
            "Lean slice artifacts must prove current sketch tokens map to modeled elements"
        );
        assert!(
            lean.contains(
                "theorem viewControlsHaveSketchTokensIsStable : viewControlsHaveSketchTokens = true := rfl"
            ),
            "Lean slice artifacts must prove current controls appear in sketches"
        );
        assert!(
            lean.contains(
                "theorem viewControlsAppearInSketchIsStable : viewControlsAppearInSketch = true := rfl"
            ),
            "Lean slice artifacts must prove current controls appear in their view sketches"
        );
        assert!(
            lean.contains(
                "theorem viewControlsReferenceKnownCommandsIsStable : viewControlsReferenceKnownCommands = true := rfl"
            ),
            "Lean slice artifacts must prove current controls reference known commands"
        );
        assert!(
            lean.contains(
                "theorem viewControlsProvideCommandInputsIsStable : viewControlsProvideCommandInputs = true := rfl"
            ),
            "Lean slice artifacts must prove current controls provide every invoked command input"
        );
        assert!(
            lean.contains(
                "theorem viewControlInputsHaveAllowedSourcesIsStable : viewControlInputsHaveAllowedSources = true := rfl"
            ),
            "Lean slice artifacts must prove current control inputs use allowed sources"
        );
        assert!(
            lean.contains(
                "theorem viewControlInputsHaveProvenanceIsStable : viewControlInputsHaveProvenance = true := rfl"
            ),
            "Lean slice artifacts must prove current control inputs carry provenance"
        );
        assert!(
            lean.contains(
                "theorem viewControlInputVisibilityIsModeledIsStable : viewControlInputVisibilityIsModeled = true := rfl"
            ),
            "Lean slice artifacts must prove current actor and decision inputs are visible"
        );
        assert!(
            lean.contains(
                "theorem viewControlsHandleCommandErrorsIsStable : viewControlsHandleCommandErrors = true := rfl"
            ),
            "Lean slice artifacts must prove current controls handle returned command errors"
        );
        assert!(
            lean.contains(
                "theorem viewControlRecoveryBehaviorIsModeledIsStable : viewControlRecoveryBehaviorIsModeled = true := rfl"
            ),
            "Lean slice artifacts must prove current control recovery behavior is modeled"
        );
        assert!(
            lean.contains(
                "theorem stateViewSlicesDoNotOwnCommandsIsStable : stateViewSlicesDoNotOwnCommands = true := rfl"
            ),
            "Lean slice artifacts must prove current state-view slices do not own commands"
        );
        assert!(
            lean.contains(
                "theorem stateViewSlicesOwnViewsIsStable : stateViewSlicesOwnViews = true := rfl"
            ),
            "Lean slice artifacts must prove current state-view slices own at least one view"
        );
        assert!(
            lean.contains(
                "theorem stateViewSlicesOwnReadModelsIsStable : stateViewSlicesOwnReadModels = true := rfl"
            ),
            "Lean slice artifacts must prove current state-view slices own read models"
        );
        assert!(
            lean.contains(
                "theorem stateViewSlicesOwnProjectionPathsIsStable : stateViewSlicesOwnProjectionPaths = true := rfl"
            ),
            "Lean slice artifacts must prove current state-view slices own projection paths"
        );
        assert!(
            lean.contains(
                "theorem stateChangeSlicesOwnCommandsIsStable : stateChangeSlicesOwnCommands = true := rfl"
            ),
            "Lean slice artifacts must prove current state-change slices own commands"
        );
        assert!(
            lean.contains(
                "theorem stateChangeSlicesOwnEventsIsStable : stateChangeSlicesOwnEvents = true := by\n  simp [stateChangeSlicesOwnEvents, sliceKind, sliceEvents, sliceEventDefinitions]"
            ),
            "Lean slice artifacts must prove current state-change slices own emitted event facts"
        );
        assert!(
            lean.contains(
                "theorem stateChangeSlicesOwnOutcomesIsStable : stateChangeSlicesOwnOutcomes = true := by\n  simp [stateChangeSlicesOwnOutcomes, sliceKind, sliceOutcomeDefinitions]"
            ),
            "Lean slice artifacts must prove current state-change slices own outcome facts"
        );
        assert!(
            lean.contains(
                "theorem stateChangeSlicesOwnErrorsIsStable : stateChangeSlicesOwnErrors = true := by\n  by_cases h : sliceKind != \"state_change\"\n  · simp [stateChangeSlicesOwnErrors, h]\n  · simp [stateChangeSlicesOwnErrors, h]\n    exact commandErrorsAreDeclaredIsStable"
            ),
            "Lean slice artifacts must prove current state-change slices own command-local errors"
        );
        assert!(
            lean.contains(
                "theorem stateChangeSlicesDoNotOwnReadModelsOrViewsIsStable : stateChangeSlicesDoNotOwnReadModelsOrViews = true := rfl"
            ),
            "Lean slice artifacts must prove current state-change slices do not own read models or views"
        );
        assert!(
            lean.contains(
                "theorem stateChangeSlicesDoNotOwnAutomationsOrTranslationsIsStable : stateChangeSlicesDoNotOwnAutomationsOrTranslations = true := rfl"
            ),
            "Lean slice artifacts must prove current state-change slices do not own automations or translations"
        );
        assert!(
            lean.contains(
                "theorem stateChangeSlicesDoNotOwnControlsOrSketchesIsStable : stateChangeSlicesDoNotOwnControlsOrSketches = true := rfl"
            ),
            "Lean slice artifacts must prove current state-change slices do not own controls or wireframe/sketch tokens"
        );
        assert!(
            lean.contains(
                "theorem translationSlicesDoNotOwnViewsIsStable : translationSlicesDoNotOwnViews = true := rfl"
            ),
            "Lean slice artifacts must prove current translation slices do not own views"
        );
        assert!(
            lean.contains(
                "theorem viewControlNavigationTypesAreModeledIsStable : viewControlNavigationTypesAreModeled = true := rfl"
            ),
            "Lean slice artifacts must prove current navigation target types are modeled"
        );
        assert!(
            lean.contains(
                "theorem viewControlNavigationTargetsAreCompleteIsStable : viewControlNavigationTargetsAreComplete = true := rfl"
            ),
            "Lean slice artifacts must prove current navigation targets are complete"
        );
        assert!(lean.contains("theorem sliceIdentityIsStable"));

        Ok(())
    }
}
