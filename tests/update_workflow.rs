#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn update_workflow_description_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
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
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens a repair ticket with priority.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("updated workflow Open ticket"));

        let index_json = read_to_string(temp_dir.path().join("model/browser/data/index.json"))?;
        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            index_json.contains("\"description\": \"Actor opens a repair ticket with priority.\""),
            "browser index must preserve the updated workflow description"
        );
        assert!(
            workflow_json
                .contains("\"description\": \"Actor opens a repair ticket with priority.\""),
            "workflow browser data must preserve the updated workflow description"
        );
        assert!(
            lean.contains(
                "def workflowDescription := \"Actor opens a repair ticket with priority.\""
            ),
            "Lean artifact must represent the updated workflow description"
        );
        assert!(
            quint.contains(
                "const workflowDescription = \"Actor opens a repair ticket with priority.\""
            ),
            "Quint artifact must represent the updated workflow description"
        );

        Ok(())
    }
}
