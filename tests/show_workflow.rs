#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::write;
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
            .stdout(predicate::str::contains("\"name\": \"Open ticket\""))
            .stdout(predicate::str::contains(
                "\"description\": \"Actor opens a repair ticket.\"",
            ));

        Ok(())
    }

    #[test]
    fn show_workflow_reports_formal_workflow_when_browser_index_is_stale()
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

        write(
            temp_dir.path().join("model/browser/data/index.json"),
            "{\n  \"project\": \"Repair Desk\",\n  \"workflows\": []\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .args(["show", "workflow", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"name\": \"Open ticket\""))
            .stdout(predicate::str::contains(
                "\"description\": \"Actor opens a repair ticket.\"",
            ));

        Ok(())
    }

    #[test]
    fn show_slice_reports_formal_slice_when_browser_slice_is_stale() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;
        add_slice(temp_dir.path())?;

        write(
            temp_dir
                .path()
                .join("model/browser/data/slices/capture-ticket.eventmodel.json"),
            "{\n  \"name\": \"Stale slice\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Stale browser projection.\"\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .args(["show", "slice", "capture-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"name\": \"Capture ticket\""))
            .stdout(predicate::str::contains(
                "\"description\": \"Actor enters repair ticket details.\"",
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
            .stdout(predicate::str::contains("\"name\": \"Capture ticket\""))
            .stdout(predicate::str::contains(
                "\"description\": \"Actor enters repair ticket details.\"",
            ));

        Ok(())
    }

    #[test]
    fn show_slice_rejects_unreferenced_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;

        write(
            temp_dir
                .path()
                .join("model/browser/data/slices/orphan-slice.eventmodel.json"),
            "{\n  \"name\": \"Orphan slice\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Not composed by a workflow.\"\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .args(["show", "slice", "orphan-slice"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice orphan-slice is not referenced by any indexed workflow",
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
