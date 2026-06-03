#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn list_workflows_reports_modeled_workflow_names() -> Result<(), Box<dyn Error>> {
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
            .args(["list", "workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Open ticket"));

        Ok(())
    }

    #[test]
    fn list_slices_reports_modeled_slice_names() -> Result<(), Box<dyn Error>> {
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
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "review-ticket",
                "--name",
                "Review ticket",
                "--type",
                "state_view",
                "--description",
                "Actor reviews repair ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["list", "slices"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Capture ticket"))
            .stdout(predicate::str::contains("Review ticket"));

        Ok(())
    }

    #[test]
    fn list_slices_requires_exact_command_subject() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["show", "slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc init --name <project-name>",
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
}
