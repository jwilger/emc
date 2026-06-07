// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::env;
    use std::error::Error;
    use std::fs::{Permissions, create_dir_all, read_dir, read_to_string, set_permissions, write};
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

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
             verify --invariant modelIdentityStable,modelVersionStable,modelDigestStable,modelWorkflowsAreDeclared,modelSlicesAreDeclared,modelSliceModulesAreDeclared,modelScenariosAreDeclared,modelScenarioDefinitionsAreDeclared,modelWorkflowCompositionStructureComplete,modelWorkflowBehaviorSurfaceIsComplete,modelScenarioDefinitionsHaveGwt,modelScenarioKindsAreFirstClass,modelDataFlowsAreDeclared,modelDataFlowsAreBitComplete,modelDataFlowSourceKindsAreModeled,modelDataFlowModeledSourcesResolve,modelDataFlowSourceChainsReachOriginals,modelDataFlowSourceChainsPreserveBitEncodingSemantics,modelDataFlowTransformationsAreModeled,modelMeaningfulDataFlowsAreCovered,modelDataFlowSourceBitEncodingsMatchModeledSources,modelViewFieldBitEncodingsMatchDataFlows,modelExternalPayloadFieldBitEncodingsMatchDataFlows,modelOutcomesAreDeclared,modelCommandErrorsAreDeclared,modelCommandsAreDeclared,modelCommandInputsAreDeclared,modelCommandInputsHaveProvenance,modelCommandInputsTraceToInvocationSources,modelReadModelsAreDeclared,modelReadModelDefinitionsAreDeclared,modelReadModelFieldsAreDeclared,modelReadModelFieldSourcesAreComplete,modelViewFieldSourcesAreComplete,modelViewFieldReadModelFieldSourcesResolve,modelDisplayedDataTraceToOriginalProvenance,modelExternalPayloadFieldsHaveProvenance,modelViewsAreDeclared,modelViewDefinitionsAreDeclared,modelViewControlsAreDeclared,modelBoardElementsAreDeclared,modelBoardConnectionsAreDeclared,modelViewFieldsAreDeclared,modelAutomationsAreDeclared,modelAutomationDefinitionsAreDeclared,modelTranslationsAreDeclared,modelTranslationDefinitionsAreDeclared,modelExternalPayloadsAreDeclared,modelExternalPayloadFieldsAreDeclared,modelStreamsAreDeclared,modelEventsAreDeclared,modelEventAttributesAreDeclared,modelViewControlsProvideCommandInputs --server-endpoint <endpoint> model/quint/RepairDesk.qnt\n"
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

        let readiness_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowReadinessDeclared")
            .ok_or("verify must export a WorkflowReadinessDeclared event")?;

        assert_eq!(readiness_event["stream_id"], "workflow::open-ticket");
        assert_eq!(readiness_event["payload"]["workflow"], "open-ticket");
        assert_eq!(
            readiness_event["payload"]["projection_fingerprint"],
            verified_frontier.trim()
        );
        assert_eq!(readiness_event["payload"]["verified_by"], "emc verify");
        assert_eq!(
            readiness_event["payload"]["review_event_id"],
            serde_json::Value::Null
        );
        assert!(
            readiness_event["payload"]["verified_at"]
                .as_str()
                .is_some_and(|value| value.ends_with('Z')),
            "readiness event must record a UTC verification timestamp"
        );
        assert!(
            readiness_event["payload"]["model_content_digest"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "readiness event must record the verified model content digest"
        );

        Ok(())
    }

    #[test]
    fn verify_rejects_workflow_readiness_when_event_frontier_changes() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let tool_dir = temp_dir.path().join("tools");

        create_frontier_mutating_tool(&tool_dir, "lake", "lake.log")?;
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

        Command::cargo_bin("emc")?
            .arg("verify")
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event frontier changed during verification",
            ));

        assert!(
            exported_events(temp_dir.path().join("model/events/v1"))?
                .into_iter()
                .all(|event| event["type"] != "WorkflowReadinessDeclared"),
            "verify must not export readiness for a changed event frontier"
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

    fn create_frontier_mutating_tool(
        tool_dir: &Path,
        tool_name: &str,
        log_file: &str,
    ) -> Result<(), Box<dyn Error>> {
        create_dir_all(tool_dir)?;
        let tool_path = tool_dir.join(tool_name);
        write(
            &tool_path,
            format!(
                "#!/bin/sh\n\
                 printf '%s\\n' \"$*\" >> {log_file}\n\
                 mkdir -p model/events/v1\n\
                 printf '%s\\n' '{{\"event_id\":\"frontier-changed\",\"parents\":[],\"type\":\"WorkflowUpdated\"}}' > model/events/v1/frontier-changed.json\n"
            ),
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

    fn exported_events(path: PathBuf) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
        let mut events = read_dir(path)?
            .map(|entry| {
                let contents = read_to_string(entry?.path())?;
                serde_json::from_str::<serde_json::Value>(&contents).map_err(Into::into)
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        events.sort_by_key(|event| event["parents"].as_array().map_or(0, Vec::len));
        Ok(events)
    }
}
