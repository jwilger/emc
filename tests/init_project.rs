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
            fs::read_to_string(temp_dir.path().join("model/lean/lakefile.lean"))?
                .contains("package EMCModel")
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("model/quint/quint.json"))?,
            "{\n  \"main\": \"RepairDesk.qnt\",\n  \"invariants\": [\n    \"workflowIdentityStable\",\n    \"workflowSliceDetailsComplete\",\n    \"workflowTransitionsStructured\",\n    \"workflowTransitionSourcesResolve\",\n    \"workflowTransitionTargetsResolve\",\n    \"workflowStepRelationshipsAreAllowed\",\n    \"workflowStepSlugsAreUnique\",\n    \"workflowHasExactlyOneEntryStep\",\n    \"workflowMainStepsHaveIncomingReachability\",\n    \"workflowNonSupportingStepsReachableFromEntry\",\n    \"workflowBranchAndAlternateStepsHaveTriggerOrRationale\",\n    \"workflowTransitionsHaveModeledKinds\",\n    \"workflowExitsNameTargetsAndRationale\",\n    \"workflowExternallyRelevantOutcomesHandled\",\n    \"workflowOutcomesSourceResolve\",\n    \"workflowCommandErrorsSourceResolve\",\n    \"workflowTransitionsDoNotUseCommandErrorsAsOutcomes\",\n    \"workflowNonEventDefinitionsAreUniquelyOwned\",\n    \"workflowSharedEventDefinitionsHaveIdenticalIdentity\",\n    \"workflowCommandTransitionsResolveControlsAndCommands\",\n    \"workflowEventTransitionsAreSharedByEndpointSlices\",\n    \"workflowNavigationTransitionsResolveControlsAndViews\",\n    \"workflowExternalTriggersDeclarePayloadContracts\",\n    \"workflowTransitionsHaveRequiredEvidence\",\n    \"workflowEntryLifecycleStatesCoverRequiredStates\"\n  ]\n}\n"
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
