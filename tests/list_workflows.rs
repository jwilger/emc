#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::write;
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::{PredicateBooleanExt, predicate};
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
    fn list_workflows_reports_formal_workflows_when_browser_index_is_stale()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;

        write(
            temp_dir.path().join("model/browser/data/index.json"),
            "{\n  \"generated_at\": \"1970-01-01T00:00:00.000Z\",\n  \"workflows\": [{\"name\":\"Stale ticket\",\"path\":\"data/workflows/stale-ticket.eventmodel.json\",\"description\":\"Stale browser projection.\"}]\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .args(["list", "workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Open ticket"))
            .stdout(predicate::str::contains("Stale ticket").not());

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
    fn list_slices_reports_formal_slices_when_browser_workflow_is_stale()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;
        add_slice(temp_dir.path(), "capture-ticket", "Capture ticket")?;
        stale_browser_workflow_projection(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["list", "slices"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Capture ticket"));

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

    #[test]
    fn list_transitions_reports_formal_transitions_when_browser_workflow_is_stale()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;
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
                "event",
                "--name",
                "TicketCaptured",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        stale_browser_workflow_projection(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["list", "transitions"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("capture-ticket"))
            .stdout(predicate::str::contains("review-ticket"))
            .stdout(predicate::str::contains("event"))
            .stdout(predicate::str::contains("TicketCaptured"));

        Ok(())
    }

    #[test]
    fn list_transitions_reports_modeled_workflow_transitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;
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
                "event",
                "--name",
                "TicketCaptured",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["list", "transitions"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("capture-ticket"))
            .stdout(predicate::str::contains("review-ticket"))
            .stdout(predicate::str::contains("event"))
            .stdout(predicate::str::contains("TicketCaptured"));

        Ok(())
    }

    #[test]
    fn list_transitions_requires_exact_command_subject() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_workflow(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["show", "transitions"])
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

    fn stale_browser_workflow_projection(cwd: &Path) -> Result<(), Box<dyn Error>> {
        write(
            cwd.join("model/browser/data/workflows/open-ticket.eventmodel.json"),
            "{\n  \"name\": \"Stale ticket\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Stale browser projection.\",\n  \"steps\": []\n}\n",
        )?;
        Ok(())
    }
}
