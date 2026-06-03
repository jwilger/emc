#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn add_slice_updates_workflow_composition_and_slice_data() -> Result<(), Box<dyn Error>> {
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
            .success()
            .stdout(predicate::str::contains("added slice Capture ticket"));

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let slice_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/slices/open-ticket-capture-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_json.contains("\"../slices/open-ticket-capture-ticket.eventmodel.json\""),
            "workflow must reference the new slice file"
        );
        assert!(
            workflow_json.contains("\"slice\": \"capture-ticket\""),
            "workflow must add a step for the new slice"
        );
        assert!(
            workflow_json.contains("\"relationship\": \"entry\""),
            "first slice must become the workflow entry"
        );
        assert!(
            slice_json.contains("\"name\": \"Capture ticket\""),
            "slice data must preserve the business slice name"
        );
        assert!(
            slice_json.contains("\"type\": \"state_view\""),
            "slice data must preserve the semantic slice type"
        );
        assert!(
            slice_json.contains("\"description\": \"Actor enters repair ticket details.\""),
            "slice data must preserve the slice description"
        );
        assert!(
            lean.contains("def workflowSlices : List String := [\"capture-ticket\"]"),
            "Lean artifact must represent the workflow's business slices"
        );
        assert!(
            quint.contains("val workflowSlices = [\"capture-ticket\"]"),
            "Quint artifact must represent the workflow's business slices"
        );

        Ok(())
    }
}
