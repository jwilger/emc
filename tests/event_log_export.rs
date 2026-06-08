// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::io::Read;
    use std::path::PathBuf;
    use std::process::Command as ProcessCommand;

    use assert_cmd::Command;
    use assert_cmd::cargo::cargo_bin;
    use eventcore_sqlite::rusqlite;
    use predicates::str::contains;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    #[test]
    fn mutating_cli_commands_export_domain_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        create_project_with_workflow_and_slice(&temp_dir)?;

        let events = exported_events(temp_dir.path().join("model/events/v1"))?;

        assert_eq!(events.len(), 3, "mutating commands must export events");
        assert_eq!(events[0]["schema_version"], "emc.events.v1");
        assert_eq!(events[0]["command_ordinal"], 0);
        assert_eq!(events[0]["stream_id"], "project");
        assert_eq!(events[0]["parents"], Value::Array(Vec::new()));
        assert_eq!(events[0]["type"], "ProjectInitialized");
        assert_eq!(events[0]["payload"]["name"], "Repair Desk");

        assert_eq!(events[1]["schema_version"], "emc.events.v1");
        assert_eq!(events[1]["command_ordinal"], 0);
        assert_eq!(events[1]["stream_id"], "workflow::open-ticket");
        assert_eq!(events[1]["parents"][0], events[0]["event_id"]);
        assert_eq!(events[1]["type"], "WorkflowAdded");
        assert_eq!(events[1]["payload"]["slug"], "open-ticket");
        assert_eq!(events[1]["payload"]["name"], "Open ticket");
        assert_eq!(
            events[1]["payload"]["description"],
            "Actor opens a repair ticket."
        );
        assert_eq!(events[2]["command_ordinal"], 0);
        assert_eq!(events[2]["stream_id"], "slice::capture-ticket");
        let mut slice_parents = events[2]["parents"]
            .as_array()
            .ok_or("slice event parents must be a JSON array")?
            .iter()
            .map(|parent| {
                parent
                    .as_str()
                    .map(str::to_owned)
                    .ok_or("slice event parent must be a string")
            })
            .collect::<Result<Vec<_>, _>>()?;
        slice_parents.sort();
        let mut expected_slice_parents = vec![
            events[0]["event_id"]
                .as_str()
                .ok_or("project event_id must be a string")?
                .to_owned(),
            events[1]["event_id"]
                .as_str()
                .ok_or("workflow event_id must be a string")?
                .to_owned(),
        ];
        expected_slice_parents.sort();
        assert_eq!(slice_parents, expected_slice_parents);
        assert_eq!(events[2]["type"], "SliceAdded");
        assert_eq!(events[2]["payload"]["workflow"], "open-ticket");
        assert_eq!(events[2]["payload"]["slug"], "capture-ticket");
        assert_eq!(events[2]["payload"]["name"], "Capture ticket");
        assert_eq!(events[2]["payload"]["kind"], "state_view");
        assert_eq!(
            events[2]["payload"]["description"],
            "Actor enters commas, pipes | semicolons; and colons: safely."
        );

        Ok(())
    }

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
    fn add_scenario_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let scenario_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceScenarioAdded")
            .ok_or("scenario mutation must export a SliceScenarioAdded event")?;

        assert_eq!(scenario_event["stream_id"], "slice::capture-ticket");
        assert_eq!(scenario_event["payload"]["slice"], "capture-ticket");
        assert_eq!(scenario_event["payload"]["kind"], "acceptance");
        assert_eq!(scenario_event["payload"]["name"], "Actor captures ticket");
        assert_eq!(
            scenario_event["payload"]["given"],
            "ticket intake screen is open"
        );
        assert_eq!(
            scenario_event["payload"]["when"],
            "the actor submits ticket details"
        );
        assert_eq!(
            scenario_event["payload"]["then"],
            "the ticket details are visible for review"
        );
        assert_eq!(
            scenario_event["payload"]["read_streams"],
            Value::Array(vec![])
        );
        assert_eq!(
            scenario_event["payload"]["written_streams"],
            Value::Array(vec![])
        );
        assert_eq!(scenario_event["payload"]["contract_kind"], Value::Null);
        assert_eq!(scenario_event["payload"]["covered_definition"], Value::Null);
        assert_eq!(
            scenario_event["payload"]["error_references"],
            Value::Array(vec![])
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
    fn add_outcome_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let outcome_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceOutcomeAdded")
            .ok_or("outcome mutation must export a SliceOutcomeAdded event")?;

        assert_eq!(outcome_event["stream_id"], "slice::capture-ticket");
        assert_eq!(outcome_event["payload"]["slice"], "capture-ticket");
        assert_eq!(outcome_event["payload"]["label"], "ticket-captured");
        assert_eq!(
            outcome_event["payload"]["events"],
            Value::Array(vec![Value::String("TicketCaptured".to_owned())])
        );
        assert_eq!(outcome_event["payload"]["externally_relevant"], true);

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
    fn add_external_payload_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let external_payload_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceExternalPayloadAdded")
            .ok_or("external payload mutation must export a SliceExternalPayloadAdded event")?;

        assert_eq!(external_payload_event["stream_id"], "slice::capture-ticket");
        assert_eq!(external_payload_event["payload"]["slice"], "capture-ticket");
        assert_eq!(external_payload_event["payload"]["name"], "intake_webhook");
        assert_eq!(external_payload_event["payload"]["field"], "ticket_title");
        assert_eq!(
            external_payload_event["payload"]["field_provenance"],
            "intake_webhook.ticket_title supplied by the external ticket intake system"
        );
        assert_eq!(
            external_payload_event["payload"]["bit_encoding"],
            "UTF-8 string"
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
    fn add_event_definition_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let event_definition_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceEventDefinitionAdded")
            .ok_or("event definition mutation must export a SliceEventDefinitionAdded event")?;

        assert_eq!(event_definition_event["stream_id"], "slice::capture-ticket");
        assert_eq!(event_definition_event["payload"]["slice"], "capture-ticket");
        assert_eq!(event_definition_event["payload"]["name"], "TicketCaptured");
        assert_eq!(event_definition_event["payload"]["stream"], "tickets");
        assert_eq!(
            event_definition_event["payload"]["attribute"]["name"],
            "ticket_title"
        );
        assert_eq!(
            event_definition_event["payload"]["attribute"]["source_kind"],
            "generated"
        );
        assert_eq!(
            event_definition_event["payload"]["attribute"]["source_name"],
            "ticket_feed_snapshot"
        );
        assert_eq!(
            event_definition_event["payload"]["attribute"]["source_field"],
            "title"
        );
        assert_eq!(
            event_definition_event["payload"]["attribute"]["generated_source_kind"],
            "ticket_feed_snapshot"
        );
        assert_eq!(
            event_definition_event["payload"]["attribute"]["provenance"],
            "ticket feed snapshot title"
        );
        assert_eq!(event_definition_event["payload"]["observed"], true);
        assert_eq!(event_definition_event["payload"]["shared"], false);

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
    fn add_command_definition_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let command_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceCommandDefinitionAdded")
            .ok_or("command mutation must export a SliceCommandDefinitionAdded event")?;

        assert_eq!(command_event["stream_id"], "slice::capture-ticket");
        assert_eq!(command_event["payload"]["slice"], "capture-ticket");
        assert_eq!(command_event["payload"]["name"], "CaptureTicket");
        assert_eq!(command_event["payload"]["input"]["name"], "ticket_title");
        assert_eq!(command_event["payload"]["input"]["source_kind"], "actor");
        assert_eq!(
            command_event["payload"]["input"]["source_description"],
            "title field on the intake form"
        );
        assert_eq!(
            command_event["payload"]["input"]["provenance_chain"],
            Value::Array(vec![Value::String(
                "actor keystrokes -> form field".to_owned()
            )])
        );
        assert_eq!(
            command_event["payload"]["emitted_events"],
            Value::Array(vec![Value::String("TicketCaptured".to_owned())])
        );
        assert_eq!(
            command_event["payload"]["observed_streams"],
            Value::Array(vec![])
        );
        assert_eq!(command_event["payload"]["errors"], Value::Array(vec![]));
        assert_eq!(
            command_event["payload"]["singleton_repeat_behavior"],
            Value::Null
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
                    "def sliceCommandDefinitions : List CommandDefinition := [{ name := \"CaptureTicket\", inputs := [{ name := \"ticket_title\", sourceKind := \"actor\", sourceDescription := \"title field on the intake form\", provenanceChain := [\"actor keystrokes -> form field\"], eventStreamSourceEvent := \"\", eventStreamSourceAttribute := \"\", externalPayloadSourceName := \"\", externalPayloadSourceField := \"\", generatedSourceName := \"\", generatedSourceField := \"\", sessionSourceName := \"\", sessionSourceField := \"\", invocationArgumentSourceName := \"\", invocationArgumentSourceField := \"\" }], emittedEvents := [\"TicketCaptured\"], observedStreams := [], errors := [], singleton := false, repeatBehavior := \"\" }]"
                ),
            "command definition must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?
                .contains(
                    "val sliceCommandDefinitions: List[CommandDefinition] = [{ name: \"CaptureTicket\", inputs: [{ name: \"ticket_title\", sourceKind: \"actor\", sourceDescription: \"title field on the intake form\", provenanceChain: [\"actor keystrokes -> form field\"], eventStreamSourceEvent: \"\", eventStreamSourceAttribute: \"\", externalPayloadSourceName: \"\", externalPayloadSourceField: \"\", generatedSourceName: \"\", generatedSourceField: \"\", sessionSourceName: \"\", sessionSourceField: \"\", invocationArgumentSourceName: \"\", invocationArgumentSourceField: \"\" }], emittedEvents: [\"TicketCaptured\"], observedStreams: [], errors: [], singleton: false, repeatBehavior: \"\" }]"
                ),
            "command definition must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn add_read_model_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let read_model_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceReadModelAdded")
            .ok_or("read model mutation must export a SliceReadModelAdded event")?;

        assert_eq!(read_model_event["stream_id"], "slice::capture-ticket");
        assert_eq!(read_model_event["payload"]["slice"], "capture-ticket");
        assert_eq!(read_model_event["payload"]["name"], "ticket_state");
        assert_eq!(read_model_event["payload"]["field"]["name"], "ticket_title");
        assert_eq!(
            read_model_event["payload"]["field"]["source_kind"],
            "event_attribute"
        );
        assert_eq!(
            read_model_event["payload"]["field"]["source_event"],
            "TicketCaptured"
        );
        assert_eq!(
            read_model_event["payload"]["field"]["source_attribute"],
            "ticket_title"
        );
        assert_eq!(
            read_model_event["payload"]["field"]["provenance"],
            "TicketCaptured.ticket_title"
        );
        assert_eq!(read_model_event["payload"]["transitive"], false);
        assert_eq!(
            read_model_event["payload"]["relationship_fields"],
            Value::Array(vec![])
        );
        assert_eq!(read_model_event["payload"]["transitive_rule"], Value::Null);
        assert_eq!(read_model_event["payload"]["example_scenario"], Value::Null);

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
    fn add_bit_level_data_flow_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let data_flow_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceBitLevelDataFlowAdded")
            .ok_or("data-flow mutation must export a SliceBitLevelDataFlowAdded event")?;

        assert_eq!(data_flow_event["stream_id"], "slice::capture-ticket");
        assert_eq!(data_flow_event["payload"]["slice"], "capture-ticket");
        assert_eq!(data_flow_event["payload"]["datum"], "ticket_title");
        assert_eq!(
            data_flow_event["payload"]["source"],
            "actor input title field"
        );
        assert_eq!(data_flow_event["payload"]["source_kind"], "original");
        assert_eq!(data_flow_event["payload"]["transformation"], "identity");
        assert_eq!(
            data_flow_event["payload"]["target"],
            "Capture ticket.ticket_title"
        );
        assert_eq!(data_flow_event["payload"]["bit_encoding"], "UTF-8 string");

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
    fn add_automation_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let automation_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceAutomationAdded")
            .ok_or("automation mutation must export a SliceAutomationAdded event")?;

        assert_eq!(automation_event["stream_id"], "slice::capture-ticket");
        assert_eq!(automation_event["payload"]["slice"], "capture-ticket");
        assert_eq!(automation_event["payload"]["name"], "title-deduplicator");
        assert_eq!(automation_event["payload"]["trigger"], "TicketCaptured");
        assert_eq!(automation_event["payload"]["command"], "CaptureTicket");
        assert_eq!(
            automation_event["payload"]["handled_errors"],
            Value::Array(vec![Value::String("DuplicateTicket".to_owned())])
        );
        assert_eq!(
            automation_event["payload"]["reaction"],
            "deduplicates captured titles by reissuing CaptureTicket when needed"
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
    fn add_board_element_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let board_element_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceBoardElementAdded")
            .ok_or("board element mutation must export a SliceBoardElementAdded event")?;

        assert_eq!(board_element_event["stream_id"], "slice::capture-ticket");
        assert_eq!(board_element_event["payload"]["slice"], "capture-ticket");
        assert_eq!(board_element_event["payload"]["name"], "CaptureTicket");
        assert_eq!(board_element_event["payload"]["kind"], "command");
        assert_eq!(board_element_event["payload"]["lane"], "actions");
        assert_eq!(
            board_element_event["payload"]["declared_name"],
            "CaptureTicket"
        );
        assert_eq!(board_element_event["payload"]["main_path"], true);

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
    fn add_board_connection_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let board_connection_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceBoardConnectionAdded")
            .ok_or("board connection mutation must export a SliceBoardConnectionAdded event")?;

        assert_eq!(board_connection_event["stream_id"], "slice::capture-ticket");
        assert_eq!(board_connection_event["payload"]["slice"], "capture-ticket");
        assert_eq!(board_connection_event["payload"]["source"], "actor-submit");
        assert_eq!(
            board_connection_event["payload"]["source_kind"],
            "workflow_trigger"
        );
        assert_eq!(board_connection_event["payload"]["target"], "CaptureTicket");
        assert_eq!(board_connection_event["payload"]["target_kind"], "command");

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
    fn add_workflow_outcome_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let workflow_outcome_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowOutcomeAdded")
            .ok_or("workflow outcome mutation must export a WorkflowOutcomeAdded event")?;

        assert_eq!(workflow_outcome_event["stream_id"], "workflow::open-ticket");
        assert_eq!(workflow_outcome_event["payload"]["workflow"], "open-ticket");
        assert_eq!(
            workflow_outcome_event["payload"]["source_slice"],
            "capture-ticket"
        );
        assert_eq!(
            workflow_outcome_event["payload"]["label"],
            "ticket_captured"
        );
        assert_eq!(
            workflow_outcome_event["payload"]["externally_relevant"],
            true
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
    fn add_workflow_command_error_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let command_error_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowCommandErrorAdded")
            .ok_or(
                "workflow command-error mutation must export a WorkflowCommandErrorAdded event",
            )?;

        assert_eq!(command_error_event["stream_id"], "workflow::open-ticket");
        assert_eq!(command_error_event["payload"]["workflow"], "open-ticket");
        assert_eq!(
            command_error_event["payload"]["source_slice"],
            "capture-ticket"
        );
        assert_eq!(command_error_event["payload"]["command"], "CaptureTicket");
        assert_eq!(command_error_event["payload"]["error"], "DuplicateTicket");

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
    fn add_workflow_owned_definition_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let owned_definition_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowOwnedDefinitionAdded")
            .ok_or(
                "workflow owned-definition mutation must export a WorkflowOwnedDefinitionAdded event",
            )?;

        assert_eq!(owned_definition_event["stream_id"], "workflow::open-ticket");
        assert_eq!(owned_definition_event["payload"]["workflow"], "open-ticket");
        assert_eq!(
            owned_definition_event["payload"]["source_slice"],
            "capture-ticket"
        );
        assert_eq!(
            owned_definition_event["payload"]["definition_kind"],
            "command"
        );
        assert_eq!(
            owned_definition_event["payload"]["definition_name"],
            "CaptureTicket"
        );
        assert_eq!(
            owned_definition_event["payload"]["definition_stream"],
            Value::Null
        );
        assert_eq!(
            owned_definition_event["payload"]["source_provenance"],
            Value::Null
        );
        assert_eq!(
            owned_definition_event["payload"]["event_participation"],
            Value::Null
        );
        assert_eq!(owned_definition_event["payload"]["view_role"], Value::Null);

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
                "def workflowOwnedDefinitions : List WorkflowOwnedDefinition := [{ sourceSlice := \"capture-ticket\", definitionKind := \"command\", definitionName := \"CaptureTicket\", definitionStream := \"\", sourceProvenance := \"\", eventParticipation := \"\", viewRole := \"\" }]"
            ),
            "workflow owned definition must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowOwnedDefinitions: List[WorkflowOwnedDefinition] = [{ sourceSlice: \"capture-ticket\", definitionKind: \"command\", definitionName: \"CaptureTicket\", definitionStream: \"\", sourceProvenance: \"\", eventParticipation: \"\", viewRole: \"\" }]"
            ),
            "workflow owned definition must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn add_workflow_transition_evidence_exports_domain_event() -> Result<(), Box<dyn Error>> {
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
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "review-ticket workflow step exposes review-ticket-screen as its entry view",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let evidence_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowTransitionEvidenceAdded")
            .ok_or(
                "workflow transition evidence mutation must export a WorkflowTransitionEvidenceAdded event",
            )?;

        assert_eq!(evidence_event["stream_id"], "workflow::open-ticket");
        assert_eq!(evidence_event["payload"]["workflow"], "open-ticket");
        assert_eq!(evidence_event["payload"]["from"], "capture-ticket");
        assert_eq!(evidence_event["payload"]["to"], "review-ticket");
        assert_eq!(evidence_event["payload"]["via"], "navigation");
        assert_eq!(evidence_event["payload"]["name"], "review-ticket-screen");
        assert_eq!(
            evidence_event["payload"]["source_evidence"],
            "capture-ticket view owns the review-ticket-screen navigation control"
        );
        assert_eq!(
            evidence_event["payload"]["target_evidence"],
            "review-ticket workflow step exposes review-ticket-screen as its entry view"
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
                "def workflowTransitionEvidences : List WorkflowTransitionEvidence := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", sourceEvidence := \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence := \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
            ),
            "workflow transition evidence must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowTransitionEvidences: List[WorkflowTransitionEvidence] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\", sourceEvidence: \"capture-ticket view owns the review-ticket-screen navigation control\", targetEvidence: \"review-ticket workflow step exposes review-ticket-screen as its entry view\" }]"
            ),
            "workflow transition evidence must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn require_workflow_entry_lifecycle_coverage_exports_domain_event() -> Result<(), Box<dyn Error>>
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

        let lifecycle_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowEntryLifecycleCoverageRequired")
            .ok_or(
                "entry lifecycle coverage requirement must export a WorkflowEntryLifecycleCoverageRequired event",
            )?;

        assert_eq!(lifecycle_event["stream_id"], "workflow::open-ticket");
        assert_eq!(lifecycle_event["payload"]["workflow"], "open-ticket");

        Ok(())
    }

    #[test]
    fn add_workflow_entry_lifecycle_state_exports_domain_event() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

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

        let lifecycle_state_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowEntryLifecycleStateAdded")
            .ok_or(
                "entry lifecycle state mutation must export a WorkflowEntryLifecycleStateAdded event",
            )?;

        assert_eq!(lifecycle_state_event["stream_id"], "workflow::open-ticket");
        assert_eq!(lifecycle_state_event["payload"]["workflow"], "open-ticket");
        assert_eq!(
            lifecycle_state_event["payload"]["state"],
            "fresh_uninitialized"
        );
        assert_eq!(lifecycle_state_event["payload"]["step"], "capture-ticket");
        assert_eq!(
            lifecycle_state_event["payload"]["evidence"],
            "capture-ticket view distinguishes first arrival before initialization"
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
                "def workflowEntryLifecycleStates : List WorkflowEntryLifecycleState := [{ state := \"fresh_uninitialized\", step := \"capture-ticket\", evidence := \"capture-ticket view distinguishes first arrival before initialization\" }]"
            ),
            "workflow lifecycle state must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_exports_domain_event() -> Result<(), Box<dyn Error>> {
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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let connect_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowConnected")
            .ok_or("connect workflow mutation must export a WorkflowConnected event")?;

        assert_eq!(connect_event["stream_id"], "workflow::open-ticket");
        assert_eq!(connect_event["payload"]["workflow"], "open-ticket");
        assert_eq!(connect_event["payload"]["from"], "capture-ticket");
        assert_eq!(connect_event["payload"]["to"], "review-ticket");
        assert_eq!(connect_event["payload"]["to_workflow"], Value::Null);
        assert_eq!(connect_event["payload"]["via"], "navigation");
        assert_eq!(connect_event["payload"]["name"], "review-ticket-screen");
        assert_eq!(connect_event["payload"]["payload_contract"], Value::Null);
        assert_eq!(connect_event["payload"]["reason"], Value::Null);

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
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\", rationale := \"\", payloadContract := \"\" }]"
            ),
            "workflow connection must be rebuilt from exported events"
        );
        assert!(
            fs::read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?.contains(
                "val workflowTransitions: List[WorkflowTransition] = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\", rationale: \"\", payloadContract: \"\" }]"
            ),
            "workflow connection must be rebuilt from exported events"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let removal_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "WorkflowTransitionRemoved")
            .ok_or("remove transition mutation must export a WorkflowTransitionRemoved event")?;

        assert_eq!(removal_event["stream_id"], "workflow::open-ticket");
        assert_eq!(removal_event["payload"]["workflow"], "open-ticket");
        assert_eq!(removal_event["payload"]["from"], "capture-ticket");
        assert_eq!(removal_event["payload"]["to"], "review-ticket");
        assert_eq!(removal_event["payload"]["to_workflow"], Value::Null);
        assert_eq!(removal_event["payload"]["via"], "navigation");
        assert_eq!(removal_event["payload"]["name"], "review-ticket-screen");

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
    fn add_view_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let view_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceViewAdded")
            .ok_or("view mutation must export a SliceViewAdded event")?;

        assert_eq!(view_event["stream_id"], "slice::capture-ticket");
        assert_eq!(view_event["payload"]["slice"], "capture-ticket");
        assert_eq!(view_event["payload"]["name"], "ticket_summary");
        assert_eq!(view_event["payload"]["field"]["name"], "ticket_title");
        assert_eq!(view_event["payload"]["field"]["source_kind"], "read_model");
        assert_eq!(
            view_event["payload"]["field"]["source_read_model"],
            "ticket_state"
        );
        assert_eq!(
            view_event["payload"]["field"]["source_field"],
            "ticket_title"
        );
        assert_eq!(
            view_event["payload"]["field"]["sketch_token"],
            "title-label"
        );
        assert_eq!(
            view_event["payload"]["field"]["provenance"],
            "ticket_state.ticket_title"
        );
        assert_eq!(
            view_event["payload"]["field"]["bit_encoding"],
            "UTF-8 string"
        );
        assert_eq!(view_event["payload"]["controls"], Value::Array(vec![]));
        assert_eq!(view_event["payload"]["local_states"], Value::Array(vec![]));
        assert_eq!(view_event["payload"]["filters"], Value::Array(vec![]));

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
    fn add_translation_exports_domain_event() -> Result<(), Box<dyn Error>> {
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

        let translation_event = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "SliceTranslationAdded")
            .ok_or("translation mutation must export a SliceTranslationAdded event")?;

        assert_eq!(translation_event["stream_id"], "slice::capture-ticket");
        assert_eq!(translation_event["payload"]["slice"], "capture-ticket");
        assert_eq!(
            translation_event["payload"]["name"],
            "intake-webhook-translator"
        );
        assert_eq!(
            translation_event["payload"]["external_event"],
            "intake_webhook_received"
        );
        assert_eq!(
            translation_event["payload"]["payload_contract"],
            "intake_webhook"
        );
        assert_eq!(translation_event["payload"]["command"], "CaptureTicket");

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
            slice_lean.contains("def sliceKind := \"state_change\""),
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
    fn list_conflicts_reports_concurrent_semantic_event_conflicts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let conflict = create_concurrent_slice_update_conflict(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["list", "conflicts"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(contains(
                "conflict slice::capture-ticket SliceUpdated capture-ticket",
            ))
            .stdout(contains(conflict.original_event_id))
            .stdout(contains(conflict.conflicting_event_id));

        Ok(())
    }

    #[test]
    fn resolve_conflict_appends_resolution_event_and_clears_conflict() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        let conflict = create_concurrent_slice_update_conflict(&temp_dir)?;
        let output = Command::cargo_bin("emc")?
            .args(["list", "conflicts"])
            .current_dir(temp_dir.path())
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let conflict_id = stdout
            .split(" id ")
            .nth(1)
            .and_then(|suffix| suffix.split_whitespace().next())
            .ok_or("conflict id must be reported")?;

        Command::cargo_bin("emc")?
            .args([
                "resolve",
                "conflict",
                "--id",
                conflict_id,
                "--choose-event",
                &conflict.original_event_id,
            ])
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(contains(format!("resolved conflict {conflict_id}")));

        Command::cargo_bin("emc")?
            .args(["list", "conflicts"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(contains("no event conflicts"));

        let resolution = exported_events(temp_dir.path().join("model/events/v1"))?
            .into_iter()
            .find(|event| event["type"] == "ConflictResolved")
            .ok_or("conflict resolution event must be exported")?;
        assert_eq!(resolution["payload"]["conflict_id"], conflict_id);
        assert_eq!(
            resolution["payload"]["chosen_event_id"],
            conflict.original_event_id
        );

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let has_resolution_event: bool = conn.query_row(
            "SELECT EXISTS (
                SELECT 1 FROM eventcore_events
                WHERE stream_id = 'project'
                  AND event_type = 'EmcEvent'
                  AND event_data LIKE '%ConflictResolved%'
                  AND event_data LIKE ?1
            )",
            rusqlite::params![format!("%{conflict_id}%")],
            |row| row.get(0),
        )?;
        assert!(
            has_resolution_event,
            "conflict resolution must append a ConflictResolved EmcEvent"
        );

        Ok(())
    }

    #[test]
    fn resolve_conflict_rejects_empty_identity_arguments() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let conflict = create_concurrent_slice_update_conflict(&temp_dir)?;
        let output = Command::cargo_bin("emc")?
            .args(["list", "conflicts"])
            .current_dir(temp_dir.path())
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let conflict_id = stdout
            .split(" id ")
            .nth(1)
            .and_then(|suffix| suffix.split_whitespace().next())
            .ok_or("conflict id must be reported")?;

        Command::cargo_bin("emc")?
            .args([
                "resolve",
                "conflict",
                "--id",
                "",
                "--choose-event",
                &conflict.original_event_id,
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(contains("invalid event conflict id"));

        Command::cargo_bin("emc")?
            .args([
                "resolve",
                "conflict",
                "--id",
                conflict_id,
                "--choose-event",
                "",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(contains("invalid chosen event id"));

        Ok(())
    }

    #[test]
    fn mutations_fail_while_exported_event_conflicts_are_unresolved() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        create_concurrent_slice_update_conflict(&temp_dir)?;
        let event_count = exported_events(temp_dir.path().join("model/events/v1"))?.len();

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--description",
                "Mutation should wait for conflict resolution.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(contains("unresolved event conflicts"));

        assert_eq!(
            exported_events(temp_dir.path().join("model/events/v1"))?.len(),
            event_count,
            "blocked mutation must not append another event"
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
        create_project_with_workflow_and_slice(&temp_dir)?;
        fs::remove_dir_all(temp_dir.path().join("model/events"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(contains("pre-release upgrade required"))
            .stderr(contains("model/events/v1"));

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

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert_eq!(
            fs::read_to_string(fingerprint_path)?,
            fingerprint,
            "unchanged event export should keep a stable projection fingerprint"
        );

        Ok(())
    }

    #[test]
    fn check_creates_operational_sqlite_cache_outside_repo() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let state_home = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .env("XDG_STATE_HOME", state_home.path())
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let project_hash = hex::encode(Sha256::digest(
            temp_dir.path().canonicalize()?.to_string_lossy().as_bytes(),
        ));
        let sqlite_path = state_home
            .path()
            .join("emc")
            .join("projects")
            .join(project_hash)
            .join("events.sqlite3");
        assert!(
            !sqlite_path.starts_with(temp_dir.path()),
            "operational SQLite cache must live outside the repository"
        );
        let mut header = [0_u8; 16];
        fs::File::open(&sqlite_path)?.read_exact(&mut header)?;
        assert_eq!(
            &header, b"SQLite format 3\0",
            "operational cache must be a SQLite database"
        );
        let event_count = exported_events(temp_dir.path().join("model/events/v1"))?.len();
        let conn = rusqlite::Connection::open(&sqlite_path)?;
        let cached_event_count: usize =
            conn.query_row("SELECT count(*) FROM eventcore_events", [], |row| {
                row.get(0)
            })?;
        assert_eq!(
            cached_event_count, event_count,
            "operational cache must sync exported event files"
        );

        Ok(())
    }

    #[test]
    fn mutating_cli_commands_execute_eventcore_commands() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");

        Command::cargo_bin("emc")?
            .args(["init", "--name", "Repair Desk"])
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
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
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
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
                "Actor enters ticket details.",
            ])
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args([
                "update",
                "workflow",
                "--slug",
                "open-ticket",
                "--description",
                "Actor opens a repair ticket with priority.",
            ])
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
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
                "Actor enters prioritized ticket details.",
            ])
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["remove", "slice", "--slug", "capture-ticket"])
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["remove", "workflow", "--slug", "open-ticket"])
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let eventcore_events = conn
            .prepare(
                "SELECT stream_id, event_type, event_data FROM eventcore_events
                 ORDER BY stream_id, stream_version",
            )?
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(eventcore_events.len(), 7);
        let stream_ids = eventcore_events
            .iter()
            .map(|(stream_id, _, _)| stream_id.as_str())
            .collect::<Vec<_>>();
        assert!(stream_ids.contains(&"project"));
        assert!(stream_ids.contains(&"workflow::open-ticket"));
        assert!(stream_ids.contains(&"slice::capture-ticket"));
        assert!(
            eventcore_events
                .iter()
                .all(|(_, event_type, _)| event_type == "EmcEvent"),
            "create mutations must append eventcore EmcEvent records"
        );
        assert!(
            eventcore_events
                .iter()
                .any(|(_, _, event_data)| event_data.contains("\"ProjectInitialized\""))
        );
        assert!(
            eventcore_events
                .iter()
                .any(|(_, _, event_data)| event_data.contains("\"WorkflowAdded\""))
        );
        assert!(
            eventcore_events
                .iter()
                .any(|(_, _, event_data)| event_data.contains("\"SliceAdded\""))
        );
        assert!(
            eventcore_events
                .iter()
                .any(
                    |(stream_id, _, event_data)| stream_id == "workflow::open-ticket"
                        && event_data.contains("\"WorkflowUpdated\"")
                        && event_data.contains("Actor opens a repair ticket with priority.")
                )
        );
        assert!(
            eventcore_events
                .iter()
                .any(
                    |(stream_id, _, event_data)| stream_id == "slice::capture-ticket"
                        && event_data.contains("\"SliceUpdated\"")
                        && event_data.contains("Actor enters prioritized ticket details.")
                )
        );
        assert!(
            eventcore_events
                .iter()
                .any(
                    |(stream_id, _, event_data)| stream_id == "slice::capture-ticket"
                        && event_data.contains("\"SliceRemoved\"")
                )
        );
        assert!(
            eventcore_events
                .iter()
                .any(
                    |(stream_id, _, event_data)| stream_id == "workflow::open-ticket"
                        && event_data.contains("\"WorkflowRemoved\"")
                )
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_cli_executes_eventcore_command() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
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
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
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
            ])
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let has_connection_event: bool = conn.query_row(
            "SELECT EXISTS (
                SELECT 1 FROM eventcore_events
                WHERE stream_id = 'workflow::open-ticket'
                  AND event_type = 'EmcEvent'
                  AND event_data LIKE '%WorkflowConnected%'
                  AND event_data LIKE '%review-ticket-screen%'
            )",
            [],
            |row| row.get(0),
        )?;
        assert!(
            has_connection_event,
            "connect workflow must append a WorkflowConnected EmcEvent"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_cli_executes_eventcore_command() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
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
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        for trigger in ["review-ticket-screen", "alternate-review-ticket-screen"] {
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
                    trigger,
                ])
                .env("EMC_EVENT_STORE_PATH", &sqlite_path)
                .current_dir(temp_dir.path())
                .assert()
                .success();
        }

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
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let has_removal_event: bool = conn.query_row(
            "SELECT EXISTS (
                SELECT 1 FROM eventcore_events
                WHERE stream_id = 'workflow::open-ticket'
                  AND event_type = 'EmcEvent'
                  AND event_data LIKE '%WorkflowTransitionRemoved%'
                  AND event_data LIKE '%review-ticket-screen%'
            )",
            [],
            |row| row.get(0),
        )?;
        assert!(
            has_removal_event,
            "remove transition must append a WorkflowTransitionRemoved EmcEvent"
        );

        Ok(())
    }

    #[test]
    fn workflow_fact_cli_commands_execute_eventcore_commands() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        create_project_with_workflow_and_slice(&temp_dir)?;

        for args in [
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
                "add",
                "workflow-transition-evidence",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "capture-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
                "--source-evidence",
                "capture-ticket view owns the review-ticket-screen navigation control",
                "--target-evidence",
                "capture-ticket workflow step exposes review-ticket-screen as its entry view",
            ],
            vec![
                "mark",
                "workflow-entry-lifecycle-required",
                "--workflow",
                "open-ticket",
            ],
            vec![
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
            ],
        ] {
            Command::cargo_bin("emc")?
                .args(args)
                .env("EMC_EVENT_STORE_PATH", &sqlite_path)
                .current_dir(temp_dir.path())
                .assert()
                .success();
        }

        let conn = rusqlite::Connection::open(sqlite_path)?;
        for (event_type, expected_fragment) in [
            ("WorkflowOutcomeAdded", "ticket_captured"),
            ("WorkflowCommandErrorAdded", "DuplicateTicket"),
            ("WorkflowOwnedDefinitionAdded", "CaptureTicket"),
            ("WorkflowTransitionEvidenceAdded", "review-ticket-screen"),
            ("WorkflowEntryLifecycleCoverageRequired", "open-ticket"),
            ("WorkflowEntryLifecycleStateAdded", "fresh_uninitialized"),
        ] {
            let has_event: bool = conn.query_row(
                "SELECT EXISTS (
                    SELECT 1 FROM eventcore_events
                    WHERE stream_id = 'workflow::open-ticket'
                      AND event_type = 'EmcEvent'
                      AND event_data LIKE ?1
                      AND event_data LIKE ?2
                )",
                rusqlite::params![format!("%{event_type}%"), format!("%{expected_fragment}%")],
                |row| row.get(0),
            )?;
            assert!(
                has_event,
                "{event_type} CLI mutation must append an eventcore EmcEvent"
            );
        }

        Ok(())
    }

    #[test]
    fn slice_fact_cli_commands_execute_eventcore_commands() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
        create_project_with_workflow_and_slice(&temp_dir)?;

        for args in [
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
            vec![
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
            ],
        ] {
            Command::cargo_bin("emc")?
                .args(args)
                .env("EMC_EVENT_STORE_PATH", &sqlite_path)
                .current_dir(temp_dir.path())
                .assert()
                .success();
        }

        let conn = rusqlite::Connection::open(sqlite_path)?;
        for (event_type, expected_fragment) in [
            ("SliceScenarioAdded", "Actor captures ticket"),
            ("SliceOutcomeAdded", "ticket-captured"),
            ("SliceExternalPayloadAdded", "intake_webhook"),
            ("SliceEventDefinitionAdded", "TicketCaptured"),
            ("SliceCommandDefinitionAdded", "CaptureTicket"),
            ("SliceReadModelAdded", "ticket_state"),
            ("SliceBitLevelDataFlowAdded", "actor input title field"),
            ("SliceAutomationAdded", "title-deduplicator"),
            ("SliceBoardElementAdded", "actions"),
            ("SliceBoardConnectionAdded", "actor-submit"),
            ("SliceViewAdded", "ticket_summary"),
            ("SliceTranslationAdded", "intake-webhook-translator"),
        ] {
            let has_event: bool = conn.query_row(
                "SELECT EXISTS (
                    SELECT 1 FROM eventcore_events
                    WHERE stream_id = 'slice::capture-ticket'
                      AND event_type = 'EmcEvent'
                      AND event_data LIKE ?1
                      AND event_data LIKE ?2
                )",
                rusqlite::params![format!("%{event_type}%"), format!("%{expected_fragment}%")],
                |row| row.get(0),
            )?;
            assert!(
                has_event,
                "{event_type} CLI mutation must append an eventcore EmcEvent"
            );
        }

        Ok(())
    }

    #[test]
    fn record_clean_review_cli_executes_eventcore_command() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let store_dir = TempDir::new()?;
        let sqlite_path = store_dir.path().join("events.sqlite3");
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
            .env("EMC_EVENT_STORE_PATH", &sqlite_path)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let conn = rusqlite::Connection::open(sqlite_path)?;
        let has_review_event: bool = conn.query_row(
            "SELECT EXISTS (
                SELECT 1 FROM eventcore_events
                WHERE stream_id = 'review::open-ticket'
                  AND event_type = 'EmcEvent'
                  AND event_data LIKE '%ReviewRecorded%'
                  AND event_data LIKE '%event-model-reviewer%'
            )",
            [],
            |row| row.get(0),
        )?;
        assert!(
            has_review_event,
            "review record mutation must append a ReviewRecorded EmcEvent"
        );

        Ok(())
    }

    #[test]
    fn check_recreates_deleted_sqlite_cache_from_exported_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let state_home = TempDir::new()?;
        create_project_with_workflow_and_slice(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .env("XDG_STATE_HOME", state_home.path())
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let project_hash = hex::encode(Sha256::digest(
            temp_dir.path().canonicalize()?.to_string_lossy().as_bytes(),
        ));
        let sqlite_path = state_home
            .path()
            .join("emc")
            .join("projects")
            .join(project_hash)
            .join("events.sqlite3");
        fs::remove_file(&sqlite_path)?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .env("XDG_STATE_HOME", state_home.path())
            .current_dir(temp_dir.path())
            .assert()
            .success();

        assert!(
            temp_dir.path().join("model/lean/RepairDesk.lean").exists(),
            "check must rebuild Lean artifacts from exported events after cache deletion"
        );
        assert!(
            temp_dir.path().join("model/quint/RepairDesk.qnt").exists(),
            "check must rebuild Quint artifacts from exported events after cache deletion"
        );
        let event_count = exported_events(temp_dir.path().join("model/events/v1"))?.len();
        let conn = rusqlite::Connection::open(&sqlite_path)?;
        let cached_event_count: usize =
            conn.query_row("SELECT count(*) FROM eventcore_events", [], |row| {
                row.get(0)
            })?;
        assert_eq!(
            cached_event_count, event_count,
            "deleted operational cache must be recreated from exported events"
        );

        Ok(())
    }

    #[test]
    fn merged_independent_slice_events_project_all_slices() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_independent_slice_event_file(&temp_dir)?;

        fs::remove_file(temp_dir.path().join("emc.toml"))?;
        fs::remove_dir_all(temp_dir.path().join("model/lean"))?;
        fs::remove_dir_all(temp_dir.path().join("model/quint"))?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean_root = fs::read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        assert!(lean_root.contains("capture-ticket"));
        assert!(lean_root.contains("classify-ticket"));
        assert!(
            fs::read_to_string(
                temp_dir
                    .path()
                    .join("model/lean/slices/ClassifyTicket.lean")
            )?
            .contains("def sliceName := \"Classify ticket\""),
            "independent merged slice event must project its slice artifact"
        );

        Ok(())
    }

    #[test]
    fn check_projects_new_event_files_when_artifacts_already_exist() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        create_independent_slice_event_file(&temp_dir)?;

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
            "runtime entrypoints must project newly merged event files even when generated artifacts already exist"
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

    fn create_independent_slice_event_file(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
        create_project_with_workflow_and_slice(temp_dir)?;

        let event_dir = temp_dir.path().join("model/events/v1");
        let mut slice_event = exported_events(event_dir.clone())?
            .into_iter()
            .find(|event| event["type"] == "SliceAdded")
            .ok_or("slice event must exist")?;
        let independent_event_id = format!(
            "{}-independent",
            slice_event["event_id"]
                .as_str()
                .ok_or("slice event_id must be a string")?
        );
        slice_event["event_id"] = Value::String(independent_event_id.clone());
        slice_event["command_id"] = Value::String(independent_event_id.clone());
        slice_event["stream_id"] = Value::String("slice::classify-ticket".to_owned());
        slice_event["payload"]["slug"] = Value::String("classify-ticket".to_owned());
        slice_event["payload"]["name"] = Value::String("Classify ticket".to_owned());
        slice_event["payload"]["description"] =
            Value::String("Actor classifies repair ticket details.".to_owned());
        fs::write(
            event_dir.join(format!("{independent_event_id}.json")),
            serde_json::to_string_pretty(&slice_event)?,
        )?;

        Ok(())
    }

    struct CreatedConflict {
        original_event_id: String,
        conflicting_event_id: String,
    }

    fn create_concurrent_slice_update_conflict(
        temp_dir: &TempDir,
    ) -> Result<CreatedConflict, Box<dyn Error>> {
        create_project_with_workflow_and_slice(temp_dir)?;

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

        Ok(CreatedConflict {
            original_event_id,
            conflicting_event_id,
        })
    }

    fn exported_events(path: PathBuf) -> Result<Vec<Value>, Box<dyn Error>> {
        let mut event_paths = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        event_paths.sort();

        let mut events = event_paths
            .into_iter()
            .map(|path| {
                let contents = fs::read_to_string(path)?;
                let event = serde_json::from_str::<Value>(&contents)?;
                Ok(event)
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        events.sort_by_key(|event| event["parents"].as_array().map_or(0, Vec::len));
        Ok(events)
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
