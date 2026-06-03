#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{remove_file, write};
    use std::path::{Path, PathBuf};

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
    fn check_reports_missing_imported_workflow_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let emc_event_model = workspace_root().join("../emc/docs/event-model");

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["import", "emc", "--source"])
            .arg(&emc_event_model)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        remove_file(temp_dir.path().join("model/lean/OrganizationAccess.lean"))?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "missing required project artifact model/lean/OrganizationAccess.lean",
            ));

        Ok(())
    }

    #[test]
    fn check_reports_imported_workflow_artifact_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let emc_event_model = workspace_root().join("../emc/docs/event-model");

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["import", "emc", "--source"])
            .arg(&emc_event_model)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        write(
            temp_dir.path().join("model/lean/OrganizationAccess.lean"),
            "namespace OrganizationAccess\n\ndef workflowName := \"Changed\"\n\nend OrganizationAccess\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "artifact digest mismatch for workflow Organization access",
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
    fn check_reports_lean_workflow_transition_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices := [\"capture-ticket\",\"review-ticket\"]\n\ndef workflowTransitions := []\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
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
    fn check_reports_lean_workflow_slice_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/lean/OpenTicket.lean"),
            "namespace OpenTicket\n\n-- EMC-DIGEST: workflow:Open ticket\ndef workflowName := \"Open ticket\"\n\ndef workflowSlug := \"open-ticket\"\n\ndef workflowDescription := \"Actor opens a repair ticket.\"\n\ndef workflowSlices := []\n\ndef workflowTransitions := [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]\n\ntheorem workflowIdentityIsStable : workflowName = \"Open ticket\" := rfl\n\nend OpenTicket\n",
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
    fn check_reports_quint_workflow_transition_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/quint/OpenTicket.qnt"),
            "module OpenTicket\n\n// EMC-DIGEST: workflow:Open ticket\nconst workflowName = \"Open ticket\"\n\nconst workflowSlug = \"open-ticket\"\n\nconst workflowDescription = \"Actor opens a repair ticket.\"\n\nconst workflowSlices = [\"capture-ticket\",\"review-ticket\"]\n\nconst workflowTransitions = []\n",
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
    fn check_reports_quint_workflow_slice_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_connected_workflow(&temp_dir)?;

        write(
            temp_dir.path().join("model/quint/OpenTicket.qnt"),
            "module OpenTicket\n\n// EMC-DIGEST: workflow:Open ticket\nconst workflowName = \"Open ticket\"\n\nconst workflowSlug = \"open-ticket\"\n\nconst workflowDescription = \"Actor opens a repair ticket.\"\n\nconst workflowSlices = []\n\nconst workflowTransitions = [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]\n",
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

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
