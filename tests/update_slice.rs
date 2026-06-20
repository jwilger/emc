// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{exists, read_to_string};
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn update_slice_description_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

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
            .success()
            .stdout(predicate::str::contains("updated slice Capture ticket"));

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            workflow_lean.contains(
                "{ slug := \"capture-ticket\", name := \"Capture ticket\", kind := SliceKindName.stateView, description := \"Actor enters repair ticket details and priority.\" }"
            ),
            "workflow Lean artifact must represent the updated slice detail"
        );
        assert!(
            workflow_quint
                .contains("description: \"Actor enters repair ticket details and priority.\""),
            "workflow Quint artifact must represent the updated slice detail"
        );
        assert!(
            slice_lean.contains(
                "def sliceDescription := \"Actor enters repair ticket details and priority.\""
            ),
            "slice Lean artifact must represent the updated slice description"
        );
        assert!(
            slice_quint.contains(
                "val sliceDescription = \"Actor enters repair ticket details and priority.\""
            ),
            "slice Quint artifact must represent the updated slice description"
        );

        Ok(())
    }

    #[test]
    fn update_slice_kind_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

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
            .success()
            .stdout(predicate::str::contains("updated slice Capture ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;

        assert!(
            workflow_lean.contains(
                "{ slug := \"capture-ticket\", name := \"Capture ticket\", kind := SliceKindName.automation, description := \"Actor enters repair ticket details.\" }"
            ),
            "workflow Lean artifact must represent the updated slice kind"
        );
        assert!(
            workflow_quint.contains("kind: SliceAutomation"),
            "workflow Quint artifact must represent the updated slice kind"
        );
        assert!(
            slice_lean.contains("def sliceKind : SliceKindName := SliceKindName.automation"),
            "slice Lean artifact must represent the updated slice kind"
        );
        assert!(
            slice_quint.contains("val sliceKind: SliceKindName = SliceAutomation"),
            "slice Quint artifact must represent the updated slice kind"
        );

        Ok(())
    }

    #[test]
    fn update_slice_name_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--name",
                "Capture repair ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated slice Capture repair ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean = read_to_string(
            temp_dir
                .path()
                .join("model/lean/slices/CaptureRepairTicket.lean"),
        )?;
        let slice_quint = read_to_string(
            temp_dir
                .path()
                .join("model/quint/slices/CaptureRepairTicket.qnt"),
        )?;

        assert!(
            workflow_lean.contains(
                "{ slug := \"capture-ticket\", name := \"Capture repair ticket\", kind := SliceKindName.stateView, description := \"Actor enters repair ticket details.\" }"
            ),
            "workflow Lean artifact must represent the updated slice name"
        );
        assert!(
            workflow_quint.contains("name: \"Capture repair ticket\""),
            "workflow Quint artifact must represent the updated slice name"
        );
        assert!(
            slice_lean.contains("namespace CaptureRepairTicket"),
            "slice Lean artifact must move to the updated module"
        );
        assert!(
            slice_lean.contains("def sliceName := \"Capture repair ticket\""),
            "slice Lean artifact must represent the updated slice name"
        );
        assert!(
            slice_quint.contains("module CaptureRepairTicket {"),
            "slice Quint artifact must move to the updated module"
        );
        assert!(
            slice_quint.contains("val sliceName = \"Capture repair ticket\""),
            "slice Quint artifact must represent the updated slice name"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?,
            "slice name update must remove the old Lean slice module"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?,
            "slice name update must remove the old Quint slice module"
        );

        Ok(())
    }

    #[test]
    fn update_slice_name_preserves_outgoing_workflow_transitions() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_navigation_chain(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "triage-intake",
                "--name",
                "Clinical triage",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("updated slice Clinical triage"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/IntakeVisit.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/IntakeVisit.qnt"))?;
        assert!(
            workflow_lean.contains(
                "{ source := \"triage-intake\", target := \"schedule-visit\", kind := WorkflowTransitionKind.navigation, trigger := \"schedule-visit-screen\", sourceControl := \"open-schedule-visit\", targetView := \"schedule-visit-screen\", rationale := \"\", payloadContract := \"\" }"
            ),
            "slice rename must preserve outgoing workflow transitions"
        );
        assert!(
            workflow_quint.contains(
                "{ source: \"triage-intake\", target: \"schedule-visit\", kind: Navigation, trigger: \"schedule-visit-screen\", sourceControl: \"open-schedule-visit\", targetView: \"schedule-visit-screen\", rationale: \"\", payloadContract: \"\" }"
            ),
            "slice rename must preserve outgoing workflow transitions"
        );

        Ok(())
    }

    #[test]
    fn update_slice_name_rejects_formal_module_name_collisions() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

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
                "update",
                "slice",
                "--slug",
                "review-ticket",
                "--name",
                "Capture-ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice module CaptureTicket already exists",
            ));

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;
        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/ReviewTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/ReviewTicket.qnt"))?;

        assert!(
            workflow_lean.contains(
                "{ slug := \"review-ticket\", name := \"Review ticket\", kind := SliceKindName.stateView, description := \"Actor reviews repair ticket details.\" }"
            ),
            "rejected slice rename must not mutate the workflow Lean artifact"
        );
        assert!(
            workflow_quint.contains("name: \"Review ticket\""),
            "rejected slice rename must not mutate the workflow Quint artifact"
        );
        assert!(
            slice_lean.contains("def sliceName := \"Review ticket\""),
            "rejected slice rename must not mutate the slice Lean artifact"
        );
        assert!(
            slice_quint.contains("val sliceName = \"Review ticket\""),
            "rejected slice rename must not mutate the slice Quint artifact"
        );

        Ok(())
    }

    #[test]
    fn update_slice_name_rejects_non_slug_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slice",
                "capture-ticket",
                "--name",
                "Capture repair ticket",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn update_slice_name_rejects_non_name_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--title",
                "Capture repair ticket",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn remove_slice_removes_synchronized_slice_artifacts_and_connected_transitions()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

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
                "open-review-ticket",
                "--target-view",
                "review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["remove", "slice", "--slug", "review-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("removed slice Review ticket"));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_lean.contains(
                "def workflowSlices : List WorkflowSlice := [{ slug := \"capture-ticket\" }]"
            ),
            "workflow Lean artifact must keep only remaining slices"
        );
        assert!(
            workflow_lean.contains("def workflowTransitions : List WorkflowTransition := []"),
            "workflow Lean artifact must remove transitions involving the slice"
        );
        assert!(
            workflow_quint.contains(
                "val workflowSlices: List[WorkflowSlice] = [{ slug: \"capture-ticket\" }]"
            ),
            "workflow Quint artifact must keep only remaining slices"
        );
        assert!(
            workflow_quint.contains("val workflowTransitions: List[WorkflowTransition] = []"),
            "workflow Quint artifact must remove transitions involving the slice"
        );
        assert!(
            !exists(temp_dir.path().join("model/lean/slices/ReviewTicket.lean"))?,
            "removed slice Lean artifact must be deleted"
        );
        assert!(
            !exists(temp_dir.path().join("model/quint/slices/ReviewTicket.qnt"))?,
            "removed slice Quint artifact must be deleted"
        );

        Ok(())
    }

    #[test]
    fn store_replays_cleanly_after_removing_a_connected_slice() -> Result<(), Box<dyn Error>> {
        // Regression for the "remove slice wedges the store" failure: once a
        // slice and its transition are removed, the event-log projection must
        // still replay cleanly so the very next commands keep working rather
        // than every subsequent command erroring.
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;

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
                "open-review-ticket",
                "--target-view",
                "review-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["remove", "slice", "--slug", "review-ticket"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("removed slice Review ticket"));

        // The very next read must succeed — the store is not wedged.
        Command::cargo_bin("emc")?
            .args(["list", "slices"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Capture ticket"));

        // And a subsequent mutation must succeed and replay cleanly.
        Command::cargo_bin("emc")?
            .args([
                "add",
                "slice",
                "--workflow",
                "open-ticket",
                "--slug",
                "close-ticket",
                "--name",
                "Close ticket",
                "--type",
                "state_view",
                "--description",
                "Actor closes the repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::cargo_bin("emc")?
            .args(["list", "slices"])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("Close ticket"));

        Ok(())
    }

    #[test]
    fn remove_middle_slice_rejects_outgoing_navigation_dependencies() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        initialize_project_with_navigation_chain(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args(["remove", "slice", "--slug", "triage-intake"])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "slice triage-intake has outgoing workflow transitions",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_lean = read_to_string(temp_dir.path().join("model/lean/IntakeVisit.lean"))?;
        let workflow_quint = read_to_string(temp_dir.path().join("model/quint/IntakeVisit.qnt"))?;
        assert!(
            workflow_lean.contains("triage-intake-screen"),
            "rejected middle slice removal must preserve incoming navigation transitions"
        );
        assert!(
            workflow_quint.contains("schedule-visit-screen"),
            "rejected middle slice removal must preserve outgoing navigation transitions"
        );

        Ok(())
    }

    #[test]
    fn remove_slice_requires_exact_slug_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["remove", "slice", "--slice", "capture-ticket"])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn remove_scenario_removes_it_from_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_acceptance_scenario(temp_dir.path(), "Actor captures ticket")?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "scenario",
                "--slice",
                "capture-ticket",
                "--name",
                "Actor captures ticket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed scenario Actor captures ticket from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_lean.contains("Actor captures ticket"),
            "removed scenario must be absent from Lean slice artifacts"
        );
        assert!(
            !slice_quint.contains("Actor captures ticket"),
            "removed scenario must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn update_scenario_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_acceptance_scenario(temp_dir.path(), "Actor captures ticket")?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "scenario",
                "--slice",
                "capture-ticket",
                "--name",
                "Actor captures ticket",
                "--kind",
                "acceptance",
                "--given",
                "ticket intake screen is open",
                "--when",
                "the actor submits corrected ticket details",
                "--then",
                "the corrected details are visible for review",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated scenario Actor captures ticket on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_lean.contains("the actor submits corrected ticket details"),
            "updated scenario must be represented in Lean slice artifacts"
        );
        assert!(
            slice_quint.contains("the corrected details are visible for review"),
            "updated scenario must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_lean.contains("the actor submits ticket details"),
            "old scenario text must be absent from Lean slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn remove_command_definition_removes_it_from_synchronized_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_capture_ticket_command(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "command",
                "--slice",
                "capture-ticket",
                "--name",
                "CaptureTicket",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed command CaptureTicket from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_lean.contains("name := \"CaptureTicket\""),
            "removed command must be absent from Lean slice artifacts"
        );
        assert!(
            !slice_quint.contains("name: \"CaptureTicket\""),
            "removed command must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn update_command_definition_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_capture_ticket_command(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "command",
                "--slice",
                "capture-ticket",
                "--name",
                "CaptureTicket",
                "--input",
                "ticket_summary",
                "--input-source",
                "actor",
                "--input-description",
                "summary field on the intake form",
                "--input-provenance",
                "actor keystrokes -> summary field",
                "--emits",
                "TicketUpdated",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated command CaptureTicket on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_lean.contains("ticket_summary"),
            "updated command input must be represented in Lean slice artifacts"
        );
        assert!(
            slice_quint.contains("TicketUpdated"),
            "updated emitted event must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_lean.contains("ticket_title"),
            "old command input must be absent from Lean slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn remove_event_definition_removes_it_from_synchronized_artifacts() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_ticket_captured_event(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed event TicketCaptured from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_lean.contains("name := \"TicketCaptured\""),
            "removed event must be absent from Lean slice artifacts"
        );
        assert!(
            !slice_quint.contains("name: \"TicketCaptured\""),
            "removed event must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn update_event_definition_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_ticket_captured_event(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "ticket-updates",
                "--attribute",
                "summary",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "ticket_summary",
                "--attribute-source-field",
                "value",
                "--generated-source-kind",
                "projection",
                "--attribute-provenance",
                "projection summary field",
                "--observed",
                "true",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated event TicketCaptured on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_lean.contains("ticket-updates"),
            "updated event stream must be represented in Lean slice artifacts"
        );
        assert!(
            slice_quint.contains("summary"),
            "updated event attribute must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_lean.contains("ticket-events"),
            "old event stream must be absent from Lean slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn remove_read_model_definition_removes_it_from_synchronized_artifacts()
    -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_ticket_captured_event(temp_dir.path())?;
        add_ticket_state_read_model(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "read-model",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_state",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed read model ticket_state from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_lean.contains("name := \"ticket_state\""),
            "removed read model must be absent from Lean slice artifacts"
        );
        assert!(
            !slice_quint.contains("name: \"ticket_state\""),
            "removed read model must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn update_read_model_definition_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_ticket_captured_event(temp_dir.path())?;
        add_ticket_state_read_model(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "read-model",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_state",
                "--field",
                "ticket_summary",
                "--field-source",
                "event_attribute",
                "--source-event",
                "TicketCaptured",
                "--source-attribute",
                "ticket_title",
                "--field-provenance",
                "TicketCaptured.ticket_title -> summary",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated read model ticket_state on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_lean.contains("ticket_summary"),
            "updated read model field must be represented in Lean slice artifacts"
        );
        assert!(
            slice_quint.contains("TicketCaptured.ticket_title -> summary"),
            "updated read model provenance must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_lean.contains("name := \"ticket_title\""),
            "old read model field must be absent from Lean slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn remove_view_definition_removes_it_from_synchronized_artifacts() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_ticket_captured_event(temp_dir.path())?;
        add_ticket_state_read_model(temp_dir.path())?;
        add_ticket_detail_view(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "remove",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_detail",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed view ticket_detail from slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            !slice_lean.contains("name := \"ticket_detail\""),
            "removed view must be absent from Lean slice artifacts"
        );
        assert!(
            !slice_quint.contains("name: \"ticket_detail\""),
            "removed view must be absent from Quint slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn update_view_definition_rewrites_synchronized_artifacts() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        initialize_project_with_slice(temp_dir.path())?;
        add_ticket_captured_event(temp_dir.path())?;
        add_ticket_state_read_model(temp_dir.path())?;
        add_ticket_detail_view(temp_dir.path())?;

        Command::cargo_bin("emc")?
            .args([
                "update",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_detail",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_summary_view",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "ticket-summary-label",
                "--field-provenance",
                "ticket_state.ticket_title -> summary label",
                "--bit-encoding",
                "plain text summary label",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "updated view ticket_detail on slice capture-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let slice_lean =
            read_to_string(temp_dir.path().join("model/lean/slices/CaptureTicket.lean"))?;
        let slice_quint =
            read_to_string(temp_dir.path().join("model/quint/slices/CaptureTicket.qnt"))?;
        assert!(
            slice_lean.contains("ticket_summary_view"),
            "updated view field must be represented in Lean slice artifacts"
        );
        assert!(
            slice_quint.contains("ticket-summary-label"),
            "updated view sketch token must be represented in Quint slice artifacts"
        );
        assert!(
            !slice_lean.contains("ticket_title_view"),
            "old view field must be absent from Lean slice artifacts"
        );

        Ok(())
    }

    #[test]
    fn update_slice_kind_rejects_non_slug_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slice",
                "capture-ticket",
                "--type",
                "automation",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    #[test]
    fn update_slice_kind_rejects_non_type_flag() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "update",
                "slice",
                "--slug",
                "capture-ticket",
                "--kind",
                "automation",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc <command> [arguments]; run emc --help",
            ));

        Ok(())
    }

    fn initialize_project_with_slice(cwd: &Path) -> Result<(), Box<dyn Error>> {
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
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_acceptance_scenario(cwd: &Path, name: &str) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "scenario",
                "--slice",
                "capture-ticket",
                "--kind",
                "acceptance",
                "--name",
                name,
                "--given",
                "ticket intake screen is open",
                "--when",
                "the actor submits ticket details",
                "--then",
                "the ticket details are visible for review",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_capture_ticket_command(cwd: &Path) -> Result<(), Box<dyn Error>> {
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
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_ticket_captured_event(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "event",
                "--slice",
                "capture-ticket",
                "--name",
                "TicketCaptured",
                "--stream",
                "ticket-events",
                "--attribute",
                "title",
                "--attribute-source",
                "generated",
                "--attribute-source-name",
                "ticket_title",
                "--attribute-source-field",
                "value",
                "--generated-source-kind",
                "projection",
                "--attribute-provenance",
                "projection title field",
                "--observed",
                "true",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_ticket_state_read_model(cwd: &Path) -> Result<(), Box<dyn Error>> {
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
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn add_ticket_detail_view(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args([
                "add",
                "view",
                "--slice",
                "capture-ticket",
                "--name",
                "ticket_detail",
                "--read-model",
                "ticket_state",
                "--field",
                "ticket_title_view",
                "--source-field",
                "ticket_title",
                "--sketch-token",
                "ticket-title-label",
                "--field-provenance",
                "ticket_state.ticket_title -> title label",
                "--bit-encoding",
                "plain text title label",
            ])
            .current_dir(cwd)
            .assert()
            .success();

        Ok(())
    }

    fn initialize_project_with_navigation_chain(cwd: &Path) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(["init", "--name", "Clinic Intake"])
            .current_dir(cwd)
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
            .current_dir(cwd)
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
                .current_dir(cwd)
                .assert()
                .success();
            Ok::<(), Box<dyn Error>>(())
        })?;

        [
            (
                "capture-intake",
                "triage-intake",
                "triage-intake-screen",
                "open-triage-intake",
            ),
            (
                "triage-intake",
                "schedule-visit",
                "schedule-visit-screen",
                "open-schedule-visit",
            ),
        ]
        .into_iter()
        .try_for_each(|(source, target, navigation, control)| {
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
                    "--source-control",
                    control,
                    "--target-view",
                    navigation,
                ])
                .current_dir(cwd)
                .assert()
                .success();
            Ok::<(), Box<dyn Error>>(())
        })?;

        Ok(())
    }
}
