#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn connect_workflow_adds_navigation_transition_to_canonical_artifacts()
    -> Result<(), Box<dyn Error>> {
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to review-ticket",
            ));

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_json.contains("\"to\": \"review-ticket\""),
            "workflow data must include the transition target"
        );
        assert!(
            workflow_json.contains("\"via_navigation\": \"review-ticket-screen\""),
            "workflow data must include the navigation trigger"
        );
        assert!(
            lean.contains(
                "def workflowTransitions := [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]"
            ),
            "Lean artifact must represent the workflow transition"
        );
        assert!(
            quint.contains(
                "const workflowTransitions = [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]"
            ),
            "Quint artifact must represent the workflow transition"
        );

        Ok(())
    }

    fn add_slice(
        cwd: &Path,
        slug: &str,
        name: &str,
        description: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                slug,
                "--name",
                name,
                "--type",
                "state_view",
                "--description",
                description,
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }
}
