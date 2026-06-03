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
}
