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

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains("def workflowName := \"Open ticket\""),
            "Lean artifact must represent the added business workflow"
        );
        assert!(
            quint.contains("val workflowName = \"Open ticket\""),
            "Quint artifact must represent the added business workflow"
        );

        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean_root.contains("def modelWorkflows : List String := [\"open-ticket\"]"),
            "Lean root artifact must explicitly enumerate modeled workflows"
        );
        assert!(
            lean_root
                .contains("theorem modelWorkflowsAreDeclared : modelWorkflows.length = 1 := rfl"),
            "Lean root artifact must prove the modeled workflow registry"
        );
        assert!(
            quint_root.contains("val modelWorkflows: List[str] = [\"open-ticket\"]"),
            "Quint root artifact must explicitly enumerate modeled workflows"
        );
        assert!(
            quint_root.contains("val modelWorkflowsAreDeclared = modelWorkflows.length() == 1"),
            "Quint root artifact must expose the modeled workflow registry invariant"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_workflow_rejects_duplicate_formal_workflow_module_names() -> Result<(), Box<dyn Error>> {
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
                "workflow",
                "--slug",
                "open-ticket-copy",
                "--name",
                "Open-ticket",
                "--description",
                "Alternate workflow with colliding formal module.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow module OpenTicket already exists",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        assert!(
            lean.contains("def workflowName := \"Open ticket\""),
            "rejected duplicate module names must not overwrite the existing formal workflow artifact"
        );

        Ok(())
    }

    #[test]
    fn add_workflow_rejects_duplicate_workflow_slugs() -> Result<(), Box<dyn Error>> {
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
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open repair ticket",
                "--description",
                "Alternate workflow with duplicate slug.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow open-ticket already exists",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        assert!(
            lean.contains("def workflowName := \"Open ticket\""),
            "rejected duplicate workflow slugs must not overwrite existing formal workflow artifacts"
        );

        Ok(())
    }
}
