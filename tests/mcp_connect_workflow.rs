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
                "def workflowTransitions : List String := [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [\"capture-ticket->review-ticket:navigation:review-ticket-screen\"]"
            )
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
                "def workflowTransitions : List String := [\"capture-ticket->submit-ticket:command:SubmitTicketForReview\",\"submit-ticket->review-ticket:event:TicketSubmittedForReview\"]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [\"capture-ticket->submit-ticket:command:SubmitTicketForReview\",\"submit-ticket->review-ticket:event:TicketSubmittedForReview\"]"
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
                "def workflowTransitions : List String := [\"capture-ticket->record-callback:external_trigger:callback_received\"]"
            )
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [\"capture-ticket->record-callback:external_trigger:callback_received\"]"
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
                "\"enum\":[\"command\",\"event\",\"navigation\",\"external_trigger\"]",
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
}
