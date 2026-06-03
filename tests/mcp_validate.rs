#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{create_dir_all, write};

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn mcp_stdio_validates_event_model_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("open-repair-ticket.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"repair_ticket\"}],\"events\":[{\"name\":\"RepairTicketOpened\",\"stream\":\"repair_ticket\",\"attributes\":[{\"name\":\"actor_user_id\",\"source\":\"command.actor_user_id\"}]}],\"commands\":[{\"name\":\"OpenRepairTicket\",\"inputs\":[\"actor_user_id\"],\"produces\":[\"RepairTicketOpened\"]}],\"read_models\":[],\"views\":[{\"name\":\"repair_queue_screen\",\"wireframe\":\"<button data-ref=\\\"Open repair ticket\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Open repair ticket\",\"command\":\"OpenRepairTicket\",\"inputs\":[{\"name\":\"actor_user_id\",\"source\":\"session.user_id\",\"description\":\"Authenticated actor identifier.\"}]}]}],\"slices\":[{\"name\":\"Show repair queue\",\"type\":\"state_view\",\"events\":[\"RepairTicketOpened\"],\"views\":[\"repair_queue_screen\"],\"acceptance_scenarios\":[{\"name\":\"show repair queue\",\"given\":[],\"when\":{},\"then\":[]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"validate_event_model\""))
            .stdout(predicate::str::contains(
                "event model is valid at model/browser/data/slices",
            ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_validates_a_single_event_model_file() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("open-repair-ticket.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"repair_ticket\"}],\"events\":[{\"name\":\"RepairTicketOpened\",\"stream\":\"repair_ticket\",\"attributes\":[{\"name\":\"actor_user_id\",\"source\":\"command.actor_user_id\"}]}],\"commands\":[{\"name\":\"OpenRepairTicket\",\"inputs\":[\"actor_user_id\"],\"produces\":[\"RepairTicketOpened\"]}],\"read_models\":[],\"views\":[{\"name\":\"repair_queue_screen\",\"wireframe\":\"<button data-ref=\\\"Open repair ticket\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Open repair ticket\",\"command\":\"OpenRepairTicket\",\"inputs\":[{\"name\":\"actor_user_id\",\"source\":\"session.user_id\",\"description\":\"Authenticated actor identifier.\"}]}]}],\"slices\":[{\"name\":\"Show repair queue\",\"type\":\"state_view\",\"events\":[\"RepairTicketOpened\"],\"views\":[\"repair_queue_screen\"],\"acceptance_scenarios\":[{\"name\":\"show repair queue\",\"given\":[],\"when\":{},\"then\":[]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_file_request())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "event model is valid at model/browser/data/slices/open-repair-ticket.eventmodel.json",
            ));

        Ok(())
    }

    #[test]
    fn mcp_stdio_validate_rejects_project_artifact_drift() -> Result<(), Box<dyn Error>> {
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

        write(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
            "{\n  \"name\": \"Changed\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Actor opens a repair ticket.\",\n  \"board\": {},\n  \"streams\": [],\n  \"events\": [],\n  \"commands\": [],\n  \"read_models\": [],\n  \"slices\": [],\n  \"slice_files\": [],\n  \"steps\": []\n}\n",
        )?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_drift_request())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "browser workflow drift for workflow Open ticket",
            ));

        Ok(())
    }

    #[test]
    fn generated_navigation_workflows_validate_after_cli_modeling() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Clinic Intake"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow",
                "--slug",
                "intake-visit",
                "--name",
                "Intake visit",
                "--description",
                "Actor completes intake for a clinic visit.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        [
            (
                "capture-intake",
                "Capture intake",
                "Actor captures intake details.",
            ),
            ("triage-intake", "Triage intake", "Actor triages intake."),
            (
                "schedule-visit",
                "Schedule visit",
                "Actor schedules a visit.",
            ),
        ]
        .into_iter()
        .try_for_each(|(slug, name, description)| {
            Command::cargo_bin("emc")?
                .args([
                    "add",
                    "slice",
                    "--workflow",
                    "intake-visit",
                    "--slug",
                    slug,
                    "--name",
                    name,
                    "--type",
                    "state_view",
                    "--description",
                    description,
                ])
                .current_dir(temp_dir.path())
                .assert()
                .success();
            Ok::<(), Box<dyn Error>>(())
        })?;

        [
            ("capture-intake", "triage-intake", "triage-intake-screen"),
            ("triage-intake", "schedule-visit", "schedule-visit-screen"),
        ]
        .into_iter()
        .try_for_each(|(source, target, navigation)| {
            Command::cargo_bin("emc")?
                .args([
                    "connect",
                    "workflow",
                    "--workflow",
                    "intake-visit",
                    "--from",
                    source,
                    "--to",
                    target,
                    "--via",
                    "navigation",
                    "--name",
                    navigation,
                ])
                .current_dir(temp_dir.path())
                .assert()
                .success();
            Ok::<(), Box<dyn Error>>(())
        })?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(generated_model_validation_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "event model is valid at model/browser/data/workflows/intake-visit.eventmodel.json",
            ))
            .stdout(predicate::str::contains(
                "event model is valid at model/browser/data/slices",
            ))
            .stdout(predicate::str::contains("\"id\":4"))
            .stdout(predicate::str::contains("\"id\":5"));

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"validate_event_model\",\"arguments\":{\"target\":\"model/browser/data/slices\"}}}\n",
        )
    }

    fn mcp_file_request() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"validate_event_model\",\"arguments\":{\"target\":\"model/browser/data/slices/open-repair-ticket.eventmodel.json\"}}}\n",
        )
    }

    fn mcp_drift_request() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"validate_event_model\",\"arguments\":{\"target\":\"model/browser/data/workflows\"}}}\n",
        )
    }

    fn generated_model_validation_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"check_project\",\"arguments\":{}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"validate_event_model\",\"arguments\":{\"target\":\"model/browser/data/workflows/intake-visit.eventmodel.json\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"validate_event_model\",\"arguments\":{\"target\":\"model/browser/data/slices\"}}}\n",
        )
    }
}
