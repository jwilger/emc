#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{create_dir_all, write};

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn validate_rejects_invalid_event_model_json() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(workflows.join("broken.eventmodel.json"), "{ not-json")?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid JSON"));

        Ok(())
    }

    #[test]
    fn validate_rejects_non_object_event_model_json() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(workflows.join("array.eventmodel.json"), "[]")?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("model must be a JSON object"));

        Ok(())
    }

    #[test]
    fn validate_rejects_missing_required_top_level_key() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("missing-name.eventmodel.json"),
            "{\"version\":\"0.1.0\",\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("missing top-level key 'name'"));

        Ok(())
    }
}
