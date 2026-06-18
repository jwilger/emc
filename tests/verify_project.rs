// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::env;
    use std::error::Error;
    use std::fs::{
        Permissions, create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file,
        set_permissions, write,
    };
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command as ProcessCommand, Output};

    use assert_cmd::Command;
    use assert_cmd::cargo::cargo_bin;
    use predicates::prelude::{PredicateBooleanExt, predicate};
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
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelWorkflowBehaviorSurfaceIsComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n"
        );

        Ok(())
    }

    #[test]
    fn verify_reports_formal_tool_progress() -> Result<(), Box<dyn Error>> {
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
            .stdout(predicate::str::contains(
                "Running Lean4 verification for model/lean/RepairDesk.lean",
            ))
            .stdout(predicate::str::contains(
                "Running Quint typecheck for model/quint/RepairDesk.qnt",
            ))
            .stdout(predicate::str::contains(
                "Running Quint verification for model/quint/RepairDesk.qnt",
            ));

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
    fn verify_requests_project_behavior_surface_invariant() -> Result<(), Box<dyn Error>> {
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
            quint_log.contains("modelWorkflowBehaviorSurfaceIsComplete"),
            "project verification must ask Quint to prove workflow branches, outcomes, command errors, navigation targets, external boundaries, and recovery paths are modeled:\n{quint_log}"
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
    fn verify_requests_slice_smallest_behavior_boundary_invariant() -> Result<(), Box<dyn Error>> {
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
            quint_log.contains("sliceRepresentsSmallestUsefulBehaviorBoundary"),
            "slice verification must ask Quint to prove each slice stays within the smallest useful modeled behavior boundary:\n{quint_log}"
        );

        Ok(())
    }

    #[test]
    fn verify_requests_state_view_single_projection_boundary_invariant()
    -> Result<(), Box<dyn Error>> {
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
            quint_log.contains("stateViewSlicesRepresentSingleViewProjectionBoundary"),
            "slice verification must ask Quint to prove state-view slices model a single view/projection boundary:\n{quint_log}"
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
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelWorkflowBehaviorSurfaceIsComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowSliceModulesComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowSharedEventDefinitionsHaveIdenticalIdentity,workflowOnlyEventsMayBeSharedAcrossSlices,workflowCommandTransitionsTargetOwnedCommands,workflowCommandTransitionsSourceOwnedControls,workflowCommandTransitionsResolveControlsAndCommands,workflowStateViewCommandTransitionsTargetStateChanges,workflowEventTransitionsAreSharedByEndpointSlices,workflowEventTransitionsHaveParticipatingEndpointEvents,workflowNavigationTransitionsResolveControlsAndViews,workflowNavigationTransitionsResolveToEntryViews,workflowExternalTriggersDeclarePayloadContracts,workflowExternalTriggerPayloadContractsHaveProvenance,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates --server-endpoint <endpoint> model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceRepresentsOneCoherentModelUnit,sliceRepresentsSmallestUsefulBehaviorBoundary,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceNamedDefinitionsAreUniquelyOwned,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,contractScenariosCoverModeledContracts,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsWithoutIssuingControlsHaveProvenance,commandSessionInputsHaveDescriptions,commandInputsTraceToInvocationSources,commandInputsSourcedFromEventStreamsResolve,commandInputsSourcedFromExternalPayloadsResolve,commandInputsSourcedFromGeneratedValuesHaveCoordinates,commandInputsSourcedFromSessionValuesHaveCoordinates,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationSlicesRepresentOneReaction,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,externalBoundariesHavePayloadContractsAndFieldProvenance,translationsTargetKnownCommands,translationsReferenceObservedExternalEvents,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,automationBoardElementsAreDeclaredAutomations,externalBoardElementsAreObservedEvents,commandEventBoardEdgesMatchEmissions,eventReadModelBoardEdgesMatchProjectionSources,viewCommandBoardEdgesMatchControls,boardConnectionsHaveCausalSemantics,externalEventTriggersMatchTranslations,externalEventsDoNotUpdateReadModels,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,storedEventFactsTraceToOriginalSources,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,displayedDataTraceToOriginalProvenance,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputsHaveDescriptions,viewControlSessionInputsHaveDescriptions,viewControlInputVisibilityIsModeled,viewControlDecisionFieldsAreVisible,viewControlActorInputsAreVisible,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateViewSlicesOwnProjectionPaths,stateViewSlicesRepresentSingleViewProjectionBoundary,stateChangeSlicesOwnCommands,stateChangeSlicesOwnEvents,stateChangeSlicesOwnOutcomes,stateChangeSlicesOwnErrors,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,stateChangeSlicesDoNotOwnControlsOrSketches,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTypesAreDeclared,viewControlModeledViewNavigationTargetsResolve,viewControlExternalWorkflowNavigationTargetsNamed,viewControlExternalSystemNavigationTargetsHaveContracts,viewControlNavigationTargetsAreComplete --server-endpoint <endpoint> model/quint/slices/CaptureTicket.qnt\n"
        );
        assert_quint_verify_endpoints_are_distinct(&read_to_string(
            temp_dir.path().join("quint.log"),
        )?);

        Ok(())
    }

    #[test]
    fn verify_runs_independent_quint_verifications_in_parallel() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");
        let state_home = temp_dir.path().join("state");

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_parallel_observing_quint_tool(&tool_dir, "quint.events")?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .env("XDG_STATE_HOME", &state_home)
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
            .env("XDG_STATE_HOME", &state_home)
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
            .env("XDG_STATE_HOME", &state_home)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .env("XDG_STATE_HOME", &state_home)
            .env("EMC_VERIFY_PARALLELISM", "8")
            .assert()
            .success();

        assert_quint_verifications_overlap(&read_to_string(temp_dir.path().join("quint.events"))?);

        Ok(())
    }

    #[test]
    fn verify_limits_quint_verification_parallelism_across_processes() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");
        let state_home = temp_dir.path().join("state");
        let events_path = temp_dir.path().join("quint.events");
        let first_project = temp_dir.path().join("first");
        let second_project = temp_dir.path().join("second");
        let event_log = events_path.to_string_lossy().into_owned();

        create_fake_tool(&tool_dir, "lake", "lake.log")?;
        create_parallel_observing_quint_tool(&tool_dir, &event_log)?;
        create_modeled_project(&first_project, &state_home)?;
        create_modeled_project(&second_project, &state_home)?;

        let emc = cargo_bin("emc");
        let tool_path = path_with_fake_tools(&tool_dir)?;
        let first_verify = ProcessCommand::new(&emc)
            .arg("verify")
            .current_dir(&first_project)
            .env("PATH", &tool_path)
            .env("XDG_STATE_HOME", &state_home)
            .env("EMC_VERIFY_PARALLELISM", "2")
            .spawn()?;
        let second_verify = ProcessCommand::new(&emc)
            .arg("verify")
            .current_dir(&second_project)
            .env("PATH", &tool_path)
            .env("XDG_STATE_HOME", &state_home)
            .env("EMC_VERIFY_PARALLELISM", "2")
            .output()?;
        let first_verify = first_verify.wait_with_output()?;

        assert_command_success("first verify", &first_verify)?;
        assert_command_success("second verify", &second_verify)?;

        let events = read_to_string(events_path)?;
        assert_quint_verify_count(&events, 6);
        assert_max_quint_verification_parallelism(&events, 2);

        Ok(())
    }

    #[test]
    fn verify_appends_workflow_readiness_event_after_successful_verification()
    -> Result<(), Box<dyn Error>> {
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
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let verified_frontier =
            read_to_string(temp_dir.path().join("model/events/projection.fingerprint"))?;

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success();

        // A second verify over the same model is idempotent and must also succeed.
        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success();

        assert_eq!(
            read_to_string(temp_dir.path().join("model/events/projection.fingerprint"))?,
            verified_frontier,
            "verifying a workflow must not change the event frontier"
        );

        Ok(())
    }

    #[test]
    fn verify_rejects_workflow_readiness_when_event_frontier_changes() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

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
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // Pre-mint a real follow-on transaction in a sibling replica so the fake
        // `lake` tool can graft it into the committed store mid-verification,
        // moving the event frontier without hand-writing any event JSON.
        let replica = TempDir::new()?;
        let staged_transaction =
            stage_extra_committed_transaction(temp_dir.path(), replica.path())?;
        create_frontier_mutating_tool(&tool_dir, "lake", "lake.log", &staged_transaction)?;

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event frontier changed during verification",
            ));

        Ok(())
    }

    #[test]
    fn verify_declared_readiness_is_not_stale_at_unchanged_frontier() -> Result<(), Box<dyn Error>>
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
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success();

        // No model-changing event follows the readiness declaration. Because
        // WorkflowReadinessDeclared events are excluded from the projection
        // fingerprint, the declaration must not invalidate its own readiness:
        // the workflow is reported, but not as stale.
        Command::cargo_bin("emc")?
            .args(["list", "workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Open ticket"))
            .stdout(
                predicate::str::contains(
                    "workflow open-ticket readiness is stale for current event frontier",
                )
                .not(),
            );

        Ok(())
    }

    #[test]
    fn list_workflows_reports_readiness_stale_after_later_workflow_event()
    -> Result<(), Box<dyn Error>> {
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
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens a repair ticket with priority.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["list", "workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Open ticket"))
            .stdout(predicate::str::contains(
                "workflow open-ticket readiness is stale for current event frontier",
            ));

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
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelWorkflowBehaviorSurfaceIsComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowSliceModulesComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowSharedEventDefinitionsHaveIdenticalIdentity,workflowOnlyEventsMayBeSharedAcrossSlices,workflowCommandTransitionsTargetOwnedCommands,workflowCommandTransitionsSourceOwnedControls,workflowCommandTransitionsResolveControlsAndCommands,workflowStateViewCommandTransitionsTargetStateChanges,workflowEventTransitionsAreSharedByEndpointSlices,workflowEventTransitionsHaveParticipatingEndpointEvents,workflowNavigationTransitionsResolveControlsAndViews,workflowNavigationTransitionsResolveToEntryViews,workflowExternalTriggersDeclarePayloadContracts,workflowExternalTriggerPayloadContractsHaveProvenance,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates --server-endpoint <endpoint> model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceRepresentsOneCoherentModelUnit,sliceRepresentsSmallestUsefulBehaviorBoundary,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceNamedDefinitionsAreUniquelyOwned,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,contractScenariosCoverModeledContracts,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsWithoutIssuingControlsHaveProvenance,commandSessionInputsHaveDescriptions,commandInputsTraceToInvocationSources,commandInputsSourcedFromEventStreamsResolve,commandInputsSourcedFromExternalPayloadsResolve,commandInputsSourcedFromGeneratedValuesHaveCoordinates,commandInputsSourcedFromSessionValuesHaveCoordinates,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationSlicesRepresentOneReaction,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,externalBoundariesHavePayloadContractsAndFieldProvenance,translationsTargetKnownCommands,translationsReferenceObservedExternalEvents,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,automationBoardElementsAreDeclaredAutomations,externalBoardElementsAreObservedEvents,commandEventBoardEdgesMatchEmissions,eventReadModelBoardEdgesMatchProjectionSources,viewCommandBoardEdgesMatchControls,boardConnectionsHaveCausalSemantics,externalEventTriggersMatchTranslations,externalEventsDoNotUpdateReadModels,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,storedEventFactsTraceToOriginalSources,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,displayedDataTraceToOriginalProvenance,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputsHaveDescriptions,viewControlSessionInputsHaveDescriptions,viewControlInputVisibilityIsModeled,viewControlDecisionFieldsAreVisible,viewControlActorInputsAreVisible,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateViewSlicesOwnProjectionPaths,stateViewSlicesRepresentSingleViewProjectionBoundary,stateChangeSlicesOwnCommands,stateChangeSlicesOwnEvents,stateChangeSlicesOwnOutcomes,stateChangeSlicesOwnErrors,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,stateChangeSlicesDoNotOwnControlsOrSketches,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTypesAreDeclared,viewControlModeledViewNavigationTargetsResolve,viewControlExternalWorkflowNavigationTargetsNamed,viewControlExternalSystemNavigationTargetsHaveContracts,viewControlNavigationTargetsAreComplete --server-endpoint <endpoint> model/quint/slices/CaptureTicket.qnt\n"
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
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelWorkflowBehaviorSurfaceIsComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowSliceModulesComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowSharedEventDefinitionsHaveIdenticalIdentity,workflowOnlyEventsMayBeSharedAcrossSlices,workflowCommandTransitionsTargetOwnedCommands,workflowCommandTransitionsSourceOwnedControls,workflowCommandTransitionsResolveControlsAndCommands,workflowStateViewCommandTransitionsTargetStateChanges,workflowEventTransitionsAreSharedByEndpointSlices,workflowEventTransitionsHaveParticipatingEndpointEvents,workflowNavigationTransitionsResolveControlsAndViews,workflowNavigationTransitionsResolveToEntryViews,workflowExternalTriggersDeclarePayloadContracts,workflowExternalTriggerPayloadContractsHaveProvenance,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates --server-endpoint <endpoint> model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceRepresentsOneCoherentModelUnit,sliceRepresentsSmallestUsefulBehaviorBoundary,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceNamedDefinitionsAreUniquelyOwned,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,contractScenariosCoverModeledContracts,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsWithoutIssuingControlsHaveProvenance,commandSessionInputsHaveDescriptions,commandInputsTraceToInvocationSources,commandInputsSourcedFromEventStreamsResolve,commandInputsSourcedFromExternalPayloadsResolve,commandInputsSourcedFromGeneratedValuesHaveCoordinates,commandInputsSourcedFromSessionValuesHaveCoordinates,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationSlicesRepresentOneReaction,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,externalBoundariesHavePayloadContractsAndFieldProvenance,translationsTargetKnownCommands,translationsReferenceObservedExternalEvents,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,automationBoardElementsAreDeclaredAutomations,externalBoardElementsAreObservedEvents,commandEventBoardEdgesMatchEmissions,eventReadModelBoardEdgesMatchProjectionSources,viewCommandBoardEdgesMatchControls,boardConnectionsHaveCausalSemantics,externalEventTriggersMatchTranslations,externalEventsDoNotUpdateReadModels,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,storedEventFactsTraceToOriginalSources,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,displayedDataTraceToOriginalProvenance,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputsHaveDescriptions,viewControlSessionInputsHaveDescriptions,viewControlInputVisibilityIsModeled,viewControlDecisionFieldsAreVisible,viewControlActorInputsAreVisible,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateViewSlicesOwnProjectionPaths,stateViewSlicesRepresentSingleViewProjectionBoundary,stateChangeSlicesOwnCommands,stateChangeSlicesOwnEvents,stateChangeSlicesOwnOutcomes,stateChangeSlicesOwnErrors,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,stateChangeSlicesDoNotOwnControlsOrSketches,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTypesAreDeclared,viewControlModeledViewNavigationTargetsResolve,viewControlExternalWorkflowNavigationTargetsNamed,viewControlExternalSystemNavigationTargetsHaveContracts,viewControlNavigationTargetsAreComplete --server-endpoint <endpoint> model/quint/slices/CaptureTicket.qnt\n"
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

    fn create_parallel_observing_quint_tool(
        tool_dir: &Path,
        log_file: &str,
    ) -> Result<(), Box<dyn Error>> {
        create_dir_all(tool_dir)?;
        let tool_path = tool_dir.join("quint");
        write(
            &tool_path,
            format!(
                "#!/bin/sh\n\
                 if [ \"$1\" = verify ]; then\n\
                   printf 'start %s\\n' \"$*\" >> {log_file}\n\
                   sleep 1\n\
                   printf 'end %s\\n' \"$*\" >> {log_file}\n\
                 else\n\
                   printf '%s\\n' \"$*\" >> {log_file}\n\
                 fi\n"
            ),
        )?;
        set_permissions(&tool_path, Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn create_modeled_project(project_dir: &Path, state_home: &Path) -> Result<(), Box<dyn Error>> {
        create_dir_all(project_dir)?;
        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(project_dir)
            .env("XDG_STATE_HOME", state_home)
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
            .current_dir(project_dir)
            .env("XDG_STATE_HOME", state_home)
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
            .current_dir(project_dir)
            .env("XDG_STATE_HOME", state_home)
            .assert()
            .success();

        Ok(())
    }

    fn create_frontier_mutating_tool(
        tool_dir: &Path,
        tool_name: &str,
        log_file: &str,
        staged_transaction: &Path,
    ) -> Result<(), Box<dyn Error>> {
        create_dir_all(tool_dir)?;
        let tool_path = tool_dir.join(tool_name);
        let staged = staged_transaction.display();
        let staged_name = staged_transaction
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("staged transaction must have a file name")?;
        // Drop a genuine eventcore-fs transaction into the committed store
        // mid-verification so the event frontier moves between the snapshot taken
        // before verification and the post-verify check. The transaction was
        // produced by a real `emc` mutation in a sibling replica, so no event JSON
        // is hand-written here.
        write(
            &tool_path,
            format!(
                "#!/bin/sh\n\
                 printf '%s\\n' \"$*\" >> {log_file}\n\
                 if [ ! -f model/events/events/{staged_name} ]; then\n\
                   cp '{staged}' model/events/events/{staged_name}\n\
                 fi\n\
                 exit 0\n"
            ),
        )?;
        set_permissions(&tool_path, Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn stage_extra_committed_transaction(
        base_dir: &Path,
        stage_into: &Path,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let base_events = base_dir.join("model/events/events");
        let before = read_dir(&base_events)?
            .map(|entry| Ok(entry?.file_name()))
            .collect::<Result<BTreeSet<_>, Box<dyn Error>>>()?;

        // Clone the project into a sibling replica that does not share the store
        // lock, then author a real mutation there so eventcore-fs mints a valid
        // committed transaction we can graft onto the base.
        let copy_status = ProcessCommand::new("cp")
            .arg("-a")
            .arg(format!("{}/.", base_dir.display()))
            .arg(stage_into)
            .status()?;
        assert!(copy_status.success(), "cloning base project must succeed");

        let replica_events = stage_into.join("model/events");
        // These paths may or may not exist on the clone; a missing entry is the
        // desired post-state, so we bind (rather than discard) each removal
        // result and intentionally ignore it without panicking.
        let _removed_eventcore = remove_dir_all(replica_events.join(".eventcore"));
        let _removed_locks = remove_dir_all(replica_events.join("locks"));
        let _removed_index = remove_dir_all(replica_events.join("index"));
        let _removed_tmp = remove_dir_all(replica_events.join("tmp"));
        let _removed_lock_file = remove_file(replica_events.join(".lock"));

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Mutated mid-verification.",
            ])
            .current_dir(stage_into)
            .assert()
            .success();

        let replica_transactions = stage_into.join("model/events/events");
        let new_transaction = read_dir(&replica_transactions)?
            .map(|entry| Ok(entry?.path()))
            .collect::<Result<Vec<PathBuf>, Box<dyn Error>>>()?
            .into_iter()
            .find(|path| {
                path.extension().and_then(|ext| ext.to_str()) == Some("jsonl")
                    && path.file_name().is_some_and(|name| !before.contains(name))
            })
            .ok_or("replica must mint a new committed transaction")?;

        Ok(new_transaction)
    }

    fn normalize_quint_log(source: &str) -> String {
        let mut lines = source
            .lines()
            .map(normalize_quint_log_line)
            .collect::<Vec<_>>();
        lines.sort_by_key(|line| quint_log_line_order(line));
        lines.join("\n") + "\n"
    }

    fn normalize_quint_log_line(line: &str) -> String {
        let mut parts = line.split_whitespace().collect::<Vec<_>>();
        if let Some(endpoint_flag) = parts.iter().position(|part| *part == "--server-endpoint")
            && let Some(endpoint_value) = endpoint_flag.checked_add(1)
            && let Some(slot) = parts.get_mut(endpoint_value)
        {
            assert_ne!(
                *slot, "localhost:8822",
                "Quint verification must not use the shared default Apalache endpoint"
            );
            *slot = "<endpoint>";
        }
        parts.join(" ")
    }

    fn quint_log_line_order(line: &str) -> usize {
        if line.starts_with("typecheck ") {
            0
        } else if line.starts_with("verify --invariant modelIdentityStable") {
            1
        } else if line.starts_with("verify --invariant workflowIdentityStable") {
            2
        } else if line.starts_with("verify --invariant sliceIdentityStable") {
            3
        } else {
            4
        }
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
                        .and_then(|index| index.checked_add(1))
                        .and_then(|value_index| parts.get(value_index))
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

    fn assert_quint_verifications_overlap(source: &str) {
        let lines = source.lines().collect::<Vec<_>>();
        let first_end = lines
            .iter()
            .position(|line| line.starts_with("end verify "))
            .unwrap_or(lines.len());
        let starts_before_first_end = lines
            .iter()
            .take(first_end)
            .filter(|line| line.starts_with("start verify "))
            .count();

        assert!(
            starts_before_first_end > 1,
            "multiple Quint verification processes must be in flight before the first one exits:\n{source}"
        );
    }

    fn assert_quint_verify_count(source: &str, expected: usize) {
        let observed = source
            .lines()
            .filter(|line| line.starts_with("start verify "))
            .count();
        assert_eq!(
            observed, expected,
            "expected {expected} Quint verify starts, got {observed}:\n{source}"
        );
    }

    fn assert_max_quint_verification_parallelism(source: &str, expected_maximum: usize) {
        let mut active = 0_usize;
        let mut observed_maximum = 0_usize;
        for line in source.lines() {
            if line.starts_with("start verify ") {
                // A start/end balance over a log never approaches `usize::MAX`,
                // so saturation is exact rather than a clamp.
                active = active.saturating_add(1);
                observed_maximum = observed_maximum.max(active);
            } else if line.starts_with("end verify ") {
                assert!(
                    active > 0,
                    "Quint verification end must follow a start:\n{source}"
                );
                // Guarded by the `active > 0` assertion above, so the decrement
                // never underflows.
                active = active.saturating_sub(1);
            }
        }

        assert_eq!(active, 0, "all Quint verifications must end:\n{source}");
        assert!(
            observed_maximum <= expected_maximum,
            "at most {expected_maximum} Quint verifications may run at once, observed {observed_maximum}:\n{source}"
        );
    }

    fn assert_command_success(command_name: &str, output: &Output) -> Result<(), Box<dyn Error>> {
        if output.status.success() {
            return Ok(());
        }
        Err(format!(
            "{command_name} failed with {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
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
