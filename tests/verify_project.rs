// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::env;
    use std::error::Error;
    use std::fs::{Permissions, create_dir_all, read_to_string, set_permissions, write};
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn verify_runs_lean_and_quint_for_initialized_project_root() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success()
            .stdout(predicate::str::contains("Lean4 artifacts verified"))
            .stdout(predicate::str::contains("Quint artifacts verified"));

        assert_eq!(
            read_to_string(temp_dir.path().join("lake.log"))?,
            "env lean model/lean/RepairDesk.lean\n"
        );
        assert_eq!(
            normalize_quint_log(&read_to_string(temp_dir.path().join("quint.log"))?),
            "typecheck model/quint/RepairDesk.qnt\n\
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n"
        );

        Ok(())
    }

    #[test]
    fn verify_requests_bit_semantics_preservation_invariant() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success();

        let quint_log = normalize_quint_log(&read_to_string(temp_dir.path().join("quint.log"))?);

        assert!(
            quint_log.contains("modelDataFlowSourceChainsPreserveBitEncodingSemantics"),
            "project verification must ask Quint to prove bit encoding semantics are preserved through data-flow source chains:\n{quint_log}"
        );

        Ok(())
    }

    #[test]
    fn verify_requests_project_root_composition_structure_invariant() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_view",
                "--description",
                "Capture ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success();

        let quint_log = normalize_quint_log(&read_to_string(temp_dir.path().join("quint.log"))?);

        assert!(
            quint_log.contains("modelWorkflowCompositionStructureComplete"),
            "project verification must ask Quint to prove workflows have explicit composition structure at the model root:\n{quint_log}"
        );

        Ok(())
    }

    #[test]
    fn verify_requests_workflow_only_events_are_shared_invariant() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_view",
                "--description",
                "Capture ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success();

        let quint_log = normalize_quint_log(&read_to_string(temp_dir.path().join("quint.log"))?);

        assert!(
            quint_log.contains("workflowOnlyEventsMayBeSharedAcrossSlices"),
            "workflow verification must ask Quint to prove cross-slice sharing is restricted to events:\n{quint_log}"
        );

        Ok(())
    }

    #[test]
    fn verify_requests_slice_coherent_model_unit_invariant() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_change",
                "--description",
                "Capture ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success();

        let quint_log = normalize_quint_log(&read_to_string(temp_dir.path().join("quint.log"))?);

        assert!(
            quint_log.contains("sliceRepresentsOneCoherentModelUnit"),
            "slice verification must ask Quint to prove each slice represents one coherent model unit:\n{quint_log}"
        );

        Ok(())
    }

    #[test]
    fn verify_runs_lean_and_quint_for_modeled_workflows() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_view",
                "--description",
                "Capture ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success()
            .stdout(predicate::str::contains("Lean4 artifacts verified"))
            .stdout(predicate::str::contains("Quint artifacts verified"));

        assert_eq!(
            read_to_string(temp_dir.path().join("lake.log"))?,
            "env lean model/lean/RepairDesk.lean\n\
             env lean model/lean/OpenTicket.lean\n\
             env lean model/lean/slices/CaptureTicket.lean\n"
        );
        assert_eq!(
            normalize_quint_log(&read_to_string(temp_dir.path().join("quint.log"))?),
            "typecheck model/quint/RepairDesk.qnt\n\
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowSliceModulesComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowSharedEventDefinitionsHaveIdenticalIdentity,workflowOnlyEventsMayBeSharedAcrossSlices,workflowCommandTransitionsTargetOwnedCommands,workflowCommandTransitionsSourceOwnedControls,workflowCommandTransitionsResolveControlsAndCommands,workflowStateViewCommandTransitionsTargetStateChanges,workflowEventTransitionsAreSharedByEndpointSlices,workflowEventTransitionsHaveParticipatingEndpointEvents,workflowNavigationTransitionsResolveControlsAndViews,workflowNavigationTransitionsResolveToEntryViews,workflowExternalTriggersDeclarePayloadContracts,workflowExternalTriggerPayloadContractsHaveProvenance,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates --server-endpoint <endpoint> model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceRepresentsOneCoherentModelUnit,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceNamedDefinitionsAreUniquelyOwned,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,contractScenariosCoverModeledContracts,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsWithoutIssuingControlsHaveProvenance,commandSessionInputsHaveDescriptions,commandInputsTraceToInvocationSources,commandInputsSourcedFromEventStreamsResolve,commandInputsSourcedFromExternalPayloadsResolve,commandInputsSourcedFromGeneratedValuesHaveCoordinates,commandInputsSourcedFromSessionValuesHaveCoordinates,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationSlicesRepresentOneReaction,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,externalBoundariesHavePayloadContractsAndFieldProvenance,translationsTargetKnownCommands,translationsReferenceObservedExternalEvents,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,automationBoardElementsAreDeclaredAutomations,externalBoardElementsAreObservedEvents,commandEventBoardEdgesMatchEmissions,eventReadModelBoardEdgesMatchProjectionSources,viewCommandBoardEdgesMatchControls,boardConnectionsHaveCausalSemantics,externalEventTriggersMatchTranslations,externalEventsDoNotUpdateReadModels,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,storedEventFactsTraceToOriginalSources,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,displayedDataTraceToOriginalProvenance,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputsHaveDescriptions,viewControlSessionInputsHaveDescriptions,viewControlInputVisibilityIsModeled,viewControlDecisionFieldsAreVisible,viewControlActorInputsAreVisible,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateViewSlicesOwnProjectionPaths,stateChangeSlicesOwnCommands,stateChangeSlicesOwnEvents,stateChangeSlicesOwnOutcomes,stateChangeSlicesOwnErrors,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,stateChangeSlicesDoNotOwnControlsOrSketches,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTypesAreDeclared,viewControlModeledViewNavigationTargetsResolve,viewControlExternalWorkflowNavigationTargetsNamed,viewControlExternalSystemNavigationTargetsHaveContracts,viewControlNavigationTargetsAreComplete --server-endpoint <endpoint> model/quint/slices/CaptureTicket.qnt\n"
        );
        assert_quint_verify_endpoints_are_distinct(&read_to_string(
            temp_dir.path().join("quint.log"),
        )?);

        Ok(())
    }

    #[test]
    fn verify_runs_lean_and_quint_for_modeled_slices() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_view",
                "--description",
                "Capture ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success()
            .stdout(predicate::str::contains("Lean4 artifacts verified"))
            .stdout(predicate::str::contains("Quint artifacts verified"));

        assert_eq!(
            read_to_string(temp_dir.path().join("lake.log"))?,
            "env lean model/lean/RepairDesk.lean\n\
             env lean model/lean/OpenTicket.lean\n\
             env lean model/lean/slices/CaptureTicket.lean\n"
        );
        assert_eq!(
            normalize_quint_log(&read_to_string(temp_dir.path().join("quint.log"))?),
            "typecheck model/quint/RepairDesk.qnt\n\
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowSliceModulesComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowSharedEventDefinitionsHaveIdenticalIdentity,workflowOnlyEventsMayBeSharedAcrossSlices,workflowCommandTransitionsTargetOwnedCommands,workflowCommandTransitionsSourceOwnedControls,workflowCommandTransitionsResolveControlsAndCommands,workflowStateViewCommandTransitionsTargetStateChanges,workflowEventTransitionsAreSharedByEndpointSlices,workflowEventTransitionsHaveParticipatingEndpointEvents,workflowNavigationTransitionsResolveControlsAndViews,workflowNavigationTransitionsResolveToEntryViews,workflowExternalTriggersDeclarePayloadContracts,workflowExternalTriggerPayloadContractsHaveProvenance,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates --server-endpoint <endpoint> model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceRepresentsOneCoherentModelUnit,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceNamedDefinitionsAreUniquelyOwned,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,contractScenariosCoverModeledContracts,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsWithoutIssuingControlsHaveProvenance,commandSessionInputsHaveDescriptions,commandInputsTraceToInvocationSources,commandInputsSourcedFromEventStreamsResolve,commandInputsSourcedFromExternalPayloadsResolve,commandInputsSourcedFromGeneratedValuesHaveCoordinates,commandInputsSourcedFromSessionValuesHaveCoordinates,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationSlicesRepresentOneReaction,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,externalBoundariesHavePayloadContractsAndFieldProvenance,translationsTargetKnownCommands,translationsReferenceObservedExternalEvents,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,automationBoardElementsAreDeclaredAutomations,externalBoardElementsAreObservedEvents,commandEventBoardEdgesMatchEmissions,eventReadModelBoardEdgesMatchProjectionSources,viewCommandBoardEdgesMatchControls,boardConnectionsHaveCausalSemantics,externalEventTriggersMatchTranslations,externalEventsDoNotUpdateReadModels,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,storedEventFactsTraceToOriginalSources,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,displayedDataTraceToOriginalProvenance,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputsHaveDescriptions,viewControlSessionInputsHaveDescriptions,viewControlInputVisibilityIsModeled,viewControlDecisionFieldsAreVisible,viewControlActorInputsAreVisible,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateViewSlicesOwnProjectionPaths,stateChangeSlicesOwnCommands,stateChangeSlicesOwnEvents,stateChangeSlicesOwnOutcomes,stateChangeSlicesOwnErrors,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,stateChangeSlicesDoNotOwnControlsOrSketches,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTypesAreDeclared,viewControlModeledViewNavigationTargetsResolve,viewControlExternalWorkflowNavigationTargetsNamed,viewControlExternalSystemNavigationTargetsHaveContracts,viewControlNavigationTargetsAreComplete --server-endpoint <endpoint> model/quint/slices/CaptureTicket.qnt\n"
        );

        Ok(())
    }

    #[test]
    fn verify_runs_formal_tools_without_json_validation_precheck() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_change",
                "--description",
                "Capture ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success()
            .stdout(predicate::str::contains("Lean4 artifacts verified"))
            .stdout(predicate::str::contains("Quint artifacts verified"));

        assert_eq!(
            read_to_string(temp_dir.path().join("lake.log"))?,
            "env lean model/lean/RepairDesk.lean\n\
             env lean model/lean/OpenTicket.lean\n\
             env lean model/lean/slices/CaptureTicket.lean\n"
        );
        assert_eq!(
            normalize_quint_log(&read_to_string(temp_dir.path().join("quint.log"))?),
            "typecheck model/quint/RepairDesk.qnt\n\
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowSliceModulesComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowSharedEventDefinitionsHaveIdenticalIdentity,workflowOnlyEventsMayBeSharedAcrossSlices,workflowCommandTransitionsTargetOwnedCommands,workflowCommandTransitionsSourceOwnedControls,workflowCommandTransitionsResolveControlsAndCommands,workflowStateViewCommandTransitionsTargetStateChanges,workflowEventTransitionsAreSharedByEndpointSlices,workflowEventTransitionsHaveParticipatingEndpointEvents,workflowNavigationTransitionsResolveControlsAndViews,workflowNavigationTransitionsResolveToEntryViews,workflowExternalTriggersDeclarePayloadContracts,workflowExternalTriggerPayloadContractsHaveProvenance,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates --server-endpoint <endpoint> model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceRepresentsOneCoherentModelUnit,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceNamedDefinitionsAreUniquelyOwned,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,contractScenariosCoverModeledContracts,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsWithoutIssuingControlsHaveProvenance,commandSessionInputsHaveDescriptions,commandInputsTraceToInvocationSources,commandInputsSourcedFromEventStreamsResolve,commandInputsSourcedFromExternalPayloadsResolve,commandInputsSourcedFromGeneratedValuesHaveCoordinates,commandInputsSourcedFromSessionValuesHaveCoordinates,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationSlicesRepresentOneReaction,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,externalBoundariesHavePayloadContractsAndFieldProvenance,translationsTargetKnownCommands,translationsReferenceObservedExternalEvents,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,automationBoardElementsAreDeclaredAutomations,externalBoardElementsAreObservedEvents,commandEventBoardEdgesMatchEmissions,eventReadModelBoardEdgesMatchProjectionSources,viewCommandBoardEdgesMatchControls,boardConnectionsHaveCausalSemantics,externalEventTriggersMatchTranslations,externalEventsDoNotUpdateReadModels,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,storedEventFactsTraceToOriginalSources,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,displayedDataTraceToOriginalProvenance,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputsHaveDescriptions,viewControlSessionInputsHaveDescriptions,viewControlInputVisibilityIsModeled,viewControlDecisionFieldsAreVisible,viewControlActorInputsAreVisible,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateViewSlicesOwnProjectionPaths,stateChangeSlicesOwnCommands,stateChangeSlicesOwnEvents,stateChangeSlicesOwnOutcomes,stateChangeSlicesOwnErrors,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,stateChangeSlicesDoNotOwnControlsOrSketches,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTypesAreDeclared,viewControlModeledViewNavigationTargetsResolve,viewControlExternalWorkflowNavigationTargetsNamed,viewControlExternalSystemNavigationTargetsHaveContracts,viewControlNavigationTargetsAreComplete --server-endpoint <endpoint> model/quint/slices/CaptureTicket.qnt\n"
        );

        Ok(())
    }

    #[test]
    fn verify_reports_actionable_lean_failure_diagnostics() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_failing_tool(&tool_dir, "lake")?;
        create_fake_tool(&tool_dir, "quint", "quint.log")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open ticket",
                "--description",
                "Actor opens a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture ticket",
                "--type",
                "state_view",
                "--description",
                "Capture ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Lean4 verification failed"))
            .stderr(predicate::str::contains("Run `emc check`"));

        Ok(())
    }

    fn create_fake_tool(
        tool_dir: &Path,
        tool_name: &str,
        log_file: &str,
    ) -> Result<(), Box<dyn Error>> {
        create_dir_all(tool_dir)?;
        let tool_path = tool_dir.join(tool_name);
        write(
            &tool_path,
            format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> {log_file}\n"),
        )?;
        set_permissions(&tool_path, Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn normalize_quint_log(source: &str) -> String {
        source
            .lines()
            .map(|line| {
                let mut parts = line.split_whitespace().collect::<Vec<_>>();
                if let Some(endpoint_flag) =
                    parts.iter().position(|part| *part == "--server-endpoint")
                {
                    let endpoint_value = endpoint_flag + 1;
                    if endpoint_value < parts.len() {
                        assert_ne!(
                            parts[endpoint_value], "localhost:8822",
                            "Quint verification must not use the shared default Apalache endpoint"
                        );
                        parts[endpoint_value] = "<endpoint>";
                    }
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    fn assert_quint_verify_endpoints_are_distinct(source: &str) {
        let endpoints = source
            .lines()
            .filter_map(|line| {
                let parts = line.split_whitespace().collect::<Vec<_>>();
                (parts.first() == Some(&"verify")).then(|| {
                    parts
                        .iter()
                        .position(|part| *part == "--server-endpoint")
                        .and_then(|index| parts.get(index + 1))
                        .copied()
                        .unwrap_or("")
                        .to_owned()
                })
            })
            .collect::<Vec<_>>();
        let unique_endpoints = endpoints.iter().collect::<BTreeSet<_>>();
        assert_eq!(
            endpoints.len(),
            unique_endpoints.len(),
            "each Quint verify invocation must receive its own Apalache endpoint"
        );
    }

    fn create_failing_tool(tool_dir: &Path, tool_name: &str) -> Result<(), Box<dyn Error>> {
        create_dir_all(tool_dir)?;
        let tool_path = tool_dir.join(tool_name);
        write(&tool_path, "#!/bin/sh\nexit 7\n")?;
        set_permissions(&tool_path, Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn path_with_fake_tools(tool_dir: &Path) -> Result<String, Box<dyn Error>> {
        let mut paths = vec![tool_dir.to_path_buf()];
        paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
        Ok(env::join_paths(paths)?.to_string_lossy().into_owned())
    }
}
