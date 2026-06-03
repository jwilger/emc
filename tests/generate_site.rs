#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::path::PathBuf;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn generate_site_emits_browsable_site_files() -> Result<(), Box<dyn Error>> {
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
            .args(["generate", "site", "--output", "site"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("generated site at site"));

        assert!(temp_dir.path().join("site/index.html").is_file());
        assert!(temp_dir.path().join("site/data/index.json").is_file());
        assert!(
            temp_dir
                .path()
                .join("site/data/workflows/organization-access.eventmodel.json")
                .is_file()
        );
        assert!(
            temp_dir
                .path()
                .join(
                    "site/data/slices/organization-access-resolve-application-entry.eventmodel.json"
                )
                .is_file()
        );

        Ok(())
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
