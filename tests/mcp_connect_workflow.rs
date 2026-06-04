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

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", rationale := \"\", payloadContract := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\", rationale: \"\", payloadContract: \"\" }]"
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

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

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

        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let lean_before = read_to_string(&lean_path)?;
        let quint_before = read_to_string(&quint_path)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_transition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"message\":\"removing transition would leave workflow step 'review-ticket' without an incoming transition",
            ));

        assert_eq!(
            lean_before,
            read_to_string(lean_path)?,
            "rejected MCP transition removal must not mutate Lean workflow data"
        );
        assert_eq!(
            quint_before,
            read_to_string(quint_path)?,
            "rejected MCP transition removal must not mutate Quint workflow data"
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

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"submit-ticket\", kind := \"command\", trigger := \"SubmitTicketForReview\", rationale := \"\", payloadContract := \"\" },{ source := \"submit-ticket\", target := \"review-ticket\", kind := \"event\", trigger := \"TicketSubmittedForReview\", rationale := \"\", payloadContract := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"submit-ticket\", kind: \"command\", trigger: \"SubmitTicketForReview\", rationale: \"\", payloadContract: \"\" },{ source: \"submit-ticket\", target: \"review-ticket\", kind: \"event\", trigger: \"TicketSubmittedForReview\", rationale: \"\", payloadContract: \"\" }]"
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

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"record-callback\", kind := \"external_trigger\", trigger := \"callback_received\", rationale := \"\", payloadContract := \"CallbackReceivedPayload\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"record-callback\", kind: \"external_trigger\", trigger: \"callback_received\", rationale: \"\", payloadContract: \"CallbackReceivedPayload\" }]"
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

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := \"workflow_exit:outcome\", trigger := \"ticket_closed\", rationale := \"Closed tickets continue to completion.\", payloadContract := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: \"workflow_exit:outcome\", trigger: \"ticket_closed\", rationale: \"Closed tickets continue to completion.\", payloadContract: \"\" }]"
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

    #[test]
    fn mcp_stdio_authors_workflow_outcome_facts() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(workflow_outcome_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_workflow_outcome\""))
            .stdout(predicate::str::contains(
                "added workflow outcome ticket_captured to workflow open-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(lean.contains(
            "def workflowOutcomes : List WorkflowOutcome := [{ sourceSlice := \"capture-ticket\", label := \"ticket_captured\", externallyRelevant := true }]"
        ));
        assert!(quint.contains(
            "val workflowOutcomes: List[WorkflowOutcome] = [{ sourceSlice: \"capture-ticket\", label: \"ticket_captured\", externallyRelevant: true }]"
        ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_workflow_command_error_facts() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(workflow_command_error_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_workflow_command_error\""))
            .stdout(predicate::str::contains(
                "added workflow command error DuplicateTicket to workflow open-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(lean.contains(
            "def workflowCommandErrors : List WorkflowCommandError := [{ sourceSlice := \"capture-ticket\", commandName := \"CaptureTicket\", errorName := \"DuplicateTicket\" }]"
        ));
        assert!(quint.contains(
            "val workflowCommandErrors: List[WorkflowCommandError] = [{ sourceSlice: \"capture-ticket\", commandName: \"CaptureTicket\", errorName: \"DuplicateTicket\" }]"
        ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_workflow_owned_definition_facts() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(workflow_owned_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"add_workflow_owned_definition\"",
            ))
            .stdout(predicate::str::contains(
                "added workflow owned definition command CaptureTicket to workflow open-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(lean.contains(
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := \"command\", definitionName := \"CaptureTicket\" }]"
        ));
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: \"command\", definitionName: \"CaptureTicket\" }]"
        ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_workflow_transition_evidence_facts() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(workflow_transition_evidence_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"add_workflow_transition_evidence\"",
            ))
            .stdout(predicate::str::contains(
                "added workflow transition evidence navigation review-ticket-screen to workflow open-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(lean.contains(
            "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", sourceEvidence := \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence := \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
        assert!(quint.contains(
            "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\", sourceEvidence: \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence: \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
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
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"record-callback\",\"via\":\"external_trigger\",\"name\":\"callback_received\",\"payload_contract\":\"CallbackReceivedPayload\"}}}\n",
        )
    }

    fn workflow_exit_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to_workflow\":\"repair-complete\",\"via\":\"outcome\",\"name\":\"ticket_closed\",\"reason\":\"Closed tickets continue to completion.\"}}}\n",
        )
    }

    fn workflow_outcome_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_outcome\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"label\":\"ticket_captured\",\"externally_relevant\":true}}}\n",
        )
    }

    fn workflow_command_error_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_command_error\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"command\":\"CaptureTicket\",\"error\":\"DuplicateTicket\"}}}\n",
        )
    }

    fn workflow_owned_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_owned_definition\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"definition_kind\":\"command\",\"definition_name\":\"CaptureTicket\"}}}\n",
        )
    }

    fn workflow_transition_evidence_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_transition_evidence\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\",\"source_evidence\":\"capture-ticket view owns the review-ticket-screen navigation control\",\"target_evidence\":\"review-ticket workflow step exposes review-ticket-screen as its entry view\"}}}\n",
        )
    }
}
