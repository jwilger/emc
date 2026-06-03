#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::write;

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
    fn show_workflow_rejects_unindexed_workflow_files() -> Result<(), Box<dyn Error>> {
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
            .failure()
            .stderr(predicate::str::contains(
                "workflow open-ticket is not indexed",
            ));

        Ok(())
    }
}
