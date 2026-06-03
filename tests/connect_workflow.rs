#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::{read_to_string, write};
    use std::path::Path;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    #[test]
    fn connect_workflow_adds_navigation_transition_to_canonical_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

        let initial_lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let initial_digest = digest_marker(&initial_lean)
            .ok_or("generated workflow artifact is missing an initial digest")?;

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
            .success()
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

        assert!(
            workflow_json.contains("\"to\": \"review-ticket\""),
            "workflow data must include the transition target"
        );
        assert!(
            workflow_json.contains("\"via_navigation\": \"review-ticket-screen\""),
            "workflow data must include the navigation trigger"
        );
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"review-ticket\", kind := \"navigation\", trigger := \"review-ticket-screen\" }]"
            ),
            "Lean artifact must represent the workflow transition"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"review-ticket\", kind: \"navigation\", trigger: \"review-ticket-screen\" }]"
            ),
            "Quint artifact must represent the workflow transition"
        );
        assert_ne!(
            initial_digest,
            digest_marker(&lean).ok_or("Lean artifact is missing an updated digest")?,
            "Lean digest must change when workflow transitions change"
        );
        assert_ne!(
            initial_digest,
            digest_marker(&quint).ok_or("Quint artifact is missing an updated digest")?,
            "Quint digest must change when workflow transitions change"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_adds_command_and_event_transitions_to_canonical_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "submit-ticket",
            "Submit ticket",
            "Actor submits repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "submit-ticket",
                "--via",
                "command",
                "--name",
                "SubmitTicketForReview",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to submit-ticket",
            ));

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "submit-ticket",
                "--to",
                "review-ticket",
                "--via",
                "event",
                "--name",
                "TicketSubmittedForReview",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
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

        assert!(
            workflow_json.contains("\"via_command\": \"SubmitTicketForReview\""),
            "workflow data must include the command trigger"
        );
        assert!(
            workflow_json.contains("\"via_event\": \"TicketSubmittedForReview\""),
            "workflow data must include the event trigger"
        );
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"submit-ticket\", kind := \"command\", trigger := \"SubmitTicketForReview\" },{ source := \"submit-ticket\", target := \"review-ticket\", kind := \"event\", trigger := \"TicketSubmittedForReview\" }]"
            ),
            "Lean artifact must represent command and event workflow transitions"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"submit-ticket\", kind: \"command\", trigger: \"SubmitTicketForReview\" },{ source: \"submit-ticket\", target: \"review-ticket\", kind: \"event\", trigger: \"TicketSubmittedForReview\" }]"
            ),
            "Quint artifact must represent command and event workflow transitions"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_removes_modeled_transition_from_canonical_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

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
            .success()
            .stdout(predicate::str::contains(
                "removed transition capture-ticket to review-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            !workflow_json.contains("\"via_navigation\": \"review-ticket-screen\""),
            "workflow data must remove the transition trigger"
        );
        assert!(
            workflow_json.contains("\"via_navigation\": \"alternate-review-ticket-screen\""),
            "workflow data must preserve the remaining transition to the target step"
        );
        assert!(
            lean.contains("alternate-review-ticket-screen"),
            "Lean artifact must preserve the remaining workflow transition"
        );
        assert!(
            quint.contains("alternate-review-ticket-screen"),
            "Quint artifact must preserve the remaining workflow transition"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_rejects_removing_required_incoming_transition_without_mutating_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

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

        let workflow_before = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;

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
            .failure()
            .stderr(predicate::str::contains(
                "removing transition would leave workflow step 'review-ticket' without an incoming transition",
            ));

        let workflow_after = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;

        assert_eq!(
            workflow_before, workflow_after,
            "rejected transition removal must not mutate workflow data"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_rejects_unknown_transition_without_mutating_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

        let workflow_before = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;

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
            .failure()
            .stderr(predicate::str::contains(
                "workflow transition capture-ticket->review-ticket:navigation:review-ticket-screen does not exist",
            ));

        let workflow_after = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;

        assert_eq!(
            workflow_before, workflow_after,
            "rejected transition removal must not mutate workflow data"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_removes_workflow_exit_transition() -> Result<(), Box<dyn Error>> {
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
                "workflow",
                "--slug",
                "close-ticket",
                "--name",
                "Close ticket",
                "--description",
                "Actor closes a repair ticket.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
                "--reason",
                "Closed tickets continue to completion.",
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
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "removed transition capture-ticket to close-ticket",
            ));

        Command::cargo_bin("emc")?
            .arg("check")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            !workflow_json.contains("\"via_outcome\": \"ticket-closed\""),
            "workflow data must remove the workflow-exit trigger"
        );
        assert!(
            lean.contains("def workflowTransitions : List WorkflowTransition := []"),
            "Lean artifact must remove the workflow-exit transition"
        );
        assert!(
            quint.contains("val workflowTransitions = []"),
            "Quint artifact must remove the workflow-exit transition"
        );

        Ok(())
    }

    #[test]
    fn remove_transition_requires_exact_in_workflow_command_shape() -> Result<(), Box<dyn Error>> {
        [
            [
                "delete",
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
            ],
            [
                "remove",
                "edge",
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
            ],
            [
                "remove",
                "transition",
                "--model",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--source",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--target",
                "review-ticket",
                "--via",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "review-ticket",
                "--kind",
                "navigation",
                "--name",
                "review-ticket-screen",
            ],
            [
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
                "--trigger",
                "review-ticket-screen",
            ],
        ]
        .into_iter()
        .try_for_each(assert_usage)?;

        Ok(())
    }

    #[test]
    fn remove_transition_requires_exact_workflow_exit_command_shape() -> Result<(), Box<dyn Error>>
    {
        [
            [
                "delete",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "edge",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--model",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--source",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--target-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--kind",
                "outcome",
                "--name",
                "ticket-closed",
            ],
            [
                "remove",
                "transition",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "close-ticket",
                "--via",
                "outcome",
                "--trigger",
                "ticket-closed",
            ],
        ]
        .into_iter()
        .try_for_each(assert_usage)?;

        Ok(())
    }

    #[test]
    fn connect_workflow_adds_external_trigger_transition_to_canonical_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "record-callback",
            "Record callback",
            "System records an external callback.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "record-callback",
                "--via",
                "external_trigger",
                "--name",
                "callback_received",
            ])
            .current_dir(temp_dir.path())
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

        assert!(
            workflow_json.contains("\"via_external_trigger\": \"callback_received\""),
            "workflow data must include the external trigger"
        );
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"record-callback\", kind := \"external_trigger\", trigger := \"callback_received\" }]"
            ),
            "Lean artifact must represent the external trigger workflow transition"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"record-callback\", kind: \"external_trigger\", trigger: \"callback_received\" }]"
            ),
            "Quint artifact must represent the external trigger workflow transition"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_adds_workflow_exit_to_canonical_artifacts() -> Result<(), Box<dyn Error>> {
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connected capture-ticket to repair-complete",
            ));

        let workflow_json = read_to_string(
            temp_dir
                .path()
                .join("model/browser/data/workflows/open-ticket.eventmodel.json"),
        )?;
        let lean = read_to_string(temp_dir.path().join("model/lean/OpenTicket.lean"))?;
        let quint = read_to_string(temp_dir.path().join("model/quint/OpenTicket.qnt"))?;

        assert!(
            workflow_json.contains("\"to_workflow\": \"repair-complete\""),
            "workflow data must include the target workflow"
        );
        assert!(
            workflow_json.contains("\"via_outcome\": \"ticket_closed\""),
            "workflow data must include the workflow exit outcome"
        );
        assert!(
            workflow_json.contains("\"exit_reason\": \"Closed tickets continue to completion.\""),
            "workflow data must include the workflow exit rationale"
        );
        assert!(
            lean.contains(
                "def workflowTransitions : List WorkflowTransition := [{ source := \"capture-ticket\", target := \"repair-complete\", kind := \"workflow_exit:outcome\", trigger := \"ticket_closed\" }]"
            ),
            "Lean artifact must represent the workflow exit"
        );
        assert!(
            quint.contains(
                "val workflowTransitions = [{ source: \"capture-ticket\", target: \"repair-complete\", kind: \"workflow_exit:outcome\", trigger: \"ticket_closed\" }]"
            ),
            "Quint artifact must represent the workflow exit"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_rejects_duplicate_transition_without_rewriting_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "submit-ticket",
            "Submit ticket",
            "Actor submits repair ticket details.",
        )?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "submit-ticket",
                "--via",
                "command",
                "--name",
                "SubmitTicketForReview",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let workflow_before = read_to_string(&workflow_path)?;
        let lean_before = read_to_string(&lean_path)?;
        let quint_before = read_to_string(&quint_path)?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "submit-ticket",
                "--via",
                "command",
                "--name",
                "SubmitTicketForReview",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "workflow transition capture-ticket->submit-ticket:command:SubmitTicketForReview already exists",
            ));

        assert_eq!(
            workflow_before,
            read_to_string(workflow_path)?,
            "duplicate transition rejection must leave browser workflow data unchanged"
        );
        assert_eq!(
            lean_before,
            read_to_string(lean_path)?,
            "duplicate transition rejection must leave Lean workflow data unchanged"
        );
        assert_eq!(
            quint_before,
            read_to_string(quint_path)?,
            "duplicate transition rejection must leave Quint workflow data unchanged"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_rejects_unknown_target_slice_without_rewriting_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;

        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let workflow_before = read_to_string(&workflow_path)?;
        let lean_before = read_to_string(&lean_path)?;
        let quint_before = read_to_string(&quint_path)?;

        Command::cargo_bin("emc")?
            .args([
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to",
                "missing-ticket",
                "--via",
                "navigation",
                "--name",
                "missing-ticket-screen",
            ])
            .current_dir(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "unknown workflow step missing-ticket",
            ));

        assert_eq!(
            workflow_before,
            read_to_string(workflow_path)?,
            "unknown target rejection must leave browser workflow data unchanged"
        );
        assert_eq!(
            lean_before,
            read_to_string(lean_path)?,
            "unknown target rejection must leave Lean workflow data unchanged"
        );
        assert_eq!(
            quint_before,
            read_to_string(quint_path)?,
            "unknown target rejection must leave Quint workflow data unchanged"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_rejects_workflow_document_identity_drift_without_rewriting_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let workflow_before = read_to_string(&workflow_path)?;
        let lean_before = read_to_string(&lean_path)?;
        let quint_before = read_to_string(&quint_path)?;
        let drifted_workflow =
            workflow_before.replace("\"name\": \"Open ticket\"", "\"name\": \"Altered ticket\"");
        write(&workflow_path, &drifted_workflow)?;

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
            .failure()
            .stderr(predicate::str::contains(
                "workflow document name 'Altered ticket' does not match index name 'Open ticket'",
            ));

        assert_eq!(
            drifted_workflow,
            read_to_string(workflow_path)?,
            "identity drift rejection must leave the drifted workflow document unchanged"
        );
        assert_eq!(
            lean_before,
            read_to_string(lean_path)?,
            "identity drift rejection must leave Lean workflow data unchanged"
        );
        assert_eq!(
            quint_before,
            read_to_string(quint_path)?,
            "identity drift rejection must leave Quint workflow data unchanged"
        );
        assert!(
            !temp_dir
                .path()
                .join("model/lean/AlteredTicket.lean")
                .exists(),
            "identity drift rejection must not create formal artifacts for a drifted workflow name"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_rejects_workflow_document_description_drift_without_rewriting_artifacts()
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

        add_slice(
            temp_dir.path(),
            "capture-ticket",
            "Capture ticket",
            "Actor enters repair ticket details.",
        )?;
        add_slice(
            temp_dir.path(),
            "review-ticket",
            "Review ticket",
            "Actor reviews repair ticket details.",
        )?;

        let workflow_path = temp_dir
            .path()
            .join("model/browser/data/workflows/open-ticket.eventmodel.json");
        let lean_path = temp_dir.path().join("model/lean/OpenTicket.lean");
        let quint_path = temp_dir.path().join("model/quint/OpenTicket.qnt");
        let workflow_before = read_to_string(&workflow_path)?;
        let lean_before = read_to_string(&lean_path)?;
        let quint_before = read_to_string(&quint_path)?;
        let drifted_workflow = workflow_before.replace(
            "\"description\": \"Actor opens a repair ticket.\"",
            "\"description\": \"Altered workflow description.\"",
        );
        write(&workflow_path, &drifted_workflow)?;

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
            .failure()
            .stderr(predicate::str::contains(
                "workflow document description 'Altered workflow description.' does not match index description 'Actor opens a repair ticket.'",
            ));

        assert_eq!(
            drifted_workflow,
            read_to_string(workflow_path)?,
            "description drift rejection must leave the drifted workflow document unchanged"
        );
        assert_eq!(
            lean_before,
            read_to_string(lean_path)?,
            "description drift rejection must leave Lean workflow data unchanged"
        );
        assert_eq!(
            quint_before,
            read_to_string(quint_path)?,
            "description drift rejection must leave Quint workflow data unchanged"
        );

        Ok(())
    }

    #[test]
    fn connect_workflow_exit_requires_exact_flag_order() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let malformed_commands = [
            [
                "wrong-command",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "wrong-subject",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--wrong-workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--wrong-from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--wrong-target",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--wrong-via",
                "outcome",
                "--name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--wrong-name",
                "ticket_closed",
                "--reason",
                "Closed tickets continue to completion.",
            ],
            [
                "connect",
                "workflow",
                "--workflow",
                "open-ticket",
                "--from",
                "capture-ticket",
                "--to-workflow",
                "repair-complete",
                "--via",
                "outcome",
                "--name",
                "ticket_closed",
                "--wrong-reason",
                "Closed tickets continue to completion.",
            ],
        ];

        for malformed_command in malformed_commands {
            Command::cargo_bin("emc")?
                .args(malformed_command)
                .current_dir(temp_dir.path())
                .assert()
                .failure()
                .stderr(predicate::str::contains(
                    "usage: emc init --name <project-name>",
                ));
        }

        Ok(())
    }

    fn assert_usage<const N: usize>(args: [&str; N]) -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("emc")?
            .args(args)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "usage: emc init --name <project-name>",
            ));
        Ok(())
    }

    fn add_slice(
        cwd: &Path,
        slug: &str,
        name: &str,
        description: &str,
    ) -> Result<(), Box<dyn Error>> {
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
                description,
            ])
            .current_dir(cwd)
            .assert()
            .success();
        Ok(())
    }

    fn digest_marker(contents: &str) -> Option<String> {
        contents.lines().find_map(|line| {
            line.trim()
                .strip_prefix("-- EMC-DIGEST: ")
                .or_else(|| line.trim().strip_prefix("// EMC-DIGEST: "))
                .map(str::to_owned)
        })
    }
}
