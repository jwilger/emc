#[cfg(test)]
mod tests {
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
            read_to_string(temp_dir.path().join("quint.log"))?,
            "typecheck model/quint/RepairDesk.qnt\n"
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
            read_to_string(temp_dir.path().join("quint.log"))?,
            "typecheck model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowCommandTransitionsResolveControlsAndCommands,workflowEventTransitionsAreSharedByEndpointSlices,workflowNavigationTransitionsResolveControlsAndViews,workflowExternalTriggersDeclarePayloadContracts,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsSourcedFromEventStreamsResolve,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,translationsTargetKnownCommands,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,boardConnectionsHaveCausalSemantics,externalEventsDoNotUpdateReadModels,readModelsDoNotFeedCommands,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputVisibilityIsModeled,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateChangeSlicesOwnCommands,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTargetsAreComplete model/quint/slices/CaptureTicket.qnt\n"
        );

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
            read_to_string(temp_dir.path().join("quint.log"))?,
            "typecheck model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowCommandTransitionsResolveControlsAndCommands,workflowEventTransitionsAreSharedByEndpointSlices,workflowNavigationTransitionsResolveControlsAndViews,workflowExternalTriggersDeclarePayloadContracts,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsSourcedFromEventStreamsResolve,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,translationsTargetKnownCommands,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,boardConnectionsHaveCausalSemantics,externalEventsDoNotUpdateReadModels,readModelsDoNotFeedCommands,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputVisibilityIsModeled,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateChangeSlicesOwnCommands,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTargetsAreComplete model/quint/slices/CaptureTicket.qnt\n"
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
            read_to_string(temp_dir.path().join("quint.log"))?,
            "typecheck model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowCommandTransitionsResolveControlsAndCommands,workflowEventTransitionsAreSharedByEndpointSlices,workflowNavigationTransitionsResolveControlsAndViews,workflowExternalTriggersDeclarePayloadContracts,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsSourcedFromEventStreamsResolve,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,translationsTargetKnownCommands,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,boardConnectionsHaveCausalSemantics,externalEventsDoNotUpdateReadModels,readModelsDoNotFeedCommands,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlInputVisibilityIsModeled,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateChangeSlicesOwnCommands,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTargetsAreComplete model/quint/slices/CaptureTicket.qnt\n"
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
