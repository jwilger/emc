// Copyright 2026 John Wilger

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
            ))
            .stdout(predicate::str::contains(
                "added acceptance scenario Actor captures ticket to project root",
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
            ))
            .stdout(predicate::str::contains(
                "added contract scenario Ticket state projector records title to project root",
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
            .success()
            .stdout(predicate::str::contains(
                "added bit-level data flow ticket_title to slice capture-ticket",
            ))
            .stdout(predicate::str::contains(
                "added bit-level data flow ticket_title to project root",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;
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
                "def sliceBitLevelDataFlows : List BitLevelDataFlow := [{ datum := \"ticket_title\", sourceKind := \"original\", source := \"actor input title field\", transformationSemantics := \"identity\", target := \"Capture ticket.ticket_title\", bitEncoding := \"UTF-8 string\" }]"
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
                "val sliceBitLevelDataFlows: List[BitLevelDataFlow] = [{ datum: \"ticket_title\", sourceKind: \"original\", source: \"actor input title field\", transformationSemantics: \"identity\", target: \"Capture ticket.ticket_title\", bitEncoding: \"UTF-8 string\" }]"
            ),
            "Quint slice artifact must carry authored bit-level data flows"
        );
        assert!(
            lean_root.contains(
                "structure ModelScenario where\n  workflow : String\n  slice : String\n  scenarioKind : String\n  scenario : String"
            ),
            "Lean project root must type authored first-class scenarios as named records"
        );
        assert!(
            lean_root.contains(
                "def modelScenarios : List ModelScenario := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", scenarioKind := \"acceptance\", scenario := \"Actor captures ticket\" },{ workflow := \"open-ticket\", slice := \"capture-ticket\", scenarioKind := \"contract\", scenario := \"Ticket state projector records title\" }]"
            ),
            "Lean project root must inventory authored first-class scenarios as named records"
        );
        assert!(
            lean_root
                .contains("theorem modelScenariosAreDeclared : modelScenarios.length = 2 := rfl"),
            "Lean project root must prove authored scenario inventory completeness"
        );
        assert!(
            lean_root.contains(
                "structure ModelScenarioDefinition where\n  workflow : String\n  slice : String\n  scenarioKind : String\n  scenario : String\n  given : String\n  when : String\n  thenStep : String\n  readStreams : List String\n  writtenStreams : List String\n  contractKind : String\n  coveredDefinition : String\n  errorReferences : List String"
            ),
            "Lean project root must type authored scenario definitions as named records"
        );
        assert!(
            lean_root.contains(
                "def modelScenarioDefinitions : List ModelScenarioDefinition := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", scenarioKind := \"acceptance\", scenario := \"Actor captures ticket\", given := \"ticket intake screen is open\", when := \"the actor submits ticket details\", thenStep := \"the ticket details are visible for review\", readStreams := [], writtenStreams := [], contractKind := \"\", coveredDefinition := \"\", errorReferences := [] },{ workflow := \"open-ticket\", slice := \"capture-ticket\", scenarioKind := \"contract\", scenario := \"Ticket state projector records title\", given := \"TicketCaptured is stored\", when := \"ticket_state projects the event\", thenStep := \"ticket_state.title equals the event title\", readStreams := [], writtenStreams := [], contractKind := \"projector\", coveredDefinition := \"ticket_state\", errorReferences := [] }]"
            ),
            "Lean project root must inventory authored scenario GWT and contract definitions as named records"
        );
        assert!(
            lean_root.contains(
                "def modelScenarioDefinitionHasGwt (scenario : ModelScenarioDefinition) : Bool := scenario.given.isEmpty == false && scenario.when.isEmpty == false && scenario.thenStep.isEmpty == false"
            ),
            "Lean project root must prove scenario GWT completeness through named fields"
        );
        assert!(
            lean_root.contains(
                "theorem modelScenarioDefinitionsAreDeclared : modelScenarioDefinitions.length = 2 := rfl"
            ),
            "Lean project root must prove authored scenario definition completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelScenario = { workflow: str, slice: str, scenarioKind: str, scenario: str }"
            ),
            "Quint project root must type authored scenario inventory entries"
        );
        assert!(
            quint_root.contains(
                "val modelScenarios: List[ModelScenario] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", scenarioKind: \"acceptance\", scenario: \"Actor captures ticket\" },{ workflow: \"open-ticket\", slice: \"capture-ticket\", scenarioKind: \"contract\", scenario: \"Ticket state projector records title\" }]"
            ),
            "Quint project root must inventory authored first-class scenarios"
        );
        assert!(
            quint_root.contains("val modelScenariosAreDeclared = modelScenarios.length() == 2"),
            "Quint project root must verify authored scenario inventory completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelScenarioDefinition = { workflow: str, slice: str, scenarioKind: str, scenario: str, given: str, when: str, then: str, readStreams: List[str], writtenStreams: List[str], contractKind: str, coveredDefinition: str, errorReferences: List[str] }"
            ),
            "Quint project root must type authored scenario definition entries"
        );
        assert!(
            quint_root.contains(
                "val modelScenarioDefinitions: List[ModelScenarioDefinition] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", scenarioKind: \"acceptance\", scenario: \"Actor captures ticket\", given: \"ticket intake screen is open\", when: \"the actor submits ticket details\", then: \"the ticket details are visible for review\", readStreams: [], writtenStreams: [], contractKind: \"\", coveredDefinition: \"\", errorReferences: [] },{ workflow: \"open-ticket\", slice: \"capture-ticket\", scenarioKind: \"contract\", scenario: \"Ticket state projector records title\", given: \"TicketCaptured is stored\", when: \"ticket_state projects the event\", then: \"ticket_state.title equals the event title\", readStreams: [], writtenStreams: [], contractKind: \"projector\", coveredDefinition: \"ticket_state\", errorReferences: [] }]"
            ),
            "Quint project root must inventory authored scenario GWT and contract definitions"
        );
        assert!(
            quint_root.contains(
                "val modelScenarioDefinitionsAreDeclared = modelScenarioDefinitions.length() == 2"
            ),
            "Quint project root must verify authored scenario definition completeness"
        );
        assert!(
            lean_root.contains(
                "inductive ModelDataFlowSourceKind where\n  | original\n  | modeledTarget\nderiving BEq, DecidableEq, Repr\n\nstructure ModelDataFlow where\n  workflow : String\n  slice : String\n  datum : String\n  sourceKind : ModelDataFlowSourceKind\n  source : String\n  transformation : String\n  target : String\n  bitEncoding : String"
            ),
            "Lean project root must type authored bit-level data-flow source kinds as closed semantic values"
        );
        assert!(
            lean_root.contains(
                "def modelDataFlows : List ModelDataFlow := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", datum := \"ticket_title\", sourceKind := ModelDataFlowSourceKind.original, source := \"actor input title field\", transformation := \"identity\", target := \"Capture ticket.ticket_title\", bitEncoding := \"UTF-8 string\" }]"
            ),
            "Lean project root must inventory authored bit-level data-flow source kinds as enum constructors"
        );
        assert!(
            lean_root.contains(
                "def modelDataFlowIsBitComplete (dataFlow : ModelDataFlow) : Bool := dataFlow.datum.isEmpty == false && dataFlow.source.isEmpty == false && dataFlow.transformation.isEmpty == false && dataFlow.target.isEmpty == false && dataFlow.bitEncoding.isEmpty == false"
            ),
            "Lean project root must prove bit-level data-flow completeness without revalidating enum source kinds as strings"
        );
        assert!(
            lean_root
                .contains("theorem modelDataFlowsAreDeclared : modelDataFlows.length = 1 := rfl"),
            "Lean project root must prove authored bit-level data-flow inventory completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelDataFlowSourceKind = Original | ModeledTarget\n  type ModelDataFlow = { workflow: str, slice: str, datum: str, sourceKind: ModelDataFlowSourceKind, source: str, transformation: str, target: str, bitEncoding: str }"
            ),
            "Quint project root must type authored bit-level data-flow source kinds as closed semantic values"
        );
        assert!(
            quint_root.contains(
                "val modelDataFlows: List[ModelDataFlow] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", datum: \"ticket_title\", sourceKind: Original, source: \"actor input title field\", transformation: \"identity\", target: \"Capture ticket.ticket_title\", bitEncoding: \"UTF-8 string\" }]"
            ),
            "Quint project root must inventory authored bit-level data-flow source kinds as enum constructors"
        );
        assert!(
            quint_root.contains("val modelDataFlowsAreDeclared = modelDataFlows.length() == 1"),
            "Quint project root must verify authored bit-level data-flow inventory completeness"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

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
                "--generated-source-kind",
                "actor_input",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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

        complete_contract_scenario_coverage(&temp_dir)?;

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
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "status",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "ticket_feed_snapshot",
                "--attribute-source-field",
                "status",
                "--generated-source-kind",
                "ticket_feed_snapshot",
                "--attribute-provenance",
                "ticket feed status field",
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
            .success()
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

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
                "ticket_status",
                "--source",
                "TicketCaptured.status",
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "status",
                "--source",
                "ticket feed snapshot",
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;
        assert!(
            lean.contains(
                "def sliceCommands : List SliceCommandReference := [{ name := \"CaptureTicket\" }]"
            ),
            "Lean slice artifact must carry the authored command name"
        );
        assert!(
            lean.contains(
                "def sliceCommandDefinitions : List CommandDefinition := [{ name := \"CaptureTicket\", inputs := [{ name := \"ticket_title\", sourceKind := \"actor\", sourceDescription := \"title field on the intake form\", provenanceChain := [\"actor keystrokes -> form field\"], eventStreamSourceEvent := \"\", eventStreamSourceAttribute := \"\", externalPayloadSourceName := \"\", externalPayloadSourceField := \"\", generatedSourceName := \"\", generatedSourceField := \"\", sessionSourceName := \"\", sessionSourceField := \"\", invocationArgumentSourceName := \"\", invocationArgumentSourceField := \"\" }], emittedEvents := [{ name := \"TicketCaptured\" }], observedStreams := [], errors := [], singleton := true, repeatBehavior := \"idempotent\" }]"
            ),
            "Lean slice artifact must carry the authored command definition"
        );
        assert!(
            quint.contains(
                "val sliceCommands: List[SliceCommandReference] = [{ name: \"CaptureTicket\" }]"
            ),
            "Quint slice artifact must carry the authored command name"
        );
        assert!(
            quint.contains(
                "val sliceCommandDefinitions: List[CommandDefinition] = [{ name: \"CaptureTicket\", inputs: [{ name: \"ticket_title\", sourceKind: \"actor\", sourceDescription: \"title field on the intake form\", provenanceChain: [\"actor keystrokes -> form field\"], eventStreamSourceEvent: \"\", eventStreamSourceAttribute: \"\", externalPayloadSourceName: \"\", externalPayloadSourceField: \"\", generatedSourceName: \"\", generatedSourceField: \"\", sessionSourceName: \"\", sessionSourceField: \"\", invocationArgumentSourceName: \"\", invocationArgumentSourceField: \"\" }], emittedEvents: [{ name: \"TicketCaptured\" }], observedStreams: [], errors: [], singleton: true, repeatBehavior: \"idempotent\" }]"
            ),
            "Quint slice artifact must carry the authored command definition"
        );
        assert!(
            lean_root.contains(
                "structure ModelCommandInput where\n  workflow : String\n  slice : String\n  command : String\n  input : String\n  sourceKind : String\n  sourceDescription : String\n  provenanceChain : List String\n  eventStreamSourceEvent : String\n  eventStreamSourceAttribute : String\n  externalPayloadSourceName : String\n  externalPayloadSourceField : String\n  generatedSourceName : String\n  generatedSourceField : String\n  sessionSourceName : String\n  sessionSourceField : String\n  invocationArgumentSourceName : String\n  invocationArgumentSourceField : String"
            ),
            "Lean project root must type authored command input source-chain inventory entries as named records"
        );
        assert!(
            lean_root.contains(
                "def modelCommandInputs : List ModelCommandInput := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", command := \"CaptureTicket\", input := \"ticket_title\", sourceKind := \"actor\", sourceDescription := \"title field on the intake form\", provenanceChain := [\"actor keystrokes -> form field\"], eventStreamSourceEvent := \"\", eventStreamSourceAttribute := \"\", externalPayloadSourceName := \"\", externalPayloadSourceField := \"\", generatedSourceName := \"\", generatedSourceField := \"\", sessionSourceName := \"\", sessionSourceField := \"\", invocationArgumentSourceName := \"\", invocationArgumentSourceField := \"\" }]"
            ),
            "Lean project root must inventory authored command input source chains"
        );
        assert!(
            lean_root.contains(
                "def modelCommandInputHasProvenance (input : ModelCommandInput) : Bool := input.sourceDescription.isEmpty == false && input.provenanceChain.isEmpty == false"
            ),
            "Lean project root must prove command input provenance through named fields"
        );
        assert!(
            lean_root.contains(
                "theorem modelCommandInputsAreDeclared : modelCommandInputs.length = 1 := rfl"
            ),
            "Lean project root must prove authored command input source-chain inventory completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelCommandInput = { workflow: str, slice: str, command: str, input: str, sourceKind: str, sourceDescription: str, provenanceChain: List[str], eventStreamSourceEvent: str, eventStreamSourceAttribute: str, externalPayloadSourceName: str, externalPayloadSourceField: str, generatedSourceName: str, generatedSourceField: str, sessionSourceName: str, sessionSourceField: str, invocationArgumentSourceName: str, invocationArgumentSourceField: str }"
            ),
            "Quint project root must type authored command input source-chain inventory entries"
        );
        assert!(
            quint_root.contains(
                "val modelCommandInputs: List[ModelCommandInput] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", command: \"CaptureTicket\", input: \"ticket_title\", sourceKind: \"actor\", sourceDescription: \"title field on the intake form\", provenanceChain: [\"actor keystrokes -> form field\"], eventStreamSourceEvent: \"\", eventStreamSourceAttribute: \"\", externalPayloadSourceName: \"\", externalPayloadSourceField: \"\", generatedSourceName: \"\", generatedSourceField: \"\", sessionSourceName: \"\", sessionSourceField: \"\", invocationArgumentSourceName: \"\", invocationArgumentSourceField: \"\" }]"
            ),
            "Quint project root must inventory authored command input source chains"
        );
        assert!(
            quint_root
                .contains("val modelCommandInputsAreDeclared = modelCommandInputs.length() == 1"),
            "Quint project root must verify authored command input source-chain inventory completeness"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_command_definition_records_event_stream_input_source() -> Result<(), Box<dyn Error>> {
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
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "tickets",
                "--attribute",
                "status",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "ticket_feed_snapshot",
                "--attribute-source-field",
                "status",
                "--generated-source-kind",
                "ticket_feed_snapshot",
                "--attribute-provenance",
                "ticket feed status field",
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
                "ticket_status",
                "--input-source",
                "event_stream_state",
                "--input-description",
                "current ticket status loaded from the ticket stream",
                "--input-provenance",
                "TicketCaptured.status -> ticket_status",
                "--emits",
                "TicketCaptured",
                "--observes",
                "tickets",
                "--source-event",
                "TicketCaptured",
                "--source-attribute",
                "status",
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
                "ticket_status",
                "--source",
                "TicketCaptured.status",
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "status",
                "--source",
                "ticket feed snapshot",
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
            lean.contains("observedStreams := [{ name := \"tickets\" }]"),
            "Lean slice artifact must carry event streams observed for command inputs"
        );
        assert!(
            lean.contains("eventStreamSourceEvent := \"TicketCaptured\"")
                && lean.contains("eventStreamSourceAttribute := \"status\""),
            "Lean slice artifact must link event-stream command inputs to an upstream event attribute"
        );
        assert!(
            quint.contains("observedStreams: [{ name: \"tickets\" }]"),
            "Quint slice artifact must carry event streams observed for command inputs"
        );
        assert!(
            quint.contains("eventStreamSourceEvent: \"TicketCaptured\"")
                && quint.contains("eventStreamSourceAttribute: \"status\""),
            "Quint slice artifact must link event-stream command inputs to an upstream event attribute"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_command_definition_records_external_payload_input_source() -> Result<(), Box<dyn Error>>
    {
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
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "ticket_title",
                "--source",
                "intake_webhook.ticket_title",
                "--source-kind",
                "original",
                "--transformation",
                "transformation",
                "--target",
                "intake_webhook",
                "--bit-encoding",
                "UTF-8 string",
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
                "external_payload",
                "--input-description",
                "ticket title loaded from the intake webhook payload",
                "--input-provenance",
                "intake_webhook.ticket_title -> CaptureTicket.ticket_title",
                "--emits",
                "TicketCaptured",
                "--source-payload",
                "intake_webhook",
                "--source-field",
                "ticket_title",
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
                "intake_webhook.ticket_title",
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
            lean.contains("externalPayloadSourceName := \"intake_webhook\"")
                && lean.contains("externalPayloadSourceField := \"ticket_title\""),
            "Lean slice artifact must link external-payload command inputs to a payload field"
        );
        assert!(
            lean.contains(
                "theorem commandInputsSourcedFromExternalPayloadsResolveIsStable : commandInputsSourcedFromExternalPayloadsResolve = true"
            ),
            "Lean slice artifact must prove external-payload command inputs resolve to payload fields"
        );
        assert!(
            quint.contains("externalPayloadSourceName: \"intake_webhook\"")
                && quint.contains("externalPayloadSourceField: \"ticket_title\""),
            "Quint slice artifact must link external-payload command inputs to a payload field"
        );
        assert!(
            quint.contains("val commandInputsSourcedFromExternalPayloadsResolve"),
            "Quint slice artifact must verify external-payload command inputs resolve to payload fields"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_command_definition_records_generated_input_source_coordinates()
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
                "ticket_id",
                "--input-source",
                "generated",
                "--input-description",
                "ticket id allocated by the ticket id generator",
                "--input-provenance",
                "ticket_id_generator.uuid -> CaptureTicket.ticket_id",
                "--emits",
                "TicketCaptured",
                "--source-name",
                "ticket_id_generator",
                "--source-field",
                "uuid",
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
                "ticket_id",
                "--attribute-source",
                "command_input",
                "--attribute-source-name",
                "ticket_id",
                "--attribute-source-field",
                "value",
                "--attribute-provenance",
                "CaptureTicket.ticket_id",
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
                "ticket_id",
                "--source",
                "ticket_id_generator.uuid",
                "--source-kind",
                "original",
                "--transformation",
                "transformation",
                "--target",
                "CaptureTicket",
                "--bit-encoding",
                "128-bit UUIDv7 string",
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
                "ticket_id",
                "--source",
                "CaptureTicket.ticket_id",
                "--source-kind",
                "original",
                "--transformation",
                "identity",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "128-bit UUIDv7 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("generatedSourceName := \"ticket_id_generator\"")
                && lean.contains("generatedSourceField := \"uuid\""),
            "Lean slice artifact must link generated command inputs to generated source coordinates"
        );
        assert!(
            lean.contains(
                "theorem commandInputsSourcedFromGeneratedValuesHaveCoordinatesIsStable : commandInputsSourcedFromGeneratedValuesHaveCoordinates = true"
            ),
            "Lean slice artifact must prove generated command inputs have source coordinates"
        );
        assert!(
            quint.contains("generatedSourceName: \"ticket_id_generator\"")
                && quint.contains("generatedSourceField: \"uuid\""),
            "Quint slice artifact must link generated command inputs to generated source coordinates"
        );
        assert!(
            quint.contains("val commandInputsSourcedFromGeneratedValuesHaveCoordinates"),
            "Quint slice artifact must verify generated command inputs have source coordinates"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_command_definition_records_session_input_source_coordinates()
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
                "organization_id",
                "--input-source",
                "session",
                "--input-description",
                "organization id loaded from authenticated session",
                "--input-provenance",
                "authenticated_session.organization_id -> CaptureTicket.organization_id",
                "--emits",
                "TicketCaptured",
                "--source-session",
                "authenticated_session",
                "--source-field",
                "organization_id",
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
                "organization_id",
                "--attribute-source",
                "command_input",
                "--attribute-source-name",
                "organization_id",
                "--attribute-source-field",
                "value",
                "--attribute-provenance",
                "CaptureTicket.organization_id",
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
                "organization_id",
                "--source",
                "authenticated_session.organization_id",
                "--source-kind",
                "original",
                "--transformation",
                "projection",
                "--target",
                "CaptureTicket",
                "--bit-encoding",
                "128-bit organization UUID string",
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
                "organization_id",
                "--source",
                "CaptureTicket.organization_id",
                "--source-kind",
                "original",
                "--transformation",
                "identity",
                "--target",
                "TicketCaptured",
                "--bit-encoding",
                "128-bit organization UUID string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("sessionSourceName := \"authenticated_session\"")
                && lean.contains("sessionSourceField := \"organization_id\""),
            "Lean slice artifact must link session command inputs to session value coordinates"
        );
        assert!(
            lean.contains(
                "theorem commandInputsSourcedFromSessionValuesHaveCoordinatesIsStable : commandInputsSourcedFromSessionValuesHaveCoordinates = true"
            ),
            "Lean slice artifact must prove session command inputs have source coordinates"
        );
        assert!(
            quint.contains("sessionSourceName: \"authenticated_session\"")
                && quint.contains("sessionSourceField: \"organization_id\""),
            "Quint slice artifact must link session command inputs to session value coordinates"
        );
        assert!(
            quint.contains("val commandInputsSourcedFromSessionValuesHaveCoordinates"),
            "Quint slice artifact must verify session command inputs have source coordinates"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_command_definition_records_invocation_argument_source_coordinates()
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
                "invocation_argument",
                "--input-description",
                "ticket title supplied by the command invocation",
                "--input-provenance",
                "invoke CaptureTicket.title -> CaptureTicket.ticket_title",
                "--emits",
                "TicketCaptured",
                "--source-argument",
                "CaptureTicket",
                "--source-field",
                "title",
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
                "CaptureTicket.title",
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains("invocationArgumentSourceName := \"CaptureTicket\"")
                && lean.contains("invocationArgumentSourceField := \"title\""),
            "Lean slice artifact must link invocation-argument command inputs to invocation argument coordinates"
        );
        assert!(
            lean.contains(
                "theorem commandInputsSourcedFromInvocationArgumentsHaveCoordinatesIsStable : commandInputsSourcedFromInvocationArgumentsHaveCoordinates = true"
            ),
            "Lean slice artifact must prove invocation-argument command inputs have source coordinates"
        );
        assert!(
            quint.contains("invocationArgumentSourceName: \"CaptureTicket\"")
                && quint.contains("invocationArgumentSourceField: \"title\""),
            "Quint slice artifact must link invocation-argument command inputs to invocation argument coordinates"
        );
        assert!(
            quint.contains("val commandInputsSourcedFromInvocationArgumentsHaveCoordinates"),
            "Quint slice artifact must verify invocation-argument command inputs have source coordinates"
        );
        assert!(
            lean_root.contains("sourceKind := \"invocation_argument\"")
                && lean_root.contains("invocationArgumentSourceName := \"CaptureTicket\"")
                && lean_root.contains("invocationArgumentSourceField := \"title\""),
            "Lean project root must carry invocation-argument command input coordinates"
        );
        assert!(
            quint_root.contains("invocationArgumentSourceName: \"CaptureTicket\"")
                && quint_root.contains("invocationArgumentSourceField: \"title\""),
            "Quint project root must carry invocation-argument command input coordinates"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains(
                "def sliceCommandDefinitions : List CommandDefinition := [{ name := \"CaptureTicket\", inputs := [{ name := \"ticket_title\", sourceKind := \"actor\", sourceDescription := \"title field on the intake form\", provenanceChain := [\"actor keystrokes -> form field\"], eventStreamSourceEvent := \"\", eventStreamSourceAttribute := \"\", externalPayloadSourceName := \"\", externalPayloadSourceField := \"\", generatedSourceName := \"\", generatedSourceField := \"\", sessionSourceName := \"\", sessionSourceField := \"\", invocationArgumentSourceName := \"\", invocationArgumentSourceField := \"\" }], emittedEvents := [{ name := \"TicketCaptured\" }], observedStreams := [], errors := [{ name := \"DuplicateTicket\", scenarioName := \"Duplicate ticket is rejected\", recoveryKind := \"retry\" }], singleton := false, repeatBehavior := \"\" }]"
            ),
            "Lean slice artifact must carry authored command errors and recovery"
        );
        assert!(
            quint.contains(
                "val sliceCommandDefinitions: List[CommandDefinition] = [{ name: \"CaptureTicket\", inputs: [{ name: \"ticket_title\", sourceKind: \"actor\", sourceDescription: \"title field on the intake form\", provenanceChain: [\"actor keystrokes -> form field\"], eventStreamSourceEvent: \"\", eventStreamSourceAttribute: \"\", externalPayloadSourceName: \"\", externalPayloadSourceField: \"\", generatedSourceName: \"\", generatedSourceField: \"\", sessionSourceName: \"\", sessionSourceField: \"\", invocationArgumentSourceName: \"\", invocationArgumentSourceField: \"\" }], emittedEvents: [{ name: \"TicketCaptured\" }], observedStreams: [], errors: [{ name: \"DuplicateTicket\", scenarioName: \"Duplicate ticket is rejected\", recoveryKind: \"retry\" }], singleton: false, repeatBehavior: \"\" }]"
            ),
            "Quint slice artifact must carry authored command errors and recovery"
        );
        assert!(
            lean_root.contains(
                "structure ModelCommandError where\n  workflow : String\n  slice : String\n  command : String\n  error : String\n  scenario : String\n  recovery : String"
            ),
            "Lean project root must type authored command errors as named records"
        );
        assert!(
            lean_root.contains(
                "def modelCommandErrors : List ModelCommandError := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", command := \"CaptureTicket\", error := \"DuplicateTicket\", scenario := \"Duplicate ticket is rejected\", recovery := \"retry\" }]"
            ),
            "Lean project root must inventory authored command errors as named records"
        );
        assert!(
            lean_root.contains(
                "theorem modelCommandErrorsAreDeclared : modelCommandErrors.length = 1 := rfl"
            ),
            "Lean project root must prove authored command errors are declared"
        );
        assert!(
            quint_root.contains(
                "type ModelCommandError = { workflow: str, slice: str, command: str, error: str, scenario: str, recovery: str }"
            ),
            "Quint project root must type authored command error inventory entries"
        );
        assert!(
            quint_root.contains(
                "val modelCommandErrors: List[ModelCommandError] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", command: \"CaptureTicket\", error: \"DuplicateTicket\", scenario: \"Duplicate ticket is rejected\", recovery: \"retry\" }]"
            ),
            "Quint project root must inventory authored command errors with scenario and recovery"
        );
        assert!(
            quint_root
                .contains("val modelCommandErrorsAreDeclared = modelCommandErrors.length() == 1"),
            "Quint project root must verify authored command error inventory completeness"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

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
    fn mcp_stdio_authors_event_stream_command_inputs() -> Result<(), Box<dyn Error>> {
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
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_event_stream_command_input_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_command_definition\""))
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("observedStreams := [{ name := \"tickets\" }]"),
            "MCP-authored event-stream command inputs must be represented in the Lean artifact"
        );
        assert!(
            lean.contains("eventStreamSourceEvent := \"TicketCaptured\"")
                && lean.contains("eventStreamSourceAttribute := \"status\""),
            "MCP-authored event-stream command inputs must link to an upstream event attribute in the Lean artifact"
        );
        assert!(
            quint.contains("observedStreams: [{ name: \"tickets\" }]"),
            "MCP-authored event-stream command inputs must be represented in the Quint artifact"
        );
        assert!(
            quint.contains("eventStreamSourceEvent: \"TicketCaptured\"")
                && quint.contains("eventStreamSourceAttribute: \"status\""),
            "MCP-authored event-stream command inputs must link to an upstream event attribute in the Quint artifact"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_generated_command_input_sources() -> Result<(), Box<dyn Error>> {
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
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_generated_command_input_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_command_definition\""))
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("generatedSourceName := \"ticket_id_generator\"")
                && lean.contains("generatedSourceField := \"uuid\""),
            "MCP-authored generated command inputs must name generated source coordinates in Lean"
        );
        assert!(
            quint.contains("generatedSourceName: \"ticket_id_generator\"")
                && quint.contains("generatedSourceField: \"uuid\""),
            "MCP-authored generated command inputs must name generated source coordinates in Quint"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_session_command_input_sources() -> Result<(), Box<dyn Error>> {
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
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_session_command_input_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_command_definition\""))
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("sessionSourceName := \"authenticated_session\"")
                && lean.contains("sessionSourceField := \"organization_id\""),
            "MCP-authored session command inputs must name session source coordinates in Lean"
        );
        assert!(
            quint.contains("sessionSourceName: \"authenticated_session\"")
                && quint.contains("sessionSourceField: \"organization_id\""),
            "MCP-authored session command inputs must name session source coordinates in Quint"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_invocation_argument_command_input_sources() -> Result<(), Box<dyn Error>> {
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
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_invocation_argument_command_input_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_command_definition\""))
            .stdout(predicate::str::contains(
                "added command CaptureTicket to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;
        assert!(
            lean.contains("invocationArgumentSourceName := \"CaptureTicket\"")
                && lean.contains("invocationArgumentSourceField := \"title\""),
            "MCP-authored invocation-argument command inputs must name invocation argument coordinates in Lean"
        );
        assert!(
            quint.contains("invocationArgumentSourceName: \"CaptureTicket\"")
                && quint.contains("invocationArgumentSourceField: \"title\""),
            "MCP-authored invocation-argument command inputs must name invocation argument coordinates in Quint"
        );
        assert!(
            lean_root.contains("sourceKind := \"invocation_argument\"")
                && lean_root.contains("invocationArgumentSourceName := \"CaptureTicket\"")
                && lean_root.contains("invocationArgumentSourceField := \"title\""),
            "MCP-authored project root facts must carry invocation-argument command input coordinates in Lean"
        );
        assert!(
            quint_root.contains("invocationArgumentSourceName: \"CaptureTicket\"")
                && quint_root.contains("invocationArgumentSourceField: \"title\""),
            "MCP-authored project root facts must carry invocation-argument command input coordinates in Quint"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

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
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains(
                "def sliceEventDefinitions : List EventDefinition := [{ name := \"TicketCaptured\""
            ),
            "MCP-authored event definition must be represented in the Lean artifact"
        );
        assert!(
            lean.contains("generatedSourceKind := \"event_store_observation\""),
            "MCP-authored generated event attributes must name the generated source kind in Lean"
        );
        assert!(
            quint.contains("generatedSourceKind: \"event_store_observation\""),
            "MCP-authored generated event attributes must name the generated source kind in Quint"
        );
        assert!(
            lean.contains("observed := true"),
            "MCP-authored observed event facts must be represented in the Lean artifact"
        );

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_shared_event_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_shared_event_definition_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_event_definition\""))
            .stdout(predicate::str::contains(
                "added event TicketTagged to slice capture-ticket",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("observed := false, shared := true"),
            "MCP-authored shared event facts must be represented in the Lean artifact"
        );
        assert!(
            quint.contains("observed: false, shared: true"),
            "MCP-authored shared event facts must be represented in the Quint artifact"
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
            ))
            .stdout(predicate::str::contains(
                "added external payload intake_webhook to project root",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

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
        assert!(
            lean_root.contains(
                "def modelExternalPayloads : List ModelExternalPayload := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", externalPayload := \"intake_webhook\" }]"
            ),
            "Lean project root must inventory authored external payload contracts"
        );
        assert!(
            lean_root.contains(
                "theorem modelExternalPayloadsAreDeclared : modelExternalPayloads.length = 1 := rfl"
            ),
            "Lean project root must prove authored external payload inventory completeness"
        );
        assert!(
            lean_root.contains(
                "def modelExternalPayloadFields : List ModelExternalPayloadField := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", externalPayload := \"intake_webhook\", field := \"ticket_title\", provenance := \"intake_webhook.ticket_title supplied by the external ticket intake system\", bitEncoding := \"UTF-8 string\" }]"
            ),
            "Lean project root must inventory authored external payload field provenance and bit encoding"
        );
        assert!(
            lean_root.contains(
                "theorem modelExternalPayloadFieldsAreDeclared : modelExternalPayloadFields.length = 1 := rfl"
            ),
            "Lean project root must prove authored external payload field completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelExternalPayload = { workflow: str, slice: str, externalPayload: str }"
            ),
            "Quint project root must type authored external payload inventory entries"
        );
        assert!(
            quint_root.contains(
                "val modelExternalPayloads: List[ModelExternalPayload] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", externalPayload: \"intake_webhook\" }]"
            ),
            "Quint project root must inventory authored external payload contracts"
        );
        assert!(
            quint_root.contains(
                "val modelExternalPayloadsAreDeclared = modelExternalPayloads.length() == 1"
            ),
            "Quint project root must verify authored external payload inventory completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelExternalPayloadField = { workflow: str, slice: str, externalPayload: str, field: str, provenance: str, bitEncoding: str }"
            ),
            "Quint project root must type authored external payload field inventory entries"
        );
        assert!(
            quint_root.contains(
                "val modelExternalPayloadFields: List[ModelExternalPayloadField] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", externalPayload: \"intake_webhook\", field: \"ticket_title\", provenance: \"intake_webhook.ticket_title supplied by the external ticket intake system\", bitEncoding: \"UTF-8 string\" }]"
            ),
            "Quint project root must inventory authored external payload field provenance and bit encoding"
        );
        assert!(
            quint_root.contains(
                "val modelExternalPayloadFieldsAreDeclared = modelExternalPayloadFields.length() == 1"
            ),
            "Quint project root must verify authored external payload field completeness"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

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
                "--source-kind",
                "original",
                "--transformation",
                "transformation",
                "--target",
                "intake_webhook",
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
                "--source-kind",
                "original",
                "--transformation",
                "transformation",
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

        complete_contract_scenario_coverage(&temp_dir)?;

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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

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
        assert!(
            lean_root.contains(
                "structure ModelOutcome where\n  workflow : String\n  slice : String\n  outcome : String\n  events : List String\n  externallyRelevant : Bool"
            ),
            "Lean project root must type authored outcomes as named records"
        );
        assert!(
            lean_root.contains(
                "def modelOutcomes : List ModelOutcome := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", outcome := \"ticket_captured\", events := [\"TicketCaptured\"], externallyRelevant := false }]"
            ),
            "Lean project root must inventory authored outcomes as named records"
        );
        assert!(
            lean_root
                .contains("theorem modelOutcomesAreDeclared : modelOutcomes.length = 1 := rfl"),
            "Lean project root must prove authored outcomes are declared"
        );
        assert!(
            quint_root.contains(
                "type ModelOutcome = { workflow: str, slice: str, outcome: str, events: List[str], externallyRelevant: bool }"
            ),
            "Quint project root must type authored outcome inventory entries"
        );
        assert!(
            quint_root.contains(
                "val modelOutcomes: List[ModelOutcome] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", outcome: \"ticket_captured\", events: [\"TicketCaptured\"], externallyRelevant: false }]"
            ),
            "Quint project root must inventory authored outcomes with backing event sets"
        );
        assert!(
            quint_root.contains("val modelOutcomesAreDeclared = modelOutcomes.length() == 1"),
            "Quint project root must verify authored outcome inventory completeness"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

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
                "--source-kind",
                "original",
                "--transformation",
                "derivation",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "derivationRule := \"derivation from TicketCaptured.ticket_title\", derivationSourceFields := [\"ticket_title\",\"raw_title\"], absenceEvent := \"\", derivationScenarioName := \"Ticket title is normalized\""
            ),
            "MCP-authored read-model derivation source fields and semantics must be represented in the Lean artifact"
        );

        complete_contract_scenario_coverage(&temp_dir)?;

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

        complete_contract_scenario_coverage(&temp_dir)?;

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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains(
                "def sliceEvents : List SliceEventReference := [{ name := \"TicketCaptured\" }]"
            ),
            "Lean slice artifact must carry the authored event name"
        );
        assert!(
            lean.contains("def sliceStreams : List StreamDefinition := [{ name := \"tickets\" }]"),
            "Lean slice artifact must carry the authored stream"
        );
        assert!(
            lean.contains(
                "def sliceEventDefinitions : List EventDefinition := [{ name := \"TicketCaptured\", stream := \"tickets\", attributes := [{ name := \"ticket_title\", sourceKind := \"command_input\", sourceName := \"ticket_title\", sourceField := \"value\", generatedSourceKind := \"\", provenanceDescription := \"CaptureTicket.ticket_title\" }], observed := false, shared := false }]"
            ),
            "Lean slice artifact must carry the authored event definition"
        );
        assert!(
            quint.contains(
                "val sliceEvents: List[SliceEventReference] = [{ name: \"TicketCaptured\" }]"
            ),
            "Quint slice artifact must carry the authored event name"
        );
        assert!(
            quint.contains("val sliceStreams: List[StreamDefinition] = [{ name: \"tickets\" }]"),
            "Quint slice artifact must carry the authored stream"
        );
        assert!(
            quint.contains(
                "val sliceEventDefinitions: List[EventDefinition] = [{ name: \"TicketCaptured\", stream: \"tickets\", attributes: [{ name: \"ticket_title\", sourceKind: \"command_input\", sourceName: \"ticket_title\", sourceField: \"value\", generatedSourceKind: \"\", provenanceDescription: \"CaptureTicket.ticket_title\" }], observed: false, shared: false }]"
            ),
            "Quint slice artifact must carry the authored event definition"
        );
        assert!(
            lean_root.contains(
                "structure ModelCommand where\n  workflow : String\n  slice : String\n  command : String"
            ),
            "Lean project root must type authored commands as named records"
        );
        assert!(
            lean_root.contains(
                "def modelCommands : List ModelCommand := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", command := \"CaptureTicket\" }]"
            ),
            "Lean project root must carry the authored command inventory as named records"
        );
        assert!(
            lean_root
                .contains("theorem modelCommandsAreDeclared : modelCommands.length = 1 := rfl"),
            "Lean project root must prove authored command inventory cardinality"
        );
        assert!(
            quint_root.contains("type ModelCommand = { workflow: str, slice: str, command: str }"),
            "Quint project root must type the authored command inventory"
        );
        assert!(
            quint_root.contains("val modelCommands: List[ModelCommand] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", command: \"CaptureTicket\" }]"),
            "Quint project root must carry the authored command inventory"
        );
        assert!(
            quint_root.contains("val modelCommandsAreDeclared = modelCommands.length() == 1"),
            "Quint project root must verify authored command inventory cardinality"
        );
        assert!(
            lean_root.contains(
                "structure ModelStream where\n  workflow : String\n  slice : String\n  stream : String"
            ),
            "Lean project root must type the authored stream inventory as named records"
        );
        assert!(
            lean_root.contains(
                "def modelStreams : List ModelStream := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", stream := \"tickets\" }]"
            ),
            "Lean project root must carry the authored stream inventory"
        );
        assert!(
            lean_root.contains("theorem modelStreamsAreDeclared : modelStreams.length = 1 := rfl"),
            "Lean project root must prove authored stream inventory cardinality"
        );
        assert!(
            quint_root.contains("type ModelStream = { workflow: str, slice: str, stream: str }"),
            "Quint project root must type the authored stream inventory"
        );
        assert!(
            quint_root.contains("val modelStreams: List[ModelStream] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", stream: \"tickets\" }]"),
            "Quint project root must carry the authored stream inventory"
        );
        assert!(
            quint_root.contains("val modelStreamsAreDeclared = modelStreams.length() == 1"),
            "Quint project root must verify authored stream inventory cardinality"
        );
        assert!(
            lean_root.contains(
                "structure ModelEvent where\n  workflow : String\n  slice : String\n  event : String\n  stream : String"
            ),
            "Lean project root must type the authored event inventory as named records"
        );
        assert!(
            lean_root.contains(
                "def modelEvents : List ModelEvent := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", event := \"TicketCaptured\", stream := \"tickets\" }]"
            ),
            "Lean project root must carry the authored event inventory"
        );
        assert!(
            lean_root.contains(
                "structure ModelEventAttribute where\n  workflow : String\n  slice : String\n  event : String\n  attributeName : String\n  sourceKind : String\n  sourceName : String\n  sourceField : String\n  generatedSourceKind : String\n  provenance : String"
            ),
            "Lean project root must type authored event attributes as named records"
        );
        assert!(
            lean_root.contains(
                "def modelEventAttributes : List ModelEventAttribute := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", event := \"TicketCaptured\", attributeName := \"ticket_title\", sourceKind := \"command_input\", sourceName := \"ticket_title\", sourceField := \"value\", generatedSourceKind := \"\", provenance := \"CaptureTicket.ticket_title\" }]"
            ),
            "Lean project root must carry authored event attribute provenance"
        );
        assert!(
            lean_root.contains("theorem modelEventsAreDeclared : modelEvents.length = 1 := rfl"),
            "Lean project root must prove authored event inventory cardinality"
        );
        assert!(
            lean_root.contains(
                "theorem modelEventAttributesAreDeclared : modelEventAttributes.length = 1 := rfl"
            ),
            "Lean project root must prove authored event attribute provenance inventory completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelEvent = { workflow: str, slice: str, event: str, stream: str }"
            ),
            "Quint project root must type the authored event inventory"
        );
        assert!(
            quint_root.contains(
                "type ModelEventAttribute = { workflow: str, slice: str, event: str, attribute: str, sourceKind: str, sourceName: str, sourceField: str, generatedSourceKind: str, provenance: str }"
            ),
            "Quint project root must type authored event attribute provenance inventory entries"
        );
        assert!(
            quint_root.contains("val modelEvents: List[ModelEvent] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", event: \"TicketCaptured\", stream: \"tickets\" }]"),
            "Quint project root must carry the authored event inventory"
        );
        assert!(
            quint_root.contains("val modelEventAttributes: List[ModelEventAttribute] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", event: \"TicketCaptured\", attribute: \"ticket_title\", sourceKind: \"command_input\", sourceName: \"ticket_title\", sourceField: \"value\", generatedSourceKind: \"\", provenance: \"CaptureTicket.ticket_title\" }]"),
            "Quint project root must carry authored event attribute provenance"
        );
        assert!(
            quint_root.contains("val modelEventsAreDeclared = modelEvents.length() == 1"),
            "Quint project root must verify authored event inventory cardinality"
        );
        assert!(
            quint_root.contains(
                "val modelEventAttributesAreDeclared = modelEventAttributes.length() == 1"
            ),
            "Quint project root must verify authored event attribute provenance inventory completeness"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_event_definition_records_shared_events() -> Result<(), Box<dyn Error>> {
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
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_ticket_summary_display_flow(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketTagged",
                "--stream",
                "tickets",
                "--attribute",
                "tag",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "tagging_policy",
                "--attribute-source-field",
                "tag",
                "--generated-source-kind",
                "tagging_policy",
                "--attribute-provenance",
                "TicketTagged.tag",
                "--shared",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "added event TicketTagged to slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "add",
                "data-flow",
                "--slice",
                "capture-ticket",
                "--datum",
                "tag",
                "--source",
                "tagging_policy.tag",
                "--source-kind",
                "original",
                "--transformation",
                "identity",
                "--target",
                "TicketTagged",
                "--bit-encoding",
                "UTF-8 string",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("name := \"TicketTagged\", stream := \"tickets\"")
                && lean.contains("observed := false, shared := true"),
            "Lean slice artifact must carry authored shared event semantics"
        );
        assert!(
            quint.contains("name: \"TicketTagged\", stream: \"tickets\"")
                && quint.contains("observed: false, shared: true"),
            "Quint slice artifact must carry authored shared event semantics"
        );

        complete_contract_scenario_coverage(&temp_dir)?;

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
                "--generated-source-kind",
                "upstream_event_store",
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
            ))
            .stdout(predicate::str::contains(
                "added read model ticket_state to project root",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains(
                "def sliceReadModels : List SliceReadModelReference := [{ name := \"ticket_state\" }]"
            ),
            "Lean slice artifact must carry the authored read model name"
        );
        assert!(
            lean.contains(
                "def sliceReadModelDefinitions : List ReadModelDefinition := [{ name := \"ticket_state\", fields := [{ name := \"ticket_title\", sourceKind := \"event_attribute\", sourceEvent := \"TicketCaptured\", sourceAttribute := \"ticket_title\", derivationRule := \"\", derivationSourceFields := [], absenceEvent := \"\", derivationScenarioName := \"\", absenceScenarioName := \"\", provenanceDescription := \"TicketCaptured.ticket_title\" }], transitive := false, relationshipFields := [], transitiveRule := \"\", exampleScenarioName := \"\" }]"
            ),
            "Lean slice artifact must carry the authored read model field source and provenance"
        );
        assert!(
            quint.contains(
                "val sliceReadModels: List[SliceReadModelReference] = [{ name: \"ticket_state\" }]"
            ),
            "Quint slice artifact must carry the authored read model name"
        );
        assert!(
            quint.contains(
                "val sliceReadModelDefinitions: List[ReadModelDefinition] = [{ name: \"ticket_state\", fields: [{ name: \"ticket_title\", sourceKind: \"event_attribute\", sourceEvent: \"TicketCaptured\", sourceAttribute: \"ticket_title\", derivationRule: \"\", derivationSourceFields: [], absenceEvent: \"\", derivationScenarioName: \"\", absenceScenarioName: \"\", provenanceDescription: \"TicketCaptured.ticket_title\" }], transitive: false, relationshipFields: [], transitiveRule: \"\", exampleScenarioName: \"\" }]"
            ),
            "Quint slice artifact must carry the authored read model field source and provenance"
        );
        assert!(
            lean_root.contains(
                "def modelReadModels : List ModelReadModel := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", readModel := \"ticket_state\" }]"
            ),
            "Lean project root must carry the authored read model inventory"
        );
        assert!(
            lean_root
                .contains("theorem modelReadModelsAreDeclared : modelReadModels.length = 1 := rfl"),
            "Lean project root must prove authored read models are declared"
        );
        assert!(
            lean_root.contains(
                "def modelReadModelFields : List ModelReadModelField := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", readModel := \"ticket_state\", field := \"ticket_title\", sourceKind := \"event_attribute\", sourceEvent := \"TicketCaptured\", sourceAttribute := \"ticket_title\", derivationRule := \"\", derivationSourceFields := [], absenceEvent := \"\", derivationScenarioName := \"\", absenceScenarioName := \"\", provenance := \"TicketCaptured.ticket_title\" }]"
            ),
            "Lean project root must carry read model field source, scenario, absence, and provenance facts"
        );
        assert!(
            lean_root.contains(
                "theorem modelReadModelFieldsAreDeclared : modelReadModelFields.length = 1 := rfl"
            ),
            "Lean project root must prove authored read model fields are declared"
        );
        assert!(
            quint_root
                .contains("type ModelReadModel = { workflow: str, slice: str, readModel: str }"),
            "Quint project root must type authored read model inventory"
        );
        assert!(
            quint_root.contains(
                "val modelReadModels: List[ModelReadModel] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", readModel: \"ticket_state\" }]"
            ),
            "Quint project root must carry the authored read model inventory"
        );
        assert!(
            quint_root.contains("val modelReadModelsAreDeclared = modelReadModels.length() == 1"),
            "Quint project root must verify authored read models are declared"
        );
        assert!(
            quint_root.contains(
                "type ModelReadModelField = { workflow: str, slice: str, readModel: str, field: str, sourceKind: str, sourceEvent: str, sourceAttribute: str, derivationRule: str, derivationSourceFields: List[str], absenceEvent: str, derivationScenarioName: str, absenceScenarioName: str, provenance: str }"
            ),
            "Quint project root must type authored read model field inventory"
        );
        assert!(
            quint_root.contains(
                "val modelReadModelFields: List[ModelReadModelField] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", readModel: \"ticket_state\", field: \"ticket_title\", sourceKind: \"event_attribute\", sourceEvent: \"TicketCaptured\", sourceAttribute: \"ticket_title\", derivationRule: \"\", derivationSourceFields: [], absenceEvent: \"\", derivationScenarioName: \"\", absenceScenarioName: \"\", provenance: \"TicketCaptured.ticket_title\" }]"
            ),
            "Quint project root must carry read model field source, scenario, absence, and provenance facts"
        );
        assert!(
            quint_root.contains(
                "val modelReadModelFieldsAreDeclared = modelReadModelFields.length() == 1"
            ),
            "Quint project root must verify authored read model fields are declared"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

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
                "derivation from TicketCaptured.ticket_title",
                "--source-fields",
                "ticket_title,raw_title",
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
                "--source-kind",
                "original",
                "--transformation",
                "derivation",
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
                "--source-kind",
                "original",
                "--transformation",
                "default",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "{ name := \"normalized_title\", sourceKind := \"derivation\", sourceEvent := \"\", sourceAttribute := \"\", derivationRule := \"derivation from TicketCaptured.ticket_title\", derivationSourceFields := [\"ticket_title\",\"raw_title\"], absenceEvent := \"\", derivationScenarioName := \"Ticket title is normalized\", absenceScenarioName := \"\", provenanceDescription := \"TicketCaptured.ticket_title -> trim\" }"
            ),
            "Lean slice artifact must carry read-model derivation source fields and semantics"
        );
        assert!(
            lean.contains(
                "{ name := \"has_ticket\", sourceKind := \"absence_default\", sourceEvent := \"\", sourceAttribute := \"\", derivationRule := \"\", derivationSourceFields := [], absenceEvent := \"TicketCaptured\", derivationScenarioName := \"\", absenceScenarioName := \"Ticket state defaults before capture\", provenanceDescription := \"absence of TicketCaptured in tickets stream\" }"
            ),
            "Lean slice artifact must carry read-model absence/default semantics"
        );
        assert!(
            quint.contains(
                "{ name: \"normalized_title\", sourceKind: \"derivation\", sourceEvent: \"\", sourceAttribute: \"\", derivationRule: \"derivation from TicketCaptured.ticket_title\", derivationSourceFields: [\"ticket_title\",\"raw_title\"], absenceEvent: \"\", derivationScenarioName: \"Ticket title is normalized\", absenceScenarioName: \"\", provenanceDescription: \"TicketCaptured.ticket_title -> trim\" }"
            ),
            "Quint slice artifact must carry read-model derivation source fields and semantics"
        );
        assert!(
            quint.contains(
                "{ name: \"has_ticket\", sourceKind: \"absence_default\", sourceEvent: \"\", sourceAttribute: \"\", derivationRule: \"\", derivationSourceFields: [], absenceEvent: \"TicketCaptured\", derivationScenarioName: \"\", absenceScenarioName: \"Ticket state defaults before capture\", provenanceDescription: \"absence of TicketCaptured in tickets stream\" }"
            ),
            "Quint slice artifact must carry read-model absence/default semantics"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains("def sliceReadModelDefinitions : List ReadModelDefinition := [{ name := \"ticket_hierarchy\", fields := [{ name := \"ancestor_ticket_id\", sourceKind := \"event_attribute\", sourceEvent := \"TicketLinked\", sourceAttribute := \"parent_ticket_id\", derivationRule := \"\", derivationSourceFields := [], absenceEvent := \"\", derivationScenarioName := \"\", absenceScenarioName := \"\", provenanceDescription := \"TicketLinked.parent_ticket_id\" }], transitive := true, relationshipFields := [\"parent_ticket_id\",\"child_ticket_id\"], transitiveRule := \"walk TicketLinked parent_ticket_id edges until root\", exampleScenarioName := \"Ticket hierarchy includes grandchild\" }]"),
            "Lean slice artifact must carry transitive read-model semantics"
        );
        assert!(
            quint.contains("val sliceReadModelDefinitions: List[ReadModelDefinition] = [{ name: \"ticket_hierarchy\", fields: [{ name: \"ancestor_ticket_id\", sourceKind: \"event_attribute\", sourceEvent: \"TicketLinked\", sourceAttribute: \"parent_ticket_id\", derivationRule: \"\", derivationSourceFields: [], absenceEvent: \"\", derivationScenarioName: \"\", absenceScenarioName: \"\", provenanceDescription: \"TicketLinked.parent_ticket_id\" }], transitive: true, relationshipFields: [\"parent_ticket_id\",\"child_ticket_id\"], transitiveRule: \"walk TicketLinked parent_ticket_id edges until root\", exampleScenarioName: \"Ticket hierarchy includes grandchild\" }]"),
            "Quint slice artifact must carry transitive read-model semantics"
        );
        assert!(
            lean_root.contains(
                "def modelReadModelDefinitions : List ModelReadModelDefinition := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", readModel := \"ticket_hierarchy\", transitive := true, relationshipFields := [\"parent_ticket_id\",\"child_ticket_id\"], transitiveRule := \"walk TicketLinked parent_ticket_id edges until root\", exampleScenarioName := \"Ticket hierarchy includes grandchild\" }]"
            ),
            "Lean project root must carry transitive read-model relationship, rule, and example semantics"
        );
        assert!(
            lean_root.contains(
                "theorem modelReadModelDefinitionsAreDeclared : modelReadModelDefinitions.length = 1 := rfl"
            ),
            "Lean project root must prove authored read-model definitions are declared"
        );
        assert!(
            quint_root.contains(
                "type ModelReadModelDefinition = { workflow: str, slice: str, readModel: str, transitive: bool, relationshipFields: List[str], transitiveRule: str, exampleScenarioName: str }"
            ),
            "Quint project root must type read-model definition semantics"
        );
        assert!(
            quint_root.contains(
                "val modelReadModelDefinitions: List[ModelReadModelDefinition] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", readModel: \"ticket_hierarchy\", transitive: true, relationshipFields: [\"parent_ticket_id\",\"child_ticket_id\"], transitiveRule: \"walk TicketLinked parent_ticket_id edges until root\", exampleScenarioName: \"Ticket hierarchy includes grandchild\" }]"
            ),
            "Quint project root must carry transitive read-model relationship, rule, and example semantics"
        );
        assert!(
            quint_root.contains(
                "val modelReadModelDefinitionsAreDeclared = modelReadModelDefinitions.length() == 1"
            ),
            "Quint project root must verify authored read-model definitions are declared"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

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
                "--generated-source-kind",
                "upstream_event_store",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
            ))
            .stdout(predicate::str::contains(
                "added view ticket_summary to project root",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains(
                "def sliceViews : List SliceViewReference := [{ name := \"ticket_summary\" }]"
            ),
            "Lean slice artifact must carry the authored view name"
        );
        assert!(
            lean.contains(
                "def sliceViewDefinitions : List ViewDefinition := [{ name := \"ticket_summary\", readModels := [\"ticket_state\"], fields := [{ name := \"ticket_title\", sourceKind := \"read_model\", sourceReadModel := \"ticket_state\", sourceField := \"ticket_title\", sketchToken := \"title-label\", provenanceDescription := \"ticket_state.ticket_title\", bitEncoding := \"UTF-8 string\" }], controls := [], sketchTokens := [\"title-label\"], localStates := [], filters := [] }]"
            ),
            "Lean slice artifact must carry the authored displayed datum source and sketch token"
        );
        assert!(
            quint.contains(
                "val sliceViews: List[SliceViewReference] = [{ name: \"ticket_summary\" }]"
            ),
            "Quint slice artifact must carry the authored view name"
        );
        assert!(
            quint.contains(
                "val sliceViewDefinitions: List[ViewDefinition] = [{ name: \"ticket_summary\", readModels: [\"ticket_state\"], fields: [{ name: \"ticket_title\", sourceKind: \"read_model\", sourceReadModel: \"ticket_state\", sourceField: \"ticket_title\", sketchToken: \"title-label\", provenanceDescription: \"ticket_state.ticket_title\", bitEncoding: \"UTF-8 string\" }], controls: [], sketchTokens: [\"title-label\"], localStates: [], filters: [] }]"
            ),
            "Quint slice artifact must carry the authored displayed datum source and sketch token"
        );
        assert!(
            lean_root.contains(
                "def modelViews : List ModelView := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", view := \"ticket_summary\" }]"
            ),
            "Lean project root must carry the authored view inventory"
        );
        assert!(
            lean_root.contains("theorem modelViewsAreDeclared : modelViews.length = 1 := rfl"),
            "Lean project root must prove authored views are declared"
        );
        assert!(
            lean_root.contains(
                "def modelViewDefinitions : List ModelViewDefinition := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", view := \"ticket_summary\", readModels := [\"ticket_state\"], sketchTokens := [\"title-label\"], localStates := [], filters := [] }]"
            ),
            "Lean project root must carry authored view read model and sketch token definitions"
        );
        assert!(
            lean_root.contains(
                "theorem modelViewDefinitionsAreDeclared : modelViewDefinitions.length = 1 := rfl"
            ),
            "Lean project root must prove authored view definitions are declared"
        );
        assert!(
            lean_root.contains(
                "def modelViewFields : List ModelViewField := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", view := \"ticket_summary\", field := \"ticket_title\", sourceKind := \"read_model\", sourceReadModel := \"ticket_state\", sourceField := \"ticket_title\", provenance := \"ticket_state.ticket_title\", bitEncoding := \"UTF-8 string\" }]"
            ),
            "Lean project root must carry displayed datum source, provenance, and bit encoding"
        );
        assert!(
            lean_root
                .contains("theorem modelViewFieldsAreDeclared : modelViewFields.length = 1 := rfl"),
            "Lean project root must prove authored displayed data are declared"
        );
        assert!(
            quint_root.contains("type ModelView = { workflow: str, slice: str, view: str }"),
            "Quint project root must type authored view inventory"
        );
        assert!(
            quint_root.contains(
                "type ModelViewField = { workflow: str, slice: str, view: str, field: str, sourceKind: str, sourceReadModel: str, sourceField: str, provenance: str, bitEncoding: str }"
            ),
            "Quint project root must type displayed datum source inventory entries"
        );
        assert!(
            quint_root.contains(
                "type ModelViewDefinition = { workflow: str, slice: str, view: str, readModels: List[str], sketchTokens: List[str], localStates: List[str], filters: List[str] }"
            ),
            "Quint project root must type authored view definitions"
        );
        assert!(
            quint_root.contains(
                "val modelViews: List[ModelView] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", view: \"ticket_summary\" }]"
            ),
            "Quint project root must carry the authored view inventory"
        );
        assert!(
            quint_root.contains(
                "val modelViewDefinitions: List[ModelViewDefinition] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", view: \"ticket_summary\", readModels: [\"ticket_state\"], sketchTokens: [\"title-label\"], localStates: [], filters: [] }]"
            ),
            "Quint project root must carry authored view read model and sketch token definitions"
        );
        assert!(
            quint_root.contains(
                "val modelViewFields: List[ModelViewField] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", view: \"ticket_summary\", field: \"ticket_title\", sourceKind: \"read_model\", sourceReadModel: \"ticket_state\", sourceField: \"ticket_title\", provenance: \"ticket_state.ticket_title\", bitEncoding: \"UTF-8 string\" }]"
            ),
            "Quint project root must carry displayed datum source, provenance, and bit encoding"
        );
        assert!(
            quint_root.contains("val modelViewsAreDeclared = modelViews.length() == 1"),
            "Quint project root must verify authored views are declared"
        );
        assert!(
            quint_root.contains(
                "val modelViewDefinitionsAreDeclared = modelViewDefinitions.length() == 1"
            ),
            "Quint project root must verify authored view definitions are declared"
        );
        assert!(
            quint_root.contains("val modelViewFieldsAreDeclared = modelViewFields.length() == 1"),
            "Quint project root must verify authored displayed data are declared"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains(
                "def sliceReferencedCommands : List SliceCommandReference := [{ name := \"CaptureTicket\" }]"
            ),
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
            quint.contains(
                "val sliceReferencedCommands: List[SliceCommandReference] = [{ name: \"CaptureTicket\" }]"
            ),
            "Quint slice artifact must carry cross-slice command references for controls"
        );
        assert!(
            quint.contains(
                "controls: [{ name: \"submit-ticket\", commandName: \"CaptureTicket\", inputs: [{ name: \"ticket_title\", sourceKind: \"actor\", sourceDescription: \"title field on the intake form\", sketchToken: \"title-input\", visibleToActor: true, decisionField: true }], handledErrors: [\"DuplicateTicket\"], recoveryBehavior: \"retry\", sketchToken: \"submit-button\", navigation: { targetType: \"modeled_view\", targetName: \"ticket_summary\", externalWorkflowName: \"\", externalSystemName: \"\", handoffContract: \"\" } }]"
            ),
            "Quint slice artifact must carry authored control input, error handling, and navigation"
        );
        assert!(
            lean_root.contains(
                "def modelViewControls : List ModelViewControl := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", view := \"ticket_summary\", control := \"submit-ticket\", command := \"CaptureTicket\", input := \"ticket_title\", inputSourceKind := \"actor\", inputSourceDescription := \"title field on the intake form\", inputSketchToken := \"title-input\", inputVisibleToActor := true, inputDecisionField := true, handledErrors := [\"DuplicateTicket\"], recoveryBehavior := \"retry\", controlSketchToken := \"submit-button\", navigationType := \"modeled_view\", navigationTarget := \"ticket_summary\", externalWorkflow := \"\", externalSystem := \"\", handoffContract := \"\" }]"
            ),
            "Lean project root must inventory authored control input, error handling, and navigation"
        );
        assert!(
            lean_root.contains(
                "theorem modelViewControlsAreDeclared : modelViewControls.length = 1 := rfl"
            ),
            "Lean project root must prove authored view controls are declared"
        );
        assert!(
            quint_root.contains(
                "type ModelViewControl = { workflow: str, slice: str, view: str, control: str, command: str, input: str, inputSourceKind: str, inputSourceDescription: str, inputSketchToken: str, inputVisibleToActor: bool, inputDecisionField: bool, handledErrors: List[str], recoveryBehavior: str, controlSketchToken: str, navigationType: str, navigationTarget: str, externalWorkflow: str, externalSystem: str, handoffContract: str }"
            ),
            "Quint project root must type authored view control inventory entries"
        );
        assert!(
            quint_root.contains(
                "val modelViewControls: List[ModelViewControl] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", view: \"ticket_summary\", control: \"submit-ticket\", command: \"CaptureTicket\", input: \"ticket_title\", inputSourceKind: \"actor\", inputSourceDescription: \"title field on the intake form\", inputSketchToken: \"title-input\", inputVisibleToActor: true, inputDecisionField: true, handledErrors: [\"DuplicateTicket\"], recoveryBehavior: \"retry\", controlSketchToken: \"submit-button\", navigationType: \"modeled_view\", navigationTarget: \"ticket_summary\", externalWorkflow: \"\", externalSystem: \"\", handoffContract: \"\" }]"
            ),
            "Quint project root must inventory authored control input, error handling, and navigation"
        );
        assert!(
            quint_root
                .contains("val modelViewControlsAreDeclared = modelViewControls.length() == 1"),
            "Quint project root must verify authored view controls are declared"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_view_definition_records_local_state_navigation_targets() -> Result<(), Box<dyn Error>> {
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
                "expand-details",
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
                "expand-button",
                "--navigation-type",
                "local_view_state",
                "--navigation-target",
                "details-expanded",
                "--local-states",
                "details-expanded",
                "--filters",
                "open-only",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;
        assert!(
            lean.contains("localStates := [\"details-expanded\"], filters := [\"open-only\"]"),
            "Lean slice artifact must carry authored local view state and filter declarations"
        );
        assert!(
            lean.contains("navigation := { targetType := \"local_view_state\", targetName := \"details-expanded\""),
            "Lean slice artifact must carry local-view-state navigation targets"
        );
        assert!(
            quint.contains("localStates: [\"details-expanded\"], filters: [\"open-only\"]"),
            "Quint slice artifact must carry authored local view state and filter declarations"
        );
        assert!(
            quint.contains(
                "navigation: { targetType: \"local_view_state\", targetName: \"details-expanded\""
            ),
            "Quint slice artifact must carry local-view-state navigation targets"
        );
        assert!(
            lean_root.contains(
                "def modelViewDefinitions : List ModelViewDefinition := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", view := \"ticket_summary\", readModels := [\"ticket_state\"], sketchTokens := [\"title-label\"], localStates := [\"details-expanded\"], filters := [\"open-only\"] }]"
            ),
            "Lean project root must inventory authored local view state and filter declarations"
        );
        assert!(
            quint_root.contains(
                "val modelViewDefinitions: List[ModelViewDefinition] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", view: \"ticket_summary\", readModels: [\"ticket_state\"], sketchTokens: [\"title-label\"], localStates: [\"details-expanded\"], filters: [\"open-only\"] }]"
            ),
            "Quint project root must inventory authored local view state and filter declarations"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

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

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn add_view_definition_records_external_workflow_navigation_targets()
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
                "open-billing-workflow",
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
                "billing-link",
                "--navigation-type",
                "external_workflow",
                "--navigation-target",
                "billing-intake",
                "--external-workflow",
                "billing-intake",
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
            lean.contains("navigation := { targetType := \"external_workflow\", targetName := \"billing-intake\", externalWorkflowName := \"billing-intake\""),
            "Lean slice artifact must carry external-workflow navigation metadata"
        );
        assert!(
            quint.contains("navigation: { targetType: \"external_workflow\", targetName: \"billing-intake\", externalWorkflowName: \"billing-intake\""),
            "Quint slice artifact must carry external-workflow navigation metadata"
        );

        complete_contract_scenario_coverage(&temp_dir)?;

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

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_external_workflow_navigation_targets() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;
        author_projected_ticket_title(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_external_workflow_view_control_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_view_definition\""))
            .stdout(predicate::str::contains(
                "added view ticket_summary to slice capture-ticket",
            ));

        complete_ticket_summary_display_flow(&temp_dir)?;

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("externalWorkflowName := \"billing-intake\""),
            "MCP-authored external-workflow navigation metadata must be represented in the Lean artifact"
        );
        assert!(
            quint.contains("externalWorkflowName: \"billing-intake\""),
            "MCP-authored external-workflow navigation metadata must be represented in the Quint artifact"
        );

        complete_contract_scenario_coverage(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["verify"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn mcp_stdio_authors_local_state_navigation_targets() -> Result<(), Box<dyn Error>> {
        let temp_dir = initialized_project_with_slice()?;
        author_projected_ticket_title(&temp_dir)?;

        Command::cargo_bin("emc")?
            .args(["mcp", "stdio"])
            .current_dir(temp_dir.path())
            .write_stdin(mcp_local_state_view_control_requests())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"add_view_definition\""))
            .stdout(predicate::str::contains(
                "added view ticket_summary to slice capture-ticket",
            ));

        complete_ticket_summary_display_flow(&temp_dir)?;

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            lean.contains("localStates := [\"details-expanded\"], filters := [\"open-only\"]"),
            "MCP-authored local view states and filters must be represented in the Lean artifact"
        );
        assert!(
            quint.contains("localStates: [\"details-expanded\"], filters: [\"open-only\"]"),
            "MCP-authored local view states and filters must be represented in the Quint artifact"
        );

        complete_contract_scenario_coverage(&temp_dir)?;

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
            ))
            .stdout(predicate::str::contains(
                "added automation title-deduplicator to project root",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains(
                "def sliceReferencedCommands : List SliceCommandReference := [{ name := \"CaptureTicket\" }]"
            ),
            "Lean slice artifact must record commands issued by authored automations"
        );
        assert!(
            lean.contains(
                "def sliceAutomations : List AutomationDefinition := [{ name := \"title-deduplicator\", triggerName := \"TicketCaptured\", commandName := \"CaptureTicket\", handledErrors := [\"DuplicateTicket\"], reactionDescription := \"deduplicates captured titles by reissuing CaptureTicket when needed\" }]"
            ),
            "Lean slice artifact must carry authored automation definitions"
        );
        assert!(
            quint.contains(
                "val sliceReferencedCommands: List[SliceCommandReference] = [{ name: \"CaptureTicket\" }]"
            ),
            "Quint slice artifact must record commands issued by authored automations"
        );
        assert!(
            quint.contains(
                "val sliceAutomations: List[AutomationDefinition] = [{ name: \"title-deduplicator\", triggerName: \"TicketCaptured\", commandName: \"CaptureTicket\", handledErrors: [\"DuplicateTicket\"], reactionDescription: \"deduplicates captured titles by reissuing CaptureTicket when needed\" }]"
            ),
            "Quint slice artifact must carry authored automation definitions"
        );
        assert!(
            lean_root.contains(
                "def modelAutomations : List ModelAutomation := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", automation := \"title-deduplicator\" }]"
            ),
            "Lean project root must carry the authored automation inventory"
        );
        assert!(
            lean_root.contains(
                "def modelAutomationDefinitions : List ModelAutomationDefinition := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", automation := \"title-deduplicator\", trigger := \"TicketCaptured\", command := \"CaptureTicket\", handledErrors := [\"DuplicateTicket\"], reaction := \"deduplicates captured titles by reissuing CaptureTicket when needed\" }]"
            ),
            "Lean project root must carry authored automation definitions"
        );
        assert!(
            lean_root.contains(
                "theorem modelAutomationsAreDeclared : modelAutomations.length = 1 := rfl"
            ),
            "Lean project root must prove authored automations are declared"
        );
        assert!(
            lean_root.contains(
                "theorem modelAutomationDefinitionsAreDeclared : modelAutomationDefinitions.length = 1 := rfl"
            ),
            "Lean project root must prove authored automation definitions are declared"
        );
        assert!(
            quint_root
                .contains("type ModelAutomation = { workflow: str, slice: str, automation: str }"),
            "Quint project root must type authored automation inventory"
        );
        assert!(
            quint_root.contains(
                "type ModelAutomationDefinition = { workflow: str, slice: str, automation: str, trigger: str, command: str, handledErrors: List[str], reaction: str }"
            ),
            "Quint project root must type authored automation definitions"
        );
        assert!(
            quint_root.contains(
                "val modelAutomations: List[ModelAutomation] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", automation: \"title-deduplicator\" }]"
            ),
            "Quint project root must carry the authored automation inventory"
        );
        assert!(
            quint_root.contains(
                "val modelAutomationDefinitions: List[ModelAutomationDefinition] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", automation: \"title-deduplicator\", trigger: \"TicketCaptured\", command: \"CaptureTicket\", handledErrors: [\"DuplicateTicket\"], reaction: \"deduplicates captured titles by reissuing CaptureTicket when needed\" }]"
            ),
            "Quint project root must carry authored automation definitions"
        );
        assert!(
            quint_root.contains("val modelAutomationsAreDeclared = modelAutomations.length() == 1"),
            "Quint project root must verify authored automations are declared"
        );
        assert!(
            quint_root.contains(
                "val modelAutomationDefinitionsAreDeclared = modelAutomationDefinitions.length() == 1"
            ),
            "Quint project root must verify authored automation definitions are declared"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();
        complete_contract_scenario_coverage(&temp_dir)?;
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
            ))
            .stdout(predicate::str::contains(
                "added translation intake-webhook-translator to project root",
            ));

        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

        assert!(
            lean.contains(
                "def sliceReferencedCommands : List SliceCommandReference := [{ name := \"CaptureTicket\" }]"
            ),
            "Lean slice artifact must record commands targeted by authored translations"
        );
        assert!(
            lean.contains(
                "def sliceTranslations : List TranslationDefinition := [{ name := \"intake-webhook-translator\", externalEventName := \"intake_webhook_received\", payloadContractName := \"intake_webhook\", commandName := \"CaptureTicket\" }]"
            ),
            "Lean slice artifact must carry authored translation definitions"
        );
        assert!(
            quint.contains(
                "val sliceReferencedCommands: List[SliceCommandReference] = [{ name: \"CaptureTicket\" }]"
            ),
            "Quint slice artifact must record commands targeted by authored translations"
        );
        assert!(
            quint.contains(
                "val sliceTranslations: List[TranslationDefinition] = [{ name: \"intake-webhook-translator\", externalEventName: \"intake_webhook_received\", payloadContractName: \"intake_webhook\", commandName: \"CaptureTicket\" }]"
            ),
            "Quint slice artifact must carry authored translation definitions"
        );
        assert!(
            lean_root.contains(
                "def modelTranslations : List ModelTranslation := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", translation := \"intake-webhook-translator\" }]"
            ),
            "Lean project root must inventory authored translations"
        );
        assert!(
            lean_root.contains(
                "def modelTranslationDefinitions : List ModelTranslationDefinition := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", translation := \"intake-webhook-translator\", externalEvent := \"intake_webhook_received\", payloadContract := \"intake_webhook\", command := \"CaptureTicket\" }]"
            ),
            "Lean project root must carry authored translation definitions"
        );
        assert!(
            lean_root.contains(
                "theorem modelTranslationsAreDeclared : modelTranslations.length = 1 := rfl"
            ),
            "Lean project root must prove authored translation inventory completeness"
        );
        assert!(
            lean_root.contains(
                "theorem modelTranslationDefinitionsAreDeclared : modelTranslationDefinitions.length = 1 := rfl"
            ),
            "Lean project root must prove authored translation definition completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelTranslation = { workflow: str, slice: str, translation: str }"
            ),
            "Quint project root must type authored translation inventory entries"
        );
        assert!(
            quint_root.contains(
                "type ModelTranslationDefinition = { workflow: str, slice: str, translation: str, externalEvent: str, payloadContract: str, command: str }"
            ),
            "Quint project root must type authored translation definitions"
        );
        assert!(
            quint_root.contains(
                "val modelTranslations: List[ModelTranslation] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", translation: \"intake-webhook-translator\" }]"
            ),
            "Quint project root must inventory authored translations"
        );
        assert!(
            quint_root.contains(
                "val modelTranslationDefinitions: List[ModelTranslationDefinition] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", translation: \"intake-webhook-translator\", externalEvent: \"intake_webhook_received\", payloadContract: \"intake_webhook\", command: \"CaptureTicket\" }]"
            ),
            "Quint project root must carry authored translation definitions"
        );
        assert!(
            quint_root
                .contains("val modelTranslationsAreDeclared = modelTranslations.length() == 1"),
            "Quint project root must verify authored translation inventory completeness"
        );
        assert!(
            quint_root.contains(
                "val modelTranslationDefinitionsAreDeclared = modelTranslationDefinitions.length() == 1"
            ),
            "Quint project root must verify authored translation definition completeness"
        );
        assert_project_root_digests_are_canonical_hashes(&lean_root, &quint_root)?;

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
                "--source-kind",
                "original",
                "--transformation",
                "transformation",
                "--target",
                "intake_webhook",
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
        complete_contract_scenario_coverage(&temp_dir)?;
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
        let lean_root = read_to_string(temp_dir.path().join("model/lean/RepairDesk.lean"))?;
        let quint_root = read_to_string(temp_dir.path().join("model/quint/RepairDesk.qnt"))?;

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
        assert!(
            lean_root.contains(
                "def modelBoardElements : List ModelBoardElement := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", element := \"CaptureTicket\", kind := \"command\", lane := \"actions\", declaredName := \"CaptureTicket\", mainPath := true },{ workflow := \"open-ticket\", slice := \"capture-ticket\", element := \"TicketCaptured\", kind := \"event\", lane := \"events\", declaredName := \"TicketCaptured\", mainPath := true }]"
            ),
            "Lean project root must inventory authored board elements"
        );
        assert!(
            lean_root.contains(
                "def modelBoardConnections : List ModelBoardConnection := [{ workflow := \"open-ticket\", slice := \"capture-ticket\", source := \"CaptureTicket\", sourceKind := \"command\", target := \"TicketCaptured\", targetKind := \"event\" },{ workflow := \"open-ticket\", slice := \"capture-ticket\", source := \"actor-submit\", sourceKind := \"workflow_trigger\", target := \"CaptureTicket\", targetKind := \"command\" }]"
            ),
            "Lean project root must inventory authored board connections"
        );
        assert!(
            lean_root.contains(
                "theorem modelBoardElementsAreDeclared : modelBoardElements.length = 2 := rfl"
            ),
            "Lean project root must prove authored board element inventory completeness"
        );
        assert!(
            lean_root.contains(
                "theorem modelBoardConnectionsAreDeclared : modelBoardConnections.length = 2 := rfl"
            ),
            "Lean project root must prove authored board connection inventory completeness"
        );
        assert!(
            quint_root.contains(
                "type ModelBoardElement = { workflow: str, slice: str, element: str, kind: str, lane: str, declaredName: str, mainPath: bool }"
            ),
            "Quint project root must type authored board element inventory entries"
        );
        assert!(
            quint_root.contains(
                "type ModelBoardConnection = { workflow: str, slice: str, source: str, sourceKind: str, target: str, targetKind: str }"
            ),
            "Quint project root must type authored board connection inventory entries"
        );
        assert!(
            quint_root.contains(
                "val modelBoardElements: List[ModelBoardElement] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", element: \"CaptureTicket\", kind: \"command\", lane: \"actions\", declaredName: \"CaptureTicket\", mainPath: true },{ workflow: \"open-ticket\", slice: \"capture-ticket\", element: \"TicketCaptured\", kind: \"event\", lane: \"events\", declaredName: \"TicketCaptured\", mainPath: true }]"
            ),
            "Quint project root must inventory authored board elements"
        );
        assert!(
            quint_root.contains(
                "val modelBoardConnections: List[ModelBoardConnection] = [{ workflow: \"open-ticket\", slice: \"capture-ticket\", source: \"CaptureTicket\", sourceKind: \"command\", target: \"TicketCaptured\", targetKind: \"event\" },{ workflow: \"open-ticket\", slice: \"capture-ticket\", source: \"actor-submit\", sourceKind: \"workflow_trigger\", target: \"CaptureTicket\", targetKind: \"command\" }]"
            ),
            "Quint project root must inventory authored board connections"
        );
        assert!(
            quint_root
                .contains("val modelBoardElementsAreDeclared = modelBoardElements.length() == 2"),
            "Quint project root must expose authored board element inventory invariant"
        );
        assert!(
            quint_root.contains(
                "val modelBoardConnectionsAreDeclared = modelBoardConnections.length() == 2"
            ),
            "Quint project root must expose authored board connection inventory invariant"
        );

        Command::cargo_bin("emc")?
            .args(["check"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        complete_contract_scenario_coverage(&temp_dir)?;

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

        complete_contract_scenario_coverage(&temp_dir)?;

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

    fn complete_contract_scenario_coverage(temp_dir: &TempDir) -> Result<(), Box<dyn Error>> {
        let lean = read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;

        if lean.contains(
            "def sliceCommandDefinitions : List CommandDefinition := [{ name := \"CaptureTicket\"",
        ) && !lean
            .contains("contractKind := \"command\", coveredDefinition := \"CaptureTicket\"")
        {
            author_capture_ticket_command_contract(temp_dir, &lean)?;
        }

        if lean.contains(
            "def sliceAutomations : List AutomationDefinition := [{ name := \"title-deduplicator\"",
        ) && !lean
            .contains("contractKind := \"automation\", coveredDefinition := \"title-deduplicator\"")
        {
            Command::cargo_bin("emc")?
                .args([
                    "add",
                    "scenario",
                    "--slice",
                    "capture-ticket",
                    "--kind",
                    "contract",
                    "--name",
                    "Title deduplicator handles duplicate tickets",
                    "--given",
                    "TicketCaptured was emitted",
                    "--when",
                    "title-deduplicator reacts to the event",
                    "--then",
                    "CaptureTicket is issued for the duplicate title",
                    "--contract-kind",
                    "automation",
                    "--covered-definition",
                    "title-deduplicator",
                ])
                .current_dir(temp_dir.path())
                .assert()
                .success();
        }

        if lean.contains(
            "def sliceTranslations : List TranslationDefinition := [{ name := \"intake-webhook-translator\"",
        ) && !lean.contains(
            "contractKind := \"translation\", coveredDefinition := \"intake-webhook-translator\"",
        ) {
            Command::cargo_bin("emc")?
                .args([
                    "add",
                    "scenario",
                    "--slice",
                    "capture-ticket",
                    "--kind",
                    "contract",
                    "--name",
                    "Intake webhook translates to CaptureTicket",
                    "--given",
                    "intake_webhook_received carries intake_webhook",
                    "--when",
                    "intake-webhook-translator handles the external event",
                    "--then",
                    "CaptureTicket receives translated payload fields",
                    "--contract-kind",
                    "translation",
                    "--covered-definition",
                    "intake-webhook-translator",
                ])
                .current_dir(temp_dir.path())
                .assert()
                .success();
        }

        if lean.contains("name := \"normalized_title\", sourceKind := \"derivation\"")
            && !lean.contains("contractKind := \"derivation\"")
        {
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
                    "TicketCaptured carries raw title text",
                    "--when",
                    "ticket_state derives normalized_title",
                    "--then",
                    "normalized_title trims surrounding whitespace",
                    "--contract-kind",
                    "derivation",
                    "--covered-definition",
                    "normalized_title",
                ])
                .current_dir(temp_dir.path())
                .assert()
                .success();
        }

        Ok(())
    }

    fn author_capture_ticket_command_contract(
        temp_dir: &TempDir,
        lean: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut args = vec![
            "add",
            "scenario",
            "--slice",
            "capture-ticket",
            "--kind",
            "contract",
            "--name",
            "CaptureTicket emits TicketCaptured",
            "--given",
            "tickets stream is available",
            "--when",
            "CaptureTicket handles ticket input",
            "--then",
            "TicketCaptured is written",
            "--contract-kind",
            "command",
            "--covered-definition",
            "CaptureTicket",
        ];

        if lean.contains("def sliceKind := \"state_change\"")
            && lean.contains("name := \"tickets\"")
        {
            args.extend(["--read-streams", "tickets", "--written-streams", "tickets"]);
        }

        Command::cargo_bin("emc")?
            .args(args)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
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
                "--generated-source-kind",
                "actor_input",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
                "--generated-source-kind",
                "upstream_event_store",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "--generated-source-kind",
                "upstream_event_store",
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
                "--source-kind",
                "original",
                "--transformation",
                "identity",
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
                "--source-kind",
                "original",
                "--transformation",
                "derivation",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
                "--source-kind",
                "original",
                "--transformation",
                "projection",
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
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_title\",\"source\":\"actor input title field\",\"source_kind\":\"original\",\"transformation\":\"identity\",\"target\":\"Capture ticket.ticket_title\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
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

    fn mcp_event_stream_command_input_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\",\"stream\":\"tickets\",\"attribute\":\"status\",\"attribute_source\":\"generated\",\"attribute_source_name\":\"ticket_feed_snapshot\",\"attribute_source_field\":\"status\",\"generated_source_kind\":\"ticket_feed_snapshot\",\"attribute_provenance\":\"ticket feed status field\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_command_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\",\"input\":\"ticket_status\",\"input_source\":\"event_stream_state\",\"input_description\":\"current ticket status loaded from the ticket stream\",\"input_provenance\":\"TicketCaptured.status -> ticket_status\",\"emits\":\"TicketCaptured\",\"observes\":\"tickets\",\"source_event\":\"TicketCaptured\",\"source_attribute\":\"status\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"add_outcome_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"label\":\"ticket_captured\",\"events\":\"TicketCaptured\",\"externally_relevant\":false}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_status\",\"source\":\"TicketCaptured.status\",\"source_kind\":\"original\",\"transformation\":\"projection\",\"target\":\"CaptureTicket\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"status\",\"source\":\"ticket feed snapshot\",\"source_kind\":\"original\",\"transformation\":\"identity\",\"target\":\"TicketCaptured\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
        )
    }

    fn mcp_generated_command_input_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_command_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\",\"input\":\"ticket_id\",\"input_source\":\"generated\",\"input_description\":\"ticket id allocated by the ticket id generator\",\"input_provenance\":\"ticket_id_generator.uuid -> CaptureTicket.ticket_id\",\"emits\":\"TicketCaptured\",\"source_name\":\"ticket_id_generator\",\"source_field\":\"uuid\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\",\"stream\":\"tickets\",\"attribute\":\"ticket_id\",\"attribute_source\":\"command_input\",\"attribute_source_name\":\"ticket_id\",\"attribute_source_field\":\"value\",\"attribute_provenance\":\"CaptureTicket.ticket_id\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"add_outcome_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"label\":\"ticket_captured\",\"events\":\"TicketCaptured\",\"externally_relevant\":false}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_id\",\"source\":\"ticket_id_generator.uuid\",\"source_kind\":\"original\",\"transformation\":\"transformation\",\"target\":\"CaptureTicket\",\"bit_encoding\":\"128-bit UUIDv7 string\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_id\",\"source\":\"CaptureTicket.ticket_id\",\"source_kind\":\"original\",\"transformation\":\"identity\",\"target\":\"TicketCaptured\",\"bit_encoding\":\"128-bit UUIDv7 string\"}}}\n",
        )
    }

    fn mcp_session_command_input_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_command_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\",\"input\":\"organization_id\",\"input_source\":\"session\",\"input_description\":\"organization id loaded from authenticated session\",\"input_provenance\":\"authenticated_session.organization_id -> CaptureTicket.organization_id\",\"emits\":\"TicketCaptured\",\"source_session\":\"authenticated_session\",\"source_field\":\"organization_id\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\",\"stream\":\"tickets\",\"attribute\":\"organization_id\",\"attribute_source\":\"command_input\",\"attribute_source_name\":\"organization_id\",\"attribute_source_field\":\"value\",\"attribute_provenance\":\"CaptureTicket.organization_id\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"add_outcome_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"label\":\"ticket_captured\",\"events\":\"TicketCaptured\",\"externally_relevant\":false}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"organization_id\",\"source\":\"authenticated_session.organization_id\",\"source_kind\":\"original\",\"transformation\":\"projection\",\"target\":\"CaptureTicket\",\"bit_encoding\":\"128-bit organization UUID string\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"organization_id\",\"source\":\"CaptureTicket.organization_id\",\"source_kind\":\"original\",\"transformation\":\"identity\",\"target\":\"TicketCaptured\",\"bit_encoding\":\"128-bit organization UUID string\"}}}\n",
        )
    }

    fn mcp_invocation_argument_command_input_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_command_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"CaptureTicket\",\"input\":\"ticket_title\",\"input_source\":\"invocation_argument\",\"input_description\":\"ticket title supplied by the command invocation\",\"input_provenance\":\"invoke CaptureTicket.title -> CaptureTicket.ticket_title\",\"emits\":\"TicketCaptured\",\"source_argument\":\"CaptureTicket\",\"source_field\":\"title\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\",\"stream\":\"tickets\",\"attribute\":\"ticket_title\",\"attribute_source\":\"command_input\",\"attribute_source_name\":\"ticket_title\",\"attribute_source_field\":\"value\",\"attribute_provenance\":\"CaptureTicket.ticket_title\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"add_outcome_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"label\":\"ticket_captured\",\"events\":\"TicketCaptured\",\"externally_relevant\":false}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_title\",\"source\":\"CaptureTicket.title\",\"source_kind\":\"original\",\"transformation\":\"projection\",\"target\":\"CaptureTicket\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_title\",\"source\":\"CaptureTicket.ticket_title\",\"source_kind\":\"original\",\"transformation\":\"identity\",\"target\":\"TicketCaptured\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
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
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketCaptured\",\"stream\":\"tickets\",\"attribute\":\"ticket_title\",\"attribute_source\":\"generated\",\"attribute_source_name\":\"upstream_event_store\",\"attribute_source_field\":\"ticket_title\",\"generated_source_kind\":\"event_store_observation\",\"attribute_provenance\":\"TicketCaptured.ticket_title\",\"observed\":true}}}\n",
        )
    }

    fn mcp_shared_event_definition_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_event_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"TicketTagged\",\"stream\":\"tickets\",\"attribute\":\"tag\",\"attribute_source\":\"generated\",\"attribute_source_name\":\"tagging_policy\",\"attribute_source_field\":\"tag\",\"generated_source_kind\":\"tagging_policy\",\"attribute_provenance\":\"TicketTagged.tag\",\"shared\":true}}}\n",
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

    fn mcp_external_workflow_view_control_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_view_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_summary\",\"read_model\":\"ticket_state\",\"field\":\"ticket_title\",\"source_field\":\"ticket_title\",\"sketch_token\":\"title-label\",\"field_provenance\":\"ticket_state.ticket_title\",\"bit_encoding\":\"UTF-8 string\",\"control\":\"open-billing-workflow\",\"control_command\":\"CaptureTicket\",\"control_input\":\"ticket_title\",\"control_input_source\":\"actor\",\"control_input_description\":\"title field on the intake form\",\"control_input_sketch_token\":\"title-input\",\"control_input_visible\":true,\"control_input_decision\":true,\"handled_errors\":\"DuplicateTicket\",\"recovery_behavior\":\"explicit_recovery_action\",\"control_sketch_token\":\"billing-link\",\"navigation_type\":\"external_workflow\",\"navigation_target\":\"billing-intake\",\"external_workflow\":\"billing-intake\"}}}\n",
        )
    }

    fn mcp_local_state_view_control_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_view_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_summary\",\"read_model\":\"ticket_state\",\"field\":\"ticket_title\",\"source_field\":\"ticket_title\",\"sketch_token\":\"title-label\",\"field_provenance\":\"ticket_state.ticket_title\",\"bit_encoding\":\"UTF-8 string\",\"control\":\"expand-details\",\"control_command\":\"CaptureTicket\",\"control_input\":\"ticket_title\",\"control_input_source\":\"actor\",\"control_input_description\":\"title field on the intake form\",\"control_input_sketch_token\":\"title-input\",\"control_input_visible\":true,\"control_input_decision\":true,\"handled_errors\":\"DuplicateTicket\",\"recovery_behavior\":\"retry\",\"control_sketch_token\":\"expand-button\",\"navigation_type\":\"local_view_state\",\"navigation_target\":\"details-expanded\",\"local_states\":\"details-expanded\",\"filters\":\"open-only\"}}}\n",
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
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"add_bit_level_data_flow\",\"arguments\":{\"slice\":\"capture-ticket\",\"datum\":\"ticket_title\",\"source\":\"intake_webhook.ticket_title\",\"source_kind\":\"original\",\"transformation\":\"identity\",\"target\":\"intake_webhook_received\",\"bit_encoding\":\"UTF-8 string\"}}}\n",
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
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_read_model_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_state\",\"field\":\"normalized_title\",\"field_source\":\"derivation\",\"derivation_rule\":\"derivation from TicketCaptured.ticket_title\",\"derivation_source_fields\":\"ticket_title,raw_title\",\"derivation_scenario\":\"Ticket title is normalized\",\"field_provenance\":\"TicketCaptured.ticket_title -> trim\"}}}\n",
        )
    }

    fn mcp_read_model_transitive_requests() -> &'static str {
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-test\",\"version\":\"0.0.0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"add_read_model_definition\",\"arguments\":{\"slice\":\"capture-ticket\",\"name\":\"ticket_hierarchy\",\"field\":\"ancestor_ticket_id\",\"field_source\":\"event_attribute\",\"source_event\":\"TicketLinked\",\"source_attribute\":\"parent_ticket_id\",\"field_provenance\":\"TicketLinked.parent_ticket_id\",\"transitive\":true,\"relationship_fields\":\"parent_ticket_id,child_ticket_id\",\"transitive_rule\":\"walk TicketLinked parent_ticket_id edges until root\",\"example_scenario\":\"Ticket hierarchy includes grandchild\"}}}\n",
        )
    }

    fn assert_project_root_digests_are_canonical_hashes(
        lean_root: &str,
        quint_root: &str,
    ) -> Result<(), Box<dyn Error>> {
        let lean_digest = generated_model_digest(lean_root, "def modelDigest := \"")?;
        let quint_digest = generated_model_digest(quint_root, "val modelDigest = \"")?;

        assert_eq!(
            lean_digest, quint_digest,
            "Lean and Quint project roots must use the same generated model digest"
        );
        assert!(
            is_lowercase_sha256_hex(lean_digest),
            "project root digest must be a canonical SHA-256 content hash"
        );

        Ok(())
    }

    fn generated_model_digest<'a>(
        artifact: &'a str,
        prefix: &str,
    ) -> Result<&'a str, Box<dyn Error>> {
        let start = artifact
            .find(prefix)
            .ok_or_else(|| format!("generated artifact must contain {prefix}"))?
            + prefix.len();
        let tail = &artifact[start..];
        let end = tail
            .find('"')
            .ok_or("generated artifact model digest must terminate with a quote")?;

        Ok(&tail[..end])
    }

    fn is_lowercase_sha256_hex(value: &str) -> bool {
        value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
    }
}
