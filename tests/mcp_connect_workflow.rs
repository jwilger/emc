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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", rationale := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\", rationale: \"\" }]"
            )
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_workflow_transition() -> Result<(), Box<dyn Error>> {
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
                "navigation",
                "--name",
                "alternate-review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_transition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_transition\""))
            .stdout(predicate::str::contains(
                "removed transition capture-ticket to review-ticket",
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
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(!workflow_json.contains("\"via_navigation\": \"review-ticket-screen\""));
        assert!(workflow_json.contains("\"via_navigation\": \"alternate-review-ticket-screen\""));
        assert!(lean.contains("alternate-review-ticket-screen"));
        assert!(quint.contains("alternate-review-ticket-screen"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_rejects_removing_required_workflow_transition() -> Result<(), Box<dyn Error>> {
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
            .success();

        let workflow_before = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_transition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"message\":\"removing transition would leave workflow step 'review-ticket' without an incoming transition",
            ));

        let workflow_after = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;

        assert_eq!(
            workflow_before, workflow_after,
            "rejected MCP transition removal must not mutate workflow data"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_connects_command_and_event_workflow_steps() -> Result<(), Box<dyn Error>> {
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
        add_slice(temp_dir.path(), "submit-ticket", "Submit ticket")?;
        add_slice(temp_dir.path(), "review-ticket", "Review ticket")?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(command_event_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to submit-ticket",
            ))
            .stdout(predicate::str::contains(
                "connected submit-ticket to review-ticket",
            ));

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(workflow_json.contains("\"via_command\": \"SubmitTicketForReview\""));
        assert!(workflow_json.contains("\"via_event\": \"TicketSubmittedForReview\""));
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"submit-ticket\", kind := \"command\", trigger := \"SubmitTicketForReview\", rationale := \"\" },{ source := \"submit-ticket\", target := \"review-ticket\", kind := \"event\", trigger := \"TicketSubmittedForReview\", rationale := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"submit-ticket\", kind: \"command\", trigger: \"SubmitTicketForReview\", rationale: \"\" },{ source: \"submit-ticket\", target: \"review-ticket\", kind: \"event\", trigger: \"TicketSubmittedForReview\", rationale: \"\" }]"
            )
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_connects_external_trigger_workflow_steps() -> Result<(), Box<dyn Error>> {
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
        add_slice(temp_dir.path(), "record-callback", "Record callback")?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(external_trigger_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to record-callback",
            ));

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(workflow_json.contains("\"via_external_trigger\": \"callback_received\""));
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"record-callback\", kind := \"external_trigger\", trigger := \"callback_received\", rationale := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"record-callback\", kind: \"external_trigger\", trigger: \"callback_received\", rationale: \"\" }]"
            )
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_connects_workflow_exit() -> Result<(), Box<dyn Error>> {
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

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(workflow_exit_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to repair-complete",
            ));

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(workflow_json.contains("\"to_workflow\": \"repair-complete\""));
        assert!(workflow_json.contains("\"via_outcome\": \"ticket_closed\""));
        assert!(
            workflow_json.contains("\"exit_reason\": \"Closed tickets continue to completion.\"")
        );
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := \"workflow_exit:outcome\", trigger := \"ticket_closed\", rationale := \"Closed tickets continue to completion.\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: \"workflow_exit:outcome\", trigger: \"ticket_closed\", rationale: \"Closed tickets continue to completion.\" }]"
            )
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_connect_workflow_schema_advertises_transition_kinds() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(tools_list_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"connect_workflow\""))
            .stdout(predicate::str::contains(
                "\"enum\":[\"command\",\"event\",\"navigation\",\"external_trigger\",\"outcome\"]",
            ));

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

    fn remove_transition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_transition\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\"}}}\n",
        )
    }

    fn tools_list_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
        )
    }

    fn command_event_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"submit-ticket\",\"via\":\"command\",\"name\":\"SubmitTicketForReview\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"submit-ticket\",\"to\":\"review-ticket\",\"via\":\"event\",\"name\":\"TicketSubmittedForReview\"}}}\n",
        )
    }

    fn external_trigger_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"record-callback\",\"via\":\"external_trigger\",\"name\":\"callback_received\"}}}\n",
        )
    }

    fn workflow_exit_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to_workflow\":\"repair-complete\",\"via\":\"outcome\",\"name\":\"ticket_closed\",\"reason\":\"Closed tickets continue to completion.\"}}}\n",
        )
    }
}
