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
}
