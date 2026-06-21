// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::process::Command as ProcessCommand;

    use assert_cmd::Command;
    use assert_cmd::cargo::cargo_bin;
    use predicates::str::contains;
    use tempfile::TempDir;

    #[test]
    fn generated_model_digests_are_canonical_hashes_when_fields_contain_delimiters()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_project_with_workflow_and_slice(&temp_dir)?;

        let lean = fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint = fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;
        let lean_digest = generated_model_digest(&lean, "def modelDigest := ")?;
        let quint_digest = generated_model_digest(&quint, "val modelDigest = ")?;

        assert!(
            is_lowercase_sha256_hex(&lean_digest),
            "generated Lean digest must be a stable canonical-content hash"
        );
        assert!(
            is_lowercase_sha256_hex(&quint_digest),
            "generated Quint digest must be a stable canonical-content hash"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_scenarios_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceAcceptanceScenarios : List EventModelScenario := [{ name := \"Actor captures ticket\", givenSteps := [\"ticket intake screen is open\"], whenSteps := [\"the actor submits ticket details\"], thenSteps := [\"the ticket details are visible for review\"], readStreams := [], writtenStreams := [], contractKind := \"\", coveredDefinition := \"\", errorReferences := [] }]"
                ),
            "slice scenario must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceAcceptanceScenarios: List[EventModelScenario] = [{ name: \"Actor captures ticket\", givenSteps: [\"ticket intake screen is open\"], whenSteps: [\"the actor submits ticket details\"], thenSteps: [\"the ticket details are visible for review\"], readStreams: [], writtenStreams: [], contractKind: \"\", coveredDefinition: \"\", errorReferences: [] }]"
                ),
            "slice scenario must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_outcomes_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceOutcomeDefinitions : List OutcomeDefinition := [{ label := \"ticket-captured\", eventSet := [\"TicketCaptured\"], externallyRelevant := true }]"
                ),
            "slice outcome must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceOutcomeDefinitions: List[OutcomeDefinition] = [{ label: \"ticket-captured\", eventSet: [\"TicketCaptured\"], externallyRelevant: true }]"
                ),
            "slice outcome must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_external_payloads_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceExternalPayloads : List ExternalPayloadDefinition := [{ name := \"intake_webhook\", fields := [{ name := \"ticket_title\", provenanceDescription := \"intake_webhook.ticket_title supplied by the external ticket intake system\", bitEncoding := \"UTF-8 string\" }] }]"
                ),
            "external payload must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceExternalPayloads: List[ExternalPayloadDefinition] = [{ name: \"intake_webhook\", fields: [{ name: \"ticket_title\", provenanceDescription: \"intake_webhook.ticket_title supplied by the external ticket intake system\", bitEncoding: \"UTF-8 string\" }] }]"
                ),
            "external payload must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_event_definitions_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
                "ticket_feed_snapshot",
                "--attribute-source-field",
                "title",
                "--generated-source-kind",
                "ticket_feed_snapshot",
                "--attribute-provenance",
                "ticket feed snapshot title",
                "--observed",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceEventDefinitions : List EventDefinition := [{ name := \"TicketCaptured\", stream := \"tickets\", attributes := [{ name := \"ticket_title\", sourceKind := \"generated\", sourceName := \"ticket_feed_snapshot\", sourceField := \"title\", generatedSourceKind := \"ticket_feed_snapshot\", provenanceDescription := \"ticket feed snapshot title\" }], observed := true, shared := false }]"
                ),
            "event definition must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceEventDefinitions: List[EventDefinition] = [{ name: \"TicketCaptured\", stream: \"tickets\", attributes: [{ name: \"ticket_title\", sourceKind: \"generated\", sourceName: \"ticket_feed_snapshot\", sourceField: \"title\", generatedSourceKind: \"ticket_feed_snapshot\", provenanceDescription: \"ticket feed snapshot title\" }], observed: true, shared: false }]"
                ),
            "event definition must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_command_definitions_from_exported_events() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceCommandDefinitions : List CommandDefinition := [{ name := \"CaptureTicket\", inputs := [{ name := \"ticket_title\", sourceKind := CommandInputSourceKind.actor, sourceDescription := \"title field on the intake form\", provenanceChain := [\"actor keystrokes -> form field\"], eventStreamSourceEvent := \"\", eventStreamSourceAttribute := \"\", externalPayloadSourceName := \"\", externalPayloadSourceField := \"\", generatedSourceName := \"\", generatedSourceField := \"\", sessionSourceName := \"\", sessionSourceField := \"\", invocationArgumentSourceName := \"\", invocationArgumentSourceField := \"\" }], emittedEvents := [{ name := \"TicketCaptured\" }], observedStreams := [], errors := [], singleton := false, repeatBehavior := \"\" }]"
                ),
            "command definition must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceCommandDefinitions: List[CommandDefinition] = [{ name: \"CaptureTicket\", inputs: [{ name: \"ticket_title\", sourceKind: CommandInputActor, sourceDescription: \"title field on the intake form\", provenanceChain: [\"actor keystrokes -> form field\"], eventStreamSourceEvent: \"\", eventStreamSourceAttribute: \"\", externalPayloadSourceName: \"\", externalPayloadSourceField: \"\", generatedSourceName: \"\", generatedSourceField: \"\", sessionSourceName: \"\", sessionSourceField: \"\", invocationArgumentSourceName: \"\", invocationArgumentSourceField: \"\" }], emittedEvents: [{ name: \"TicketCaptured\" }], observedStreams: [], errors: [], singleton: false, repeatBehavior: \"\" }]"
                ),
            "command definition must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_read_models_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceReadModelDefinitions : List ReadModelDefinition := [{ name := \"ticket_state\", fields := [{ name := \"ticket_title\", sourceKind := \"event_attribute\", sourceEvent := \"TicketCaptured\", sourceAttribute := \"ticket_title\", derivationRule := \"\", derivationSourceFields := [], absenceEvent := \"\", derivationScenarioName := \"\", absenceScenarioName := \"\", provenanceDescription := \"TicketCaptured.ticket_title\" }], transitive := false, relationshipFields := [], transitiveRule := \"\", exampleScenarioName := \"\" }]"
                ),
            "read model definition must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceReadModelDefinitions: List[ReadModelDefinition] = [{ name: \"ticket_state\", fields: [{ name: \"ticket_title\", sourceKind: \"event_attribute\", sourceEvent: \"TicketCaptured\", sourceAttribute: \"ticket_title\", derivationRule: \"\", derivationSourceFields: [], absenceEvent: \"\", derivationScenarioName: \"\", absenceScenarioName: \"\", provenanceDescription: \"TicketCaptured.ticket_title\" }], transitive: false, relationshipFields: [], transitiveRule: \"\", exampleScenarioName: \"\" }]"
                ),
            "read model definition must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_bit_level_data_flows_from_exported_events() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
                "--target",
                "Capture ticket.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceBitLevelDataFlows : List BitLevelDataFlow := [{ datum := \"ticket_title\", sourceKind := \"original\", source := \"actor input title field\", transformationSemantics := \"identity\", target := \"Capture ticket.ticket_title\", bitEncoding := \"UTF-8 string\" }]"
                ),
            "bit-level data flow must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = [{ datum: \"ticket_title\", sourceKind: \"original\", source: \"actor input title field\", transformationSemantics: \"identity\", target: \"Capture ticket.ticket_title\", bitEncoding: \"UTF-8 string\" }]"
                ),
            "bit-level data flow must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_bit_level_data_flow_updates_from_exported_events()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        add_ticket_title_data_flow(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "actor input title field",
                "--source-kind",
                "original",
                "--transformation",
                "identity",
                "--target",
                "Capture ticket.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
                "--new-datum",
                "ticket_title",
                "--new-source",
                "reviewed title field",
                "--new-source-kind",
                "modeled_target",
                "--new-transformation",
                "identity",
                "--new-target",
                "Capture ticket.reviewed_title",
                "--new-bit-encoding",
                "UTF-8 normalized string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_quint.contains("source: \"reviewed title field\""),
            "updated bit-level data flow must be rebuilt from exported events"
        );
        assert!(
            !slice_quint.contains("source: \"actor input title field\""),
            "previous bit-level data flow must not be rebuilt after update"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_bit_level_data_flow_removals_from_exported_events()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        add_ticket_title_data_flow(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "actor input title field",
                "--source-kind",
                "original",
                "--transformation",
                "identity",
                "--target",
                "Capture ticket.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_quint =
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_quint.contains("datum: \"ticket_title\""),
            "removed bit-level data flow must not be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_automations_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceAutomations : List AutomationDefinition := [{ name := \"title-deduplicator\", triggerName := \"TicketCaptured\", commandName := \"CaptureTicket\", handledErrors := [\"DuplicateTicket\"], reactionDescription := \"deduplicates captured titles by reissuing CaptureTicket when needed\" }]"
                ),
            "slice automation must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceAutomations: List[AutomationDefinition] = [{ name: \"title-deduplicator\", triggerName: \"TicketCaptured\", commandName: \"CaptureTicket\", handledErrors: [\"DuplicateTicket\"], reactionDescription: \"deduplicates captured titles by reissuing CaptureTicket when needed\" }]"
                ),
            "slice automation must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_board_elements_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceBoardElements : List BoardElement := [{ name := \"CaptureTicket\", kind := \"command\", lane := \"actions\", declaredName := \"CaptureTicket\", mainPath := true }]"
                ),
            "slice board element must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceBoardElements: List[BoardElement] = [{ name: \"CaptureTicket\", kind: \"command\", lane: \"actions\", declaredName: \"CaptureTicket\", mainPath: true }]"
                ),
            "slice board element must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_board_connections_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceBoardConnections : List BoardConnection := [{ source := \"actor-submit\", sourceKind := \"workflow_trigger\", target := \"CaptureTicket\", targetKind := \"command\" }]"
                ),
            "slice board connection must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceBoardConnections: List[BoardConnection] = [{ source: \"actor-submit\", sourceKind: \"workflow_trigger\", target: \"CaptureTicket\", targetKind: \"command\" }]"
                ),
            "slice board connection must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_outcomes_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-outcome",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--label",
                "ticket_captured",
                "--externally-relevant",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?.contains(
                "def workflowOutcomes : List WorkflowOutcome := [{ sourceSlice := \"capture-ticket\", label := \"ticket_captured\", externallyRelevant := true }]"
            ),
            "workflow outcome must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowOutcomes: List[WorkflowOutcome] = [{ sourceSlice: \"capture-ticket\", label: \"ticket_captured\", externallyRelevant: true }]"
            ),
            "workflow outcome must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_command_errors_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-command-error",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--command",
                "CaptureTicket",
                "--error",
                "DuplicateTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?.contains(
                "def workflowCommandErrors : List WorkflowCommandError := [{ sourceSlice := \"capture-ticket\", commandName := \"CaptureTicket\", errorName := \"DuplicateTicket\" }]"
            ),
            "workflow command error must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowCommandErrors: List[WorkflowCommandError] = [{ sourceSlice: \"capture-ticket\", commandName: \"CaptureTicket\", errorName: \"DuplicateTicket\" }]"
            ),
            "workflow command error must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_owned_definitions_from_exported_events() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-owned-definition",
                "--workflow",
                "open-ticket",
                "--source-slice",
                "capture-ticket",
                "--definition-kind",
                "command",
                "--definition-name",
                "CaptureTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?.contains(
                "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := WorkflowOwnedDefinitionKind.command, definitionName := \"CaptureTicket\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" }]"
            ),
            "workflow owned definition must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: OwnedCommand, definitionName: \"CaptureTicket\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" }]"
            ),
            "workflow owned definition must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_transition_evidence_from_exported_events()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
                "add",
                "workflow-transition-evidence",
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
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "review-ticket workflow step exposes review-ticket-screen as its entry view",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?.contains(
                "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.navigation, trigger := \"review-ticket-screen\", sourceControl := \"review-ticket-screen\", targetView := \"review-ticket-screen\", sourceEvidence := \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence := \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
            ),
            "workflow transition evidence must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: Navigation, trigger: \"review-ticket-screen\", sourceControl: \"review-ticket-screen\", targetView: \"review-ticket-screen\", sourceEvidence: \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence: \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
            ),
            "workflow transition evidence must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_entry_lifecycle_from_exported_events() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "mark",
                "workflow-entry-lifecycle-required",
                "--workflow",
                "open-ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "add",
                "workflow-entry-lifecycle-state",
                "--workflow",
                "open-ticket",
                "--state",
                "fresh_uninitialized",
                "--step",
                "capture-ticket",
                "--evidence",
                "capture-ticket view distinguishes first arrival before initialization",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = fs::read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        assert!(
            workflow_lean.contains("def workflowRequiresEntryLifecycleCoverage : Bool := true"),
            "workflow lifecycle coverage requirement must be rebuilt from exported events"
        );
        assert!(
            workflow_lean.contains(
                "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := [{ state := WorkflowEntryLifecycleStateName.freshUninitialized, step := \"capture-ticket\", evidence := \"capture-ticket view distinguishes first arrival before initialization\" }]"
            ),
            "workflow lifecycle state must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_connections_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := WorkflowTransitionKind.navigation, trigger := \"review-ticket-screen\", sourceControl := \"review-ticket-screen\", targetView := \"review-ticket-screen\", rationale := \"\", payloadContract := \"\" }]"
            ),
            "workflow connection must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: Navigation, trigger: \"review-ticket-screen\", sourceControl: \"review-ticket-screen\", targetView: \"review-ticket-screen\", rationale: \"\", payloadContract: \"\" }]"
            ),
            "workflow connection must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_transition_removals_from_exported_events()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
            .args([
                "remove",
                "transition",
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

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = fs::read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        assert!(
            !workflow_lean.contains("trigger := \"review-ticket-screen\""),
            "removed workflow transition must not be rebuilt from exported events"
        );
        assert!(
            workflow_lean.contains("trigger := \"alternate-review-ticket-screen\""),
            "unremoved workflow transition must still be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_views_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceViewDefinitions : List ViewDefinition := [{ name := \"ticket_summary\", readModels := [\"ticket_state\"], fields := [{ name := \"ticket_title\", sourceKind := \"read_model\", sourceReadModel := \"ticket_state\", sourceField := \"ticket_title\", sketchToken := \"title-label\", provenanceDescription := \"ticket_state.ticket_title\", bitEncoding := \"UTF-8 string\" }], controls := [], sketchTokens := [\"title-label\"], localStates := [], filters := [] }]"
                ),
            "view definition must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceViewDefinitions: List[ViewDefinition] = [{ name: \"ticket_summary\", readModels: [\"ticket_state\"], fields: [{ name: \"ticket_title\", sourceKind: \"read_model\", sourceReadModel: \"ticket_state\", sourceField: \"ticket_title\", sketchToken: \"title-label\", provenanceDescription: \"ticket_state.ticket_title\", bitEncoding: \"UTF-8 string\" }], controls: [], sketchTokens: [\"title-label\"], localStates: [], filters: [] }]"
                ),
            "view definition must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_translations_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceTranslations : List TranslationDefinition := [{ name := \"intake-webhook-translator\", externalEventName := \"intake_webhook_received\", payloadContractName := \"intake_webhook\", commandName := \"CaptureTicket\" }]"
                ),
            "slice translation must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceTranslations: List[TranslationDefinition] = [{ name: \"intake-webhook-translator\", externalEventName: \"intake_webhook_received\", payloadContractName: \"intake_webhook\", commandName: \"CaptureTicket\" }]"
                ),
            "slice translation must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_generated_artifacts_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?.contains(
                "def modelWorkflows : List ModelWorkflow := [{ workflow := \"open-ticket\" }]"
            ),
            "project root must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?
                .contains(
                    "def sliceDescription := \"Actor enters commas, pipes | semicolons; and colons: safely.\""
                ),
            "slice artifact must preserve punctuation when rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowSlices: List[WorkflowSlice] = [{ slug: \"capture-ticket\" }]"
            ),
            "workflow artifact must be rebuilt with projected slice membership"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_updates_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--description",
                "Updated commas, pipes | semicolons; and colons: survive.",
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
                "--type",
                "state_change",
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
                "--name",
                "Capture ticket command",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean = fs::read_to_string(
            temp_dir
                .path()
                .join("model/lean/slices/CaptureTicketCommand.lean"),
        )?;
        assert!(
            slice_lean.contains("def sliceName := \"Capture ticket command\""),
            "slice name update must be rebuilt from exported events"
        );
        assert!(
            slice_lean.contains("def sliceKind : SliceKindName := SliceKindName.stateChange"),
            "slice kind update must be rebuilt from exported events"
        );
        assert!(
            slice_lean.contains(
                "def sliceDescription := \"Updated commas, pipes | semicolons; and colons: survive.\""
            ),
            "slice description update must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowSliceModules: List[WorkflowSliceModule] = [{ slice: \"capture-ticket\", formalModule: \"CaptureTicketCommand\" }]"
            ),
            "workflow projection must rebuild the renamed slice module reference"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_slice_removal_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["remove", "slice", "--slug", "capture-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            !temp_dir
                .path()
                .join("model/lean/slices/CaptureTicket.lean")
                .exists(),
            "removed slice Lean artifact must not be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?
                .contains("def workflowSlices : List WorkflowSlice := []"),
            "workflow projection must rebuild without removed slice membership"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelSlices: List[ModelSlice] = []"),
            "project root projection must rebuild without removed slice membership"
        );

        Ok(())
    }

    #[test]
    fn re_add_reconciles_event_index_from_jsonl_after_cache_is_dropped()
    -> Result<(), Box<dyn Error>> {
        // Bug #2: the gitignored `model/events/index/` cache (the ingestion log)
        // is derived state — a fresh `git clone`, a `git clean`, or any tooling
        // that prunes derived files leaves only the authoritative
        // `events/*.jsonl`. Loading the store must rebuild the cache from those
        // files so that re-adding a fact to an existing slice computes the
        // correct expected stream version, rather than failing with a stale
        // optimistic-concurrency conflict.
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        // First mutation on the slice stream.
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

        // Disturb the store: drop the derived index cache, keeping only the
        // committed authoritative event files.
        let index_dir = temp_dir.path().join("model/events/index");
        assert!(
            index_dir.exists(),
            "the event store must maintain a derived index cache to drop"
        );
        fs::remove_dir_all(&index_dir)?;
        assert!(
            temp_dir.path().join("model/events/events").exists(),
            "the authoritative jsonl event files must remain after dropping the cache"
        );

        // Re-add another fact to the SAME slice. Loading must reconcile the
        // cache from the jsonl and compute the correct expected version, so this
        // re-add succeeds instead of hitting a stale OCC conflict.
        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "acceptance",
                "--name",
                "Actor reviews captured ticket",
                "--given",
                "the ticket details are captured",
                "--when",
                "the actor reopens the ticket",
                "--then",
                "the captured details are shown",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // The store still replays cleanly after the reconciliation.
        Command::cargo_bin("emc")?
            .args(["list", "slices"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(contains("Capture ticket"));

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_updates_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens a repair ticket with commas, pipes | semicolons; and colons: intact.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--name",
                "Open repair ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean =
            fs::read_to_string(temp_dir.path().join("model/lean/OpenRepairTicket.lean"))?;
        assert!(
            workflow_lean.contains("def workflowName := \"Open repair ticket\""),
            "workflow name update must be rebuilt from exported events"
        );
        assert!(
            workflow_lean.contains(
                "def workflowDescription := \"Actor opens a repair ticket with commas, pipes | semicolons; and colons: intact.\""
            ),
            "workflow description update must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelWorkflows: List[str] = [\"open-ticket\"]"),
            "project root projection must preserve the workflow slug after rename"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_workflow_removal_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["remove", "workflow", "--slug", "open-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            !temp_dir.path().join("model/lean/OpenTicket.lean").exists(),
            "removed workflow Lean artifact must not be rebuilt from exported events"
        );
        assert!(
            !temp_dir.path().join("model/quint/OpenTicket.qnt").exists(),
            "removed workflow Quint artifact must not be rebuilt from exported events"
        );
        assert!(
            !temp_dir
                .path()
                .join("model/lean/slices/CaptureTicket.lean")
                .exists(),
            "owned slice Lean artifact must not be rebuilt after workflow removal"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?
                .contains("val modelWorkflows: List[str] = []"),
            "project root projection must rebuild without the removed workflow"
        );

        Ok(())
    }

    #[test]
    fn concurrent_cli_add_slice_commands_preserve_all_project_artifacts()
    -> Result<(), Box<dyn Error>> {
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

        let emc = cargo_bin("emc");
        let slices = [
            ("capture-ticket", "Capture ticket"),
            ("classify-ticket", "Classify ticket"),
            ("assign-ticket", "Assign ticket"),
            ("prioritize-ticket", "Prioritize ticket"),
        ];
        let children = slices
            .iter()
            .map(|(slug, name)| {
                ProcessCommand::new(&emc)
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
                        "Concurrent slice addition.",
                    ])
                    .current_dir(temp_dir.path())
                    .spawn()
            })
            .collect::<Result<Vec<_>, _>>()?;

        for child in children {
            let output = child.wait_with_output()?;
            assert!(
                output.status.success(),
                "concurrent add slice command must succeed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        let lean_root = fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        for (slug, _) in slices {
            assert!(
                lean_root.contains(slug),
                "concurrent add slice commands must preserve {slug}"
            );
        }

        Ok(())
    }

    #[test]
    fn artifact_only_projects_without_event_export_report_upgrade_error()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("emc.toml"), "name = \"Repair Desk\"\n")?;
        fs::create_dir_all(temp_dir.path().join("model/lean"))?;
        fs::write(
            temp_dir.path().join("model/lean/RepairDesk.lean"),
            "-- generated lean artifact\n",
        )?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(contains(
                "pre-release upgrade required: generated artifacts exist without a populated event store at model/events",
            ));

        Ok(())
    }

    #[test]
    fn check_records_projection_fingerprint_when_rebuilding_from_events()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let fingerprint_path = temp_dir.path().join("model/events/projection.fingerprint");
        let fingerprint = fs::read_to_string(&fingerprint_path)?;
        assert!(
            is_lowercase_sha256_hex(fingerprint.trim()),
            "projection fingerprint must be a lowercase sha256 hex string"
        );

        Ok(())
    }

    #[test]
    fn check_projects_new_event_files_when_artifacts_already_exist() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "classify-ticket",
                "--name",
                "Classify ticket",
                "--type",
                "state_view",
                "--description",
                "Actor classifies repair ticket details.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(
                temp_dir
                    .path()
                    .join("model/lean/slices/ClassifyTicket.lean")
            )?
            .contains("def sliceName := \"Classify ticket\""),
            "runtime entrypoints must project newly added slices even when generated artifacts already exist"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_clean_review_records_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "review",
                "record",
                "--workflow",
                "open-ticket",
                "--reviewer",
                "event-model-reviewer",
                "--reviewed-at",
                "2026-06-03T00:00:00.000Z",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;
        fs::remove_dir_all(temp_dir.path().join("reviews"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let review_record =
            fs::read_to_string(temp_dir.path().join("reviews/open-ticket.review.json"))?;
        assert!(
            review_record.contains("\"reviewer_id\": \"event-model-reviewer\""),
            "projected review record must preserve reviewer id"
        );
        assert!(
            review_record.contains("\"reviewed_at\": \"2026-06-03T00:00:00.000Z\""),
            "projected review record must preserve reviewed_at"
        );
        assert!(
            review_record.contains("\"status\": \"clean\""),
            "projected review record must preserve clean status"
        );

        Ok(())
    }

    #[test]
    fn check_rebuilds_missing_reviews_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "review",
                "record",
                "--workflow",
                "open-ticket",
                "--reviewer",
                "event-model-reviewer",
                "--reviewed-at",
                "2026-06-03T00:00:00.000Z",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::remove_dir_all(temp_dir.path().join("reviews"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            fs::read_to_string(temp_dir.path().join("reviews/open-ticket.review.json"))?
                .contains("\"reviewer_id\": \"event-model-reviewer\""),
            "missing review directory must be rebuilt from exported events"
        );

        Ok(())
    }

    fn create_project_with_workflow_and_slice(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
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
                "Actor enters commas, pipes | semicolons; and colons: safely.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn add_ticket_title_data_flow(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
                "--target",
                "Capture ticket.ticket_title",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn generated_model_digest(contents: &str, marker: &str) -> Result<String, Box<dyn Error>> {
        let declaration = contents
            .lines()
            .find_map(|line| line.trim_start().strip_prefix(marker))
            .ok_or("generated artifact must declare modelDigest")?;
        serde_json::from_str(declaration.trim()).map_err(Into::into)
    }

    fn is_lowercase_sha256_hex(value: &str) -> bool {
        value.len() == 64
            && value
                .as_bytes()
                .iter()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    }
}
