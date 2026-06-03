#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{exists, read_to_string};

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_updates_workflow_description() -> Result<(), Box<dyn Error>> {
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
            .stdout(predicate::str::contains("\"update_workflow\""))
            .stdout(predicate::str::contains("updated workflow Open ticket"));

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_json
                .contains("\"description\": \"Actor opens a repair ticket with priority.\""),
            "workflow browser data must preserve the MCP-updated description"
        );
        assert!(
            lean.contains(
                "def workflowDescription := \"Actor opens a repair ticket with priority.\""
            ),
            "Lean artifact must represent the MCP-updated description"
        );
        assert!(
            quint.contains(
                "val workflowDescription = \"Actor opens a repair ticket with priority.\""
            ),
            "Quint artifact must represent the MCP-updated description"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_workflow_name() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(mcp_name_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_workflow_name\""))
            .stdout(predicate::str::contains(
                "updated workflow Open repair ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenRepairTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenRepairTicket.qnt"))?;

        assert!(
            workflow_json.contains("\"name\": \"Open repair ticket\""),
            "workflow browser data must preserve the MCP-updated name"
        );
        assert!(
            lean.contains("def workflowName := \"Open repair ticket\""),
            "Lean artifact must represent the MCP-updated name"
        );
        assert!(
            quint.contains("val workflowName = \"Open repair ticket\""),
            "Quint artifact must represent the MCP-updated name"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_workflow() -> Result<(), Box<dyn Error>> {
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
                "capture-request",
                "--name",
                "Capture request",
                "--type",
                "state_change",
                "--description",
                "Actor captures the request.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_remove_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_workflow\""))
            .stdout(predicate::str::contains("removed workflow Open ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let index_json = read_to_string(temp_dir.path().join("model/browser/data/index.json"))?;
        assert!(
            !index_json.contains("open-ticket"),
            "MCP-removed workflow must be removed from the browser index"
        );
        assert!(
            !exists(
                temp_dir
                    .path()
                    .join("model/browser/data/workflows/open-ticket.eventmodel.json")
            )?,
            "MCP-removed workflow browser JSON must be deleted"
        );
        assert!(
            !exists(
                temp_dir
                    .path()
                    .join("model/browser/data/slices/capture-request.eventmodel.json")
            )?,
            "MCP-removed workflow must delete owned slice browser JSON"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/OpenTicket.lean"))?,
            "MCP-removed workflow Lean module must be deleted"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/OpenTicket.qnt"))?,
            "MCP-removed workflow Quint module must be deleted"
        );
        assert!(
            !exists(
                temp_dir
                    .path()
                    .join("model/lean/slices/CaptureRequest.lean")
            )?,
            "MCP-removed workflow must delete owned slice Lean module"
        );
        assert!(
            !exists(
                temp_dir
                    .path()
                    .join("model/quint/slices/CaptureRequest.qnt")
            )?,
            "MCP-removed workflow must delete owned slice Quint module"
        );

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_workflow\",\"arguments\":{\"slug\":\"open-ticket\",\"description\":\"Actor opens a repair ticket with priority.\"}}}\n",
        )
    }

    fn mcp_name_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_workflow_name\",\"arguments\":{\"slug\":\"open-ticket\",\"name\":\"Open repair ticket\"}}}\n",
        )
    }

    fn mcp_remove_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_workflow\",\"arguments\":{\"slug\":\"open-ticket\"}}}\n",
        )
    }
}
