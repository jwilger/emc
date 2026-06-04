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
    fn mcp_stdio_verifies_modeled_workflow_artifacts() -> Result<(), Box<dyn Error>> {
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

        let assert = Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .write_stdin(mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"verify_project\""))
            .stdout(predicate::str::contains("Lean4 artifacts verified"))
            .stdout(predicate::str::contains("Quint artifacts verified"));

        let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
        assert!(
            stdout.lines().all(|line| line.starts_with('{')),
            "MCP stdout must contain only JSON-RPC frames, got:\n{stdout}"
        );

        assert_eq!(
            read_to_string(temp_dir.path().join("lake.log"))?,
            "env lean model/lean/RepairDesk.lean\n\
             env lean model/lean/OpenTicket.lean\n\
             env lean model/lean/slices/CaptureTicket.lean\n"
        );
        assert_eq!(
            read_to_string(temp_dir.path().join("quint.log"))?,
            "typecheck model/quint/RepairDesk.qnt\n\
             verify --invariant workflowIdentityStable,workflowSliceDetailsComplete,workflowTransitionsStructured,workflowTransitionSourcesResolve,workflowTransitionTargetsResolve,workflowStepRelationshipsAreAllowed,workflowStepSlugsAreUnique,workflowHasExactlyOneEntryStep,workflowMainStepsHaveIncomingReachability,workflowNonSupportingStepsReachableFromEntry,workflowBranchAndAlternateStepsHaveTriggerOrRationale,workflowTransitionsHaveModeledKinds,workflowExitsNameTargetsAndRationale,workflowExternallyRelevantOutcomesHandled,workflowOutcomesSourceResolve,workflowCommandErrorsSourceResolve,workflowTransitionsDoNotUseCommandErrorsAsOutcomes,workflowNonEventDefinitionsAreUniquelyOwned,workflowSharedEventDefinitionsHaveIdenticalIdentity,workflowCommandTransitionsTargetOwnedCommands,workflowCommandTransitionsSourceOwnedControls,workflowCommandTransitionsResolveControlsAndCommands,workflowEventTransitionsAreSharedByEndpointSlices,workflowNavigationTransitionsResolveControlsAndViews,workflowExternalTriggersDeclarePayloadContracts,workflowExternalTriggerPayloadContractsHaveProvenance,workflowTransitionsHaveRequiredEvidence,workflowEntryLifecycleStatesCoverRequiredStates model/quint/OpenTicket.qnt\n\
             verify --invariant sliceIdentityStable,sliceStateChangeRequiresEvent,sliceBitLevelDataFlowsStructured,modeledDataFlowsAreBitComplete,sliceScenariosHaveGwt,sliceScenarioNamesAreUnique,sliceScenarioStreamsResolve,stateChangeScenariosNameStreams,acceptanceScenariosAreUserFacing,stateViewReadModelsHaveProjectorContracts,contractScenariosTargetKnownDefinitions,commandInputsHaveAllowedSources,commandInputsHaveProvenance,commandInputsWithoutIssuingControlsHaveProvenance,commandSessionInputsHaveDescriptions,commandInputsTraceToInvocationSources,commandInputsSourcedFromEventStreamsResolve,commandErrorsAreDeclared,commandErrorsHaveAllowedRecovery,commandErrorsHaveScenarioCoverage,scenarioErrorReferencesAreDeclared,singletonCommandsDeclareRepeatBehavior,automationSlicesDeclareTriggers,automationSlicesRepresentOneReaction,automationsIssueKnownCommands,automationsHandleCommandErrors,translationSlicesDeclareExternalContracts,externalBoundariesHavePayloadContractsAndFieldProvenance,translationsTargetKnownCommands,translationsReferenceObservedExternalEvents,boardLanesAreCanonical,boardElementsUseCanonicalLanes,boardElementsReferenceDeclarations,automationBoardElementsAreDeclaredAutomations,externalBoardElementsAreObservedEvents,commandEventBoardEdgesMatchEmissions,eventReadModelBoardEdgesMatchProjectionSources,viewCommandBoardEdgesMatchControls,boardConnectionsHaveCausalSemantics,externalEventTriggersMatchTranslations,externalEventsDoNotUpdateReadModels,readModelsDoNotFeedCommands,readModelsFeedingViewsHaveIncomingEventUpdates,commandsHaveIncomingTriggers,mainPathBoardHasNoDisconnectedIslands,outcomeLabelsAreUnique,outcomeEventSetsAreNonEmpty,outcomeEventSetsAreDistinct,outcomeEventsAreKnownToSlice,eventsReferenceKnownStreams,commandEmittedEventsAreKnown,locallyEmittedEventsAreProducedByCommands,externalPayloadFieldsHaveProvenance,eventAttributesHaveAllowedSources,eventAttributesHaveProvenance,eventAttributeSourcesAreComplete,storedEventFactsTraceToOriginalSources,readModelFieldsHaveAllowedSources,readModelFieldsHaveProvenance,readModelFieldSourcesAreComplete,readModelFieldEventAttributeSourcesResolve,derivedReadModelFieldsHaveScenarioCoverage,absenceReadModelFieldsHaveScenarioCoverage,transitiveReadModelsHaveSemantics,viewFieldsHaveAllowedSources,viewFieldsHaveProvenance,viewFieldSourcesAreComplete,viewFieldsSourceFromUsedReadModels,viewsHaveInformationSketches,viewFieldsAppearInSketch,viewSketchTokensMapToModeledElements,viewFieldReadModelFieldSourcesResolve,displayedDataTraceToOriginalProvenance,viewControlsHaveSketchTokens,viewControlsAppearInSketch,viewControlsReferenceKnownCommands,viewControlsProvideCommandInputs,viewControlInputsHaveAllowedSources,viewControlInputsHaveProvenance,viewControlSessionInputsHaveDescriptions,viewControlInputVisibilityIsModeled,viewControlDecisionFieldsAreVisible,viewControlActorInputsAreVisible,viewControlsHandleCommandErrors,viewControlRecoveryBehaviorIsModeled,stateViewSlicesDoNotOwnCommands,stateViewSlicesOwnViews,stateViewSlicesOwnReadModels,stateViewSlicesOwnProjectionPaths,stateChangeSlicesOwnCommands,stateChangeSlicesOwnEvents,stateChangeSlicesOwnOutcomes,stateChangeSlicesOwnErrors,stateChangeSlicesDoNotOwnReadModelsOrViews,stateChangeSlicesDoNotOwnAutomationsOrTranslations,stateChangeSlicesDoNotOwnControlsOrSketches,translationSlicesDoNotOwnViews,viewControlNavigationTypesAreModeled,viewControlNavigationTargetsAreComplete model/quint/slices/CaptureTicket.qnt\n"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_reports_verify_failures_as_json_rpc_errors() -> Result<(), Box<dyn Error>> {
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
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .env("PATH", path_with_fake_tools(&tool_dir)?)
            .write_stdin(mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"error\""))
            .stdout(predicate::str::contains("\"code\":-32000"))
            .stdout(predicate::str::contains("Lean4 verification failed"))
            .stdout(predicate::str::contains("failing tool stdout"))
            .stdout(predicate::str::contains("failing tool stderr"))
            .stdout(predicate::str::contains("Run `emc check`"));

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"verify_project\",\"arguments\":{}}}\n",
        )
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
            format!(
                "#!/bin/sh\nprintf 'tool stdout noise\\n'\nprintf 'tool stderr noise\\n' >&2\nprintf '%s\\n' \"$*\" >> {log_file}\n"
            ),
        )?;
        set_permissions(&tool_path, Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn create_failing_tool(tool_dir: &Path, tool_name: &str) -> Result<(), Box<dyn Error>> {
        create_dir_all(tool_dir)?;
        let tool_path = tool_dir.join(tool_name);
        write(
            &tool_path,
            "#!/bin/sh\nprintf 'failing tool stdout\\n'\nprintf 'failing tool stderr\\n' >&2\nexit 7\n",
        )?;
        set_permissions(&tool_path, Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn path_with_fake_tools(tool_dir: &Path) -> Result<String, Box<dyn Error>> {
        let mut paths = vec![tool_dir.to_path_buf()];
        paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
        Ok(env::join_paths(paths)?.to_string_lossy().into_owned())
    }
}
