// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::io::{BufRead, BufReader, Write};
    use std::path::PathBuf;
    use std::process::{Command as ProcessCommand, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use assert_cmd::Command;
    use assert_cmd::cargo::cargo_bin;
    use predicates::Predicate;
    use predicates::prelude::predicate;
    use serde_json::Value;
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
                "EMC project Repair Desk layout is present",
            ));

        assert!(temp_dir.path().join("emc.toml").is_file());
        assert!(temp_dir.path().join("model/lean/RepairDesk.lean").is_file());
        assert!(temp_dir.path().join("model/quint/RepairDesk.qnt").is_file());
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
            .stdout(predicate::str::contains("Open ticket"))
            .stdout(predicate::str::contains("Close ticket"))
            .stdout(predicate::str::contains("# model/lean/OpenTicket.lean"))
            .stdout(predicate::str::contains(
                "def workflowName := \\\"Open ticket\\\"",
            ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_adds_workflow_after_cli_init() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(add_workflow_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_workflow\""))
            .stdout(predicate::str::contains("added workflow Open ticket"));

        Command::cargo_bin("emc")?
            .args(["list", "workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Open ticket"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_adds_workflow_after_file_level_reset() -> Result<(), Box<dyn Error>> {
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

        fs::remove_dir_all(temp_dir.path().join("model"))?;
        fs::remove_file(temp_dir.path().join("emc.toml"))?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(add_workflow_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_workflow\""))
            .stdout(predicate::str::contains("added workflow Open ticket"));

        Command::cargo_bin("emc")?
            .args(["list", "workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Open ticket"));

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

    #[test]
    fn mcp_stdio_negotiates_codex_initialize_protocol_and_keeps_tools_available()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let output = Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(codex_protocol_mcp_requests())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output)?;
        let responses = stdout
            .lines()
            .map(serde_json::from_str::<Value>)
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(responses.len(), 3);
        assert_eq!(
            responses[0]["result"]["protocolVersion"],
            Value::String("2024-11-05".to_owned())
        );
        assert!(responses[1]["result"]["tools"].is_array());
        assert!(
            stdout.contains("\"check_project\""),
            "tools/list must remain available after initialize"
        );
        assert!(
            stdout.contains("project layout is complete"),
            "tools/call must remain available after initialize"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_echoes_current_codex_initialize_protocol() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        let output = Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(current_codex_initialize_request())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output)?;
        let response = serde_json::from_str::<Value>(stdout.trim())?;

        assert_eq!(
            response["result"]["protocolVersion"],
            Value::String("2025-06-18".to_owned())
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_exposes_list_conflicts_tool() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_concurrent_slice_update_conflict(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(list_conflicts_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"list_conflicts\""))
            .stdout(predicate::str::contains(
                "conflict slice::capture-ticket SliceUpdated capture-ticket",
            ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_resolves_event_conflicts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let chosen_event_id = create_concurrent_slice_update_conflict(&temp_dir)?;
        let conflict_output = Command::cargo_bin("emc")?
            .args(["list", "conflicts"])
            .current_dir(temp_dir.path())
            .output()?;
        let conflict_stdout = String::from_utf8(conflict_output.stdout)?;
        let conflict_id = conflict_stdout
            .split(" id ")
            .nth(1)
            .and_then(|suffix| suffix.split_whitespace().next())
            .ok_or("conflict id must be reported")?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(resolve_conflict_mcp_requests(conflict_id, &chosen_event_id))
            .assert()
            .success()
            .stdout(predicate::str::contains("\"resolve_conflict\""))
            .stdout(predicate::str::contains(format!(
                "resolved conflict {conflict_id}"
            )));

        Command::cargo_bin("emc")?
            .args(["list", "conflicts"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("no event conflicts"));

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"list_workflows\",\"arguments\":{}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"show_workflow\",\"arguments\":{\"slug\":\"open-ticket\"}}}\n",
        )
    }

    fn init_project_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"init_project\",\"arguments\":{\"name\":\"Repair Desk\"}}}\n",
        )
    }

    fn add_workflow_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow\",\"arguments\":{\"slug\":\"open-ticket\",\"name\":\"Open ticket\",\"description\":\"Actor opens a repair ticket.\"}}}\n",
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

    fn list_conflicts_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"list_conflicts\",\"arguments\":{}}}\n",
        )
    }

    fn resolve_conflict_mcp_requests(conflict_id: &str, chosen_event_id: &str) -> String {
        format!(
            "{}{}{}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            format_args!(
                "{{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{{\"name\":\"resolve_conflict\",\"arguments\":{{\"id\":\"{conflict_id}\",\"choose_event\":\"{chosen_event_id}\"}}}}}}"
            )
        )
    }

    fn create_concurrent_slice_update_conflict(
        temp_dir: &TempDir,
    ) -> Result<String, Box<dyn Error>> {
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
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--description",
                "First merged branch description.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let event_dir = temp_dir.path().join("model/events/v1");
        let mut update_event = exported_events(event_dir.clone())?
            .into_iter()
            .find(|event| event["type"] == "SliceUpdated")
            .ok_or("slice update event must exist")?;
        let original_event_id = update_event["event_id"]
            .as_str()
            .ok_or("slice update event_id must be a string")?
            .to_owned();
        let conflicting_event_id = format!("{original_event_id}-conflicting");
        update_event["event_id"] = Value::String(conflicting_event_id.clone());
        update_event["command_id"] = Value::String(conflicting_event_id.clone());
        update_event["payload"]["description"] =
            Value::String("Second merged branch description.".to_owned());
        fs::write(
            event_dir.join(format!("{conflicting_event_id}.json")),
            serde_json::to_string_pretty(&update_event)?,
        )?;

        Ok(original_event_id)
    }

    fn exported_events(path: PathBuf) -> Result<Vec<Value>, Box<dyn Error>> {
        let mut event_paths = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        event_paths.sort();

        event_paths
            .into_iter()
            .map(|path| {
                let contents = fs::read_to_string(path)?;
                let event = serde_json::from_str::<Value>(&contents)?;
                Ok(event)
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()
    }

    fn initialize_request() -> &'static str {
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}"
    }

    fn codex_protocol_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"probe\",\"version\":\"0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"check_project\",\"arguments\":{}}}\n",
        )
    }

    fn current_codex_initialize_request() -> &'static str {
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{\"elicitation\":{}},\"clientInfo\":{\"name\":\"codex-mcp-client\",\"title\":\"Codex\",\"version\":\"0.137.0\"}}}\n"
    }
}
