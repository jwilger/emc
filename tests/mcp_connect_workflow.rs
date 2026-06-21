// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use serde_json::Value;
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.navigation, trigger := \"review-ticket-screen\", sourceControl := \"review-ticket-screen\", targetView := \"review-ticket-screen\", rationale := \"\", payloadContract := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: Navigation, trigger: \"review-ticket-screen\", sourceControl: \"review-ticket-screen\", targetView: \"review-ticket-screen\", rationale: \"\", payloadContract: \"\" }]"
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
                "--source-control",
                "review-ticket-screen",
                "--target-view",
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
                "--source-control",
                "alternate-review-ticket-screen",
                "--target-view",
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
    fn mcp_stdio_updates_workflow_transition() -> Result<(), Box<dyn Error>> {
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
                "--source-control",
                "review-ticket-screen",
                "--target-view",
                "review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_transition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_transition\""))
            .stdout(predicate::str::contains(
                "updated transition capture-ticket to review-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            !lean.contains("trigger := \"review-ticket-screen\""),
            "Lean artifact must remove the replaced workflow transition"
        );
        assert!(
            !quint.contains("trigger: \"review-ticket-screen\""),
            "Quint artifact must remove the replaced workflow transition"
        );
        assert!(
            lean.contains("trigger := \"alternate-review-ticket-screen\""),
            "Lean artifact must contain the replacement workflow transition"
        );
        assert!(
            quint.contains("trigger: \"alternate-review-ticket-screen\""),
            "Quint artifact must contain the replacement workflow transition"
        );

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
                "--source-control",
                "review-ticket-screen",
                "--target-view",
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"submit-ticket\", kind := WorkflowTransitionKind.command, trigger := \"SubmitTicketForReview\", sourceControl := \"\", targetView := \"\", rationale := \"\", payloadContract := \"\" },{ source := \"submit-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.event, trigger := \"TicketSubmittedForReview\", sourceControl := \"\", targetView := \"\", rationale := \"\", payloadContract := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"submit-ticket\", kind: Command, trigger: \"SubmitTicketForReview\", sourceControl: \"\", targetView: \"\", rationale: \"\", payloadContract: \"\" },{ source: \"submit-ticket\", target: \"review-ticket\", kind: Event, trigger: \"TicketSubmittedForReview\", sourceControl: \"\", targetView: \"\", rationale: \"\", payloadContract: \"\" }]"
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"record-callback\", kind := WorkflowTransitionKind.externalTrigger, trigger := \"callback_received\", sourceControl := \"\", targetView := \"\", rationale := \"\", payloadContract := \"CallbackReceivedPayload\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"record-callback\", kind: ExternalTrigger, trigger: \"callback_received\", sourceControl: \"\", targetView: \"\", rationale: \"\", payloadContract: \"CallbackReceivedPayload\" }]"
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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := WorkflowTransitionKind.workflowExitOutcome, trigger := \"ticket_closed\", sourceControl := \"\", targetView := \"\", rationale := \"Closed tickets continue to completion.\", payloadContract := \"\" }]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: WorkflowExitOutcome, trigger: \"ticket_closed\", sourceControl: \"\", targetView: \"\", rationale: \"Closed tickets continue to completion.\", payloadContract: \"\" }]"
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
    fn mcp_stdio_transition_tool_schemas_have_openai_compatible_top_level_shape()
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
            .write_stdin(tools_list_requests())
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
        let tools_response = responses
            .iter()
            .find(|response| response["id"] == 2)
            .ok_or("tools/list response must be present")?;
        let tools = tools_response
            .get("result")
            .and_then(|result| result.get("tools"))
            .and_then(Value::as_array)
            .ok_or("tools/list response must include tools")?;

        assert_openai_compatible_transition_tool_schema(transition_tool_schema(
            tools,
            "connect_workflow",
        )?)?;
        assert_openai_compatible_transition_tool_schema(transition_tool_schema(
            tools,
            "remove_transition",
        )?)?;

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
    fn mcp_stdio_updates_workflow_outcome_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_open_ticket_workflow(temp_dir.path())?;
        add_workflow_outcome(temp_dir.path(), "ticket_captured", true)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_workflow_outcome_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"update_workflow_outcome\""))
            .stdout(predicate::str::contains(
                "updated workflow outcome ticket_captured on workflow open-ticket",
            ));

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(quint.contains(
            "val workflowOutcomes: List[WorkflowOutcome] = [{ sourceSlice: \"capture-ticket\", label: \"ticket_ready\", externallyRelevant: false }]"
        ));
        assert!(!quint.contains("label: \"ticket_captured\""));

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_workflow_outcome_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_open_ticket_workflow(temp_dir.path())?;
        add_workflow_outcome(temp_dir.path(), "ticket_captured", true)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_workflow_outcome_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"remove_workflow_outcome\""))
            .stdout(predicate::str::contains(
                "removed workflow outcome ticket_captured from workflow open-ticket",
            ));

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(quint.contains("val workflowOutcomes: List[WorkflowOutcome] = []"));

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
    fn mcp_stdio_updates_workflow_command_error_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_open_ticket_workflow(temp_dir.path())?;
        add_workflow_command_error(temp_dir.path(), "CaptureTicket", "DuplicateTicket")?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_workflow_command_error_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"update_workflow_command_error\"",
            ))
            .stdout(predicate::str::contains(
                "updated workflow command error DuplicateTicket on workflow open-ticket",
            ));

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(quint.contains(
            "val workflowCommandErrors: List[WorkflowCommandError] = [{ sourceSlice: \"capture-ticket\", commandName: \"SubmitTicket\", errorName: \"TicketAlreadySubmitted\" }]"
        ));
        assert!(!quint.contains("DuplicateTicket"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_workflow_command_error_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_open_ticket_workflow(temp_dir.path())?;
        add_workflow_command_error(temp_dir.path(), "CaptureTicket", "DuplicateTicket")?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_workflow_command_error_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"remove_workflow_command_error\"",
            ))
            .stdout(predicate::str::contains(
                "removed workflow command error DuplicateTicket from workflow open-ticket",
            ));

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(quint.contains("val workflowCommandErrors: List[WorkflowCommandError] = []"));

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
            ))
            .stdout(predicate::str::contains(
                "added workflow owned definition event TicketSubmitted to workflow open-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(lean.contains(
            "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := WorkflowOwnedDefinitionKind.command, definitionName := \"CaptureTicket\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" },{ sourceSlice := \"capture-ticket\", definitionKind := WorkflowOwnedDefinitionKind.event, definitionName := \"TicketSubmitted\", definitionStream := \"tickets\", sourceProvenance := \"CaptureTicket command input\", eventParticipation := \"emitted\", viewRole := \"\" },{ sourceSlice := \"capture-ticket\", definitionKind := WorkflowOwnedDefinitionKind.view, definitionName := \"ticket-entry-screen\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"entry\" }]"
        ));
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: OwnedCommand, definitionName: \"CaptureTicket\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" },{ sourceSlice: \"capture-ticket\", definitionKind: OwnedEvent, definitionName: \"TicketSubmitted\", definitionStream: \"tickets\", sourceProvenance: \"CaptureTicket command input\", eventParticipation: \"emitted\", viewRole: \"\" },{ sourceSlice: \"capture-ticket\", definitionKind: OwnedView, definitionName: \"ticket-entry-screen\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"entry\" }]"
        ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_workflow_owned_definition_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_open_ticket_workflow(temp_dir.path())?;
        add_workflow_owned_definition(
            temp_dir.path(),
            "open-ticket",
            "capture-ticket",
            "command",
            "CaptureTicket",
        )?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_workflow_owned_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"update_workflow_owned_definition\"",
            ))
            .stdout(predicate::str::contains(
                "updated workflow owned definition command CaptureTicket on workflow open-ticket",
            ));

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(quint.contains(
            "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: OwnedCommand, definitionName: \"SubmitTicket\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" }]"
        ));
        assert!(!quint.contains("definitionName: \"CaptureTicket\""));

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_workflow_owned_definition_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_open_ticket_workflow(temp_dir.path())?;
        add_workflow_owned_definition(
            temp_dir.path(),
            "open-ticket",
            "capture-ticket",
            "command",
            "CaptureTicket",
        )?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_workflow_owned_definition_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"remove_workflow_owned_definition\"",
            ))
            .stdout(predicate::str::contains(
                "removed workflow owned definition command CaptureTicket from workflow open-ticket",
            ));

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(quint.contains("val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = []"));

        Ok(())
    }

    #[test]
    fn mcp_stdio_rejects_event_participation_without_event_identity() -> Result<(), Box<dyn Error>>
    {
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
            .write_stdin(incomplete_event_participation_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"error\""))
            .stdout(predicate::str::contains(
                "event_participation requires definition_stream and source_provenance",
            ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_rejects_view_role_for_non_view_definitions() -> Result<(), Box<dyn Error>> {
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
            .write_stdin(non_view_role_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"error\""))
            .stdout(predicate::str::contains(
                "view_role requires definition_kind view",
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
            "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.navigation, trigger := \"review-ticket-screen\", sourceControl := \"review-ticket-screen\", targetView := \"review-ticket-screen\", sourceEvidence := \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence := \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));
        assert!(quint.contains(
            "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: Navigation, trigger: \"review-ticket-screen\", sourceControl: \"review-ticket-screen\", targetView: \"review-ticket-screen\", sourceEvidence: \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence: \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
        ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_workflow_transition_evidence_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_open_ticket_review_workflow(temp_dir.path())?;
        add_transition_evidence_with_mcp(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_workflow_transition_evidence_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"update_workflow_transition_evidence\"",
            ))
            .stdout(predicate::str::contains(
                "updated workflow transition evidence navigation review-ticket-screen on workflow open-ticket",
            ));

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(quint.contains(
            "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: Navigation, trigger: \"review-ticket-screen\", sourceControl: \"review-ticket-screen\", targetView: \"review-ticket-screen\", sourceEvidence: \"capture-ticket control is the modeled navigation source\", targetEvidence: \"review-ticket entry view is the modeled navigation target\" }]"
        ));
        assert!(
            !quint.contains("capture-ticket view owns the review-ticket-screen navigation control")
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_workflow_transition_evidence_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_open_ticket_review_workflow(temp_dir.path())?;
        add_transition_evidence_with_mcp(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_workflow_transition_evidence_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"remove_workflow_transition_evidence\"",
            ))
            .stdout(predicate::str::contains(
                "removed workflow transition evidence navigation review-ticket-screen from workflow open-ticket",
            ));

        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        assert!(
            quint
                .contains("val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = []")
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_workflow_entry_lifecycle_coverage() -> Result<(), Box<dyn Error>> {
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
                "application-entry",
                "--name",
                "Application entry",
                "--description",
                "Actor enters the application.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "application-entry",
                "--slug",
                "entry-state",
                "--name",
                "Entry state",
                "--type",
                "state_view",
                "--description",
                "Slice description.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(workflow_entry_lifecycle_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"require_workflow_entry_lifecycle_coverage\"",
            ))
            .stdout(predicate::str::contains(
                "\"add_workflow_entry_lifecycle_state\"",
            ))
            .stdout(predicate::str::contains(
                "marked workflow application-entry as requiring entry lifecycle coverage",
            ))
            .stdout(predicate::str::contains(
                "added workflow entry lifecycle state fully_configured to workflow application-entry",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/ApplicationEntry.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/ApplicationEntry.qnt"))?;

        assert!(lean.contains("def workflowRequiresEntryLifecycleCoverage : Bool := true"));
        assert!(quint.contains("val workflowRequiresEntryLifecycleCoverage = true"));
        assert!(lean.contains(
            "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := [{ state := WorkflowEntryLifecycleStateName.freshUninitialized, step := \"entry-state\", evidence := \"entry-state view distinguishes first arrival before initialization\" },{ state := WorkflowEntryLifecycleStateName.initializedUnauthenticated, step := \"entry-state\", evidence := \"entry-state view distinguishes initialized unauthenticated sessions\" },{ state := WorkflowEntryLifecycleStateName.initializedAuthenticated, step := \"entry-state\", evidence := \"entry-state view distinguishes initialized authenticated sessions\" },{ state := WorkflowEntryLifecycleStateName.partiallyConfigured, step := \"entry-state\", evidence := \"entry-state view distinguishes partially configured accounts\" },{ state := WorkflowEntryLifecycleStateName.fullyConfigured, step := \"entry-state\", evidence := \"entry-state view distinguishes fully configured accounts\" }]"
        ));
        assert!(quint.contains(
            "val workflowEntryLifecycleStates: List[WorkflowEntryLifecycleState] = [{ state: FreshUninitialized, step: \"entry-state\", evidence: \"entry-state view distinguishes first arrival before initialization\" },{ state: InitializedUnauthenticated, step: \"entry-state\", evidence: \"entry-state view distinguishes initialized unauthenticated sessions\" },{ state: InitializedAuthenticated, step: \"entry-state\", evidence: \"entry-state view distinguishes initialized authenticated sessions\" },{ state: PartiallyConfigured, step: \"entry-state\", evidence: \"entry-state view distinguishes partially configured accounts\" },{ state: FullyConfigured, step: \"entry-state\", evidence: \"entry-state view distinguishes fully configured accounts\" }]"
        ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_updates_workflow_entry_lifecycle_state() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_application_entry_workflow(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(update_workflow_entry_lifecycle_state_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"update_workflow_entry_lifecycle_state\"",
            ))
            .stdout(predicate::str::contains(
                "updated workflow entry lifecycle state fresh_uninitialized on workflow application-entry",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/ApplicationEntry.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/ApplicationEntry.qnt"))?;

        assert!(lean.contains("evidence := \"entry-state view confirms first-arrival routing\""));
        assert!(quint.contains("evidence: \"entry-state view confirms first-arrival routing\""));
        assert!(
            !lean.contains("entry-state view distinguishes first arrival before initialization")
        );
        assert!(
            !quint.contains("entry-state view distinguishes first arrival before initialization")
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_removes_workflow_entry_lifecycle_coverage_and_state() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        initialize_application_entry_workflow(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(remove_workflow_entry_lifecycle_mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"remove_workflow_entry_lifecycle_coverage\"",
            ))
            .stdout(predicate::str::contains(
                "\"remove_workflow_entry_lifecycle_state\"",
            ))
            .stdout(predicate::str::contains(
                "removed workflow entry lifecycle coverage requirement from workflow application-entry",
            ))
            .stdout(predicate::str::contains(
                "removed workflow entry lifecycle state fresh_uninitialized from workflow application-entry",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/ApplicationEntry.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/ApplicationEntry.qnt"))?;

        assert!(lean.contains("def workflowRequiresEntryLifecycleCoverage : Bool := false"));
        assert!(quint.contains("val workflowRequiresEntryLifecycleCoverage = false"));
        assert!(
            lean.contains(
                "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := []"
            )
        );
        assert!(
            quint.contains(
                "val workflowEntryLifecycleStates: List[WorkflowEntryLifecycleState] = []"
            )
        );

        Ok(())
    }

    fn initialize_application_entry_workflow(cwd: &Path) -> Result<(), Box<dyn Error>> {
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
                "application-entry",
                "--name",
                "Application entry",
                "--description",
                "Actor enters the application.",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "application-entry",
                "--slug",
                "entry-state",
                "--name",
                "Entry state",
                "--type",
                "state_view",
                "--description",
                "Slice description.",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "mark",
                "workflow-entry-lifecycle-required",
                "--workflow",
                "application-entry",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-entry-lifecycle-state",
                "--workflow",
                "application-entry",
                "--state",
                "fresh_uninitialized",
                "--step",
                "entry-state",
                "--evidence",
                "entry-state view distinguishes first arrival before initialization",
            ])
            .current_dir(cwd)
            .assert()
            .success();

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

    fn initialize_open_ticket_workflow(cwd: &Path) -> Result<(), Box<dyn Error>> {
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

        add_slice(cwd, "capture-ticket", "Capture ticket")
    }

    fn initialize_open_ticket_review_workflow(cwd: &Path) -> Result<(), Box<dyn Error>> {
        initialize_open_ticket_workflow(cwd)?;
        add_slice(cwd, "review-ticket", "Review ticket")
    }

    fn add_transition_evidence_with_mcp(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(cwd)
            .write_stdin(workflow_transition_evidence_mcp_requests())
            .assert()
            .success();
        Ok(())
    }

    fn add_workflow_outcome(
        cwd: &Path,
        label: &str,
        externally_relevant: bool,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-outcome",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--label",
                label,
                "--externally-relevant",
                if externally_relevant { "true" } else { "false" },
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_workflow_command_error(
        cwd: &Path,
        command: &str,
        error: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-command-error",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--command",
                command,
                "--error",
                error,
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_workflow_owned_definition(
        cwd: &Path,
        workflow: &str,
        source_slice: &str,
        definition_kind: &str,
        definition_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-owned-definition",
                "--workflow",
                workflow,
                "--source-slice",
                source_slice,
                "--definition-kind",
                definition_kind,
                "--definition-name",
                definition_name,
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn transition_tool_schema<'tools>(
        tools: &'tools [Value],
        name: &str,
    ) -> Result<&'tools Value, Box<dyn Error>> {
        tools
            .iter()
            .find(|tool| tool["name"] == Value::String(name.to_owned()))
            .and_then(|tool| tool.get("inputSchema"))
            .ok_or_else(|| format!("{name} input schema must be advertised").into())
    }

    fn assert_openai_compatible_transition_tool_schema(
        schema: &Value,
    ) -> Result<(), Box<dyn Error>> {
        assert_eq!(schema["type"], Value::String("object".to_owned()));
        for keyword in ["oneOf", "anyOf", "allOf", "enum", "not"] {
            assert!(
                schema.get(keyword).is_none(),
                "transition tool schemas must not use top-level {keyword}: {schema}"
            );
        }

        let properties = schema["properties"]
            .as_object()
            .ok_or("transition tool schema must advertise object properties")?;
        for property in ["workflow", "from", "to", "to_workflow", "via", "name"] {
            assert!(
                properties.contains_key(property),
                "transition tool schema must advertise {property}: {schema}"
            );
        }
        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\",\"source_control\":\"review-ticket-screen\",\"target_view\":\"review-ticket-screen\"}}}\n",
        )
    }

    fn remove_transition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_transition\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\",\"source_control\":\"review-ticket-screen\",\"target_view\":\"review-ticket-screen\"}}}\n",
        )
    }

    fn update_transition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_transition\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\",\"new_from\":\"capture-ticket\",\"new_to\":\"review-ticket\",\"new_via\":\"navigation\",\"new_name\":\"alternate-review-ticket-screen\",\"new_source_control\":\"alternate-review-ticket-screen\",\"new_target_view\":\"alternate-review-ticket-screen\"}}}\n",
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

    fn update_workflow_outcome_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_workflow_outcome\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"label\":\"ticket_captured\",\"externally_relevant\":true,\"new_source_slice\":\"capture-ticket\",\"new_label\":\"ticket_ready\",\"new_externally_relevant\":false}}}\n",
        )
    }

    fn remove_workflow_outcome_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_workflow_outcome\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"label\":\"ticket_captured\",\"externally_relevant\":true}}}\n",
        )
    }

    fn workflow_command_error_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_command_error\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"command\":\"CaptureTicket\",\"error\":\"DuplicateTicket\"}}}\n",
        )
    }

    fn update_workflow_command_error_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_workflow_command_error\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"command\":\"CaptureTicket\",\"error\":\"DuplicateTicket\",\"new_source_slice\":\"capture-ticket\",\"new_command\":\"SubmitTicket\",\"new_error\":\"TicketAlreadySubmitted\"}}}\n",
        )
    }

    fn remove_workflow_command_error_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_workflow_command_error\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"command\":\"CaptureTicket\",\"error\":\"DuplicateTicket\"}}}\n",
        )
    }

    fn workflow_owned_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_owned_definition\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"definition_kind\":\"command\",\"definition_name\":\"CaptureTicket\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_owned_definition\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"definition_kind\":\"event\",\"definition_name\":\"TicketSubmitted\",\"definition_stream\":\"tickets\",\"source_provenance\":\"CaptureTicket command input\",\"event_participation\":\"emitted\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_owned_definition\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"definition_kind\":\"view\",\"definition_name\":\"ticket-entry-screen\",\"view_role\":\"entry\"}}}\n",
        )
    }

    fn update_workflow_owned_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_workflow_owned_definition\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"definition_kind\":\"command\",\"definition_name\":\"CaptureTicket\",\"new_source_slice\":\"capture-ticket\",\"new_definition_kind\":\"command\",\"new_definition_name\":\"SubmitTicket\"}}}\n",
        )
    }

    fn remove_workflow_owned_definition_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_workflow_owned_definition\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"definition_kind\":\"command\",\"definition_name\":\"CaptureTicket\"}}}\n",
        )
    }

    fn incomplete_event_participation_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_owned_definition\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"definition_kind\":\"event\",\"definition_name\":\"TicketSubmitted\",\"event_participation\":\"emitted\"}}}\n",
        )
    }

    fn non_view_role_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_owned_definition\",\"arguments\":{\"workflow\":\"open-ticket\",\"source_slice\":\"capture-ticket\",\"definition_kind\":\"command\",\"definition_name\":\"CaptureTicket\",\"view_role\":\"entry\"}}}\n",
        )
    }

    fn workflow_transition_evidence_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"connect_workflow\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\",\"source_control\":\"review-ticket-screen\",\"target_view\":\"review-ticket-screen\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_transition_evidence\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\",\"source_control\":\"review-ticket-screen\",\"target_view\":\"review-ticket-screen\",\"source_evidence\":\"capture-ticket view owns the review-ticket-screen navigation control\",\"target_evidence\":\"review-ticket workflow step exposes review-ticket-screen as its entry view\"}}}\n",
        )
    }

    fn update_workflow_transition_evidence_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_workflow_transition_evidence\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\",\"source_control\":\"review-ticket-screen\",\"target_view\":\"review-ticket-screen\",\"source_evidence\":\"capture-ticket view owns the review-ticket-screen navigation control\",\"target_evidence\":\"review-ticket workflow step exposes review-ticket-screen as its entry view\",\"new_from\":\"capture-ticket\",\"new_to\":\"review-ticket\",\"new_via\":\"navigation\",\"new_name\":\"review-ticket-screen\",\"new_source_control\":\"review-ticket-screen\",\"new_target_view\":\"review-ticket-screen\",\"new_source_evidence\":\"capture-ticket control is the modeled navigation source\",\"new_target_evidence\":\"review-ticket entry view is the modeled navigation target\"}}}\n",
        )
    }

    fn remove_workflow_transition_evidence_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_workflow_transition_evidence\",\"arguments\":{\"workflow\":\"open-ticket\",\"from\":\"capture-ticket\",\"to\":\"review-ticket\",\"via\":\"navigation\",\"name\":\"review-ticket-screen\",\"source_control\":\"review-ticket-screen\",\"target_view\":\"review-ticket-screen\",\"source_evidence\":\"capture-ticket view owns the review-ticket-screen navigation control\",\"target_evidence\":\"review-ticket workflow step exposes review-ticket-screen as its entry view\"}}}\n",
        )
    }

    fn workflow_entry_lifecycle_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"require_workflow_entry_lifecycle_coverage\",\"arguments\":{\"workflow\":\"application-entry\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_entry_lifecycle_state\",\"arguments\":{\"workflow\":\"application-entry\",\"state\":\"fresh_uninitialized\",\"step\":\"entry-state\",\"evidence\":\"entry-state view distinguishes first arrival before initialization\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_entry_lifecycle_state\",\"arguments\":{\"workflow\":\"application-entry\",\"state\":\"initialized_unauthenticated\",\"step\":\"entry-state\",\"evidence\":\"entry-state view distinguishes initialized unauthenticated sessions\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_entry_lifecycle_state\",\"arguments\":{\"workflow\":\"application-entry\",\"state\":\"initialized_authenticated\",\"step\":\"entry-state\",\"evidence\":\"entry-state view distinguishes initialized authenticated sessions\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_entry_lifecycle_state\",\"arguments\":{\"workflow\":\"application-entry\",\"state\":\"partially_configured\",\"step\":\"entry-state\",\"evidence\":\"entry-state view distinguishes partially configured accounts\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":8,\"method\":\"tools/call\",\"params\":{\"name\":\"add_workflow_entry_lifecycle_state\",\"arguments\":{\"workflow\":\"application-entry\",\"state\":\"fully_configured\",\"step\":\"entry-state\",\"evidence\":\"entry-state view distinguishes fully configured accounts\"}}}\n",
        )
    }

    fn update_workflow_entry_lifecycle_state_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"update_workflow_entry_lifecycle_state\",\"arguments\":{\"workflow\":\"application-entry\",\"state\":\"fresh_uninitialized\",\"step\":\"entry-state\",\"evidence\":\"entry-state view distinguishes first arrival before initialization\",\"new_state\":\"fresh_uninitialized\",\"new_step\":\"entry-state\",\"new_evidence\":\"entry-state view confirms first-arrival routing\"}}}\n",
        )
    }

    fn remove_workflow_entry_lifecycle_mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_workflow_entry_lifecycle_coverage\",\"arguments\":{\"workflow\":\"application-entry\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"remove_workflow_entry_lifecycle_state\",\"arguments\":{\"workflow\":\"application-entry\",\"state\":\"fresh_uninitialized\",\"step\":\"entry-state\",\"evidence\":\"entry-state view distinguishes first arrival before initialization\"}}}\n",
        )
    }
}
