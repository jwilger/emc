#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::read_to_string;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn add_scenario_and_data_flow_updates_formal_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

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
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added acceptance scenario Actor captures ticket to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket state projector records title",
                "--given",
                "TicketCaptured is stored",
                "--when",
                "ticket_state projects the event",
                "--then",
                "ticket_state.title equals the event title",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_state",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added contract scenario Ticket state projector records title to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "actor input title field",
                "--transformation",
                "copied without normalization",
                "--target",
                "Capture ticket.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added bit-level data flow ticket_title to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains(
                "def sliceAcceptanceScenarios : List EventModelScenario := [{ name := \"Actor captures ticket\", givenSteps := [\"ticket intake screen is open\"], whenSteps := [\"the actor submits ticket details\"], thenSteps := [\"the ticket details are visible for review\"], readStreams := [], writtenStreams := [], contractKind := \"\", coveredDefinition := \"\", errorReferences := [] }]"
            ),
            "Lean slice artifact must carry authored acceptance scenarios"
        );
        assert!(
            lean.contains(
                "def sliceContractScenarios : List EventModelScenario := [{ name := \"Ticket state projector records title\", givenSteps := [\"TicketCaptured is stored\"], whenSteps := [\"ticket_state projects the event\"], thenSteps := [\"ticket_state.title equals the event title\"], readStreams := [], writtenStreams := [], contractKind := \"projector\", coveredDefinition := \"ticket_state\", errorReferences := [] }]"
            ),
            "Lean slice artifact must carry authored contract scenarios"
        );
        assert!(
            lean.contains(
                "def sliceBitLevelDataFlows : List BitLevelDataFlow := [{ datum := \"ticket_title\", source := \"actor input title field\", transformationSemantics := \"copied without normalization\", target := \"Capture ticket.ticket_title\", bitEncoding := \"UTF-8 string\" }]"
            ),
            "Lean slice artifact must carry authored bit-level data flows"
        );

        assert!(
            quint.contains(
                "val sliceAcceptanceScenarios: List[EventModelScenario] = [{ name: \"Actor captures ticket\", givenSteps: [\"ticket intake screen is open\"], whenSteps: [\"the actor submits ticket details\"], thenSteps: [\"the ticket details are visible for review\"], readStreams: [], writtenStreams: [], contractKind: \"\", coveredDefinition: \"\", errorReferences: [] }]"
            ),
            "Quint slice artifact must carry authored acceptance scenarios"
        );
        assert!(
            quint.contains(
                "val sliceContractScenarios: List[EventModelScenario] = [{ name: \"Ticket state projector records title\", givenSteps: [\"TicketCaptured is stored\"], whenSteps: [\"ticket_state projects the event\"], thenSteps: [\"ticket_state.title equals the event title\"], readStreams: [], writtenStreams: [], contractKind: \"projector\", coveredDefinition: \"ticket_state\", errorReferences: [] }]"
            ),
            "Quint slice artifact must carry authored contract scenarios"
        );
        assert!(
            quint.contains(
                "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = [{ datum: \"ticket_title\", source: \"actor input title field\", transformationSemantics: \"copied without normalization\", target: \"Capture ticket.ticket_title\", bitEncoding: \"UTF-8 string\" }]"
            ),
            "Quint slice artifact must carry authored bit-level data flows"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_slice_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_slice_scenario\""))
            .stdout(predicate::str::contains("\"add_bit_level_data_flow\""))
            .stdout(predicate::str::contains(
                "added acceptance scenario Actor captures ticket to slice capture-ticket",
            ))
            .stdout(predicate::str::contains(
                "added bit-level data flow ticket_title to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains("def sliceAcceptanceScenarios : List EventModelScenario := [{ name := \"Actor captures ticket\""),
            "MCP-authored scenario must be represented in the Lean artifact"
        );
        assert!(
            lean.contains(
                "def sliceBitLevelDataFlows : List BitLevelDataFlow := [{ datum := \"ticket_title\""
            ),
            "MCP-authored data flow must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn add_state_change_scenario_records_read_and_written_streams() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "state_change",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
                "--singleton",
                "true",
                "--repeat-behavior",
                "idempotent",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "actor_input",
                "--attribute-source-field",
                "ticket_title",
                "--attribute-provenance",
                "CaptureTicket.ticket_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "outcome",
                "--slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--events",
                "TicketCaptured",
                "--externally-relevant",
                "false",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "actor input title field",
                "--transformation",
                "captured without normalization",
                "--target",
                "CaptureTicket",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "CaptureTicket.ticket_title",
                "--transformation",
                "copied into TicketCaptured.ticket_title",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
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
                "tickets stream is available",
                "--when",
                "the actor submits ticket details",
                "--then",
                "TicketCaptured is written",
                "--read-streams",
                "tickets",
                "--written-streams",
                "tickets",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added acceptance scenario Actor captures ticket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains(
                "def sliceAcceptanceScenarios : List EventModelScenario := [{ name := \"Actor captures ticket\", givenSteps := [\"tickets stream is available\"], whenSteps := [\"the actor submits ticket details\"], thenSteps := [\"TicketCaptured is written\"], readStreams := [\"tickets\"], writtenStreams := [\"tickets\"], contractKind := \"\", coveredDefinition := \"\", errorReferences := [] }]"
            ),
            "Lean slice artifact must carry authored scenario stream reads and writes"
        );
        assert!(
            quint.contains(
                "val sliceAcceptanceScenarios: List[EventModelScenario] = [{ name: \"Actor captures ticket\", givenSteps: [\"tickets stream is available\"], whenSteps: [\"the actor submits ticket details\"], thenSteps: [\"TicketCaptured is written\"], readStreams: [\"tickets\"], writtenStreams: [\"tickets\"], contractKind: \"\", coveredDefinition: \"\", errorReferences: [] }]"
            ),
            "Quint slice artifact must carry authored scenario stream reads and writes"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_scenario_streams() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_scenario_stream_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added acceptance scenario Actor captures ticket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains("readStreams := [\"tickets\"], writtenStreams := [\"tickets\"]"),
            "MCP-authored scenario stream facts must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn update_slice_preserves_authored_formal_slice_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

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
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "actor input title field",
                "--transformation",
                "copied without normalization",
                "--target",
                "Capture ticket.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
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
                "Actor enters repair ticket details and priority.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains(
                "def sliceDescription := \"Actor enters repair ticket details and priority.\""
            ),
            "Lean slice artifact must carry the updated description"
        );
        assert!(
            lean.contains("def sliceAcceptanceScenarios : List EventModelScenario := [{ name := \"Actor captures ticket\""),
            "Lean slice artifact must preserve authored scenarios after update"
        );
        assert!(
            lean.contains(
                "def sliceBitLevelDataFlows : List BitLevelDataFlow := [{ datum := \"ticket_title\""
            ),
            "Lean slice artifact must preserve authored data flows after update"
        );
        assert!(
            quint.contains(
                "val sliceDescription = \"Actor enters repair ticket details and priority.\""
            ),
            "Quint slice artifact must carry the updated description"
        );
        assert!(
            quint.contains("val sliceAcceptanceScenarios: List[EventModelScenario] = [{ name: \"Actor captures ticket\""),
            "Quint slice artifact must preserve authored scenarios after update"
        );
        assert!(
            quint.contains(
                "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = [{ datum: \"ticket_title\""
            ),
            "Quint slice artifact must preserve authored data flows after update"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_command_definition_updates_formal_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

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
                "--singleton",
                "true",
                "--repeat-behavior",
                "idempotent",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("def sliceCommands : List String := [\"CaptureTicket\"]"),
            "Lean slice artifact must carry the authored command name"
        );
        assert!(
            lean.contains(
                "def sliceCommandDefinitions : List CommandDefinition := [{ name := \"CaptureTicket\", inputs := [{ name := \"ticket_title\", sourceKind := \"actor\", sourceDescription := \"title field on the intake form\", provenanceChain := [\"actor keystrokes -> form field\"] }], emittedEvents := [\"TicketCaptured\"], observedStreams := [], errors := [], singleton := true, repeatBehavior := \"idempotent\" }]"
            ),
            "Lean slice artifact must carry the authored command definition"
        );
        assert!(
            quint.contains("val sliceCommands: List[str] = [\"CaptureTicket\"]"),
            "Quint slice artifact must carry the authored command name"
        );
        assert!(
            quint.contains(
                "val sliceCommandDefinitions: List[CommandDefinition] = [{ name: \"CaptureTicket\", inputs: [{ name: \"ticket_title\", sourceKind: \"actor\", sourceDescription: \"title field on the intake form\", provenanceChain: [\"actor keystrokes -> form field\"] }], emittedEvents: [\"TicketCaptured\"], observedStreams: [], errors: [], singleton: true, repeatBehavior: \"idempotent\" }]"
            ),
            "Quint slice artifact must carry the authored command definition"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_command_definition_with_error_updates_formal_slice_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "state_change",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
                "--read-streams",
                "tickets",
                "--written-streams",
                "tickets",
                "--error-references",
                "DuplicateTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "command_input",
                "--attribute-source-name",
                "ticket_title",
                "--attribute-source-field",
                "value",
                "--attribute-provenance",
                "CaptureTicket.ticket_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "outcome",
                "--slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--events",
                "TicketCaptured",
                "--externally-relevant",
                "false",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "actor input title field",
                "--transformation",
                "captured without normalization",
                "--target",
                "CaptureTicket",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "CaptureTicket.ticket_title",
                "--transformation",
                "copied into TicketCaptured.ticket_title",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains(
                "def sliceCommandDefinitions : List CommandDefinition := [{ name := \"CaptureTicket\", inputs := [{ name := \"ticket_title\", sourceKind := \"actor\", sourceDescription := \"title field on the intake form\", provenanceChain := [\"actor keystrokes -> form field\"] }], emittedEvents := [\"TicketCaptured\"], observedStreams := [], errors := [{ name := \"DuplicateTicket\", scenarioName := \"Duplicate ticket is rejected\", recoveryKind := \"retry\" }], singleton := false, repeatBehavior := \"\" }]"
            ),
            "Lean slice artifact must carry authored command errors and recovery"
        );
        assert!(
            quint.contains(
                "val sliceCommandDefinitions: List[CommandDefinition] = [{ name: \"CaptureTicket\", inputs: [{ name: \"ticket_title\", sourceKind: \"actor\", sourceDescription: \"title field on the intake form\", provenanceChain: [\"actor keystrokes -> form field\"] }], emittedEvents: [\"TicketCaptured\"], observedStreams: [], errors: [{ name: \"DuplicateTicket\", scenarioName: \"Duplicate ticket is rejected\", recoveryKind: \"retry\" }], singleton: false, repeatBehavior: \"\" }]"
            ),
            "Quint slice artifact must carry authored command errors and recovery"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_command_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_command_definition_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_command_definition\""))
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "def sliceCommandDefinitions : List CommandDefinition := [{ name := \"CaptureTicket\""
            ),
            "MCP-authored command definition must be represented in the Lean artifact"
        );
        assert!(
            lean.contains("singleton := true, repeatBehavior := \"idempotent\""),
            "MCP-authored singleton repeat behavior must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_command_errors() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_command_error_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_command_definition\""))
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "errors := [{ name := \"DuplicateTicket\", scenarioName := \"Duplicate ticket is rejected\", recoveryKind := \"retry\" }]"
            ),
            "MCP-authored command errors must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_event_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_event_definition_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_event_definition\""))
            .stdout(predicate::str::contains(
                "added event TicketCaptured to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "def sliceEventDefinitions : List EventDefinition := [{ name := \"TicketCaptured\""
            ),
            "MCP-authored event definition must be represented in the Lean artifact"
        );
        assert!(
            lean.contains("observed := true"),
            "MCP-authored observed event facts must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn add_external_payload_definition_updates_formal_slice_artifacts() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = initialized_project_with_slice()?;

        author_state_change_ticket_capture(&temp_dir)?;
        author_ticket_captured_outcome(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "external-payload",
                "--slice",
                "capture-ticket",
                "--name",
                "intake_webhook",
                "--field",
                "ticket_title",
                "--field-provenance",
                "intake_webhook.ticket_title supplied by the external ticket intake system",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added external payload intake_webhook to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains(
                "def sliceExternalPayloads : List ExternalPayloadDefinition := [{ name := \"intake_webhook\", fields := [{ name := \"ticket_title\", provenanceDescription := \"intake_webhook.ticket_title supplied by the external ticket intake system\", bitEncoding := \"UTF-8 string\" }] }]"
            ),
            "Lean slice artifact must carry authored external payload field provenance and bit encoding"
        );
        assert!(
            quint.contains(
                "val sliceExternalPayloads: List[ExternalPayloadDefinition] = [{ name: \"intake_webhook\", fields: [{ name: \"ticket_title\", provenanceDescription: \"intake_webhook.ticket_title supplied by the external ticket intake system\", bitEncoding: \"UTF-8 string\" }] }]"
            ),
            "Quint slice artifact must carry authored external payload field provenance and bit encoding"
        );

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketImported",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "external_payload",
                "--attribute-source-name",
                "intake_webhook",
                "--attribute-source-field",
                "ticket_title",
                "--attribute-provenance",
                "intake_webhook.ticket_title",
                "--observed",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "intake webhook ticket_title field",
                "--transformation",
                "decoded as external payload field",
                "--target",
                "TicketImported",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_external_payload_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_external_payload_definition_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\"add_external_payload_definition\"",
            ))
            .stdout(predicate::str::contains(
                "added external payload intake_webhook to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "def sliceExternalPayloads : List ExternalPayloadDefinition := [{ name := \"intake_webhook\", fields := [{ name := \"ticket_title\", provenanceDescription := \"intake_webhook.ticket_title supplied by the external ticket intake system\", bitEncoding := \"UTF-8 string\" }] }]"
            ),
            "MCP-authored external payload definition must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn add_outcome_definition_updates_formal_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        author_state_change_ticket_capture(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "outcome",
                "--slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--events",
                "TicketCaptured",
                "--externally-relevant",
                "false",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added outcome ticket_captured to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains(
                "def sliceOutcomeDefinitions : List OutcomeDefinition := [{ label := \"ticket_captured\", eventSet := [\"TicketCaptured\"], externallyRelevant := false }]"
            ),
            "Lean slice artifact must carry authored outcome event sets"
        );
        assert!(
            quint.contains(
                "val sliceOutcomeDefinitions: List[OutcomeDefinition] = [{ label: \"ticket_captured\", eventSet: [\"TicketCaptured\"], externallyRelevant: false }]"
            ),
            "Quint slice artifact must carry authored outcome event sets"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_outcome_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_outcome_definition_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_outcome_definition\""))
            .stdout(predicate::str::contains(
                "added outcome ticket_imported to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "def sliceOutcomeDefinitions : List OutcomeDefinition := [{ label := \"ticket_imported\""
            ),
            "MCP-authored outcome definition must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_read_model_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_read_model_definition_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_read_model_definition\""))
            .stdout(predicate::str::contains(
                "added read model ticket_state to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "def sliceReadModelDefinitions : List ReadModelDefinition := [{ name := \"ticket_state\""
            ),
            "MCP-authored read model definition must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_read_model_derivations() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket state projector covers derived fields",
                "--given",
                "TicketCaptured carries a padded title",
                "--when",
                "ticket_state projects the stream",
                "--then",
                "normalized_title is available",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_state",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket title is normalized",
                "--given",
                "TicketCaptured carries a padded title",
                "--when",
                "ticket_state derives normalized_title",
                "--then",
                "normalized_title is trimmed and case preserved",
                "--contract-kind",
                "derivation",
                "--covered-definition",
                "ticket_state.normalized_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_read_model_derivation_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_read_model_definition\""))
            .stdout(predicate::str::contains(
                "added read model ticket_state to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_summary",
                "--read-model",
                "ticket_state",
                "--field",
                "normalized_title",
                "--source-field",
                "normalized_title",
                "--sketch-token",
                "normalized-title",
                "--field-provenance",
                "ticket_state.normalized_title",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "normalized_title",
                "--source",
                "TicketCaptured.ticket_title",
                "--transformation",
                "trim surrounding whitespace",
                "--target",
                "ticket_state",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "normalized_title",
                "--source",
                "ticket_state.normalized_title",
                "--transformation",
                "displayed without transformation",
                "--target",
                "ticket_summary",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "derivationRule := \"trim surrounding whitespace from TicketCaptured.ticket_title\", absenceEvent := \"\", derivationScenarioName := \"Ticket title is normalized\""
            ),
            "MCP-authored read-model derivation semantics must be represented in the Lean artifact"
        );

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_transitive_read_models() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;
        author_transitive_ticket_hierarchy_context(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_read_model_transitive_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_read_model_definition\""))
            .stdout(predicate::str::contains(
                "added read model ticket_hierarchy to slice capture-ticket",
            ));

        complete_transitive_ticket_hierarchy(&temp_dir)?;

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains("transitive := true, relationshipFields := [\"parent_ticket_id\",\"child_ticket_id\"], transitiveRule := \"walk TicketLinked parent_ticket_id edges until root\", exampleScenarioName := \"Ticket hierarchy includes grandchild\""),
            "MCP-authored transitive read model semantics must be represented in the Lean artifact"
        );

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_event_definition_completes_command_event_data_flow_verification()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "state_change",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "command_input",
                "--attribute-source-name",
                "ticket_title",
                "--attribute-source-field",
                "value",
                "--attribute-provenance",
                "CaptureTicket.ticket_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added event TicketCaptured to slice capture-ticket",
            ));

        author_ticket_captured_outcome(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "title field on the intake form",
                "--transformation",
                "copied without normalization",
                "--target",
                "CaptureTicket",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "CaptureTicket.ticket_title",
                "--transformation",
                "copied into TicketCaptured.ticket_title",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("def sliceEvents : List String := [\"TicketCaptured\"]"),
            "Lean slice artifact must carry the authored event name"
        );
        assert!(
            lean.contains("def sliceStreams : List StreamDefinition := [{ name := \"tickets\" }]"),
            "Lean slice artifact must carry the authored stream"
        );
        assert!(
            lean.contains(
                "def sliceEventDefinitions : List EventDefinition := [{ name := \"TicketCaptured\", stream := \"tickets\", attributes := [{ name := \"ticket_title\", sourceKind := \"command_input\", sourceName := \"ticket_title\", sourceField := \"value\", provenanceDescription := \"CaptureTicket.ticket_title\" }], observed := false, shared := false }]"
            ),
            "Lean slice artifact must carry the authored event definition"
        );
        assert!(
            quint.contains("val sliceEvents: List[str] = [\"TicketCaptured\"]"),
            "Quint slice artifact must carry the authored event name"
        );
        assert!(
            quint.contains("val sliceStreams: List[StreamDefinition] = [{ name: \"tickets\" }]"),
            "Quint slice artifact must carry the authored stream"
        );
        assert!(
            quint.contains(
                "val sliceEventDefinitions: List[EventDefinition] = [{ name: \"TicketCaptured\", stream: \"tickets\", attributes: [{ name: \"ticket_title\", sourceKind: \"command_input\", sourceName: \"ticket_title\", sourceField: \"value\", provenanceDescription: \"CaptureTicket.ticket_title\" }], observed: false, shared: false }]"
            ),
            "Quint slice artifact must carry the authored event definition"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_read_model_definition_completes_projection_data_flow_verification()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket state projects title",
                "--given",
                "TicketCaptured carries ticket_title",
                "--when",
                "ticket_state projects TicketCaptured",
                "--then",
                "ticket_state.ticket_title equals TicketCaptured.ticket_title",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_state",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "upstream_event_store",
                "--attribute-source-field",
                "ticket_title",
                "--attribute-provenance",
                "TicketCaptured.ticket_title",
                "--observed",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
                "ticket_title",
                "--field-provenance",
                "TicketCaptured.ticket_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added read model ticket_state to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_summary",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_title",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "title-label",
                "--field-provenance",
                "ticket_state.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "upstream event store",
                "--transformation",
                "observed without transformation",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "TicketCaptured.ticket_title",
                "--transformation",
                "projected without transformation",
                "--target",
                "ticket_state",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "ticket_state.ticket_title",
                "--transformation",
                "displayed without transformation",
                "--target",
                "ticket_summary",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("def sliceReadModels : List String := [\"ticket_state\"]"),
            "Lean slice artifact must carry the authored read model name"
        );
        assert!(
            lean.contains(
                "def sliceReadModelDefinitions : List ReadModelDefinition := [{ name := \"ticket_state\", fields := [{ name := \"ticket_title\", sourceKind := \"event_attribute\", sourceEvent := \"TicketCaptured\", sourceAttribute := \"ticket_title\", derivationRule := \"\", absenceEvent := \"\", derivationScenarioName := \"\", absenceScenarioName := \"\", provenanceDescription := \"TicketCaptured.ticket_title\" }], transitive := false, relationshipFields := [], transitiveRule := \"\", exampleScenarioName := \"\" }]"
            ),
            "Lean slice artifact must carry the authored read model field source and provenance"
        );
        assert!(
            quint.contains("val sliceReadModels: List[str] = [\"ticket_state\"]"),
            "Quint slice artifact must carry the authored read model name"
        );
        assert!(
            quint.contains(
                "val sliceReadModelDefinitions: List[ReadModelDefinition] = [{ name: \"ticket_state\", fields: [{ name: \"ticket_title\", sourceKind: \"event_attribute\", sourceEvent: \"TicketCaptured\", sourceAttribute: \"ticket_title\", derivationRule: \"\", absenceEvent: \"\", derivationScenarioName: \"\", absenceScenarioName: \"\", provenanceDescription: \"TicketCaptured.ticket_title\" }], transitive: false, relationshipFields: [], transitiveRule: \"\", exampleScenarioName: \"\" }]"
            ),
            "Quint slice artifact must carry the authored read model field source and provenance"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_read_model_definition_records_derivation_and_absence_semantics()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket state projector covers derived and absent fields",
                "--given",
                "TicketCaptured may or may not exist",
                "--when",
                "ticket_state projects the stream",
                "--then",
                "derived and absence fields are populated",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_state",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket state defaults projector covers absence",
                "--given",
                "TicketCaptured is absent from the stream",
                "--when",
                "ticket_state_defaults projects the empty stream",
                "--then",
                "has_ticket defaults to false",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_state_defaults",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket title is normalized",
                "--given",
                "TicketCaptured carries a padded title",
                "--when",
                "ticket_state derives normalized_title",
                "--then",
                "normalized_title is trimmed and case preserved",
                "--contract-kind",
                "derivation",
                "--covered-definition",
                "ticket_state.normalized_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket state defaults before capture",
                "--given",
                "TicketCaptured is absent from the stream",
                "--when",
                "ticket_state projects the empty stream",
                "--then",
                "has_ticket defaults to false",
                "--contract-kind",
                "absence",
                "--covered-definition",
                "ticket_state.has_ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "read-model",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_state",
                "--field",
                "normalized_title",
                "--field-source",
                "derivation",
                "--derivation-rule",
                "trim surrounding whitespace from TicketCaptured.ticket_title",
                "--derivation-scenario",
                "Ticket title is normalized",
                "--field-provenance",
                "TicketCaptured.ticket_title -> trim",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added read model ticket_state to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "read-model",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_state_defaults",
                "--field",
                "has_ticket",
                "--field-source",
                "absence_default",
                "--absence-event",
                "TicketCaptured",
                "--absence-scenario",
                "Ticket state defaults before capture",
                "--field-provenance",
                "absence of TicketCaptured in tickets stream",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_summary",
                "--read-model",
                "ticket_state",
                "--field",
                "normalized_title",
                "--source-field",
                "normalized_title",
                "--sketch-token",
                "normalized-title",
                "--field-provenance",
                "ticket_state.normalized_title",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "normalized_title",
                "--source",
                "TicketCaptured.ticket_title",
                "--transformation",
                "trim surrounding whitespace",
                "--target",
                "ticket_state",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "has_ticket",
                "--source",
                "absence of TicketCaptured",
                "--transformation",
                "default false when no TicketCaptured fact exists",
                "--target",
                "ticket_state_defaults",
                "--bit-encoding",
                "boolean",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "normalized_title",
                "--source",
                "ticket_state.normalized_title",
                "--transformation",
                "displayed without transformation",
                "--target",
                "ticket_summary",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains(
                "{ name := \"normalized_title\", sourceKind := \"derivation\", sourceEvent := \"\", sourceAttribute := \"\", derivationRule := \"trim surrounding whitespace from TicketCaptured.ticket_title\", absenceEvent := \"\", derivationScenarioName := \"Ticket title is normalized\", absenceScenarioName := \"\", provenanceDescription := \"TicketCaptured.ticket_title -> trim\" }"
            ),
            "Lean slice artifact must carry read-model derivation semantics"
        );
        assert!(
            lean.contains(
                "{ name := \"has_ticket\", sourceKind := \"absence_default\", sourceEvent := \"\", sourceAttribute := \"\", derivationRule := \"\", absenceEvent := \"TicketCaptured\", derivationScenarioName := \"\", absenceScenarioName := \"Ticket state defaults before capture\", provenanceDescription := \"absence of TicketCaptured in tickets stream\" }"
            ),
            "Lean slice artifact must carry read-model absence/default semantics"
        );
        assert!(
            quint.contains(
                "{ name: \"normalized_title\", sourceKind: \"derivation\", sourceEvent: \"\", sourceAttribute: \"\", derivationRule: \"trim surrounding whitespace from TicketCaptured.ticket_title\", absenceEvent: \"\", derivationScenarioName: \"Ticket title is normalized\", absenceScenarioName: \"\", provenanceDescription: \"TicketCaptured.ticket_title -> trim\" }"
            ),
            "Quint slice artifact must carry read-model derivation semantics"
        );
        assert!(
            quint.contains(
                "{ name: \"has_ticket\", sourceKind: \"absence_default\", sourceEvent: \"\", sourceAttribute: \"\", derivationRule: \"\", absenceEvent: \"TicketCaptured\", derivationScenarioName: \"\", absenceScenarioName: \"Ticket state defaults before capture\", provenanceDescription: \"absence of TicketCaptured in tickets stream\" }"
            ),
            "Quint slice artifact must carry read-model absence/default semantics"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_read_model_definition_records_transitive_semantics() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;
        author_transitive_ticket_hierarchy_context(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "read-model",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_hierarchy",
                "--field",
                "ancestor_ticket_id",
                "--field-source",
                "event_attribute",
                "--source-event",
                "TicketLinked",
                "--source-attribute",
                "parent_ticket_id",
                "--field-provenance",
                "TicketLinked.parent_ticket_id",
                "--transitive",
                "true",
                "--relationship-fields",
                "parent_ticket_id,child_ticket_id",
                "--transitive-rule",
                "walk TicketLinked parent_ticket_id edges until root",
                "--example-scenario",
                "Ticket hierarchy includes grandchild",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added read model ticket_hierarchy to slice capture-ticket",
            ));

        complete_transitive_ticket_hierarchy(&temp_dir)?;

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("def sliceReadModelDefinitions : List ReadModelDefinition := [{ name := \"ticket_hierarchy\", fields := [{ name := \"ancestor_ticket_id\", sourceKind := \"event_attribute\", sourceEvent := \"TicketLinked\", sourceAttribute := \"parent_ticket_id\", derivationRule := \"\", absenceEvent := \"\", derivationScenarioName := \"\", absenceScenarioName := \"\", provenanceDescription := \"TicketLinked.parent_ticket_id\" }], transitive := true, relationshipFields := [\"parent_ticket_id\",\"child_ticket_id\"], transitiveRule := \"walk TicketLinked parent_ticket_id edges until root\", exampleScenarioName := \"Ticket hierarchy includes grandchild\" }]"),
            "Lean slice artifact must carry transitive read-model semantics"
        );
        assert!(
            quint.contains("val sliceReadModelDefinitions: List[ReadModelDefinition] = [{ name: \"ticket_hierarchy\", fields: [{ name: \"ancestor_ticket_id\", sourceKind: \"event_attribute\", sourceEvent: \"TicketLinked\", sourceAttribute: \"parent_ticket_id\", derivationRule: \"\", absenceEvent: \"\", derivationScenarioName: \"\", absenceScenarioName: \"\", provenanceDescription: \"TicketLinked.parent_ticket_id\" }], transitive: true, relationshipFields: [\"parent_ticket_id\",\"child_ticket_id\"], transitiveRule: \"walk TicketLinked parent_ticket_id edges until root\", exampleScenarioName: \"Ticket hierarchy includes grandchild\" }]"),
            "Quint slice artifact must carry transitive read-model semantics"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_view_definition_completes_displayed_datum_verification() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket state projects title",
                "--given",
                "TicketCaptured carries ticket_title",
                "--when",
                "ticket_state projects TicketCaptured",
                "--then",
                "ticket_state.ticket_title equals TicketCaptured.ticket_title",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_state",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "upstream_event_store",
                "--attribute-source-field",
                "ticket_title",
                "--attribute-provenance",
                "TicketCaptured.ticket_title",
                "--observed",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
                "ticket_title",
                "--field-provenance",
                "TicketCaptured.ticket_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "upstream event store",
                "--transformation",
                "observed without transformation",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_summary",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_title",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "title-label",
                "--field-provenance",
                "ticket_state.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added view ticket_summary to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "TicketCaptured.ticket_title",
                "--transformation",
                "projected without transformation",
                "--target",
                "ticket_state",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "ticket_state.ticket_title",
                "--transformation",
                "displayed without transformation",
                "--target",
                "ticket_summary",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("def sliceViews : List String := [\"ticket_summary\"]"),
            "Lean slice artifact must carry the authored view name"
        );
        assert!(
            lean.contains(
                "def sliceViewDefinitions : List ViewDefinition := [{ name := \"ticket_summary\", readModels := [\"ticket_state\"], fields := [{ name := \"ticket_title\", sourceKind := \"read_model\", sourceReadModel := \"ticket_state\", sourceField := \"ticket_title\", sketchToken := \"title-label\", provenanceDescription := \"ticket_state.ticket_title\", bitEncoding := \"UTF-8 string\" }], controls := [], sketchTokens := [\"title-label\"], localStates := [], filters := [] }]"
            ),
            "Lean slice artifact must carry the authored displayed datum source and sketch token"
        );
        assert!(
            quint.contains("val sliceViews: List[str] = [\"ticket_summary\"]"),
            "Quint slice artifact must carry the authored view name"
        );
        assert!(
            quint.contains(
                "val sliceViewDefinitions: List[ViewDefinition] = [{ name: \"ticket_summary\", readModels: [\"ticket_state\"], fields: [{ name: \"ticket_title\", sourceKind: \"read_model\", sourceReadModel: \"ticket_state\", sourceField: \"ticket_title\", sketchToken: \"title-label\", provenanceDescription: \"ticket_state.ticket_title\", bitEncoding: \"UTF-8 string\" }], controls: [], sketchTokens: [\"title-label\"], localStates: [], filters: [] }]"
            ),
            "Quint slice artifact must carry the authored displayed datum source and sketch token"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_view_definition_with_control_updates_formal_slice_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;
        author_projected_ticket_title(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_summary",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_title",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "title-label",
                "--field-provenance",
                "ticket_state.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
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
                "ticket_summary",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added view ticket_summary to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "ticket_state.ticket_title",
                "--transformation",
                "displayed without transformation",
                "--target",
                "ticket_summary",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("def sliceReferencedCommands : List String := [\"CaptureTicket\"]"),
            "Lean slice artifact must carry cross-slice command references for controls"
        );
        assert!(
            lean.contains(
                "controls := [{ name := \"submit-ticket\", commandName := \"CaptureTicket\", inputs := [{ name := \"ticket_title\", sourceKind := \"actor\", sourceDescription := \"title field on the intake form\", sketchToken := \"title-input\", visibleToActor := true, decisionField := true }], handledErrors := [\"DuplicateTicket\"], recoveryBehavior := \"retry\", sketchToken := \"submit-button\", navigation := { targetType := \"modeled_view\", targetName := \"ticket_summary\", externalWorkflowName := \"\", externalSystemName := \"\", handoffContract := \"\" } }]"
            ),
            "Lean slice artifact must carry authored control input, error handling, and navigation"
        );
        assert!(
            lean.contains("sketchTokens := [\"title-label\",\"submit-button\",\"title-input\"]"),
            "Lean slice artifact must map field, control, and actor input sketch tokens"
        );
        assert!(
            quint.contains("val sliceReferencedCommands: List[str] = [\"CaptureTicket\"]"),
            "Quint slice artifact must carry cross-slice command references for controls"
        );
        assert!(
            quint.contains(
                "controls: [{ name: \"submit-ticket\", commandName: \"CaptureTicket\", inputs: [{ name: \"ticket_title\", sourceKind: \"actor\", sourceDescription: \"title field on the intake form\", sketchToken: \"title-input\", visibleToActor: true, decisionField: true }], handledErrors: [\"DuplicateTicket\"], recoveryBehavior: \"retry\", sketchToken: \"submit-button\", navigation: { targetType: \"modeled_view\", targetName: \"ticket_summary\", externalWorkflowName: \"\", externalSystemName: \"\", handoffContract: \"\" } }]"
            ),
            "Quint slice artifact must carry authored control input, error handling, and navigation"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_view_definition_records_external_system_navigation_contract()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;
        author_projected_ticket_title(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_summary",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_title",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "title-label",
                "--field-provenance",
                "ticket_state.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
                "--control",
                "open-vendor-portal",
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
                "explicit_recovery_action",
                "--control-sketch-token",
                "vendor-link",
                "--navigation-type",
                "external_system",
                "--navigation-target",
                "vendor_portal",
                "--external-system",
                "Vendor Portal",
                "--handoff-contract",
                "ticket_export_payload",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added view ticket_summary to slice capture-ticket",
            ));

        complete_ticket_summary_display_flow(&temp_dir)?;

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("navigation := { targetType := \"external_system\", targetName := \"vendor_portal\", externalWorkflowName := \"\", externalSystemName := \"Vendor Portal\", handoffContract := \"ticket_export_payload\" }"),
            "Lean slice artifact must carry external-system navigation target and handoff contract"
        );
        assert!(
            quint.contains("navigation: { targetType: \"external_system\", targetName: \"vendor_portal\", externalWorkflowName: \"\", externalSystemName: \"Vendor Portal\", handoffContract: \"ticket_export_payload\" }"),
            "Quint slice artifact must carry external-system navigation target and handoff contract"
        );

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_view_controls() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;
        author_projected_ticket_title(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_view_control_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_view_definition\""))
            .stdout(predicate::str::contains(
                "added view ticket_summary to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "controls := [{ name := \"submit-ticket\", commandName := \"CaptureTicket\""
            ),
            "MCP-authored controls must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_external_system_navigation_contract() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;
        author_projected_ticket_title(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_external_system_view_control_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_view_definition\""))
            .stdout(predicate::str::contains(
                "added view ticket_summary to slice capture-ticket",
            ));

        complete_ticket_summary_display_flow(&temp_dir)?;

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains("externalSystemName := \"Vendor Portal\", handoffContract := \"ticket_export_payload\""),
            "MCP-authored external-system navigation metadata must be represented in the Lean artifact"
        );

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_automation_definition_updates_formal_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "automation",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "automation",
                "--slice",
                "capture-ticket",
                "--name",
                "title-deduplicator",
                "--trigger",
                "TicketCaptured",
                "--command",
                "CaptureTicket",
                "--handled-errors",
                "DuplicateTicket",
                "--reaction",
                "deduplicates captured titles by reissuing CaptureTicket when needed",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added automation title-deduplicator to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("def sliceReferencedCommands : List String := [\"CaptureTicket\"]"),
            "Lean slice artifact must record commands issued by authored automations"
        );
        assert!(
            lean.contains(
                "def sliceAutomations : List AutomationDefinition := [{ name := \"title-deduplicator\", triggerName := \"TicketCaptured\", commandName := \"CaptureTicket\", handledErrors := [\"DuplicateTicket\"], reactionDescription := \"deduplicates captured titles by reissuing CaptureTicket when needed\" }]"
            ),
            "Lean slice artifact must carry authored automation definitions"
        );
        assert!(
            quint.contains("val sliceReferencedCommands: List[str] = [\"CaptureTicket\"]"),
            "Quint slice artifact must record commands issued by authored automations"
        );
        assert!(
            quint.contains(
                "val sliceAutomations: List[AutomationDefinition] = [{ name: \"title-deduplicator\", triggerName: \"TicketCaptured\", commandName: \"CaptureTicket\", handledErrors: [\"DuplicateTicket\"], reactionDescription: \"deduplicates captured titles by reissuing CaptureTicket when needed\" }]"
            ),
            "Quint slice artifact must carry authored automation definitions"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_automation_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "automation",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_automation_definition_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_automation_definition\""))
            .stdout(predicate::str::contains(
                "added automation title-deduplicator to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "def sliceAutomations : List AutomationDefinition := [{ name := \"title-deduplicator\""
            ),
            "MCP-authored automation must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn add_translation_definition_updates_formal_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "translation",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "external-payload",
                "--slice",
                "capture-ticket",
                "--name",
                "intake_webhook",
                "--field",
                "ticket_title",
                "--field-provenance",
                "intake_webhook.ticket_title supplied by the external ticket intake system",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "intake_webhook_received",
                "--stream",
                "intake-webhooks",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "external_payload",
                "--attribute-source-name",
                "intake_webhook",
                "--attribute-source-field",
                "ticket_title",
                "--attribute-provenance",
                "intake_webhook.ticket_title",
                "--observed",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "intake_webhook.ticket_title",
                "--transformation",
                "observed without transformation",
                "--target",
                "intake_webhook_received",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "translation",
                "--slice",
                "capture-ticket",
                "--name",
                "intake-webhook-translator",
                "--external-event",
                "intake_webhook_received",
                "--payload-contract",
                "intake_webhook",
                "--command",
                "CaptureTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added translation intake-webhook-translator to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains("def sliceReferencedCommands : List String := [\"CaptureTicket\"]"),
            "Lean slice artifact must record commands targeted by authored translations"
        );
        assert!(
            lean.contains(
                "def sliceTranslations : List TranslationDefinition := [{ name := \"intake-webhook-translator\", externalEventName := \"intake_webhook_received\", payloadContractName := \"intake_webhook\", commandName := \"CaptureTicket\" }]"
            ),
            "Lean slice artifact must carry authored translation definitions"
        );
        assert!(
            quint.contains("val sliceReferencedCommands: List[str] = [\"CaptureTicket\"]"),
            "Quint slice artifact must record commands targeted by authored translations"
        );
        assert!(
            quint.contains(
                "val sliceTranslations: List[TranslationDefinition] = [{ name: \"intake-webhook-translator\", externalEventName: \"intake_webhook_received\", payloadContractName: \"intake_webhook\", commandName: \"CaptureTicket\" }]"
            ),
            "Quint slice artifact must carry authored translation definitions"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_translation_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "translation",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "external-payload",
                "--slice",
                "capture-ticket",
                "--name",
                "intake_webhook",
                "--field",
                "ticket_title",
                "--field-provenance",
                "intake_webhook.ticket_title supplied by the external ticket intake system",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_translation_definition_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_translation_definition\""))
            .stdout(predicate::str::contains(
                "added translation intake-webhook-translator to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "def sliceTranslations : List TranslationDefinition := [{ name := \"intake-webhook-translator\""
            ),
            "MCP-authored translation must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn add_board_facts_updates_formal_slice_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        author_state_change_ticket_capture(&temp_dir)?;
        author_ticket_captured_outcome(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "board-element",
                "--slice",
                "capture-ticket",
                "--name",
                "CaptureTicket",
                "--kind",
                "command",
                "--lane",
                "actions",
                "--declared-name",
                "CaptureTicket",
                "--main-path",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added board element CaptureTicket to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "board-element",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--kind",
                "event",
                "--lane",
                "events",
                "--declared-name",
                "TicketCaptured",
                "--main-path",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "board-connection",
                "--slice",
                "capture-ticket",
                "--source",
                "actor-submit",
                "--source-kind",
                "workflow_trigger",
                "--target",
                "CaptureTicket",
                "--target-kind",
                "command",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added board connection actor-submit -> CaptureTicket to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "board-connection",
                "--slice",
                "capture-ticket",
                "--source",
                "CaptureTicket",
                "--source-kind",
                "command",
                "--target",
                "TicketCaptured",
                "--target-kind",
                "event",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            lean.contains(
                "def sliceBoardElements : List BoardElement := [{ name := \"CaptureTicket\", kind := \"command\", lane := \"actions\", declaredName := \"CaptureTicket\", mainPath := true },{ name := \"TicketCaptured\", kind := \"event\", lane := \"events\", declaredName := \"TicketCaptured\", mainPath := true }]"
            ),
            "Lean slice artifact must carry authored board elements"
        );
        assert!(
            lean.contains(
                "def sliceBoardConnections : List BoardConnection := [{ source := \"actor-submit\", sourceKind := \"workflow_trigger\", target := \"CaptureTicket\", targetKind := \"command\" },{ source := \"CaptureTicket\", sourceKind := \"command\", target := \"TicketCaptured\", targetKind := \"event\" }]"
            ),
            "Lean slice artifact must carry authored board connections"
        );
        assert!(
            quint.contains(
                "val sliceBoardElements: List[BoardElement] = [{ name: \"CaptureTicket\", kind: \"command\", lane: \"actions\", declaredName: \"CaptureTicket\", mainPath: true },{ name: \"TicketCaptured\", kind: \"event\", lane: \"events\", declaredName: \"TicketCaptured\", mainPath: true }]"
            ),
            "Quint slice artifact must carry authored board elements"
        );
        assert!(
            quint.contains(
                "val sliceBoardConnections: List[BoardConnection] = [{ source: \"actor-submit\", sourceKind: \"workflow_trigger\", target: \"CaptureTicket\", targetKind: \"command\" },{ source: \"CaptureTicket\", sourceKind: \"command\", target: \"TicketCaptured\", targetKind: \"event\" }]"
            ),
            "Quint slice artifact must carry authored board connections"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_formal_board_facts() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        author_state_change_ticket_capture(&temp_dir)?;
        author_ticket_captured_outcome(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_board_fact_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_board_element\""))
            .stdout(predicate::str::contains("\"add_board_connection\""))
            .stdout(predicate::str::contains(
                "added board element CaptureTicket to slice capture-ticket",
            ))
            .stdout(predicate::str::contains(
                "added board connection actor-submit -> CaptureTicket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        assert!(
            lean.contains(
                "def sliceBoardElements : List BoardElement := [{ name := \"CaptureTicket\""
            ),
            "MCP-authored board elements must be represented in the Lean artifact"
        );
        assert!(
            lean.contains(
                "def sliceBoardConnections : List BoardConnection := [{ source := \"actor-submit\""
            ),
            "MCP-authored board connections must be represented in the Lean artifact"
        );

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn initialized_project_with_slice() -> Result<TempDir, Box<dyn Error>> {
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

        Ok(temp_dir)
    }

    fn author_ticket_captured_outcome(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "outcome",
                "--slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--events",
                "TicketCaptured",
                "--externally-relevant",
                "false",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn author_state_change_ticket_capture(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--type",
                "state_change",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "actor_input",
                "--attribute-source-field",
                "ticket_title",
                "--attribute-provenance",
                "CaptureTicket.ticket_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "actor input title field",
                "--transformation",
                "captured without normalization",
                "--target",
                "CaptureTicket",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "CaptureTicket.ticket_title",
                "--transformation",
                "copied into TicketCaptured.ticket_title",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn author_projected_ticket_title(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket state projects title",
                "--given",
                "TicketCaptured carries ticket_title",
                "--when",
                "ticket_state projects TicketCaptured",
                "--then",
                "ticket_state.ticket_title equals TicketCaptured.ticket_title",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_state",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "ticket_title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "upstream_event_store",
                "--attribute-source-field",
                "ticket_title",
                "--attribute-provenance",
                "TicketCaptured.ticket_title",
                "--observed",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
                "ticket_title",
                "--field-provenance",
                "TicketCaptured.ticket_title",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "upstream event store",
                "--transformation",
                "observed without transformation",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "TicketCaptured.ticket_title",
                "--transformation",
                "projected without transformation",
                "--target",
                "ticket_state",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn author_transitive_ticket_hierarchy_context(
        temp_dir: &TempDir,
    ) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket hierarchy projector covers ancestors",
                "--given",
                "TicketLinked carries parent and child identifiers",
                "--when",
                "ticket_hierarchy projects the link chain",
                "--then",
                "ancestor_ticket_id is available",
                "--contract-kind",
                "projector",
                "--covered-definition",
                "ticket_hierarchy",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "contract",
                "--name",
                "Ticket hierarchy includes grandchild",
                "--given",
                "TicketLinked contains parent and child edges",
                "--when",
                "ticket_hierarchy follows the relationship fields",
                "--then",
                "grandchild tickets include the root ancestor",
                "--contract-kind",
                "transitive",
                "--covered-definition",
                "ticket_hierarchy",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketLinked",
                "--stream",
                "ticket-links",
                "--attribute",
                "parent_ticket_id",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "upstream_event_store",
                "--attribute-source-field",
                "parent_ticket_id",
                "--attribute-provenance",
                "TicketLinked.parent_ticket_id",
                "--observed",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn complete_transitive_ticket_hierarchy(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "hierarchy_summary",
                "--read-model",
                "ticket_hierarchy",
                "--field",
                "ancestor_ticket_id",
                "--source-field",
                "ancestor_ticket_id",
                "--sketch-token",
                "ancestor-id",
                "--field-provenance",
                "ticket_hierarchy.ancestor_ticket_id",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "parent_ticket_id",
                "--source",
                "upstream event store",
                "--transformation",
                "observed as TicketLinked.parent_ticket_id",
                "--target",
                "TicketLinked",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ancestor_ticket_id",
                "--source",
                "TicketLinked.parent_ticket_id",
                "--transformation",
                "transitive closure over parent_ticket_id and child_ticket_id",
                "--target",
                "ticket_hierarchy",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ancestor_ticket_id",
                "--source",
                "ticket_hierarchy.ancestor_ticket_id",
                "--transformation",
                "displayed without transformation",
                "--target",
                "hierarchy_summary",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn complete_ticket_summary_display_flow(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "ticket_state.ticket_title",
                "--transformation",
                "displayed without transformation",
                "--target",
                "ticket_summary",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn mcp_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_slice_scenario\",\"arguments\":{\"slice\":\"capture-ticket\",\"kind\":\"acceptance\",\"name\":\"Actor captures ticket\",\"given\":\"ticket intake screen is open\",\"when\":\"the actor submits ticket details\",\"then\":\"the ticket details are visible for review\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_title\",\"source\":\"actor input title field\",\"transformation\":\"copied without normalization\",\"target\":\"Capture ticket.ticket_title\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
        )
    }

    fn mcp_scenario_stream_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_slice_scenario\",\"arguments\":{\"slice\":\"capture-ticket\",\"kind\":\"acceptance\",\"name\":\"Actor captures ticket\",\"given\":\"tickets stream is available\",\"when\":\"the actor submits ticket details\",\"then\":\"TicketCaptured is written\",\"read_streams\":\"tickets\",\"written_streams\":\"tickets\"}}}\n",
        )
    }

    fn mcp_command_definition_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_command_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\",\"input\":\"ticket_title\",\"input_source\":\"actor\",\"input_description\":\"title field on the intake form\",\"input_provenance\":\"actor keystrokes -> form field\",\"emits\":\"TicketCaptured\",\"singleton\":true,\"repeat_behavior\":\"idempotent\"}}}\n",
        )
    }

    fn mcp_command_error_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_slice_scenario\",\"arguments\":{\"slice\":\"capture-ticket\",\"kind\":\"contract\",\"name\":\"Duplicate ticket is rejected\",\"given\":\"tickets stream already contains duplicate title\",\"when\":\"CaptureTicket handles the duplicate title\",\"then\":\"DuplicateTicket is returned\",\"contract_kind\":\"command\",\"covered_definition\":\"CaptureTicket\",\"error_references\":\"DuplicateTicket\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_command_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\",\"input\":\"ticket_title\",\"input_source\":\"actor\",\"input_description\":\"title field on the intake form\",\"input_provenance\":\"actor keystrokes -> form field\",\"emits\":\"TicketCaptured\",\"error\":\"DuplicateTicket\",\"error_scenario\":\"Duplicate ticket is rejected\",\"error_recovery\":\"retry\"}}}\n",
        )
    }

    fn mcp_event_definition_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\",\"stream\":\"tickets\",\"attribute\":\"ticket_title\",\"attribute_source\":\"generated\",\"attribute_source_name\":\"upstream_event_store\",\"attribute_source_field\":\"ticket_title\",\"attribute_provenance\":\"TicketCaptured.ticket_title\",\"observed\":true}}}\n",
        )
    }

    fn mcp_external_payload_definition_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_external_payload_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"intake_webhook\",\"field\":\"ticket_title\",\"field_provenance\":\"intake_webhook.ticket_title supplied by the external ticket intake system\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
        )
    }

    fn mcp_outcome_definition_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_outcome_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"label\":\"ticket_imported\",\"events\":\"TicketImported\",\"externally_relevant\":false}}}\n",
        )
    }

    fn mcp_view_control_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_view_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_summary\",\"read_model\":\"ticket_state\",\"field\":\"ticket_title\",\"source_field\":\"ticket_title\",\"sketch_token\":\"title-label\",\"field_provenance\":\"ticket_state.ticket_title\",\"bit_encoding\":\"UTF-8 string\",\"control\":\"submit-ticket\",\"control_command\":\"CaptureTicket\",\"control_input\":\"ticket_title\",\"control_input_source\":\"actor\",\"control_input_description\":\"title field on the intake form\",\"control_input_sketch_token\":\"title-input\",\"control_input_visible\":true,\"control_input_decision\":true,\"handled_errors\":\"DuplicateTicket\",\"recovery_behavior\":\"retry\",\"control_sketch_token\":\"submit-button\",\"navigation_type\":\"modeled_view\",\"navigation_target\":\"ticket_summary\"}}}\n",
        )
    }

    fn mcp_external_system_view_control_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_view_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_summary\",\"read_model\":\"ticket_state\",\"field\":\"ticket_title\",\"source_field\":\"ticket_title\",\"sketch_token\":\"title-label\",\"field_provenance\":\"ticket_state.ticket_title\",\"bit_encoding\":\"UTF-8 string\",\"control\":\"open-vendor-portal\",\"control_command\":\"CaptureTicket\",\"control_input\":\"ticket_title\",\"control_input_source\":\"actor\",\"control_input_description\":\"title field on the intake form\",\"control_input_sketch_token\":\"title-input\",\"control_input_visible\":true,\"control_input_decision\":true,\"handled_errors\":\"DuplicateTicket\",\"recovery_behavior\":\"explicit_recovery_action\",\"control_sketch_token\":\"vendor-link\",\"navigation_type\":\"external_system\",\"navigation_target\":\"vendor_portal\",\"external_system\":\"Vendor Portal\",\"handoff_contract\":\"ticket_export_payload\"}}}\n",
        )
    }

    fn mcp_automation_definition_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_automation_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"title-deduplicator\",\"trigger\":\"TicketCaptured\",\"command\":\"CaptureTicket\",\"handled_errors\":\"DuplicateTicket\",\"reaction\":\"deduplicates captured titles by reissuing CaptureTicket when needed\"}}}\n",
        )
    }

    fn mcp_translation_definition_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"intake_webhook_received\",\"stream\":\"intake-webhooks\",\"attribute\":\"ticket_title\",\"attribute_source\":\"external_payload\",\"attribute_source_name\":\"intake_webhook\",\"attribute_source_field\":\"ticket_title\",\"attribute_provenance\":\"intake_webhook.ticket_title\",\"observed\":true}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_title\",\"source\":\"intake_webhook.ticket_title\",\"transformation\":\"observed without transformation\",\"target\":\"intake_webhook_received\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"add_translation_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"intake-webhook-translator\",\"external_event\":\"intake_webhook_received\",\"payload_contract\":\"intake_webhook\",\"command\":\"CaptureTicket\"}}}\n",
        )
    }

    fn mcp_board_fact_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_board_element\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\",\"kind\":\"command\",\"lane\":\"actions\",\"declared_name\":\"CaptureTicket\",\"main_path\":true}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_board_element\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\",\"kind\":\"event\",\"lane\":\"events\",\"declared_name\":\"TicketCaptured\",\"main_path\":true}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"add_board_connection\",\"arguments\":{\"slice\":\"capture-ticket\",\"source\":\"actor-submit\",\"source_kind\":\"workflow_trigger\",\"target\":\"CaptureTicket\",\"target_kind\":\"command\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"tools/call\",\"params\":{\"name\":\"add_board_connection\",\"arguments\":{\"slice\":\"capture-ticket\",\"source\":\"CaptureTicket\",\"source_kind\":\"command\",\"target\":\"TicketCaptured\",\"target_kind\":\"event\"}}}\n",
        )
    }

    fn mcp_read_model_definition_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_read_model_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_state\",\"field\":\"ticket_title\",\"field_source\":\"event_attribute\",\"source_event\":\"TicketCaptured\",\"source_attribute\":\"ticket_title\",\"field_provenance\":\"TicketCaptured.ticket_title\"}}}\n",
        )
    }

    fn mcp_read_model_derivation_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_read_model_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_state\",\"field\":\"normalized_title\",\"field_source\":\"derivation\",\"derivation_rule\":\"trim surrounding whitespace from TicketCaptured.ticket_title\",\"derivation_scenario\":\"Ticket title is normalized\",\"field_provenance\":\"TicketCaptured.ticket_title -> trim\"}}}\n",
        )
    }

    fn mcp_read_model_transitive_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_read_model_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_hierarchy\",\"field\":\"ancestor_ticket_id\",\"field_source\":\"event_attribute\",\"source_event\":\"TicketLinked\",\"source_attribute\":\"parent_ticket_id\",\"field_provenance\":\"TicketLinked.parent_ticket_id\",\"transitive\":true,\"relationship_fields\":\"parent_ticket_id,child_ticket_id\",\"transitive_rule\":\"walk TicketLinked parent_ticket_id edges until root\",\"example_scenario\":\"Ticket hierarchy includes grandchild\"}}}\n",
        )
    }
}
