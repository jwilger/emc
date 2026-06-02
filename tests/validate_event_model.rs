#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{create_dir_all, write};

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn validate_rejects_invalid_event_model_json() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(workflows.join("broken.eventmodel.json"), "{ not-json")?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid JSON"));

        Ok(())
    }

    #[test]
    fn validate_rejects_non_object_event_model_json() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(workflows.join("array.eventmodel.json"), "[]")?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("model must be a JSON object"));

        Ok(())
    }

    #[test]
    fn validate_rejects_missing_required_top_level_key() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("missing-name.eventmodel.json"),
            "{\"version\":\"0.1.0\",\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("missing top-level key 'name'"));

        Ok(())
    }

    #[test]
    fn validate_rejects_missing_explicit_board() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("missing-board.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("missing explicit board"));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_command_names() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("duplicate-command.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[{\"name\":\"OpenRepairTicket\"},{\"name\":\"OpenRepairTicket\"}],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "duplicate command name 'OpenRepairTicket'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_slice_files_with_two_slices() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("two-slices.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit ticket\"},{\"name\":\"Review ticket\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice file must contain exactly one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_slice_legacy_scenarios_field() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("legacy-scenarios.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Submit lesson' uses legacy 'scenarios'; use 'acceptance_scenarios' and 'contract_scenarios'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_first_class_scenario_without_when() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("missing-when.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"acceptance_scenarios\":[{\"name\":\"reader sees lesson\",\"given\":[],\"then\":[]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Submit lesson' scenario 'reader sees lesson' is missing 'when'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_scenario_names_across_first_class_fields()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("duplicate-scenarios.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"acceptance_scenarios\":[{\"name\":\"duplicate scenario\",\"given\":[],\"when\":{},\"then\":[]}],\"contract_scenarios\":[{\"name\":\"duplicate scenario\",\"given\":[],\"when\":{},\"then\":[]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Submit lesson' has duplicate scenario name 'duplicate scenario' across acceptance_scenarios and contract_scenarios",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_acceptance_scenario_event_references() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("acceptance-event-reference.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[{\"name\":\"RootOrganizationBootstrapped\"}],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Resolve application entry\",\"acceptance_scenarios\":[{\"name\":\"fresh install sees bootstrap setup\",\"given\":[],\"when\":{},\"then\":[\"RootOrganizationBootstrapped\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Resolve application entry' acceptance scenario 'fresh install sees bootstrap setup' references event 'RootOrganizationBootstrapped'; acceptance_scenarios must describe user-facing behavior only",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_state_view_read_model_without_projector_contract_scenario()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("missing-projector-contract.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[{\"name\":\"application_entry_state\"}],\"views\":[{\"name\":\"application_entry_screen\",\"uses_read_models\":[\"application_entry_state\"]}],\"slices\":[{\"name\":\"Resolve application entry\",\"type\":\"state_view\",\"views\":[\"application_entry_screen\"],\"acceptance_scenarios\":[{\"name\":\"reader sees entry\",\"given\":[],\"when\":{},\"then\":[\"entry is visible\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "state_view slice 'Resolve application entry' read model 'application_entry_state' requires a contract_scenarios GWT for the projector",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_slice_outcome_labels() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("duplicate-outcomes.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"outcomes\":[{\"label\":\"submitted\"},{\"label\":\"submitted\"}],\"acceptance_scenarios\":[{\"name\":\"reader submits lesson\",\"given\":[],\"when\":{},\"then\":[\"lesson is submitted\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice 'Submit lesson' has duplicate outcome label 'submitted'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_slice_outcome_event_sets() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("duplicate-outcome-events.eventmodel.json"),
            "{\"name\":\"Activate member workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"member\"}],\"events\":[{\"name\":\"OrganizationMemberActivated\",\"stream\":\"member\",\"attributes\":[]}],\"commands\":[{\"name\":\"ActivateMember\",\"inputs\":[],\"produces\":[\"OrganizationMemberActivated\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Activate member\",\"type\":\"state_change\",\"commands\":[\"ActivateMember\"],\"events\":[\"OrganizationMemberActivated\"],\"outcomes\":[{\"label\":\"activated\",\"events\":[\"OrganizationMemberActivated\"]},{\"label\":\"already_active\",\"events\":[\"OrganizationMemberActivated\"]}],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"activate member\",\"given\":[],\"given_streams\":[{\"stream\":\"member\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"OrganizationMemberActivated\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "outcomes 'activated' and 'already_active' use the same event set",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_slice_outcomes_without_events() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("outcome-without-events.eventmodel.json"),
            "{\"name\":\"Activate member workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"member\"}],\"events\":[{\"name\":\"OrganizationMemberActivated\",\"stream\":\"member\",\"attributes\":[]}],\"commands\":[{\"name\":\"ActivateMember\",\"inputs\":[],\"produces\":[\"OrganizationMemberActivated\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Activate member\",\"type\":\"state_change\",\"commands\":[\"ActivateMember\"],\"events\":[\"OrganizationMemberActivated\"],\"outcomes\":[{\"label\":\"activated\",\"events\":[]}],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"activate member\",\"given\":[],\"given_streams\":[{\"stream\":\"member\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"OrganizationMemberActivated\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "outcome 'activated' must declare at least one event",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_slice_outcomes_that_reference_events_outside_the_slice()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("outcome-with-external-event.eventmodel.json"),
            "{\"name\":\"Activate member workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"member\"}],\"events\":[{\"name\":\"OrganizationMemberActivated\",\"stream\":\"member\",\"attributes\":[]},{\"name\":\"OrganizationMemberSuspended\",\"stream\":\"member\",\"attributes\":[]}],\"commands\":[{\"name\":\"ActivateMember\",\"inputs\":[],\"produces\":[\"OrganizationMemberActivated\",\"OrganizationMemberSuspended\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Activate member\",\"type\":\"state_change\",\"commands\":[\"ActivateMember\"],\"events\":[\"OrganizationMemberActivated\"],\"outcomes\":[{\"label\":\"activated\",\"events\":[\"OrganizationMemberSuspended\"]}],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"activate member\",\"given\":[],\"given_streams\":[{\"stream\":\"member\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"OrganizationMemberActivated\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "outcome 'activated' references event 'OrganizationMemberSuspended' that is not emitted or observed by slice 'Activate member'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_slice_outcomes_that_reference_unknown_events() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("outcome-with-unknown-event.eventmodel.json"),
            "{\"name\":\"Activate member workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"member\"}],\"events\":[{\"name\":\"OrganizationMemberActivated\",\"stream\":\"member\",\"attributes\":[]}],\"commands\":[{\"name\":\"ActivateMember\",\"inputs\":[],\"produces\":[\"OrganizationMemberActivated\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Activate member\",\"type\":\"state_change\",\"commands\":[\"ActivateMember\"],\"events\":[\"OrganizationMemberActivated\"],\"outcomes\":[{\"label\":\"activated\",\"events\":[\"MissingEvent\"]}],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"activate member\",\"given\":[],\"given_streams\":[{\"stream\":\"member\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"OrganizationMemberActivated\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "outcome 'activated' references unknown event 'MissingEvent'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_events_that_reference_unknown_streams() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("unknown-stream.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[{\"name\":\"RepairTicketOpened\",\"stream\":\"missing_stream\"}],\"commands\":[],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'RepairTicketOpened' references unknown stream 'missing_stream'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_events_without_command_producers() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("unproduced-event.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"repair_ticket\"}],\"events\":[{\"name\":\"RepairTicketOpened\",\"stream\":\"repair_ticket\"}],\"commands\":[],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'RepairTicketOpened' is not produced by any command",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_command_sourced_event_attributes_without_command_input()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("invalid-command-source.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"repair_ticket\"}],\"events\":[{\"name\":\"RepairTicketOpened\",\"stream\":\"repair_ticket\",\"attributes\":[{\"name\":\"customer_name\",\"source\":\"command.customer_name\"}]}],\"commands\":[{\"name\":\"OpenRepairTicket\",\"inputs\":[],\"produces\":[\"RepairTicketOpened\"]}],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'RepairTicketOpened' attribute 'customer_name' has invalid source 'command.customer_name'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_external_event_attributes_without_external_input()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("invalid-external-source.eventmodel.json"),
            "{\"name\":\"Record payment\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"payment\"}],\"events\":[{\"name\":\"PaymentRecorded\",\"stream\":\"payment\",\"attributes\":[{\"name\":\"provider_payment_id\",\"source\":\"external.payment_webhook.payment_id\"}]}],\"commands\":[{\"name\":\"RecordPayment\",\"inputs\":[],\"external_inputs\":[],\"produces\":[\"PaymentRecorded\"]}],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'PaymentRecorded' attribute 'provider_payment_id' has invalid source 'external.payment_webhook.payment_id'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_external_event_attributes_with_undeclared_payload_field()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("undeclared-external-field.eventmodel.json"),
            "{\"name\":\"Record payment\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"payment\"}],\"events\":[{\"name\":\"PaymentRecorded\",\"stream\":\"payment\",\"attributes\":[{\"name\":\"provider_status\",\"source\":\"external.payment_webhook.status\"}]}],\"commands\":[{\"name\":\"RecordPayment\",\"inputs\":[],\"external_inputs\":[\"payment_webhook\"],\"external_input_schemas\":[{\"name\":\"payment_webhook\",\"fields\":[{\"name\":\"payment_id\"}]}],\"produces\":[\"PaymentRecorded\"]}],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'PaymentRecorded' attribute 'provider_status' references undeclared external input field 'status'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_event_attributes_sourced_from_read_models() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("read-model-event-source.eventmodel.json"),
            "{\"name\":\"Escalate repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"repair_ticket\"}],\"events\":[{\"name\":\"RepairTicketEscalated\",\"stream\":\"repair_ticket\",\"attributes\":[{\"name\":\"priority\",\"source\":\"read_model.repair_ticket_summary.priority\"}]}],\"commands\":[{\"name\":\"EscalateRepairTicket\",\"inputs\":[],\"produces\":[\"RepairTicketEscalated\"]}],\"read_models\":[{\"name\":\"repair_ticket_summary\"}],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'RepairTicketEscalated' attribute 'priority' has invalid source 'read_model.repair_ticket_summary.priority'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_event_attributes_with_empty_generated_source() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("empty-generated-source.eventmodel.json"),
            "{\"name\":\"Open repair ticket\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"repair_ticket\"}],\"events\":[{\"name\":\"RepairTicketOpened\",\"stream\":\"repair_ticket\",\"attributes\":[{\"name\":\"repair_ticket_id\",\"source\":\"generated.\"}]}],\"commands\":[{\"name\":\"OpenRepairTicket\",\"inputs\":[],\"produces\":[\"RepairTicketOpened\"]}],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'RepairTicketOpened' attribute 'repair_ticket_id' has invalid source 'generated.'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_read_model_fields_that_reference_unknown_event_attributes()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("unknown-event-attribute.eventmodel.json"),
            "{\"name\":\"Repair queue\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"repair_ticket\"}],\"events\":[{\"name\":\"RepairTicketOpened\",\"stream\":\"repair_ticket\",\"attributes\":[]}],\"commands\":[{\"name\":\"OpenRepairTicket\",\"inputs\":[],\"produces\":[\"RepairTicketOpened\"]}],\"read_models\":[{\"name\":\"repair_queue\",\"fields\":[{\"name\":\"customer_name\",\"source\":\"RepairTicketOpened.customer_name\"}]}],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "read model 'repair_queue' field 'customer_name' references unknown event attribute 'RepairTicketOpened.customer_name'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_derived_read_model_fields_without_derivation_provenance()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("derived-field-without-provenance.eventmodel.json"),
            "{\"name\":\"Manager visibility\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"report_visibility\"}],\"events\":[{\"name\":\"ReportVisibilityGranted\",\"stream\":\"report_visibility\",\"attributes\":[{\"name\":\"report_user_id\",\"source\":\"command.report_user_id\"}]}],\"commands\":[{\"name\":\"GrantReportVisibility\",\"inputs\":[\"report_user_id\"],\"produces\":[\"ReportVisibilityGranted\"]}],\"read_models\":[{\"name\":\"manager_visibility\",\"fields\":[{\"name\":\"visible_report_count\",\"source\":\"derivation.visible_report_count\",\"derived\":true}]}],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "derived read model field 'visible_report_count' must declare source fields and derivation",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_derived_read_model_fields_without_derivation_scenarios()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("derived-field-without-scenarios.eventmodel.json"),
            "{\"name\":\"Manager visibility\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"report_visibility\"}],\"events\":[{\"name\":\"ReportVisibilityGranted\",\"stream\":\"report_visibility\",\"attributes\":[{\"name\":\"report_user_id\",\"source\":\"command.report_user_id\"}]}],\"commands\":[{\"name\":\"GrantReportVisibility\",\"inputs\":[\"report_user_id\"],\"produces\":[\"ReportVisibilityGranted\"]}],\"read_models\":[{\"name\":\"manager_visibility\",\"fields\":[{\"name\":\"visible_report_count\",\"source\":\"derivation.visible_report_count\",\"derived\":true,\"derivation_source_fields\":[\"ReportVisibilityGranted.report_user_id\"],\"derivation_description\":\"count visible reports\"}]}],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "derived read model field 'visible_report_count' must have a derivation scenario",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_transitive_read_models_without_derivation_rule_and_examples()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("transitive-read-model-without-rule.eventmodel.json"),
            "{\"name\":\"Manager progress visibility\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"report_visibility\"}],\"events\":[{\"name\":\"ReportVisibilityGranted\",\"stream\":\"report_visibility\",\"attributes\":[{\"name\":\"report_user_id\",\"source\":\"command.report_user_id\"}]}],\"commands\":[{\"name\":\"GrantReportVisibility\",\"inputs\":[\"report_user_id\"],\"produces\":[\"ReportVisibilityGranted\"]}],\"read_models\":[{\"name\":\"manager_progress_visibility\",\"transitive\":true,\"fields\":[{\"name\":\"visible_report_user_id\",\"source\":\"ReportVisibilityGranted.report_user_id\"}]}],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "transitive read model 'manager_progress_visibility' must declare source fields, derivation rule, and scenarios",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_absence_default_fields_without_absence_event() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("absence-default-without-event.eventmodel.json"),
            "{\"name\":\"Application entry\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"organization\"}],\"events\":[{\"name\":\"RootOrganizationBootstrapped\",\"stream\":\"organization\",\"attributes\":[]}],\"commands\":[{\"name\":\"BootstrapRootOrganization\",\"inputs\":[],\"produces\":[\"RootOrganizationBootstrapped\"]}],\"read_models\":[{\"name\":\"application_entry_state\",\"fields\":[{\"name\":\"is_bootstrapped\",\"source\":\"absence\",\"defaulted_from_absence\":true}]}],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "absence/default field 'is_bootstrapped' must declare the event absence it derives from",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_absence_default_fields_without_absence_scenarios()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("absence-default-without-scenarios.eventmodel.json"),
            "{\"name\":\"Application entry\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"organization\"}],\"events\":[{\"name\":\"RootOrganizationBootstrapped\",\"stream\":\"organization\",\"attributes\":[]}],\"commands\":[{\"name\":\"BootstrapRootOrganization\",\"inputs\":[],\"produces\":[\"RootOrganizationBootstrapped\"]}],\"read_models\":[{\"name\":\"application_entry_state\",\"fields\":[{\"name\":\"is_bootstrapped\",\"source\":\"absence\",\"defaulted_from_absence\":true,\"absence_event\":\"RootOrganizationBootstrapped\"}]}],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "absence/default field 'is_bootstrapped' must have an absence scenario",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_command_input_sources_with_undeclared_external_fields()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("command-input-undeclared-external-field.eventmodel.json"),
            "{\"name\":\"Record checkpoint result\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"checkpoint\"}],\"events\":[{\"name\":\"CheckpointResultRecorded\",\"stream\":\"checkpoint\",\"attributes\":[{\"name\":\"output_excerpt\",\"source\":\"command.output_excerpt\"}]}],\"commands\":[{\"name\":\"RecordCheckpointResult\",\"inputs\":[\"output_excerpt\"],\"input_sources\":[{\"name\":\"output_excerpt\",\"source\":\"external.lesson_checkpoint_result.output_excerpt\"}],\"external_inputs\":[\"lesson_checkpoint_result\"],\"external_input_schemas\":[{\"name\":\"lesson_checkpoint_result\",\"fields\":[{\"name\":\"score\"}]}],\"produces\":[\"CheckpointResultRecorded\"]}],\"read_models\":[],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "command input 'output_excerpt' references undeclared external input field 'output_excerpt'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_commands_with_legacy_read_model_reads() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("legacy-command-reads.eventmodel.json"),
            "{\"name\":\"Submit lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_submission\"}],\"events\":[{\"name\":\"LessonSubmittedForReview\",\"stream\":\"lesson_submission\",\"attributes\":[]}],\"commands\":[{\"name\":\"SubmitLessonForReview\",\"inputs\":[],\"reads\":[\"lesson_submission_context\"],\"produces\":[\"LessonSubmittedForReview\"]}],\"read_models\":[{\"name\":\"lesson_submission_context\",\"fields\":[]}],\"slices\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "command 'SubmitLessonForReview' uses legacy read-model reads",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_state_change_scenarios_without_written_stream_in_given_streams()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("state-change-missing-given-stream.eventmodel.json"),
            "{\"name\":\"Submit lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_submission\"}],\"events\":[{\"name\":\"LessonSubmittedForReview\",\"stream\":\"lesson_submission\",\"attributes\":[]}],\"commands\":[{\"name\":\"SubmitLessonForReview\",\"inputs\":[],\"produces\":[\"LessonSubmittedForReview\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"type\":\"state_change\",\"events\":[\"LessonSubmittedForReview\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"submit lesson\",\"given\":[],\"given_streams\":[],\"when\":{},\"then\":[\"LessonSubmittedForReview\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "state-change scenario 'submit lesson' writes stream 'lesson_submission' but does not name it in given_streams",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_singleton_state_change_slices_without_repeat_behavior()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("singleton-state-change-without-repeat.eventmodel.json"),
            "{\"name\":\"Bootstrap root organization\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"organization\"}],\"events\":[{\"name\":\"RootOrganizationBootstrapped\",\"stream\":\"organization\",\"attributes\":[]}],\"commands\":[{\"name\":\"BootstrapRootOrganization\",\"inputs\":[],\"produces\":[\"RootOrganizationBootstrapped\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Bootstrap Root organization\",\"type\":\"state_change\",\"singleton\":true,\"events\":[\"RootOrganizationBootstrapped\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"bootstrap root organization\",\"given\":[],\"given_streams\":[{\"stream\":\"organization\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"RootOrganizationBootstrapped\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "singleton state_change slice 'Bootstrap Root organization' must declare already-exists or idempotent behavior",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_translation_slices_without_external_signal_or_payload_contract()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("translation-without-external-contract.eventmodel.json"),
            "{\"name\":\"Record SCIM member provisioning\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"member\"}],\"events\":[{\"name\":\"SCIMMemberProvisioned\",\"stream\":\"member\",\"attributes\":[]}],\"commands\":[{\"name\":\"RecordSCIMMemberProvisioning\",\"inputs\":[],\"produces\":[\"SCIMMemberProvisioned\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Record SCIM member provisioning\",\"type\":\"translation\",\"events\":[\"SCIMMemberProvisioned\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"record scim member\",\"given\":[],\"when\":{},\"then\":[\"SCIMMemberProvisioned\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "translation slice 'Record SCIM member provisioning' must declare an external event or payload contract",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_translation_slices_that_own_views() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("translation-owning-view.eventmodel.json"),
            "{\"name\":\"Activate member from SAML claim\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"member\"}],\"events\":[{\"name\":\"MemberActivatedFromSAMLClaim\",\"stream\":\"member\",\"attributes\":[]}],\"commands\":[{\"name\":\"ActivateMemberFromSAMLClaim\",\"inputs\":[],\"produces\":[\"MemberActivatedFromSAMLClaim\"]}],\"read_models\":[],\"views\":[{\"name\":\"organization_sign_in_screen\",\"uses_read_models\":[]}],\"slices\":[{\"name\":\"Activate member from SAML claim\",\"type\":\"translation\",\"external_event\":\"SAMLClaimReceived\",\"views\":[\"organization_sign_in_screen\"],\"events\":[\"MemberActivatedFromSAMLClaim\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"activate member from saml claim\",\"given\":[],\"when\":{},\"then\":[\"MemberActivatedFromSAMLClaim\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "translation slice 'Activate member from SAML claim' must not own view 'organization_sign_in_screen'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_translation_slices_without_payload_variant_scenario()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("translation-without-payload-variant-scenario.eventmodel.json"),
            "{\"name\":\"Receive webhook\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"webhook\"}],\"events\":[{\"name\":\"WebhookProcessed\",\"stream\":\"webhook\",\"attributes\":[]}],\"commands\":[{\"name\":\"ProcessWebhook\",\"inputs\":[],\"produces\":[\"WebhookProcessed\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Receive webhook\",\"type\":\"translation\",\"commands\":[\"ProcessWebhook\"],\"external_input_schemas\":[{\"name\":\"webhook_payload\",\"variants\":[\"create\",\"update\"]}],\"events\":[\"WebhookProcessed\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"create webhook\",\"given\":[\"create webhook received\"],\"when\":\"process create\",\"then\":[\"WebhookProcessed\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "translation slice 'Receive webhook' must include a scenario for external payload variant 'update'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflow_transitions_using_command_errors_as_outcomes()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("organization-access.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\",\"transitions\":[{\"to\":\"activate-member\"}]},{\"slice\":\"activate-member\",\"name\":\"Activate member\",\"type\":\"state_change\",\"relationship\":\"main\",\"transitions\":[{\"to_workflow\":\"member-access\",\"via_error\":\"member_suspended\",\"exit_reason\":\"incorrectly modeled local command error as branch outcome\"}]}]}",
        )?;
        write(
            workflows.join("activate-member.eventmodel.json"),
            "{\"name\":\"Activate member\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"members\"}],\"events\":[{\"name\":\"MemberActivationAttempted\",\"stream\":\"members\",\"attributes\":[]}],\"commands\":[{\"name\":\"ActivateMember\",\"inputs\":[],\"produces\":[\"MemberActivationAttempted\"],\"errors\":[\"member_suspended\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Activate member\",\"type\":\"state_change\",\"commands\":[\"ActivateMember\"],\"events\":[\"MemberActivationAttempted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"activate error\",\"given\":[\"member suspended\"],\"given_streams\":[{\"stream\":\"members\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"error member_suspended is returned\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow transition cannot use command-local error 'member_suspended' as a business outcome",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflows_that_do_not_handle_slice_outcomes() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("organization-access.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\",\"transitions\":[{\"to\":\"activate-member\"}]},{\"slice\":\"activate-member\",\"name\":\"Activate member\",\"type\":\"state_change\",\"relationship\":\"main\",\"transitions\":[{\"to_workflow\":\"member-access\",\"via_outcome\":\"activated\",\"exit_reason\":\"activated members continue to member access\"}]}]}",
        )?;
        write(
            workflows.join("activate-member.eventmodel.json"),
            "{\"name\":\"Activate member\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"members\"}],\"events\":[{\"name\":\"OrganizationMemberActivated\",\"stream\":\"members\",\"attributes\":[]},{\"name\":\"MemberActivationNotAuthorized\",\"stream\":\"members\",\"attributes\":[]}],\"commands\":[{\"name\":\"ActivateMember\",\"inputs\":[],\"produces\":[\"OrganizationMemberActivated\",\"MemberActivationNotAuthorized\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Activate member\",\"type\":\"state_change\",\"commands\":[\"ActivateMember\"],\"events\":[\"OrganizationMemberActivated\",\"MemberActivationNotAuthorized\"],\"outcomes\":[{\"label\":\"activated\",\"events\":[\"OrganizationMemberActivated\"]},{\"label\":\"not_authorized\",\"events\":[\"MemberActivationNotAuthorized\"]}],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"activate succeeds\",\"given\":[\"member exists\"],\"given_streams\":[{\"stream\":\"members\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"OrganizationMemberActivated\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow 'Organization access' does not handle outcome 'not_authorized' from slice 'Activate member'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflow_compositions_without_steps() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./lesson-entry.eventmodel.json\"],\"steps\":[]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow composition must declare steps",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflow_steps_that_do_not_reference_composed_slices()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./entry.eventmodel.json\"],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\",\"transitions\":[{\"to\":\"show-lesson\",\"via_navigation\":\"show_lesson_screen\"}]},{\"slice\":\"show-lesson\",\"name\":\"Show lesson\",\"type\":\"state_view\",\"relationship\":\"main\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow step 'show-lesson' does not reference a composed slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_referenced_slices_without_workflow_steps() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./entry.eventmodel.json\",\"./submit-lesson.eventmodel.json\"],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "referenced slice 'submit-lesson' is not used by workflow steps",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflows_without_exactly_one_entry_step() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./resolve-entry.eventmodel.json\",\"./show-lesson.eventmodel.json\"],\"steps\":[{\"slice\":\"resolve-entry\",\"name\":\"Resolve entry\",\"type\":\"state_view\",\"relationship\":\"entry\"},{\"slice\":\"show-lesson\",\"name\":\"Show lesson\",\"type\":\"state_view\",\"relationship\":\"entry\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow must declare exactly one entry step",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_workflow_step_slices() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./show-lesson.eventmodel.json\"],\"steps\":[{\"slice\":\"show-lesson\",\"name\":\"Show lesson\",\"type\":\"state_view\",\"relationship\":\"entry\"},{\"slice\":\"show-lesson\",\"name\":\"Show lesson again\",\"type\":\"state_view\",\"relationship\":\"main\",\"trigger\":\"manual_retry\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow step slice 'show-lesson' is duplicated",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_non_entry_workflow_steps_without_incoming_reachability()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./entry.eventmodel.json\",\"./submit-lesson.eventmodel.json\"],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\"},{\"slice\":\"submit-lesson\",\"name\":\"Submit lesson\",\"type\":\"state_change\",\"relationship\":\"main\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow step 'submit-lesson' has no incoming transition",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflow_steps_unreachable_from_entry() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./entry.eventmodel.json\",\"./submit.eventmodel.json\",\"./review.eventmodel.json\"],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\"},{\"slice\":\"submit\",\"name\":\"Submit\",\"type\":\"state_view\",\"relationship\":\"main\",\"transitions\":[{\"to\":\"review\"}]},{\"slice\":\"review\",\"name\":\"Review\",\"type\":\"state_view\",\"relationship\":\"main\",\"transitions\":[{\"to\":\"submit\"}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow step 'submit' is not reachable from entry step 'entry'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflow_transitions_targeting_unknown_steps() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./entry.eventmodel.json\"],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\",\"transitions\":[{\"to\":\"missing-step\"}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "transition targets unknown workflow step 'missing-step'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_alternate_workflow_steps_without_trigger_or_incoming_transition()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./entry.eventmodel.json\",\"./revision.eventmodel.json\"],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\"},{\"slice\":\"revision\",\"name\":\"Revision\",\"type\":\"state_view\",\"relationship\":\"alternate\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "alternate workflow step 'revision' must declare a trigger or incoming transition",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_async_lifecycle_workflow_steps_modeled_as_main()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("organization-access.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./entry.eventmodel.json\",\"./record-member-suspension.eventmodel.json\"],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\"},{\"slice\":\"record-member-suspension\",\"name\":\"Record member suspension\",\"type\":\"state_change\",\"relationship\":\"main\",\"trigger\":\"scim_member_suspended\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "async lifecycle step 'record-member-suspension' must be alternate or async_lifecycle",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflow_files_with_internal_definitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[{\"name\":\"SubmitLessonForReview\",\"produces\":[]}],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./course-submit-lesson-for-review.eventmodel.json\"],\"steps\":[{\"slice\":\"course-submit-lesson-for-review\",\"name\":\"Submit lesson\",\"type\":\"state_change\",\"relationship\":\"entry\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow files must not define commands, views, read models, automations, or scenarios",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflow_steps_selecting_internal_scenarios() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./submit-lesson.eventmodel.json\"],\"steps\":[{\"slice\":\"submit-lesson\",\"name\":\"Submit lesson\",\"type\":\"state_change\",\"relationship\":\"entry\",\"scenario\":\"missing evidence\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow step 'submit-lesson' must compose the whole slice, not scenario 'missing evidence'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_bootstrap_workflows_without_application_entry_state_view()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("organization-access.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./bootstrap-root-organization.eventmodel.json\"],\"steps\":[{\"slice\":\"bootstrap-root-organization\",\"name\":\"Bootstrap root organization\",\"type\":\"state_change\",\"relationship\":\"entry\"}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "should model the application entry root bootstrap state view before bootstrap",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_application_entry_slices_without_lifecycle_state_coverage()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("organization-access.eventmodel.json"),
            "{\"name\":\"Organization access\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./resolve-application-entry.eventmodel.json\"],\"steps\":[{\"slice\":\"resolve-application-entry\",\"name\":\"Resolve application entry\",\"type\":\"state_view\",\"relationship\":\"entry\"}]}",
        )?;
        write(
            workflows.join("resolve-application-entry.eventmodel.json"),
            "{\"name\":\"Resolve application entry\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Resolve application entry\",\"type\":\"state_view\",\"views\":[\"resolve_application_entry_screen\"],\"acceptance_scenarios\":[{\"name\":\"already initialized and unauthenticated\",\"given\":[],\"when\":{},\"then\":[]},{\"name\":\"already initialized and authenticated\",\"given\":[],\"when\":{},\"then\":[]},{\"name\":\"partially configured\",\"given\":[],\"when\":{},\"then\":[]},{\"name\":\"fully configured\",\"given\":[],\"when\":{},\"then\":[]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "application entry slice 'resolve-application-entry' must cover fresh and uninitialized state",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_workflow_event_transitions_not_shared_by_adjacent_slices()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("lesson-01.eventmodel.json"),
            "{\"name\":\"Lesson 01\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[],\"slice_files\":[\"./entry.eventmodel.json\",\"./submit-lesson.eventmodel.json\",\"./review-lesson.eventmodel.json\"],\"steps\":[{\"slice\":\"entry\",\"name\":\"Entry\",\"type\":\"state_view\",\"relationship\":\"entry\",\"transitions\":[{\"to\":\"submit-lesson\"}]},{\"slice\":\"submit-lesson\",\"name\":\"Submit lesson\",\"type\":\"state_change\",\"relationship\":\"main\",\"transitions\":[{\"to\":\"review-lesson\",\"via_event\":\"LessonSubmittedForReview\"}]},{\"slice\":\"review-lesson\",\"name\":\"Review lesson\",\"type\":\"state_view\",\"relationship\":\"main\"}]}",
        )?;
        write(
            workflows.join("submit-lesson.eventmodel.json"),
            "{\"name\":\"Submit lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"type\":\"state_change\",\"events\":[],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;
        write(
            workflows.join("review-lesson.eventmodel.json"),
            "{\"name\":\"Review lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Review lesson\",\"type\":\"state_view\",\"events\":[],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "transition event 'LessonSubmittedForReview' is not shared by source and target slices",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_automation_slices_without_trigger() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("automation-without-trigger.eventmodel.json"),
            "{\"name\":\"Review lesson submission\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_review\"}],\"events\":[{\"name\":\"TeacherReviewRecorded\",\"stream\":\"lesson_review\",\"attributes\":[]}],\"commands\":[{\"name\":\"RecordTeacherReview\",\"inputs\":[],\"produces\":[\"TeacherReviewRecorded\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Review lesson submission\",\"type\":\"automation\",\"commands\":[\"RecordTeacherReview\"],\"events\":[\"TeacherReviewRecorded\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"review lesson submission\",\"given\":[],\"when\":{},\"then\":[\"TeacherReviewRecorded\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "automation slice 'Review lesson submission' must declare a trigger",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_automation_slices_that_issue_multiple_commands()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("automation-with-multiple-commands.eventmodel.json"),
            "{\"name\":\"Review lesson submission\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_review\"},{\"name\":\"instructor_notification\"}],\"events\":[{\"name\":\"AcceptedReviewRecorded\",\"stream\":\"lesson_review\",\"attributes\":[]},{\"name\":\"InstructorNotified\",\"stream\":\"instructor_notification\",\"attributes\":[]}],\"commands\":[{\"name\":\"RecordAcceptedReview\",\"inputs\":[],\"produces\":[\"AcceptedReviewRecorded\"]},{\"name\":\"NotifyInstructor\",\"inputs\":[],\"produces\":[\"InstructorNotified\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Review lesson submission\",\"type\":\"automation\",\"trigger\":\"LessonSubmittedForReview\",\"commands\":[\"RecordAcceptedReview\",\"NotifyInstructor\"],\"events\":[\"AcceptedReviewRecorded\",\"InstructorNotified\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"review lesson submission\",\"given\":[\"LessonSubmittedForReview\"],\"when\":{},\"then\":[\"AcceptedReviewRecorded\",\"InstructorNotified\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "automation slice 'Review lesson submission' must issue one command per operation",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_automation_slices_without_trigger_scenario() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("automation-without-trigger-scenario.eventmodel.json"),
            "{\"name\":\"Review lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_review\"}],\"events\":[{\"name\":\"TeacherReviewRecorded\",\"stream\":\"lesson_review\",\"attributes\":[]}],\"commands\":[{\"name\":\"RecordTeacherReview\",\"inputs\":[],\"produces\":[\"TeacherReviewRecorded\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Review lesson\",\"type\":\"automation\",\"trigger\":\"LessonSubmittedForReview\",\"commands\":[\"RecordTeacherReview\"],\"events\":[\"TeacherReviewRecorded\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"other trigger\",\"given\":[\"LessonReviewRetried\"],\"when\":\"record review\",\"then\":[\"TeacherReviewRecorded\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "automation slice 'Review lesson' must include a scenario for trigger event 'LessonSubmittedForReview'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_automation_slices_without_command_error_handling()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("automation-without-command-error-handling.eventmodel.json"),
            "{\"name\":\"Review lesson submission\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_review\"}],\"events\":[{\"name\":\"TeacherReviewRecorded\",\"stream\":\"lesson_review\",\"attributes\":[]}],\"commands\":[{\"name\":\"RecordTeacherReview\",\"inputs\":[],\"produces\":[\"TeacherReviewRecorded\"],\"errors\":[\"review_decision_required\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Review lesson submission\",\"type\":\"automation\",\"trigger\":\"LessonSubmittedForReview\",\"commands\":[\"RecordTeacherReview\"],\"events\":[\"TeacherReviewRecorded\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"review lesson submission\",\"given\":[\"LessonSubmittedForReview\"],\"when\":{},\"then\":[\"TeacherReviewRecorded\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "automation slice 'Review lesson submission' does not handle command error 'review_decision_required'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_scenarios_that_reference_undeclared_command_errors()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("scenario-with-undeclared-command-error.eventmodel.json"),
            "{\"name\":\"Submit lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_submission\"}],\"events\":[{\"name\":\"LessonSubmittedForReview\",\"stream\":\"lesson_submission\",\"attributes\":[]}],\"commands\":[{\"name\":\"SubmitLesson\",\"inputs\":[],\"produces\":[\"LessonSubmittedForReview\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"type\":\"state_change\",\"commands\":[\"SubmitLesson\"],\"events\":[\"LessonSubmittedForReview\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"submit lesson requires reflection\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_submission\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"error reflection_required is returned\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "scenario references undeclared command error 'reflection_required'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_declared_command_errors_without_state_change_scenario_coverage()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("command-error-without-scenario.eventmodel.json"),
            "{\"name\":\"Submit lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_submission\"}],\"events\":[{\"name\":\"LessonSubmittedForReview\",\"stream\":\"lesson_submission\",\"attributes\":[]}],\"commands\":[{\"name\":\"SubmitLesson\",\"inputs\":[],\"produces\":[\"LessonSubmittedForReview\"],\"errors\":[\"reflection_required\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"type\":\"state_change\",\"commands\":[\"SubmitLesson\"],\"events\":[\"LessonSubmittedForReview\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"submit lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_submission\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonSubmittedForReview\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "command error 'reflection_required' must be covered by a state-change scenario",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_controls_that_do_not_handle_command_errors() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("control-with-unhandled-command-error.eventmodel.json"),
            "{\"name\":\"Submit lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_submission\"}],\"events\":[{\"name\":\"LessonSubmittedForReview\",\"stream\":\"lesson_submission\",\"attributes\":[]}],\"commands\":[{\"name\":\"SubmitLesson\",\"inputs\":[],\"produces\":[\"LessonSubmittedForReview\"],\"errors\":[\"evidence_required\",\"reflection_required\"]}],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"wireframe\":\"<button data-ref=\\\"Submit for review\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Submit for review\",\"command\":\"SubmitLesson\",\"error_handling\":[{\"error\":\"evidence_required\"}]}]}],\"slices\":[{\"name\":\"Submit lesson\",\"type\":\"state_change\",\"commands\":[\"SubmitLesson\"],\"events\":[\"LessonSubmittedForReview\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"submit lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_submission\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonSubmittedForReview\"]},{\"name\":\"evidence required\",\"given\":[],\"when\":{},\"then\":[\"error evidence_required is returned\"]},{\"name\":\"reflection required\",\"given\":[],\"when\":{},\"then\":[\"error reflection_required is returned\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "control 'Submit for review' does not handle command error 'reflection_required'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_error_handling_without_recovery_behavior() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("error-handling-without-recovery.eventmodel.json"),
            "{\"name\":\"Submit lesson\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_submission\"}],\"events\":[{\"name\":\"LessonSubmittedForReview\",\"stream\":\"lesson_submission\",\"attributes\":[]}],\"commands\":[{\"name\":\"SubmitLesson\",\"inputs\":[],\"produces\":[\"LessonSubmittedForReview\"],\"errors\":[\"evidence_required\"]}],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"wireframe\":\"<button data-ref=\\\"Submit for review\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Submit for review\",\"command\":\"SubmitLesson\",\"error_handling\":[{\"error\":\"evidence_required\"}]}]}],\"slices\":[{\"name\":\"Submit lesson\",\"type\":\"state_change\",\"commands\":[\"SubmitLesson\"],\"events\":[\"LessonSubmittedForReview\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"submit lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_submission\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonSubmittedForReview\"]},{\"name\":\"evidence required\",\"given\":[],\"when\":{},\"then\":[\"error evidence_required is returned\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "error handling for 'evidence_required' must describe recovery behavior",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_navigation_controls_without_navigation_type() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("navigation-without-type.eventmodel.json"),
            "{\"name\":\"Repair queue\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"repair_queue_screen\",\"wireframe\":\"<button data-ref=\\\"Open settings\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Open settings\",\"navigation\":\"settings_screen\"}]}],\"slices\":[{\"name\":\"Show repair queue\",\"type\":\"state_view\",\"views\":[\"repair_queue_screen\"],\"acceptance_scenarios\":[{\"name\":\"show queue\",\"given\":[\"queue exists\"],\"when\":{},\"then\":[\"queue shown\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "navigation target must be classified as modeled_view, local_view_state, external_system, or external_workflow",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_modeled_view_navigation_targets_that_do_not_exist()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("modeled-view-navigation-missing-target.eventmodel.json"),
            "{\"name\":\"Lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"wireframe\":\"<button data-ref=\\\"Open settings\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Open settings\",\"navigation\":\"missing_settings_screen\",\"navigation_type\":\"modeled_view\"}]}],\"slices\":[{\"name\":\"Show lesson\",\"type\":\"state_view\",\"views\":[\"lesson_screen\"],\"acceptance_scenarios\":[{\"name\":\"show lesson\",\"given\":[\"lesson exists\"],\"when\":{},\"then\":[\"lesson shown\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "references unknown modeled view navigation target 'missing_settings_screen'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_local_view_state_navigation_targets_not_declared_by_view()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("local-view-state-navigation-missing-target.eventmodel.json"),
            "{\"name\":\"Manager progress\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"manager_progress_visibility_screen\",\"wireframe\":\"<button data-ref=\\\"Filter direct reports\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Filter direct reports\",\"navigation\":\"direct_reports\",\"navigation_type\":\"local_view_state\"}]}],\"slices\":[{\"name\":\"Show manager progress\",\"type\":\"state_view\",\"views\":[\"manager_progress_visibility_screen\"],\"acceptance_scenarios\":[{\"name\":\"show progress\",\"given\":[\"manager exists\"],\"when\":{},\"then\":[\"progress shown\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "local view state navigation target 'direct_reports' is not declared by view 'manager_progress_visibility_screen'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_external_workflow_navigation_without_workflow_target()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("external-workflow-navigation-without-target.eventmodel.json"),
            "{\"name\":\"Repair queue\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"repair_queue_screen\",\"wireframe\":\"<button data-ref=\\\"Open report detail\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Open report detail\",\"navigation\":\"report_detail\",\"navigation_type\":\"external_workflow\"}]}],\"slices\":[{\"name\":\"Show repair queue\",\"type\":\"state_view\",\"views\":[\"repair_queue_screen\"],\"acceptance_scenarios\":[{\"name\":\"show queue\",\"given\":[\"queue exists\"],\"when\":{},\"then\":[\"queue shown\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "external workflow navigation must name the target workflow",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_external_system_navigation_without_system_or_payload_contract()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("external-system-navigation-without-contract.eventmodel.json"),
            "{\"name\":\"Repair queue\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"repair_queue_screen\",\"wireframe\":\"<button data-ref=\\\"Run checkpoint\\\"></button>\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Run checkpoint\",\"navigation\":\"checkpoint\",\"navigation_type\":\"external_system\"}]}],\"slices\":[{\"name\":\"Show repair queue\",\"type\":\"state_view\",\"views\":[\"repair_queue_screen\"],\"acceptance_scenarios\":[{\"name\":\"show queue\",\"given\":[\"queue exists\"],\"when\":{},\"then\":[\"queue shown\"]}],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "external system navigation must name the external system and returned payload contract",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_state_view_slices_without_empty_read_model_state()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("state-view-without-empty-state.eventmodel.json"),
            "{\"name\":\"Lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson\"}],\"events\":[{\"name\":\"LessonCreated\",\"stream\":\"lesson\",\"attributes\":[{\"name\":\"title\",\"source\":\"command.title\"}]}],\"commands\":[{\"name\":\"CreateLesson\",\"inputs\":[\"title\"],\"produces\":[\"LessonCreated\"]}],\"read_models\":[{\"name\":\"lesson_read_model\",\"fields\":[{\"name\":\"title\",\"source\":\"LessonCreated.title\"}]}],\"views\":[{\"name\":\"lesson_screen\",\"wireframe\":\"<div data-ref=\\\"title\\\"></div>\",\"uses_read_models\":[\"lesson_read_model\"]}],\"slices\":[{\"name\":\"Show lesson\",\"type\":\"state_view\",\"events\":[\"LessonCreated\"],\"views\":[\"lesson_screen\"],\"acceptance_scenarios\":[{\"name\":\"populated lesson\",\"given\":[\"lesson exists\"],\"when\":{},\"then\":[\"lesson shown\"],\"read_model_states\":{\"lesson_read_model\":\"populated\"}}],\"contract_scenarios\":[{\"name\":\"lesson projector\",\"given\":[\"LessonCreated\"],\"when\":{},\"read_model_states\":{\"lesson_read_model\":\"populated\"},\"then\":[\"lesson_read_model is populated\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "state-view slice 'Show lesson' must include a scenario for empty state of read model 'lesson_read_model'",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_commands_across_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("submit-lesson.eventmodel.json"),
            "{\"name\":\"Submit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_submission\"}],\"events\":[{\"name\":\"LessonSubmittedForReview\",\"stream\":\"lesson_submission\",\"attributes\":[]}],\"commands\":[{\"name\":\"SubmitLessonForReview\",\"inputs\":[],\"produces\":[\"LessonSubmittedForReview\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Submit lesson\",\"type\":\"state_change\",\"commands\":[\"SubmitLessonForReview\"],\"events\":[\"LessonSubmittedForReview\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"submit lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_submission\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonSubmittedForReview\"]}]}]}",
        )?;
        write(
            slices.join("resubmit-lesson.eventmodel.json"),
            "{\"name\":\"Resubmit lesson workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_resubmission\"}],\"events\":[{\"name\":\"LessonResubmittedForReview\",\"stream\":\"lesson_resubmission\",\"attributes\":[]}],\"commands\":[{\"name\":\"SubmitLessonForReview\",\"inputs\":[],\"produces\":[\"LessonResubmittedForReview\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Resubmit lesson\",\"type\":\"state_change\",\"commands\":[\"SubmitLessonForReview\"],\"events\":[\"LessonResubmittedForReview\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"resubmit lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_resubmission\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonResubmittedForReview\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "command 'SubmitLessonForReview' is defined by more than one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_read_models_across_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("lesson-context.eventmodel.json"),
            "{\"name\":\"Lesson context workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[{\"name\":\"lesson_submission_context\",\"fields\":[]}],\"slices\":[{\"name\":\"Show lesson context\",\"read_models\":[\"lesson_submission_context\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;
        write(
            slices.join("review-context.eventmodel.json"),
            "{\"name\":\"Review context workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[{\"name\":\"lesson_submission_context\",\"fields\":[]}],\"slices\":[{\"name\":\"Review lesson context\",\"read_models\":[\"lesson_submission_context\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "read model 'lesson_submission_context' is defined by more than one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_views_across_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("lesson-screen.eventmodel.json"),
            "{\"name\":\"Lesson screen workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"uses_read_models\":[]}],\"slices\":[{\"name\":\"Show lesson screen\",\"views\":[\"lesson_screen\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;
        write(
            slices.join("review-screen.eventmodel.json"),
            "{\"name\":\"Review screen workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"uses_read_models\":[]}],\"slices\":[{\"name\":\"Review lesson screen\",\"views\":[\"lesson_screen\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "view 'lesson_screen' is defined by more than one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_controls_across_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("submit-control.eventmodel.json"),
            "{\"name\":\"Submit control workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Submit for review\",\"description\":\"Submit the lesson.\"}]}],\"slices\":[{\"name\":\"Submit lesson control\",\"views\":[\"lesson_screen\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;
        write(
            slices.join("review-control.eventmodel.json"),
            "{\"name\":\"Review control workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"uses_read_models\":[],\"controls\":[{\"label\":\"Submit for review\",\"description\":\"Request review.\"}]}],\"slices\":[{\"name\":\"Review lesson control\",\"views\":[\"lesson_screen\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "control 'Submit for review' on view 'lesson_screen' is defined by more than one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_automations_across_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("review-automation.eventmodel.json"),
            "{\"name\":\"Review automation workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Review lesson submission owner\",\"automations\":[\"Review lesson submission\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;
        write(
            slices.join("retry-review-automation.eventmodel.json"),
            "{\"name\":\"Retry review automation workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Retry review lesson submission owner\",\"automations\":[\"Review lesson submission\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "automation 'Review lesson submission' is defined by more than one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_translations_across_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("checkpoint-translation.eventmodel.json"),
            "{\"name\":\"Checkpoint translation workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Record checkpoint result owner\",\"translations\":[\"Record checkpoint result\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;
        write(
            slices.join("retry-checkpoint-translation.eventmodel.json"),
            "{\"name\":\"Retry checkpoint translation workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Retry record checkpoint result owner\",\"translations\":[\"Record checkpoint result\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "translation 'Record checkpoint result' is defined by more than one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_scenarios_across_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("teacher-accepts-progress.eventmodel.json"),
            "{\"name\":\"Teacher accepts progress workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Review progress\",\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"teacher accepts progress\",\"given\":[],\"when\":{},\"then\":[]}]}]}",
        )?;
        write(
            slices.join("mentor-accepts-progress.eventmodel.json"),
            "{\"name\":\"Mentor accepts progress workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"slices\":[{\"name\":\"Mentor review progress\",\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"teacher accepts progress\",\"given\":[],\"when\":{},\"then\":[]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "scenario 'teacher accepts progress' is ambiguously defined across slices",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_wireframes_across_slice_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("lesson-wireframe.eventmodel.json"),
            "{\"name\":\"Lesson wireframe workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"uses_read_models\":[],\"wireframe\":\"lesson-screen-v1\"}],\"slices\":[{\"name\":\"Show lesson wireframe\",\"views\":[\"lesson_screen\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;
        write(
            slices.join("review-wireframe.eventmodel.json"),
            "{\"name\":\"Review wireframe workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[],\"events\":[],\"commands\":[],\"read_models\":[],\"views\":[{\"name\":\"lesson_screen\",\"uses_read_models\":[],\"wireframe\":\"lesson-screen-v2\"}],\"slices\":[{\"name\":\"Review lesson wireframe\",\"views\":[\"lesson_screen\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "wireframe for view 'lesson_screen' is defined by more than one slice",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_conflicting_event_definitions_across_slice_files()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("lesson-accepted.eventmodel.json"),
            "{\"name\":\"Lesson accepted workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_progress\"}],\"events\":[{\"name\":\"LessonAccepted\",\"stream\":\"lesson_progress\",\"attributes\":[{\"name\":\"learner_id\",\"source\":\"command.learner_id\"}]}],\"commands\":[{\"name\":\"AcceptLesson\",\"inputs\":[\"learner_id\"],\"produces\":[\"LessonAccepted\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Accept lesson\",\"type\":\"state_change\",\"commands\":[\"AcceptLesson\"],\"events\":[\"LessonAccepted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"accept lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_progress\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonAccepted\"]}]}]}",
        )?;
        write(
            slices.join("lesson-accepted-with-reviewer.eventmodel.json"),
            "{\"name\":\"Lesson accepted with reviewer workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_progress\"}],\"events\":[{\"name\":\"LessonAccepted\",\"stream\":\"lesson_progress\",\"attributes\":[{\"name\":\"learner_id\",\"source\":\"command.learner_id\"},{\"name\":\"reviewer_id\",\"source\":\"command.reviewer_id\"}]}],\"commands\":[{\"name\":\"AcceptLessonWithReviewer\",\"inputs\":[\"learner_id\",\"reviewer_id\"],\"produces\":[\"LessonAccepted\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Accept lesson with reviewer\",\"type\":\"state_change\",\"commands\":[\"AcceptLessonWithReviewer\"],\"events\":[\"LessonAccepted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"accept lesson with reviewer\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_progress\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonAccepted\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'LessonAccepted' has conflicting definitions across slices",
            ));

        Ok(())
    }

    #[test]
    fn validate_allows_identical_event_definitions_across_slice_files() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("lesson-accepted.eventmodel.json"),
            "{\"name\":\"Lesson accepted workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_progress\"}],\"events\":[{\"name\":\"LessonAccepted\",\"stream\":\"lesson_progress\",\"attributes\":[{\"name\":\"learner_id\",\"source\":\"command.learner_id\"}]}],\"commands\":[{\"name\":\"AcceptLesson\",\"inputs\":[\"learner_id\"],\"produces\":[\"LessonAccepted\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Accept lesson\",\"type\":\"state_change\",\"commands\":[\"AcceptLesson\"],\"events\":[\"LessonAccepted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"accept lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_progress\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonAccepted\"]}]}]}",
        )?;
        write(
            slices.join("lesson-accepted-repeat.eventmodel.json"),
            "{\"name\":\"Lesson accepted repeat workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_progress\"}],\"events\":[{\"name\":\"LessonAccepted\",\"stream\":\"lesson_progress\",\"attributes\":[{\"name\":\"learner_id\",\"source\":\"command.learner_id\"}]}],\"commands\":[{\"name\":\"RepeatLessonAcceptance\",\"inputs\":[\"learner_id\"],\"produces\":[\"LessonAccepted\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Repeat lesson acceptance\",\"type\":\"state_change\",\"commands\":[\"RepeatLessonAcceptance\"],\"events\":[\"LessonAccepted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"repeat lesson acceptance\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_progress\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonAccepted\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn validate_rejects_shared_event_definitions_with_different_streams()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("course-progress-accepted.eventmodel.json"),
            "{\"name\":\"Course progress accepted workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"course_progress\"}],\"events\":[{\"name\":\"LessonAccepted\",\"stream\":\"course_progress\",\"attributes\":[{\"name\":\"learner_id\",\"source\":\"command.learner_id\"}]}],\"commands\":[{\"name\":\"AcceptCourseLesson\",\"inputs\":[\"learner_id\"],\"produces\":[\"LessonAccepted\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Accept course lesson\",\"type\":\"state_change\",\"commands\":[\"AcceptCourseLesson\"],\"events\":[\"LessonAccepted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"accept course lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"course_progress\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonAccepted\"]}]}]}",
        )?;
        write(
            slices.join("lesson-progress-accepted.eventmodel.json"),
            "{\"name\":\"Lesson progress accepted workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_progress\"}],\"events\":[{\"name\":\"LessonAccepted\",\"stream\":\"lesson_progress\",\"attributes\":[{\"name\":\"learner_id\",\"source\":\"command.learner_id\"}]}],\"commands\":[{\"name\":\"AcceptLessonProgress\",\"inputs\":[\"learner_id\"],\"produces\":[\"LessonAccepted\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Accept lesson progress\",\"type\":\"state_change\",\"commands\":[\"AcceptLessonProgress\"],\"events\":[\"LessonAccepted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"accept lesson progress\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_progress\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonAccepted\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'LessonAccepted' has conflicting definitions across slices",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_shared_event_definitions_with_different_source_provenance()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let slices = temp_dir.path().join("model/browser/data/slices");
        create_dir_all(&slices)?;
        write(
            slices.join("command-sourced-lesson-accepted.eventmodel.json"),
            "{\"name\":\"Command sourced lesson accepted workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_progress\"}],\"events\":[{\"name\":\"LessonAccepted\",\"stream\":\"lesson_progress\",\"attributes\":[{\"name\":\"learner_id\",\"source\":\"command.learner_id\"}]}],\"commands\":[{\"name\":\"AcceptLessonFromCommand\",\"inputs\":[\"learner_id\"],\"produces\":[\"LessonAccepted\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Accept command lesson\",\"type\":\"state_change\",\"commands\":[\"AcceptLessonFromCommand\"],\"events\":[\"LessonAccepted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"accept command lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_progress\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonAccepted\"]}]}]}",
        )?;
        write(
            slices.join("external-sourced-lesson-accepted.eventmodel.json"),
            "{\"name\":\"External sourced lesson accepted workflow\",\"version\":\"0.1.0\",\"board\":{},\"streams\":[{\"name\":\"lesson_progress\"}],\"events\":[{\"name\":\"LessonAccepted\",\"stream\":\"lesson_progress\",\"attributes\":[{\"name\":\"learner_id\",\"source\":\"external.review_packet.learner_id\"}]}],\"commands\":[{\"name\":\"AcceptLessonFromPacket\",\"inputs\":[],\"external_inputs\":[\"review_packet\"],\"external_input_schemas\":[{\"name\":\"review_packet\",\"fields\":[{\"name\":\"learner_id\"}]}],\"produces\":[\"LessonAccepted\"]}],\"read_models\":[],\"slices\":[{\"name\":\"Accept packet lesson\",\"type\":\"state_change\",\"commands\":[\"AcceptLessonFromPacket\"],\"events\":[\"LessonAccepted\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"accept packet lesson\",\"given\":[],\"given_streams\":[{\"stream\":\"lesson_progress\",\"state\":\"empty\"}],\"when\":{},\"then\":[\"LessonAccepted\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/slices"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "event 'LessonAccepted' has conflicting definitions across slices",
            ));

        Ok(())
    }

    #[test]
    fn validate_rejects_undeclared_board_automation_between_read_model_and_command()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let workflows = temp_dir.path().join("model/browser/data/workflows");
        create_dir_all(&workflows)?;
        write(
            workflows.join("undeclared-board-automation.eventmodel.json"),
            "{\"name\":\"Record SCIM member provisioning\",\"version\":\"0.1.0\",\"board\":{\"slices\":[{\"name\":\"Record SCIM member provisioning\",\"elements\":[{\"id\":\"scim_configuration\",\"kind\":\"read_model\",\"name\":\"scim_configuration\"},{\"id\":\"fake_automation\",\"kind\":\"automation\",\"name\":\"Undeclared automation\"},{\"id\":\"record_scim_member\",\"kind\":\"command\",\"name\":\"RecordSCIMMember\"}],\"connections\":[{\"from\":\"scim_configuration\",\"to\":\"fake_automation\"},{\"from\":\"fake_automation\",\"to\":\"record_scim_member\"}]}]},\"streams\":[{\"name\":\"member\"}],\"events\":[{\"name\":\"SCIMMemberRecorded\",\"stream\":\"member\",\"attributes\":[]}],\"commands\":[{\"name\":\"RecordSCIMMember\",\"inputs\":[],\"produces\":[\"SCIMMemberRecorded\"]}],\"read_models\":[{\"name\":\"scim_configuration\",\"fields\":[]}],\"slices\":[{\"name\":\"Record SCIM member provisioning\",\"type\":\"translation\",\"external_event\":\"SCIMMemberProvisioned\",\"events\":[\"SCIMMemberRecorded\"],\"acceptance_scenarios\":[],\"contract_scenarios\":[{\"name\":\"record scim member\",\"given\":[],\"when\":{},\"then\":[\"SCIMMemberRecorded\"]}]}]}",
        )?;

        Command::cargo_bin("emc")?
            .args(["validate", "model/browser/data/workflows"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "board element between read model 'scim_configuration' and command 'RecordSCIMMember' is not a declared automation",
            ));

        Ok(())
    }
}
