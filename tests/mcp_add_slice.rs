// Copyright 2026 John Wilger

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

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;

        assert!(
            slice_lean.contains("def sliceDescription := \"Actor enters repair ticket details.\""),
            "slice artifact must preserve the MCP-created slice description"
        );
        assert!(
            lean.contains(
                "def workflowSlices : List WorkflowSlice := [{ slug := \"capture-ticket\" }]"
            ),
            "Lean artifact must represent the MCP-created workflow slice"
        );
        assert!(
            quint.contains(
                "val workflowSlices: List[WorkflowSlice] = [{ slug: \"capture-ticket\" }]"
            ),
            "Quint artifact must represent the MCP-created workflow slice"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_shows_modeled_slice_documents() -> Result<(), Box<dyn Error>> {
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
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(show_slice_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"show_slice\""))
            .stdout(predicate::str::contains(
                "# model/lean/slices/CaptureTicket.lean",
            ))
            .stdout(predicate::str::contains(
                "# model/quint/slices/CaptureTicket.qnt",
            ))
            .stdout(predicate::str::contains(
                "def sliceName := \\\"Capture ticket\\\"",
            ))
            .stdout(predicate::str::contains(
                "val sliceDescription = \\\"Actor enters repair ticket details.\\\"",
            ));

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_slice\",\"arguments\":{\"workflow\":\"open-ticket\",\"slug\":\"capture-ticket\",\"name\":\"Capture ticket\",\"type\":\"state_view\",\"description\":\"Actor enters repair ticket details.\"}}}\n",
        )
    }

    fn show_slice_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"show_slice\",\"arguments\":{\"slug\":\"capture-ticket\"}}}\n",
        )
    }
}
