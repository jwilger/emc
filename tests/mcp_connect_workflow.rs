#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_connects_workflow_steps() -> Result<(), Box<dyn Error>> {
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

        add_slice(temp_dir.path(), "capture-ticket", "Capture ticket")?;
        add_slice(temp_dir.path(), "review-ticket", "Review ticket")?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"connect_workflow\""))
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

        assert!(workflow_json.contains("\"via_navigation\": \"review-ticket-screen\""));
        assert!(
            lean.contains(
                "def workflowTransitions : List String := [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]"
            )
        );

        Ok(())
    }

    fn add_slice(cwd: &Path, slug: &str, name: &str) -> Result<(), Box<dyn Error>> {
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
                "Slice description.",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\"}}}\n",
        )
    }
}
