#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_adds_slice_to_workflow() -> Result<(), Box<dyn Error>> {
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
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_slice\""))
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
            workflow_json.contains("\"slice\": \"capture-ticket\""),
            "workflow must add a step for the MCP-created slice"
        );
        assert!(
            slice_json.contains("\"description\": \"Actor enters repair ticket details.\""),
            "slice data must preserve the MCP-created slice description"
        );
        assert!(
            lean.contains("def workflowSlices : List String := [\"capture-ticket\"]"),
            "Lean artifact must represent the MCP-created workflow slice"
        );
        assert!(
            quint.contains("val workflowSlices = [\"capture-ticket\"]"),
            "Quint artifact must represent the MCP-created workflow slice"
        );

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_slice\",\"arguments\":{\"workflow\":\"open-ticket\",\"slug\":\"capture-ticket\",\"name\":\"Capture ticket\",\"type\":\"state_view\",\"description\":\"Actor enters repair ticket details.\"}}}\n",
        )
    }
}
