#[cfg(test)]
mod tests {
    use std::error::Error;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_exposes_list_workflows_tool() -> Result<(), Box<dyn Error>> {
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
                "close-ticket",
                "--name",
                "Close ticket",
                "--description",
                "Actor closes a repair ticket.",
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
            .stdout(predicate::str::contains("\"serverInfo\""))
            .stdout(predicate::str::contains("\"list_workflows\""))
            .stdout(predicate::str::contains("\"show_workflow\""))
            .stdout(predicate::str::contains("\"generate_site\""))
            .stdout(predicate::str::contains("Open ticket"))
            .stdout(predicate::str::contains("Close ticket"))
            .stdout(predicate::str::contains("generated site at mcp-site"))
            .stdout(predicate::str::contains(
                "\\\"name\\\": \\\"Open ticket\\\"",
            ));

        assert!(temp_dir.path().join("mcp-site/index.html").is_file());
        assert!(
            temp_dir
                .path()
                .join("mcp-site/assets/index-CTzj-YfP.js")
                .is_file()
        );
        assert!(temp_dir.path().join("mcp-site/data/index.json").is_file());

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"list_workflows\",\"arguments\":{}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"show_workflow\",\"arguments\":{\"slug\":\"open-ticket\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"generate_site\",\"arguments\":{\"output\":\"mcp-site\"}}}\n",
        )
    }
}
