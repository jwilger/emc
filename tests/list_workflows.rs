#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::path::PathBuf;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn list_workflows_reports_imported_workflow_names() -> Result<(), Box<dyn Error>> {
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
            .args(["list", "workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Organization access"));

        Ok(())
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
