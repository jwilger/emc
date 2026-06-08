// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{exists, read_to_string};

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_updates_slice_description() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_slice\""))
            .stdout(predicate::str::contains("updated slice Capture ticket"));

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            slice_lean.contains(
                "def sliceDescription := \"Actor enters repair ticket details and priority.\""
            ),
            "slice Lean artifact must represent the MCP-updated description"
        );
        assert!(
            slice_quint.contains(
                "val sliceDescription = \"Actor enters repair ticket details and priority.\""
            ),
            "slice Quint artifact must represent the MCP-updated description"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_slice_kind() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(mcp_kind_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_slice_kind\""))
            .stdout(predicate::str::contains("updated slice Capture ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            slice_lean.contains("def sliceKind := \"automation\""),
            "slice Lean artifact must represent the MCP-updated kind"
        );
        assert!(
            slice_quint.contains("val sliceKind = \"automation\""),
            "slice Quint artifact must represent the MCP-updated kind"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_slice_name() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(mcp_name_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_slice_name\""))
            .stdout(predicate::str::contains(
                "updated slice Capture repair ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean = read_to_string(
            temp_dir
                .path()
                .join("model/lean/slices/CaptureRepairTicket.lean"),
        )?;
        let slice_quint = read_to_string(
            temp_dir
                .path()
                .join("model/quint/slices/CaptureRepairTicket.qnt"),
        )?;

        assert!(
            slice_lean.contains("def sliceName := \"Capture repair ticket\""),
            "slice Lean artifact must represent the MCP-updated name"
        );
        assert!(
            slice_quint.contains("val sliceName = \"Capture repair ticket\""),
            "slice Quint artifact must represent the MCP-updated name"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?,
            "slice rename must remove the stale Lean artifact"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?,
            "slice rename must remove the stale Quint artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_slice() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(remove_slice_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_slice\""))
            .stdout(predicate::str::contains("removed slice Capture ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_lean.contains("def workflowSlices : List WorkflowSlice := []"),
            "workflow Lean artifact must remove the MCP-removed slice"
        );
        assert!(
            workflow_quint.contains("val workflowSlices: List[WorkflowSlice] = []"),
            "workflow Quint artifact must remove the MCP-removed slice"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?,
            "slice Lean artifact must be deleted by MCP removal"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?,
            "slice Quint artifact must be deleted by MCP removal"
        );

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_slice\",\"arguments\":{\"slug\":\"capture-ticket\",\"description\":\"Actor enters repair ticket details and priority.\"}}}\n",
        )
    }

    fn mcp_kind_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_slice_kind\",\"arguments\":{\"slug\":\"capture-ticket\",\"type\":\"automation\"}}}\n",
        )
    }

    fn mcp_name_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_slice_name\",\"arguments\":{\"slug\":\"capture-ticket\",\"name\":\"Capture repair ticket\"}}}\n",
        )
    }

    fn remove_slice_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_slice\",\"arguments\":{\"slug\":\"capture-ticket\"}}}\n",
        )
    }
}
