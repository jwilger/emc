#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command as ProcessCommand, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use assert_cmd::Command;
    use assert_cmd::cargo::cargo_bin;
    use predicates::Predicate;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_initializes_project_layout() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(init_project_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"init_project\""))
            .stdout(predicate::str::contains(
                "initialized EMC project Repair Desk",
            ));

        assert!(temp_dir.path().join("emc.toml").is_file());
        assert!(temp_dir.path().join("model/lean/RepairDesk.lean").is_file());
        assert!(temp_dir.path().join("model/quint/RepairDesk.qnt").is_file());
        assert!(
            temp_dir
                .path()
                .join("model/browser/data/index.json")
                .is_file()
        );

        Ok(())
    }

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

    #[test]
    fn mcp_stdio_exposes_list_slices_tool() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(list_slices_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"list_slices\""))
            .stdout(predicate::str::contains("Capture ticket"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_exposes_list_transitions_tool() -> Result<(), Box<dyn Error>> {
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
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "review-ticket",
                "--name",
                "Review ticket",
                "--type",
                "state_view",
                "--description",
                "Actor reviews repair ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
                "event",
                "--name",
                "TicketCaptured",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(list_transitions_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"list_transitions\""))
            .stdout(predicate::str::contains("capture-ticket"))
            .stdout(predicate::str::contains("review-ticket"))
            .stdout(predicate::str::contains("TicketCaptured"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_responds_to_each_request_without_waiting_for_stdin_eof()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let mut server = ProcessCommand::new(cargo_bin("emc"))
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = server
            .stdin
            .take()
            .ok_or("MCP stdio child stdin is unavailable")?;
        let stdout = server
            .stdout
            .take()
            .ok_or("MCP stdio child stdout is unavailable")?;

        writeln!(stdin, "{}", initialize_request())?;
        stdin.flush()?;

        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut lines = BufReader::new(stdout).lines();
            let line = lines.next().transpose();
            let _send_result = sender.send(line);
        });

        let line = receiver
            .recv_timeout(Duration::from_millis(500))
            .map_err(|_| "MCP stdio did not respond before stdin EOF")??
            .ok_or("MCP stdio closed stdout without a response")?;

        server.kill()?;
        let output = server.wait_with_output()?;

        assert!(output.status.success() || output.status.code().is_none());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert!(
            predicate::str::contains("\"serverInfo\"").eval(&line),
            "MCP stdio response must be sent before the client closes stdin"
        );

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

    fn init_project_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"init_project\",\"arguments\":{\"name\":\"Repair Desk\"}}}\n",
        )
    }

    fn list_slices_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"list_slices\",\"arguments\":{}}}\n",
        )
    }

    fn list_transitions_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"list_transitions\",\"arguments\":{}}}\n",
        )
    }

    fn initialize_request() -> &'static str {
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}"
    }
}
