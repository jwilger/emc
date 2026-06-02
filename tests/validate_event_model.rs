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

    #[test]
    fn validate_rejects_missing_explicit_board() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("missing-board.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("missing explicit board"));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_command_names() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("duplicate-command.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[{\"name\":\"OpenRepairTicket\"},{\"name\":\"OpenRepairTicket\"}],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "duplicate command name 'OpenRepairTicket'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_slice_files_with_two_slices() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("two-slices.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit ticket\"},{\"name\":\"Review ticket\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice file must contain exactly one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_slice_legacy_scenarios_field() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("legacy-scenarios.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Submit lesson' uses legacy 'scenarios'; use 'acceptance_scenarios' and 'contract_scenarios'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_first_class_scenario_without_when() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("missing-when.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"acceptance_scenarios\":[{\"name\":\"reader sees lesson\",\"given\":[],\"then\":[]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Submit lesson' scenario 'reader sees lesson' is missing 'when'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_scenario_names_across_first_class_fields()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("duplicate-scenarios.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"acceptance_scenarios\":[{\"name\":\"duplicate scenario\",\"given\":[],\"when\":{},\"then\":[]}],\"contract_scenarios\":[{\"name\":\"duplicate scenario\",\"given\":[],\"when\":{},\"then\":[]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Submit lesson' has duplicate scenario name 'duplicate scenario' across acceptance_scenarios and contract_scenarios",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_acceptance_scenario_event_references() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("acceptance-event-reference.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[{\"name\":\"RootOrganizationBootstrapped\"}],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Resolve application entry\",\"acceptance_scenarios\":[{\"name\":\"fresh install sees bootstrap setup\",\"given\":[],\"when\":{},\"then\":[\"RootOrganizationBootstrapped\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Resolve application entry' acceptance scenario 'fresh install sees bootstrap setup' references event 'RootOrganizationBootstrapped'; acceptance_scenarios must describe user-facing behavior only",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_state_view_read_model_without_projector_contract_scenario()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("missing-projector-contract.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[{\"name\":\"application_entry_state\"}],\"views\":[{\"name\":\"application_entry_screen\",\"uses_read_models\":[\"application_entry_state\"]}],\"slices\":[{\"name\":\"Resolve application entry\",\"type\":\"state_view\",\"views\":[\"application_entry_screen\"],\"acceptance_scenarios\":[{\"name\":\"reader sees entry\",\"given\":[],\"when\":{},\"then\":[\"entry is visible\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "state_view slice 'Resolve application entry' read model 'application_entry_state' requires a contract_scenarios GWT for the projector",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_slice_outcome_labels() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("duplicate-outcomes.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"outcomes\":[{\"label\":\"submitted\"},{\"label\":\"submitted\"}],\"acceptance_scenarios\":[{\"name\":\"reader submits lesson\",\"given\":[],\"when\":{},\"then\":[\"lesson is submitted\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Submit lesson' has duplicate outcome label 'submitted'",
            ));

        Ok(())
    }
}
