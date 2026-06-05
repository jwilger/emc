// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn init_creates_deterministic_project_layout() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "EMC project Repair Desk layout is present",
            ));

        let expected_paths = [
            "emc.toml",
            "model/lean/lakefile.lean",
            "model/lean/lean-toolchain",
            "model/lean/RepairDesk.lean",
            "model/lean/slices/.gitkeep",
            "model/quint/quint.json",
            "model/quint/RepairDesk.qnt",
            "model/quint/slices/.gitkeep",
            "reviews/.gitkeep",
        ];

        expected_paths
            .iter()
            .map(|relative_path| temp_dir.path().join(relative_path))
            .for_each(|path| assert!(path.exists(), "expected {} to exist", path.display()));

        assert_eq!(
            fs::read_to_string(temp_dir.path().join("model/lean/lean-toolchain"))?,
            "leanprover/lean4:4.29.1\n"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("emc.toml"))?.contains("version = \"0.1.0\""),
            "project manifest must carry the formal model version"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("def modelVersion := \"0.1.0\""),
            "Lean project root must carry the formal model version"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("def modelName := \"Repair Desk\""),
            "Lean project root must carry the project model name"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelDigest := \"project:name=Repair Desk;version=0.1.0;workflows=;slices=;scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=\""
            ),
            "Lean project root must carry a deterministic project model digest"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("theorem modelVersionIsStable : modelVersion = \"0.1.0\" := rfl"),
            "Lean project root must prove the formal model version"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?
                .contains("theorem modelIdentityIsStable : modelName = \"Repair Desk\" := rfl"),
            "Lean project root must prove project model identity"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelDigestIsStable : modelDigest = \"project:name=Repair Desk;version=0.1.0;workflows=;slices=;scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=\" := rfl"
            ),
            "Lean project root must prove project model digest stability"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelControlProvidesCommandInput (control : String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) (input : String × String × String × String × String × String × List String × String × String × String × String × String × String × String × String) : Bool := control.1 == input.1 && control.2.2.2.2.1 == input.2.2.1 && control.2.2.2.2.2.1 == input.2.2.2.1"
            ),
            "Lean project root must be able to prove controls provide target command inputs across composed slices"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelViewControlProvidesEveryCommandInput (control : String × String × String × String × String × String × String × String × String × Bool × Bool × List String × String × String × String × String × String × String × String) : Bool := modelCommandInputs.all (fun input => input.1 != control.1 || input.2.2.1 != control.2.2.2.2.1 || modelViewControls.any (fun providedInput => providedInput.1 == control.1 && providedInput.2.1 == control.2.1 && providedInput.2.2.1 == control.2.2.1 && providedInput.2.2.2.1 == control.2.2.2.1 && providedInput.2.2.2.2.1 == control.2.2.2.2.1 && modelControlProvidesCommandInput providedInput input))"
            ),
            "Lean project root must prove each control invocation supplies every input required by its target command"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "theorem modelViewControlsProvideCommandInputs : modelViewControls.all modelViewControlProvidesEveryCommandInput = true := rfl"
            ),
            "Lean project root must expose cross-slice control input completeness as a theorem"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelVersion = \"0.1.0\""),
            "Quint project root must carry the formal model version"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelName = \"Repair Desk\""),
            "Quint project root must carry the project model name"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDigest = \"project:name=Repair Desk;version=0.1.0;workflows=;slices=;scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=\""
            ),
            "Quint project root must carry a deterministic project model digest"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelVersionStable = modelVersion == \"0.1.0\""),
            "Quint project root must expose the formal model version check"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelIdentityStable = modelName == \"Repair Desk\""),
            "Quint project root must expose the project model identity check"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelDigestStable = modelDigest == \"project:name=Repair Desk;version=0.1.0;workflows=;slices=;scenarios=;scenario-definitions=;data-flows=;outcomes=;command-errors=;commands=;command-inputs=;read-models=;read-model-definitions=;read-model-fields=;views=;view-definitions=;view-controls=;board-elements=;board-connections=;view-fields=;automations=;automation-definitions=;translations=;translation-definitions=;external-payloads=;external-payload-fields=;streams=;events=;event-attributes=\""
            ),
            "Quint project root must expose the project model digest invariant"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelControlProvidesCommandInput(control, input) = control.workflow == input.workflow and control.command == input.command and control.input == input.input"
            ),
            "Quint project root must be able to verify controls provide target command inputs across composed slices"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "def modelViewControlProvidesEveryCommandInput(control) = modelCommandInputs.select(input => input.workflow != control.workflow or input.command != control.command or modelViewControls.select(providedInput => providedInput.workflow == control.workflow and providedInput.slice == control.slice and providedInput.view == control.view and providedInput.control == control.control and providedInput.command == control.command and modelControlProvidesCommandInput(providedInput, input)).length() > 0).length() == modelCommandInputs.length()"
            ),
            "Quint project root must verify each control invocation supplies every input required by its target command"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?.contains(
                "val modelViewControlsProvideCommandInputs = modelViewControls.select(control => modelViewControlProvidesEveryCommandInput(control)).length() == modelViewControls.length()"
            ),
            "Quint project root must expose cross-slice control input completeness as an invariant"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/lakefile.lean"))?
                .contains("package EMCModel")
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("model/quint/quint.json"))?,
            "{\n  \"main\": \"RepairDesk.qnt\",\n  \"invariants\": [\n    \"workflowIdentityStable\",\n    \"workflowSliceDetailsComplete\",\n    \"workflowSliceModulesComplete\",\n    \"workflowTransitionsStructured\",\n    \"workflowTransitionSourcesResolve\",\n    \"workflowTransitionTargetsResolve\",\n    \"workflowStepRelationshipsAreAllowed\",\n    \"workflowStepSlugsAreUnique\",\n    \"workflowHasExactlyOneEntryStep\",\n    \"workflowMainStepsHaveIncomingReachability\",\n    \"workflowNonSupportingStepsReachableFromEntry\",\n    \"workflowBranchAndAlternateStepsHaveTriggerOrRationale\",\n    \"workflowTransitionsHaveModeledKinds\",\n    \"workflowExitsNameTargetsAndRationale\",\n    \"workflowExternallyRelevantOutcomesHandled\",\n    \"workflowOutcomesSourceResolve\",\n    \"workflowCommandErrorsSourceResolve\",\n    \"workflowTransitionsDoNotUseCommandErrorsAsOutcomes\",\n    \"workflowNonEventDefinitionsAreUniquelyOwned\",\n    \"workflowSharedEventDefinitionsHaveIdenticalIdentity\",\n    \"workflowCommandTransitionsResolveControlsAndCommands\",\n    \"workflowEventTransitionsAreSharedByEndpointSlices\",\n    \"workflowNavigationTransitionsResolveControlsAndViews\",\n    \"workflowExternalTriggersDeclarePayloadContracts\",\n    \"workflowExternalTriggerPayloadContractsHaveProvenance\",\n    \"workflowTransitionsHaveRequiredEvidence\",\n    \"workflowEntryLifecycleStatesCoverRequiredStates\",\n    \"modelViewControlsProvideCommandInputs\"\n  ]\n}\n"
        );
        Ok(())
    }

    #[test]
    fn init_does_not_overwrite_existing_project_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let manifest_path = temp_dir.path().join("emc.toml");
        let user_manifest = "[project]\nname = \"User Edited\"\n";

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::write(&manifest_path, user_manifest)?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let actual_manifest = fs::read_to_string(&manifest_path)?;
        assert_eq!(
            actual_manifest, user_manifest,
            "re-running init must not overwrite existing project files"
        );

        Ok(())
    }

    #[test]
    fn init_requires_exact_name_flag() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--wrong-name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc init --name <project-name>",
            ));

        assert!(
            !temp_dir.path().join("emc.toml").exists(),
            "unsupported init syntax must not create project files"
        );

        Ok(())
    }
}
