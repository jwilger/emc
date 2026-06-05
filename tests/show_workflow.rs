// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn show_workflow_reports_the_modeled_workflow_document() -> Result<(), Box<dyn Error>> {
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

        Command::cargo_bin("emc")?
            .args(["show", "workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("# model/lean/OpenTicket.lean"))
            .stdout(predicate::str::contains("# model/quint/OpenTicket.qnt"))
            .stdout(predicate::str::contains(
                "def workflowName := \"Open ticket\"",
            ))
            .stdout(predicate::str::contains(
                "val workflowName = \"Open ticket\"",
            ))
            .stdout(predicate::str::contains(
                "def workflowDescription := \"Actor opens a repair ticket.\"",
            ));

        Ok(())
    }

    #[test]
    fn show_workflow_accepts_explicit_slug_flag() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["show", "workflow", "--slug", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "def workflowName := \"Open ticket\"",
            ))
            .stdout(predicate::str::contains(
                "val workflowName = \"Open ticket\"",
            ));

        Ok(())
    }

    #[test]
    fn show_slice_accepts_explicit_slug_flag() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;
        add_slice(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["show", "slice", "--slug", "capture-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "def sliceName := \"Capture ticket\"",
            ))
            .stdout(predicate::str::contains(
                "val sliceName = \"Capture ticket\"",
            ));

        Ok(())
    }

    #[test]
    fn show_slice_with_slug_flag_requires_exact_subject() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;
        add_slice(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["show", "slices", "--slug", "capture-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn show_slice_reports_a_referenced_modeled_slice_document() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;

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
                "Actor enters repair ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["show", "slice", "capture-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "# model/lean/slices/CaptureTicket.lean",
            ))
            .stdout(predicate::str::contains(
                "# model/quint/slices/CaptureTicket.qnt",
            ))
            .stdout(predicate::str::contains(
                "def sliceName := \"Capture ticket\"",
            ))
            .stdout(predicate::str::contains(
                "val sliceName = \"Capture ticket\"",
            ))
            .stdout(predicate::str::contains(
                "def sliceDescription := \"Actor enters repair ticket details.\"",
            ));

        Ok(())
    }

    #[test]
    fn show_slice_rejects_unknown_modeled_slices() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["show", "slice", "orphan-slice"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice orphan-slice is not referenced by any modeled workflow",
            ));

        Ok(())
    }

    #[test]
    fn show_slice_requires_exact_command_subject() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;

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
                "Actor enters repair ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["list", "slice", "capture-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    fn initialize_project_with_workflow(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(cwd)
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
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_slice(cwd: &Path) -> Result<(), Box<dyn Error>> {
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
                "Actor enters repair ticket details.",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }
}
