// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{exists, read_to_string, remove_file, write};
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
    fn check_reports_project_root_digest_drift_without_rewriting_it() -> Result<(), Box<dyn Error>>
    {
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

        let lean_root_path = temp_dir.path().join("model/lean/RepairDesk.lean");
        let lean_root = read_to_string(&lean_root_path)?;
        let current_digest = generated_model_digest(&lean_root, "def modelDigest := \"")?;
        let stale_digest = if current_digest == "0".repeat(64) {
            "1".repeat(64)
        } else {
            "0".repeat(64)
        };
        let drifted_root = lean_root.replace(current_digest, &stale_digest);
        write(&lean_root_path, &drifted_root)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("Lean project root drift"));

        assert_eq!(read_to_string(&lean_root_path)?, drifted_root);

        Ok(())
    }

    #[test]
    fn sync_repairs_corrupted_artifacts_after_check_reports_drift() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_connected_workflow(&temp_dir)?;

        let path = temp_dir.path().join("model/lean/OpenTicket.lean");
        write(
            &path,
            "namespace OpenTicket\n-- corrupted\nend OpenTicket\n",
        )?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("Lean workflow field drift"));

        assert_eq!(
            read_to_string(&path)?,
            "namespace OpenTicket\n-- corrupted\nend OpenTicket\n"
        );

        Command::cargo_bin("emc")?
            .arg("sync")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("project layout is complete"));

        let healed = read_to_string(&path)?;
        assert!(
            healed.contains("def workflowName := \"Open ticket\"")
                && healed.contains(
                    "def workflowSlices : List WorkflowSlice := [{ slug := \"capture-ticket\" },{ slug := \"review-ticket\" }]"
                ),
            "sync must regenerate the corrupted Lean workflow artifact from the event log"
        );

        Ok(())
    }

    #[test]
    fn check_reports_missing_artifacts_without_creating_them() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_connected_workflow(&temp_dir)?;

        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        remove_file(&lean_path)?;
        remove_file(&quint_path)?;

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .failure();

        assert!(
            !exists(&lean_path)? && !exists(&quint_path)?,
            "check must not create missing workflow artifacts"
        );

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

    fn stabilize_project(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
                "--source-control",
                "review-ticket-screen",
                "--target-view",
                "review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        stabilize_project(temp_dir)?;
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

        stabilize_project(temp_dir)?;
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

    fn generated_model_digest<'a>(
        artifact: &'a str,
        prefix: &str,
    ) -> Result<&'a str, Box<dyn Error>> {
        let (_, tail) = artifact
            .split_once(prefix)
            .ok_or_else(|| format!("generated artifact must contain {prefix}"))?;
        let end = tail
            .find('"')
            .ok_or("generated artifact model digest must terminate with a quote")?;

        tail.get(..end).ok_or_else(|| {
            "digest quote offset must fall on a char boundary within the tail".into()
        })
    }
}
