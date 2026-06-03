#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::path::PathBuf;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn show_workflow_reports_the_imported_workflow_document() -> Result<(), Box<dyn Error>> {
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

        Command::cargo_bin("emc")?
            .args(["show", "workflow", "organization-access"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"name\": \"Organization access\"",
            ))
            .stdout(predicate::str::contains(
                "\"description\": \"An actor arrives at EMC",
            ));

        Ok(())
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
