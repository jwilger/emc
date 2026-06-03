#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{read_to_string, remove_file, write};
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn check_reports_initialized_project_artifacts_are_present() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("project layout is complete"));

        Ok(())
    }

    #[test]
    fn check_uses_the_initialized_project_name() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Inventory Intake"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("project layout is complete"));

        Ok(())
    }

    #[test]
    fn check_reports_missing_project_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        remove_file(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "missing required project artifact model/quint/RepairDesk.qnt",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_project_manifest_lean_module_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("emc.toml"),
            "[project]\nname = \"Repair Desk\"\nlean_module = \"StaleRoot\"\nquint_module = \"RepairDesk\"\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "project manifest drift for Repair Desk",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_project_manifest_quint_module_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("emc.toml"),
            "[project]\nname = \"Repair Desk\"\nlean_module = \"RepairDesk\"\nquint_module = \"StaleRoot\"\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "project manifest drift for Repair Desk",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_verification_entrypoint_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        remove_file(temp_dir.path().join("model/lean/lakefile.lean"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "missing required project artifact model/lean/lakefile.lean",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_formal_slice_layout_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        remove_file(temp_dir.path().join("model/quint/slices/.gitkeep"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "missing required project artifact model/quint/slices/.gitkeep",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_project_lakefile_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("model/lean/lakefile.lean"),
            "package StaleModel\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean project config drift for Repair Desk",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_project_toolchain_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("model/lean/lean-toolchain"),
            "leanprover/lean4:4.28.0\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean project config drift for Repair Desk",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_project_root_formal_artifact_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("model/lean/RepairDesk.lean"),
            "namespace StaleRoot\n\n-- EMC generated Lean4 model root.\n\nend StaleRoot\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean project root drift for Repair Desk",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_project_root_lean_end_module_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("model/lean/RepairDesk.lean"),
            "namespace RepairDesk\n\n-- EMC generated Lean4 model root.\n\nend StaleRoot\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean project root drift for Repair Desk",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_project_root_quint_module_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("model/quint/RepairDesk.qnt"),
            "module StaleRoot {\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint project root drift for Repair Desk",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_project_quint_config_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("model/quint/quint.json"),
            "{\n  \"main\": \"RepairDesk.qnt\",\n  \"invariants\": []\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint project config drift for Repair Desk",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_modeled_workflow_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

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

        remove_file(temp_dir.path().join("model/lean/OpenTicket.lean"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "missing required project artifact model/lean/OpenTicket.lean",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unmodeled_lean_workflow_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

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

        write(
            temp_dir.path().join("model/lean/OrphanWorkflow.lean"),
            "namespace OrphanWorkflow\n\nend OrphanWorkflow\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean model artifact drift for OrphanWorkflow.lean",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unmodeled_quint_workflow_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

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

        write(
            temp_dir.path().join("model/quint/OrphanWorkflow.qnt"),
            "module OrphanWorkflow\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint model artifact drift for OrphanWorkflow.qnt",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_modeled_workflow_artifact_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

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

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\ndef workflowName := \"Changed\"\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow field drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_digest_marker_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str("\n-- EMC-DIGEST: workflow:Stale ticket\n");
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "artifact digest mismatch for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_digest_drift_after_workflow_description_changes()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

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

        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let old_lean = read_to_string(&lean_path)?;
        let old_digest = old_lean
            .lines()
            .find(|line| line.starts_with("-- EMC-DIGEST: "))
            .ok_or("generated Lean artifact is missing digest marker")?
            .to_owned();

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

        let updated_lean = read_to_string(&lean_path)?;
        let current_digest = updated_lean
            .lines()
            .find(|line| line.starts_with("-- EMC-DIGEST: "))
            .ok_or("updated Lean artifact is missing digest marker")?;
        assert_ne!(
            old_digest, current_digest,
            "workflow digest must change when semantic workflow content changes"
        );

        write(lean_path, updated_lean.replace(current_digest, &old_digest))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "artifact digest mismatch for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_module_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let lean = read_to_string(&lean_path)?.replace("namespace OpenTicket", "namespace Stale");
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow module drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_end_module_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let lean = read_to_string(&lean_path)?.replace("end OpenTicket", "end Stale");
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow module drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_module_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let quint = read_to_string(&quint_path)?.replace("module OpenTicket {", "module Stale {");
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow module drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_workflow_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

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

        write(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
            "{\n  \"name\": \"Changed\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Actor opens a repair ticket.\",\n  \"slice_files\": [],\n  \"steps\": []\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_workflow_extra_name_field_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let workflow = read_to_string(&workflow_path)?.replace(
            "  \"name\": \"Open ticket\",\n",
            "  \"name\": \"Stale ticket\",\n  \"name\": \"Open ticket\",\n",
        );
        write(workflow_path, workflow)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_workflow_extra_description_field_drift() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let workflow = read_to_string(&workflow_path)?.replace(
            "  \"description\": \"Actor opens a repair ticket.\",\n",
            "  \"description\": \"Stale description.\",\n  \"description\": \"Actor opens a repair ticket.\",\n",
        );
        write(workflow_path, workflow)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_workflow_duplicate_steps_field_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let workflow = read_to_string(&workflow_path)?
            .replace("  \"steps\": [\n", "  \"steps\": [],\n  \"steps\": [\n");
        write(workflow_path, workflow)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_workflow_duplicate_numeric_field_drift() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let workflow = read_to_string(&workflow_path)?.replace(
            "  \"events\": [],\n",
            "  \"rank\": 1,\n  \"rank\": 2,\n  \"events\": [],\n",
        );
        write(workflow_path, workflow)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_workflow_duplicate_escaped_field_drift() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let workflow = read_to_string(&workflow_path)?.replace(
            "  \"events\": [],\n",
            "  \"escaped\\\"key\": \"first\",\n  \"escaped\\\"key\": \"second\",\n  \"events\": [],\n",
        );
        write(workflow_path, workflow)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_workflow_duplicate_after_escaped_string_value_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let workflow = read_to_string(&workflow_path)?.replace(
            "  \"events\": [],\n",
            "  \"note\": \"contains an escaped \\\" quote\",\n  \"rank\": 1,\n  \"rank\": 2,\n  \"events\": [],\n",
        );
        write(workflow_path, workflow)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_referenced_browser_slice_duplicate_field_drift() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let slice_path = temp_dir
            .path()
            .join("model/browser/data/slices/capture-ticket.eventmodel.json");
        let slice = read_to_string(&slice_path)?.replace(
            "  \"name\": \"Capture ticket\",\n",
            "  \"name\": \"Stale capture ticket\",\n  \"name\": \"Capture ticket\",\n",
        );
        write(slice_path, slice)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_referenced_browser_slice_invalid_json_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        write(
            temp_dir
                .path()
                .join("model/browser/data/slices/capture-ticket.eventmodel.json"),
            "{ not-json",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unindexed_browser_workflow_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

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

        write(
            temp_dir.path().join("model/browser/data/index.json"),
            "{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": []\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser workflow index drift for open-ticket.eventmodel.json",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_index_duplicate_workflows_field_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let index_path = temp_dir.path().join("model/browser/data/index.json");
        let index = read_to_string(&index_path)?.replace(
            "  \"workflows\": [\n",
            "  \"workflows\": [],\n  \"workflows\": [\n",
        );
        write(index_path, index)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("browser index drift"));

        Ok(())
    }

    #[test]
    fn check_reports_browser_index_duplicate_workflow_path_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let index_path = temp_dir.path().join("model/browser/data/index.json");
        let workflow_entry = concat!(
            "    {\n",
            "      \"name\": \"Open ticket\",\n",
            "      \"path\": \"data/workflows/open-ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    }"
        );
        let duplicate_workflows = concat!(
            "    {\n",
            "      \"name\": \"Open ticket\",\n",
            "      \"path\": \"data/workflows/open-ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    },\n",
            "    {\n",
            "      \"name\": \"Open ticket\",\n",
            "      \"path\": \"data/workflows/open-ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    }"
        );
        let index = read_to_string(&index_path)?.replace(workflow_entry, duplicate_workflows);
        write(index_path, index)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser index workflow path is duplicated",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_index_duplicate_workflow_name_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let duplicate_workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/duplicate-ticket.eventmodel.json");
        write(&duplicate_workflow_path, read_to_string(workflow_path)?)?;

        let index_path = temp_dir.path().join("model/browser/data/index.json");
        let workflow_entry = concat!(
            "    {\n",
            "      \"name\": \"Open ticket\",\n",
            "      \"path\": \"data/workflows/open-ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    }"
        );
        let duplicate_workflows = concat!(
            "    {\n",
            "      \"name\": \"Open ticket\",\n",
            "      \"path\": \"data/workflows/open-ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    },\n",
            "    {\n",
            "      \"name\": \"Open ticket\",\n",
            "      \"path\": \"data/workflows/duplicate-ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    }"
        );
        let index = read_to_string(&index_path)?.replace(workflow_entry, duplicate_workflows);
        write(index_path, index)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser index workflow name is duplicated",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_browser_index_duplicate_workflow_slug_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let duplicate_workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open_ticket.eventmodel.json");
        let duplicate_workflow = read_to_string(workflow_path)?.replace(
            "\"name\": \"Open ticket\"",
            "\"name\": \"Open ticket duplicate\"",
        );
        write(&duplicate_workflow_path, duplicate_workflow)?;

        let index_path = temp_dir.path().join("model/browser/data/index.json");
        let workflow_entry = concat!(
            "    {\n",
            "      \"name\": \"Open ticket\",\n",
            "      \"path\": \"data/workflows/open-ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    }"
        );
        let duplicate_workflows = concat!(
            "    {\n",
            "      \"name\": \"Open ticket\",\n",
            "      \"path\": \"data/workflows/open-ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    },\n",
            "    {\n",
            "      \"name\": \"Open ticket duplicate\",\n",
            "      \"path\": \"data/workflows/open_ticket.eventmodel.json\",\n",
            "      \"description\": \"Actor opens a repair ticket.\"\n",
            "    }"
        );
        let index = read_to_string(&index_path)?.replace(workflow_entry, duplicate_workflows);
        write(index_path, index)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser index workflow slug is duplicated",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_field_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

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

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Changed\"\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow field drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_name_declaration_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str("\ndef workflowName := \"Stale ticket\"\n");
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow field drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_extra_description_declaration_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let mut quint = read_to_string(&quint_path)?;
        quint.push_str("  val workflowDescription = \"Stale description.\"\n");
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow field drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_transition_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices : List String := [\"capture-ticket\",\"review-ticket\"]\n\ndef workflowSliceDetails : List (String × String × String × String) := [(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Slice description.\"),(\"review-ticket\", \"Review ticket\", \"state_view\", \"Slice description.\")]\n\ndef workflowTransitions : List WorkflowTransition := []\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_transition_declaration_drift() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str(
            "\ndef workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"stale-screen\" }]\n",
        );
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_exit_transition_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_workflow_exit(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices : List String := [\"capture-ticket\"]\n\ndef workflowSliceDetails : List (String × String × String × String) := [(\"capture-ticket\", \"Capture ticket\", \"state_view\", \"Slice description.\")]\n\ndef workflowTransitions : List WorkflowTransition := []\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_accepts_synchronized_workflow_exit_transition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_workflow_exit(&temp_dir)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("project layout is complete"));

        Ok(())
    }

    #[test]
    fn check_accepts_synchronized_workflow_navigation_transition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("project layout is complete"));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_slice_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices : List String := []\n\ndef workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\" }]\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_slice_declaration_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str("\ndef workflowSlices : List String := [\"stale-slice\"]\n");
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_slice_detail_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices : List String := [\"capture-ticket\",\"review-ticket\"]\n\ndef workflowSliceDetails : List (String × String × String × String) := []\n\ndef workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\" }]\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow slice detail drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_extra_slice_detail_declaration_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let mut lean = read_to_string(&lean_path)?;
        lean.push_str(
            "\ndef workflowSliceDetails : List (String × String × String × String) := [(\"stale-slice\", \"Stale slice\", \"state_view\", \"Stale description.\")]\n",
        );
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow slice detail drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_invariant_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let lean = read_to_string(&lean_path)?.replace(
            "theorem workflowSlicesHaveDetails : workflowSlices.length = workflowSliceDetails.length := rfl",
            "theorem workflowSlicesHaveDetails : workflowSlices.length = 0 := rfl",
        );
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow invariant drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_identity_invariant_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let lean = read_to_string(&lean_path)?.replace(
            "theorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl",
            "theorem workflowIdentityIsStable : workflowName = \"Stale ticket\" := rfl",
        );
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow invariant drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_workflow_transition_invariant_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let lean = read_to_string(&lean_path)?.replace(
            "theorem workflowTransitionsAreStructured : workflowTransitions.all (fun transition => transition.source.isEmpty == false && transition.target.isEmpty == false && transition.kind.isEmpty == false && transition.trigger.isEmpty == false) = true := rfl",
            "theorem workflowTransitionsAreStructured : workflowTransitions.length = 0 := rfl",
        );
        write(lean_path, lean)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean workflow invariant drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_transition_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/quint/OpenTicket.qnt"),
            "module OpenTicket {\n  // EMC-DIGEST: workflow:Open ticket\n  val workflowName = \"Open ticket\"\n  val workflowSlug = \"open-ticket\"\n  val workflowDescription = \"Actor opens a repair ticket.\"\n  val workflowSlices = [\"capture-ticket\",\"review-ticket\"]\n  val workflowSliceDetails = [{ slug: \"capture-ticket\", name: \"Capture ticket\", kind: \"state_view\", description: \"Slice description.\" },{ slug: \"review-ticket\", name: \"Review ticket\", kind: \"state_view\", description: \"Slice description.\" }]\n  val workflowTransitions = []\n  val workflowIdentityStable = workflowName == \"Open ticket\"\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_extra_transition_declaration_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let mut quint = read_to_string(&quint_path)?;
        quint.push_str(
            "  val workflowTransitions = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"stale-screen\" }]\n",
        );
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow transition drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_slice_detail_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/quint/OpenTicket.qnt"),
            "module OpenTicket {\n  // EMC-DIGEST: workflow:Open ticket\n  val workflowName = \"Open ticket\"\n  val workflowSlug = \"open-ticket\"\n  val workflowDescription = \"Actor opens a repair ticket.\"\n  val workflowSlices = [\"capture-ticket\",\"review-ticket\"]\n  val workflowSliceDetails = []\n  val workflowTransitions = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\" }]\n  val workflowIdentityStable = workflowName == \"Open ticket\"\n  var modelState: int\n  action init = modelState' = 0\n  action step = modelState' = modelState\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow slice detail drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_extra_slice_detail_declaration_drift()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let mut quint = read_to_string(&quint_path)?;
        quint.push_str(
            "  val workflowSliceDetails = [{ slug: \"stale-slice\", name: \"Stale slice\", kind: \"state_view\", description: \"Stale description.\" }]\n",
        );
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow slice detail drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_invariant_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let quint = read_to_string(&quint_path)?.replace(
            "  val workflowSliceDetailsComplete = workflowSlicesHaveDetails\n",
            "  val workflowSliceDetailsComplete = false\n",
        );
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow invariant drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_identity_invariant_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let quint = read_to_string(&quint_path)?.replace(
            "  val workflowIdentityStable = workflowName == \"Open ticket\"\n",
            "  val workflowIdentityStable = true\n",
        );
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow invariant drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_transition_invariant_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let quint = read_to_string(&quint_path)?.replace(
            "  val workflowTransitionsStructured = workflowTransitions.select(transition => transition.source != \"\" and transition.target != \"\" and transition.kind != \"\" and transition.trigger != \"\").length() == workflowTransitions.length()\n",
            "  val workflowTransitionsStructured = true\n",
        );
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow invariant drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_slice_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/quint/OpenTicket.qnt"),
            "module OpenTicket {\n\n// EMC-DIGEST: workflow:Open ticket\nval workflowName = \"Open ticket\"\n\nval workflowSlug = \"open-ticket\"\n\nval workflowDescription = \"Actor opens a repair ticket.\"\n\nval workflowSlices = []\n\nval workflowTransitions = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\" }]\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_workflow_extra_slice_declaration_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let mut quint = read_to_string(&quint_path)?;
        quint.push_str("  val workflowSlices = [\"stale-slice\"]\n");
        write(quint_path, quint)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint workflow slice drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_referenced_browser_slice_file() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        remove_file(
            temp_dir
                .path()
                .join("model/browser/data/slices/capture-ticket.eventmodel.json"),
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow Open ticket references missing slice artifact",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_formal_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        remove_file(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean slice artifact drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_missing_quint_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        remove_file(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint slice artifact drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unmodeled_lean_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        write(
            temp_dir.path().join("model/lean/slices/OrphanSlice.lean"),
            "namespace OrphanSlice\n\nend OrphanSlice\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean slice artifact drift for OrphanSlice.lean",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unmodeled_quint_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        write(
            temp_dir.path().join("model/quint/slices/OrphanSlice.qnt"),
            "module OrphanSlice {\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint slice artifact drift for OrphanSlice.qnt",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_slice_artifact_field_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_slice_path = temp_dir.path().join("model/lean/slices/CaptureTicket.lean");
        let lean_slice = read_to_string(&lean_slice_path)?.replace(
            "def sliceName := \"Capture ticket\"",
            "def sliceName := \"Stale slice\"",
        );
        write(lean_slice_path, lean_slice)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean slice artifact drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_slice_artifact_digest_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_slice_path = temp_dir.path().join("model/lean/slices/CaptureTicket.lean");
        let mut lean_slice = read_to_string(&lean_slice_path)?;
        lean_slice.push_str("\n-- EMC-DIGEST: slice:name=Stale slice\n");
        write(lean_slice_path, lean_slice)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean slice artifact drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_lean_slice_artifact_invariant_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let lean_slice_path = temp_dir.path().join("model/lean/slices/CaptureTicket.lean");
        let lean_slice = read_to_string(&lean_slice_path)?.replace(
            "theorem sliceIdentityIsStable : sliceName = \"Capture ticket\" := rfl",
            "theorem sliceIdentityIsStable : sliceName = sliceName := rfl",
        );
        write(lean_slice_path, lean_slice)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Lean slice artifact drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_slice_artifact_field_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_slice_path = temp_dir.path().join("model/quint/slices/CaptureTicket.qnt");
        let quint_slice = read_to_string(&quint_slice_path)?.replace(
            "val sliceKind = \"state_view\"",
            "val sliceKind = \"stale_kind\"",
        );
        write(quint_slice_path, quint_slice)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint slice artifact drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_slice_artifact_invariant_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_slice_path = temp_dir.path().join("model/quint/slices/CaptureTicket.qnt");
        let quint_slice = read_to_string(&quint_slice_path)?.replace(
            "val sliceIdentityStable = sliceName == \"Capture ticket\"",
            "val sliceIdentityStable = sliceName == sliceName",
        );
        write(quint_slice_path, quint_slice)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint slice artifact drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_quint_slice_artifact_digest_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let quint_slice_path = temp_dir.path().join("model/quint/slices/CaptureTicket.qnt");
        let mut quint_slice = read_to_string(&quint_slice_path)?;
        quint_slice.push_str("\n  // EMC-DIGEST: slice:name=Stale slice\n");
        write(quint_slice_path, quint_slice)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Quint slice artifact drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_unreferenced_browser_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        write(
            temp_dir
                .path()
                .join("model/browser/data/slices/orphan-slice.eventmodel.json"),
            "{\"name\":\"Orphan slice\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[],\"slices\":[{\"name\":\"Orphan slice\",\"type\":\"state_view\",\"events\":[],\"views\":[],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser slice reference drift for orphan-slice.eventmodel.json",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_referenced_browser_slice_file_identity_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;
        let slices_path = temp_dir.path().join("model/browser/data/slices");
        let canonical_slice_path = slices_path.join("capture-ticket.eventmodel.json");
        let drifted_slice_path = slices_path.join("capture_ticket.eventmodel.json");
        write(&drifted_slice_path, read_to_string(&canonical_slice_path)?)?;
        remove_file(canonical_slice_path)?;

        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let workflow = read_to_string(&workflow_path)?.replace(
            "\"../slices/capture-ticket.eventmodel.json\"",
            "\"../slices/capture_ticket.eventmodel.json\"",
        );
        write(workflow_path, workflow)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "browser slice identity drift for capture_ticket.eventmodel.json",
            ));

        Ok(())
    }

    fn create_connected_workflow(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
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

        add_slice(temp_dir.path(), "capture-ticket", "Capture ticket")?;
        add_slice(temp_dir.path(), "review-ticket", "Review ticket")?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn create_workflow_exit(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
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

        add_slice(temp_dir.path(), "capture-ticket", "Capture ticket")?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn add_slice(cwd: &Path, slug: &str, name: &str) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                slug,
                "--name",
                name,
                "--type",
                "state_view",
                "--description",
                "Slice description.",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }
}
