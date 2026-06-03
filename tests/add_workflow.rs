#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn add_workflow_emits_business_model_artifacts() -> Result<(), Box<dyn Error>> {
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
            .success()
            .stdout(predicate::str::contains("added workflow Open ticket"));

        let index_json = read_to_string(temp_dir.path().join("model/browser/data/index.json"))?;
        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            index_json.contains("\"name\": \"Open ticket\""),
            "browser index must list the added workflow"
        );
        assert!(
            index_json.contains("\"path\": \"data/workflows/open-ticket.eventmodel.json\""),
            "browser index must point at the added workflow data"
        );
        assert!(
            workflow_json.contains("\"name\": \"Open ticket\""),
            "workflow browser data must represent the added business workflow"
        );
        assert!(
            workflow_json.contains("\"description\": \"Actor opens a repair ticket.\""),
            "workflow browser data must preserve the workflow description"
        );
        assert!(
            lean.contains("def workflowName := \"Open ticket\""),
            "Lean artifact must represent the added business workflow"
        );
        assert!(
            quint.contains("const workflowName = \"Open ticket\""),
            "Quint artifact must represent the added business workflow"
        );

        Ok(())
    }
}
