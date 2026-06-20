// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::io::{self, BufRead, BufReader, ErrorKind, Write};
    use std::path::Path;
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
            .map_err(|err| format!("MCP stdio did not respond before stdin EOF: {err}"))??
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

        assert_eq!(responses.len(), 3, "expected three MCP responses");
        let first_protocol_version = responses
            .first()
            .and_then(|response| response.get("result"))
            .and_then(|result| result.get("protocolVersion"))
            .ok_or("protocolVersion in first MCP result")?;
        assert_eq!(
            *first_protocol_version,
            Value::String("2024-11-05".to_owned()),
            "first MCP result must report the negotiated protocol version"
        );
        let second_tools = responses
            .get(1)
            .and_then(|response| response.get("result"))
            .and_then(|result| result.get("tools"))
            .ok_or("tools in second MCP result")?;
        assert!(
            second_tools.is_array(),
            "second MCP result must list tools as an array"
        );
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

        let protocol_version = response
            .get("result")
            .and_then(|result| result.get("protocolVersion"))
            .ok_or("protocolVersion in MCP result")?;
        assert_eq!(
            *protocol_version,
            Value::String("2025-06-18".to_owned()),
            "MCP result must report the negotiated protocol version"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_exposes_list_conflicts_tool() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_slice_update_fork(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(list_conflicts_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"list_conflicts\""))
            .stdout(predicate::str::contains("conflict slice::capture-ticket"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_exposes_modeling_guidance_tool() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(modeling_guidance_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"get_modeling_guidance\""))
            .stdout(predicate::str::contains("EMC Modeling Process"))
            .stdout(predicate::str::contains("Phase-By-Phase Modeling Order"))
            .stdout(predicate::str::contains("Acceptance Scenarios"))
            .stdout(predicate::str::contains("external actor's point of view"))
            .stdout(predicate::str::contains("Contract Scenarios"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_slice_scenario() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_scenario(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_slice_scenario_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_slice_scenario\""))
            .stdout(predicate::str::contains(
                "updated scenario Actor captures ticket on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            slice_lean.contains("the actor submits corrected ticket details"),
            "MCP-updated scenario must be represented in Lean slice artifacts"
        );
        assert!(
            !slice_lean.contains("the actor submits ticket details"),
            "old scenario text must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_slice_scenario() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_scenario(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_slice_scenario_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_slice_scenario\""))
            .stdout(predicate::str::contains(
                "removed scenario Actor captures ticket from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("Actor captures ticket"),
            "MCP-removed scenario must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_command_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_command(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_command_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_command_definition\""))
            .stdout(predicate::str::contains(
                "updated command CaptureTicket on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            slice_lean.contains("ticket_summary"),
            "MCP-updated command input must be represented in Lean slice artifacts"
        );
        assert!(
            !slice_lean.contains("ticket_title"),
            "old command input must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_command_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_command(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_command_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_command_definition\""))
            .stdout(predicate::str::contains(
                "removed command CaptureTicket from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("name: \"CaptureTicket\""),
            "MCP-removed command must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_event_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_event(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_event_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_event_definition\""))
            .stdout(predicate::str::contains(
                "updated event TicketCaptured on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            slice_lean.contains("ticket-updates"),
            "MCP-updated event stream must be represented in Lean slice artifacts"
        );
        assert!(
            !slice_lean.contains("ticket-events"),
            "old event stream must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_event_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_event(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_event_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_event_definition\""))
            .stdout(predicate::str::contains(
                "removed event TicketCaptured from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("name: \"TicketCaptured\""),
            "MCP-removed event must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_read_model_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_read_model(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_read_model_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_read_model_definition\""))
            .stdout(predicate::str::contains(
                "updated read model ticket_state on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_lean.contains("ticket_summary"),
            "MCP-updated read model field must be represented in Lean slice artifacts"
        );
        assert!(
            slice_quint.contains("TicketCaptured.title -> summary"),
            "MCP-updated read model provenance must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_lean.contains("name := \"ticket_title\""),
            "old read model field must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_read_model_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_read_model(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_read_model_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_read_model_definition\""))
            .stdout(predicate::str::contains(
                "removed read model ticket_state from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("name: \"ticket_state\""),
            "MCP-removed read model must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_view_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_view(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_view_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_view_definition\""))
            .stdout(predicate::str::contains(
                "updated view ticket_detail on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_lean.contains("ticket_summary_view"),
            "MCP-updated view field must be represented in Lean slice artifacts"
        );
        assert!(
            slice_quint.contains("summary-label"),
            "MCP-updated view sketch token must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_lean.contains("ticket_title_view"),
            "old view field must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_view_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_view(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_view_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_view_definition\""))
            .stdout(predicate::str::contains(
                "removed view ticket_detail from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("name: \"ticket_detail\""),
            "MCP-removed view must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_control_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_control(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_control_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_control_definition\""))
            .stdout(predicate::str::contains(
                "updated control submit-ticket on view ticket_detail in slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let root_quint = fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;
        assert!(
            slice_lean.contains("corrected title field on the intake form"),
            "MCP-updated control input description must be represented in Lean slice artifacts"
        );
        assert!(
            root_quint.contains("controlSketchToken: \"resubmit-button\""),
            "MCP-updated control sketch token must be represented in Quint project inventory"
        );
        assert!(
            !slice_lean.contains("sketchToken := \"submit-button\""),
            "old control sketch token must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_control_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_control(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_control_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_control_definition\""))
            .stdout(predicate::str::contains(
                "removed control submit-ticket from view ticket_detail in slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        let root_quint = fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;
        assert!(
            !slice_quint.contains("controls: [{ name: \"submit-ticket\""),
            "MCP-removed control must be absent from Quint slice artifacts"
        );
        assert!(
            root_quint.contains("val modelViewControls: List[ModelViewControl] = []"),
            "MCP-removed control must be absent from Quint project inventory"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_outcome_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_outcome(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_outcome_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_outcome_definition\""))
            .stdout(predicate::str::contains(
                "updated outcome ticket-captured on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_quint.contains(
                "{ label: \"ticket-captured\", eventSet: [\"TicketRouted\"], externallyRelevant: false }"
            ),
            "MCP-updated outcome must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_quint.contains("TicketCaptured"),
            "old outcome event set must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_outcome_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_outcome(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_outcome_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_outcome_definition\""))
            .stdout(predicate::str::contains(
                "removed outcome ticket-captured from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("label: \"ticket-captured\""),
            "MCP-removed outcome must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_automation_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_automation(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_automation_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_automation_definition\""))
            .stdout(predicate::str::contains(
                "updated automation assign-duplicate-ticket on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_quint.contains("DuplicateTicketEscalated"),
            "MCP-updated automation trigger must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_quint.contains("DuplicateTicketDetected"),
            "old automation trigger must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_automation_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_automation(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_automation_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_automation_definition\""))
            .stdout(predicate::str::contains(
                "removed automation assign-duplicate-ticket from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("name: \"assign-duplicate-ticket\""),
            "MCP-removed automation must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_translation_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_translation(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_translation_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"update_translation_definition\"",
            ))
            .stdout(predicate::str::contains(
                "updated translation capture-ticket-from-webhook on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_quint.contains("TicketWebhookRetried"),
            "MCP-updated translation external event must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_quint.contains("TicketWebhookReceived"),
            "old translation external event must be absent after MCP update"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_translation_definition() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_translation(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_translation_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"remove_translation_definition\"",
            ))
            .stdout(predicate::str::contains(
                "removed translation capture-ticket-from-webhook from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("name: \"capture-ticket-from-webhook\""),
            "MCP-removed translation must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_resolves_event_conflicts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_slice_update_fork(&temp_dir)?;

        let conflict_output = Command::cargo_bin("emc")?
            .args(["list", "conflicts"])
            .current_dir(temp_dir.path())
            .output()?;
        let conflict_stdout = String::from_utf8(conflict_output.stdout)?;
        let conflict_line = conflict_stdout
            .lines()
            .find(|line| line.contains("conflict slice::capture-ticket"))
            .ok_or("conflict on slice::capture-ticket must be reported")?;
        let branch_tx = conflict_line
            .split(" branches ")
            .nth(1)
            .and_then(|branches| branches.split(',').next())
            .map(str::trim)
            .ok_or("conflict branches must be reported")?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(resolve_conflict_mcp_requests(
                "slice::capture-ticket",
                branch_tx,
            ))
            .assert()
            .success()
            .stdout(predicate::str::contains("\"resolve_conflict\""))
            .stdout(predicate::str::contains(
                "resolved conflict slice::capture-ticket",
            ));

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

    fn modeling_guidance_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"get_modeling_guidance\",\"arguments\":{}}}\n",
        )
    }

    fn update_slice_scenario_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_slice_scenario\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"Actor captures ticket\",\"kind\":\"acceptance\",\"given\":\"ticket intake screen is open\",\"when\":\"the actor submits corrected ticket details\",\"then\":\"the corrected details are visible for review\"}}}\n",
        )
    }

    fn remove_slice_scenario_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_slice_scenario\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"Actor captures ticket\"}}}\n",
        )
    }

    fn update_command_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_command_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\",\"input\":\"ticket_summary\",\"input_source\":\"actor\",\"input_description\":\"summary field on the intake form\",\"input_provenance\":\"actor keystrokes -> summary field\",\"emits\":\"TicketUpdated\"}}}\n",
        )
    }

    fn remove_command_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_command_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\"}}}\n",
        )
    }

    fn update_event_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\",\"stream\":\"ticket-updates\",\"attribute\":\"summary\",\"attribute_source\":\"generated\",\"attribute_source_name\":\"ticket_summary\",\"attribute_source_field\":\"value\",\"generated_source_kind\":\"projection\",\"attribute_provenance\":\"projection summary field\",\"observed\":true}}}\n",
        )
    }

    fn remove_event_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\"}}}\n",
        )
    }

    fn update_read_model_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_read_model_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_state\",\"field\":\"ticket_summary\",\"field_source\":\"event_attribute\",\"source_event\":\"TicketCaptured\",\"source_attribute\":\"title\",\"field_provenance\":\"TicketCaptured.title -> summary\"}}}\n",
        )
    }

    fn remove_read_model_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_read_model_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_state\"}}}\n",
        )
    }

    fn update_view_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_view_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_detail\",\"read_model\":\"ticket_state\",\"field\":\"ticket_summary_view\",\"source_field\":\"ticket_title\",\"sketch_token\":\"summary-label\",\"field_provenance\":\"ticket_state.ticket_title -> summary label\",\"bit_encoding\":\"UTF-8 summary string\",\"control\":\"submit-ticket\",\"control_command\":\"CaptureTicket\",\"control_input\":\"ticket_title\",\"control_input_source\":\"actor\",\"control_input_description\":\"title field on the intake form\",\"control_input_sketch_token\":\"title-input\",\"control_input_visible\":true,\"control_input_decision\":true,\"handled_errors\":\"DuplicateTicket\",\"recovery_behavior\":\"retry\",\"control_sketch_token\":\"submit-button\",\"navigation_type\":\"modeled_view\",\"navigation_target\":\"ticket_detail\"}}}\n",
        )
    }

    fn remove_view_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_view_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_detail\"}}}\n",
        )
    }

    fn update_control_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_control_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"view\":\"ticket_detail\",\"name\":\"submit-ticket\",\"command\":\"CaptureTicket\",\"input\":\"ticket_title\",\"input_source\":\"actor\",\"input_description\":\"corrected title field on the intake form\",\"input_sketch_token\":\"corrected-title-input\",\"input_visible\":true,\"input_decision\":true,\"handled_errors\":\"DuplicateTicket\",\"recovery_behavior\":\"retry\",\"sketch_token\":\"resubmit-button\",\"navigation_type\":\"modeled_view\",\"navigation_target\":\"ticket_detail\"}}}\n",
        )
    }

    fn remove_control_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_control_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"view\":\"ticket_detail\",\"name\":\"submit-ticket\"}}}\n",
        )
    }

    fn update_outcome_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_outcome_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"label\":\"ticket-captured\",\"events\":\"TicketRouted\",\"externally_relevant\":false}}}\n",
        )
    }

    fn remove_outcome_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_outcome_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"label\":\"ticket-captured\"}}}\n",
        )
    }

    fn update_automation_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_automation_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"assign-duplicate-ticket\",\"trigger\":\"DuplicateTicketEscalated\",\"command\":\"CaptureTicket\",\"handled_errors\":\"DuplicateTicket\",\"reaction\":\"escalate duplicate tickets to a human assignment queue\"}}}\n",
        )
    }

    fn remove_automation_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_automation_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"assign-duplicate-ticket\"}}}\n",
        )
    }

    fn update_translation_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_translation_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"capture-ticket-from-webhook\",\"external_event\":\"TicketWebhookRetried\",\"payload_contract\":\"TicketWebhookRetryPayload\",\"command\":\"CaptureTicket\"}}}\n",
        )
    }

    fn remove_translation_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_translation_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"capture-ticket-from-webhook\"}}}\n",
        )
    }

    fn resolve_conflict_mcp_requests(stream_id: &str, branch_tx: &str) -> String {
        format!(
            "{}{}{}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            format_args!(
                "{{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{{\"name\":\"resolve_conflict\",\"arguments\":{{\"id\":\"{stream_id}\",\"choose_event\":\"{branch_tx}\"}}}}}}"
            )
        )
    }

    fn initialize_project_with_scenario(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(cwd)
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
            .current_dir(cwd)
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
            .current_dir(cwd)
            .assert()
            .success();
        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "acceptance",
                "--name",
                "Actor captures ticket",
                "--given",
                "ticket intake screen is open",
                "--when",
                "the actor submits ticket details",
                "--then",
                "the ticket details are visible for review",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn initialize_project_with_command(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_project_with_scenario(cwd)?;
        Command::cargo_bin("emc")?
            .args([
                "add",
                "command",
                "--slice",
                "capture-ticket",
                "--name",
                "CaptureTicket",
                "--input",
                "ticket_title",
                "--input-source",
                "actor",
                "--input-description",
                "title field on the intake form",
                "--input-provenance",
                "actor keystrokes -> form field",
                "--emits",
                "TicketCaptured",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn initialize_project_with_event(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_project_with_scenario(cwd)?;
        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "ticket-events",
                "--attribute",
                "title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "ticket_title",
                "--attribute-source-field",
                "value",
                "--generated-source-kind",
                "projection",
                "--attribute-provenance",
                "projection title field",
                "--observed",
                "true",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn initialize_project_with_read_model(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_project_with_event(cwd)?;
        Command::cargo_bin("emc")?
            .args([
                "add",
                "read-model",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_state",
                "--field",
                "ticket_title",
                "--field-source",
                "event_attribute",
                "--source-event",
                "TicketCaptured",
                "--source-attribute",
                "title",
                "--field-provenance",
                "TicketCaptured.title",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn initialize_project_with_view(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_project_with_read_model(cwd)?;
        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_detail",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_title_view",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "title-label",
                "--field-provenance",
                "ticket_state.ticket_title -> title label",
                "--bit-encoding",
                "UTF-8 title string",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn initialize_project_with_control(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_project_with_read_model(cwd)?;
        add_duplicate_ticket_contract_scenario(cwd)?;
        add_capture_ticket_command_with_error(cwd)?;
        add_controlled_ticket_detail_view(cwd)?;
        Ok(())
    }

    fn initialize_project_with_outcome(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_project_with_scenario(cwd)?;
        Command::cargo_bin("emc")?
            .args([
                "add",
                "outcome",
                "--slice",
                "capture-ticket",
                "--label",
                "ticket-captured",
                "--events",
                "TicketCaptured",
                "--externally-relevant",
                "true",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn initialize_project_with_automation(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_project_with_scenario(cwd)?;
        add_duplicate_ticket_contract_scenario(cwd)?;
        add_capture_ticket_command_with_error(cwd)?;
        Command::cargo_bin("emc")?
            .args([
                "add",
                "automation",
                "--slice",
                "capture-ticket",
                "--name",
                "assign-duplicate-ticket",
                "--trigger",
                "DuplicateTicketDetected",
                "--command",
                "CaptureTicket",
                "--handled-errors",
                "DuplicateTicket",
                "--reaction",
                "route duplicate tickets to manual assignment",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn initialize_project_with_translation(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_project_with_command(cwd)?;
        Command::cargo_bin("emc")?
            .args([
                "add",
                "translation",
                "--slice",
                "capture-ticket",
                "--name",
                "capture-ticket-from-webhook",
                "--external-event",
                "TicketWebhookReceived",
                "--payload-contract",
                "TicketWebhookPayload",
                "--command",
                "CaptureTicket",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn add_duplicate_ticket_contract_scenario(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Duplicate ticket is rejected",
                "--given",
                "tickets stream already contains duplicate title",
                "--when",
                "CaptureTicket handles the duplicate title",
                "--then",
                "DuplicateTicket is returned",
                "--contract-kind",
                "command",
                "--covered-definition",
                "CaptureTicket",
                "--error-references",
                "DuplicateTicket",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn add_capture_ticket_command_with_error(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "command",
                "--slice",
                "capture-ticket",
                "--name",
                "CaptureTicket",
                "--input",
                "ticket_title",
                "--input-source",
                "actor",
                "--input-description",
                "title field on the intake form",
                "--input-provenance",
                "actor keystrokes -> form field",
                "--emits",
                "TicketCaptured",
                "--error",
                "DuplicateTicket",
                "--error-scenario",
                "Duplicate ticket is rejected",
                "--error-recovery",
                "retry",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn add_controlled_ticket_detail_view(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_detail",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_title_view",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "title-label",
                "--field-provenance",
                "ticket_state.ticket_title -> title label",
                "--bit-encoding",
                "UTF-8 title string",
                "--control",
                "submit-ticket",
                "--control-command",
                "CaptureTicket",
                "--control-input",
                "ticket_title",
                "--control-input-source",
                "actor",
                "--control-input-description",
                "title field on the intake form",
                "--control-input-sketch-token",
                "title-input",
                "--control-input-visible",
                "true",
                "--control-input-decision",
                "true",
                "--handled-errors",
                "DuplicateTicket",
                "--recovery-behavior",
                "retry",
                "--control-sketch-token",
                "submit-button",
                "--navigation-type",
                "modeled_view",
                "--navigation-target",
                "ticket_detail",
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn create_slice_update_fork(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
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

        // Clone the shared base into a second replica.
        let replica = TempDir::new()?;
        let copy_status = ProcessCommand::new("cp")
            .arg("-a")
            .arg(format!("{}/.", temp_dir.path().display()))
            .arg(replica.path())
            .status()?;
        assert!(copy_status.success(), "copying base project must succeed");

        // Strip replica-local store metadata so the clone mints a fresh replica id.
        let replica_events = replica.path().join("model/events");
        ignore_not_found(fs::remove_dir_all(replica_events.join(".eventcore")))?;
        ignore_not_found(fs::remove_dir_all(replica_events.join("locks")))?;
        ignore_not_found(fs::remove_dir_all(replica_events.join("index")))?;
        ignore_not_found(fs::remove_dir_all(replica_events.join("tmp")))?;
        ignore_not_found(fs::remove_file(replica_events.join(".lock")))?;

        // Diverge: each replica authors a different description from the shared base.
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--description",
                "Description authored on replica A.",
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
                "Description authored on replica B.",
            ])
            .current_dir(replica.path())
            .assert()
            .success();

        // Union replica B's transactions into the original to produce the fork.
        let original_events = temp_dir.path().join("model/events/events");
        let replica_events_dir = replica.path().join("model/events/events");
        for entry in fs::read_dir(&replica_events_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                continue;
            }
            let file_name = entry.file_name();
            let destination = original_events.join(&file_name);
            if !destination.exists() {
                fs::copy(&path, &destination)?;
            }
        }

        Ok(())
    }

    fn ignore_not_found(result: io::Result<()>) -> io::Result<()> {
        match result {
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            other => other,
        }
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
